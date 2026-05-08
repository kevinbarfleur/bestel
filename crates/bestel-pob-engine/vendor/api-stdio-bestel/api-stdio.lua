-- api-stdio-bestel: ndJSON-over-stdio harness for the Bestel sidecar.
--
-- Provenance:
--   Conceptually inspired by ianderse/PathOfBuilding (branch `api-stdio`),
--   but rewritten from scratch against the upstream HeadlessWrapper.lua
--   so we can extend `set_config` to expose the full Calcs key set
--   (enemyIsBoss, charges, flasks, impale stacks, onslaught buff). The
--   protocol shape (single-line JSON in / single-line JSON out, `action`
--   field, `ok`/`error` reply envelope) deliberately matches the upstream
--   contract for compatibility.
--
-- Invocation:
--   luajit api-stdio.lua <pob_root_dir> <game>
--   - <pob_root_dir>: absolute path to PathOfBuilding{,-PoE2}/src/
--   - <game>: "poe1" or "poe2" (currently informational only)
--
-- Reads JSON commands from stdin, one per line. Writes JSON replies to
-- stdout, one per line. Logs go to stderr.

local pob_root = arg and arg[1]
local game_tag = (arg and arg[2]) or "poe1"

if not pob_root or pob_root == "" then
    io.stderr:write("api-stdio-bestel: missing pob_root argument\n")
    os.exit(1)
end

-- Redirect Lua's `print` to stderr so PoB's chatty boot logs do not
-- contaminate stdout (which is reserved for ndJSON replies).
local _print_stderr = function(...)
    local args = {...}
    for i = 1, select("#", ...) do
        if i > 1 then io.stderr:write("\t") end
        io.stderr:write(tostring(args[i]))
    end
    io.stderr:write("\n")
end
print = _print_stderr

-- Make PoB modules importable. PoB expects the cwd to be its src/ dir.
-- Normalise separators; Lua's package.path accepts forward slashes on Windows.
local pob_root_norm = pob_root:gsub("\\", "/")
package.path = pob_root_norm .. "/?.lua;" .. pob_root_norm .. "/?/init.lua;" .. package.path
package.path = pob_root_norm .. "/Modules/?.lua;" .. package.path
package.path = pob_root_norm .. "/Classes/?.lua;" .. package.path
local runtime_lua = pob_root_norm:gsub("/src/?$", "") .. "/runtime/lua"
package.path = runtime_lua .. "/?.lua;" .. runtime_lua .. "/?/init.lua;" .. package.path

-- Bring in dkjson (PoB ships it under runtime/lua/).
local ok_json, dkjson = pcall(require, "dkjson")
if not ok_json then
    io.stderr:write("api-stdio-bestel: failed to load dkjson: " .. tostring(dkjson) .. "\n")
    os.exit(1)
end

-- Stub additional UI globals that the upstream HeadlessWrapper does not
-- cover. PoB's Launch.lua occasionally references newer callbacks than
-- HeadlessWrapper anticipates; we patch them in pre-load so the headless
-- bootstrap succeeds.
function GetVirtualScreenSize()
    return 1920, 1080
end
function ConPrintf(...) end
function ConPrintTable(...) end

-- Stub PoB UI control factory callbacks. The Calcs/build-flag pipeline
-- occasionally walks DropDown / EditBox / ButtonControl trees during
-- recompute (e.g. when initialising a fresh skill group's UI bindings).
-- Headless mode never paints them, so we return inert tables that
-- swallow any subsequent method call without raising. Keeps the calc
-- pipeline alive even when PoB tries to mutate UI state mid-frame.
local function inert_control()
    local t = {}
    setmetatable(t, {
        __index = function(self, _key)
            return function() end
        end,
        __newindex = function(self, _key, _val) end,
        __call = function(self, ...) return self end,
    })
    return t
end
function NewDropDown(...) return inert_control() end
function NewEditControl(...) return inert_control() end
function NewButtonControl(...) return inert_control() end
function NewSliderControl(...) return inert_control() end
function NewLabelControl(...) return inert_control() end

-- Pre-stub `lua-utf8` (PoB normally loads it as a C extension via lua-utf8.dll;
-- we don't bundle that — fall back to the standard `string` library which is
-- correct for ASCII and approximately right for PoB's number-formatting use).
package.preload["lua-utf8"] = function()
    local utf8stub = {}
    utf8stub.reverse = string.reverse
    utf8stub.gsub = string.gsub
    utf8stub.find = string.find
    utf8stub.sub = string.sub
    utf8stub.len = string.len
    utf8stub.upper = string.upper
    utf8stub.lower = string.lower
    utf8stub.match = string.match
    utf8stub.gmatch = string.gmatch
    utf8stub.format = string.format
    return utf8stub
end

-- The HeadlessWrapper expects to dofile("Launch.lua") relative to cwd.
-- It must be loaded AFTER package.path setup but BEFORE any build ops.
local headless_path = pob_root .. "/HeadlessWrapper.lua"
local headless, herr = loadfile(headless_path)
if not headless then
    io.stderr:write("api-stdio-bestel: cannot load HeadlessWrapper at "
        .. headless_path .. ": " .. tostring(herr) .. "\n")
    os.exit(1)
end

-- HeadlessWrapper does dofile("Launch.lua") — needs cwd at PoB src/.
-- We assume the parent process set cwd correctly via Command::current_dir().
local ok_init, init_err = pcall(headless)
if not ok_init then
    io.stderr:write("api-stdio-bestel: HeadlessWrapper init failed: "
        .. tostring(init_err) .. "\n")
    os.exit(1)
end

-- The wrapper exposes:
--   build = mainObject.main.modes["BUILD"]
--   loadBuildFromXML(xmlText, name)

local HARNESS_VERSION = "1.0.0-bestel"

-- Walk a Lua table, replacing cycles and unencodable types with markers
-- so dkjson never throws "reference cycle" mid-stream. Functions and
-- userdata become a string sentinel; tables seen on the current branch
-- are replaced with `"<cycle>"`.
local function sanitise(value, seen, depth)
    seen = seen or {}
    depth = depth or 0
    if depth > 16 then
        return "<deep>"
    end
    local t = type(value)
    if t == "table" then
        if seen[value] then
            return "<cycle>"
        end
        seen[value] = true
        local copy
        local is_array = #value > 0
        if is_array then
            copy = {}
            for i, v in ipairs(value) do
                copy[i] = sanitise(v, seen, depth + 1)
            end
        else
            copy = {}
            for k, v in pairs(value) do
                if type(k) == "string" or type(k) == "number" then
                    copy[k] = sanitise(v, seen, depth + 1)
                end
            end
        end
        seen[value] = nil
        return copy
    elseif t == "function" or t == "userdata" or t == "thread" then
        return "<" .. t .. ">"
    else
        return value
    end
end

local function reply(tbl)
    local safe_tbl = sanitise(tbl)
    local ok, s = pcall(dkjson.encode, safe_tbl)
    if not ok then
        s = dkjson.encode({ ok = false, error = "json encode failed: " .. tostring(s) })
    end
    io.write(s)
    io.write("\n")
    io.flush()
end

local function ok_reply(extra)
    local r = { ok = true }
    if extra then
        for k, v in pairs(extra) do r[k] = v end
    end
    reply(r)
end

local function err_reply(msg)
    reply({ ok = false, error = tostring(msg) })
end

local function safe(fn, ...)
    local ok, result = pcall(fn, ...)
    if not ok then
        return nil, tostring(result)
    end
    return result, nil
end

-- Translate a Bestel-flavoured `enemyIsBoss` payload value to the
-- PoB config string. Accepts:
--   bool true  -> "Pinnacle" (the common load-bearing case)
--   bool false -> "None"
--   string -> passed through ("None"/"Boss"/"Pinnacle"/"Uber")
local function normalise_enemy_is_boss(v)
    if type(v) == "boolean" then
        return v and "Pinnacle" or "None"
    elseif type(v) == "string" then
        return v
    end
    return nil
end

-- Map of supported set_config keys → coercion handlers. Anything not in
-- this map is rejected to surface unsupported config in the reply.
local CONFIG_KEYS = {
    bandit              = "string",
    pantheonMajorGod    = "string",
    pantheonMinorGod    = "string",
    enemyLevel          = "number",
    enemyIsBoss         = "enemy_is_boss",
    usePowerCharges     = "boolean",
    useFrenzyCharges    = "boolean",
    useEnduranceCharges = "boolean",
    forceBuffOnslaught  = "boolean",
    multiplierImpaleStacks = "number",
    useFlask1 = "flask:1",
    useFlask2 = "flask:2",
    useFlask3 = "flask:3",
    useFlask4 = "flask:4",
    useFlask5 = "flask:5",
}

local function set_one_config(key, value)
    local handler = CONFIG_KEYS[key]
    if not handler then
        return false, "unsupported config key: " .. key
    end
    if handler == "enemy_is_boss" then
        local v = normalise_enemy_is_boss(value)
        if not v then
            return false, "enemyIsBoss expects bool or string"
        end
        build.configTab.input[key] = v
    elseif handler == "boolean" then
        if type(value) ~= "boolean" then
            return false, key .. " expects boolean"
        end
        build.configTab.input[key] = value
    elseif handler == "number" then
        if type(value) ~= "number" then
            return false, key .. " expects number"
        end
        build.configTab.input[key] = value
    elseif handler == "string" then
        if type(value) ~= "string" then
            return false, key .. " expects string"
        end
        build.configTab.input[key] = value
    elseif handler:sub(1,6) == "flask:" then
        local idx = tonumber(handler:sub(7))
        -- Flask activation lives on the items tab in PoB. We toggle the
        -- flask in slot N if the build has one there. Best-effort.
        if type(value) ~= "boolean" then
            return false, key .. " expects boolean"
        end
        if build.itemsTab and build.itemsTab.activeItemSet and
           build.itemsTab.activeItemSet["Flask " .. idx] then
            local slot = build.itemsTab.activeItemSet["Flask " .. idx]
            slot.active = value
        end
    end
    return true, nil
end

local function recompute_build()
    if not build then return end
    build.buildFlag = true
    build.modFlag = true
    if build.OnFrame then
        build:OnFrame({})
    elseif runCallback then
        runCallback("OnFrame")
    end
end

-- Subset stat keys per category. The schema mirrors doc 25.
local CATEGORY_KEYS = {
    offence = {
        "TotalDPS","CombinedDPS","FullDPS","AverageHit","Speed","HitChance",
        "CritChance","CritMultiplier","IgniteDPS","BleedDPS","PoisonDPS","TotalDot",
    },
    defence = {
        "EHP","LifeUnreserved","LifeRecoverable","EnergyShield","Mana",
        "Armour","Evasion","MeleeEvadeChance","PhysicalDamageReduction",
        "MaxHitFire","MaxHitCold","MaxHitLightning","MaxHitPhysical","MaxHitChaos",
        "BlockChance","SpellBlockChance","SpellSuppressionChance",
    },
    charges = {
        "PowerChargesMax","FrenzyChargesMax","EnduranceChargesMax",
    },
    reservation = {
        "LifeReserved","ManaReserved","LifeReservedPercent",
        "ManaReservedPercent","SpiritReserved",
    },
    ailments = {
        "EnemyShockChance","ShockEffect","EnemyChillEffect",
        "EnemyFreezeChance","IgniteChance",
    },
}

local function copy_keys(src, keys)
    local out = {}
    if not src then return out end
    for _, k in ipairs(keys) do
        if src[k] ~= nil then out[k] = src[k] end
    end
    return out
end

local function get_stats(category)
    if not build or not build.calcsTab then
        return nil, "no build loaded"
    end
    local mainOutput = build.calcsTab.mainOutput or {}
    if category == "all" then
        return mainOutput, nil
    end
    local keys = CATEGORY_KEYS[category]
    if not keys then
        return nil, "unknown category: " .. tostring(category)
    end
    return copy_keys(mainOutput, keys), nil
end

local function snapshot_config()
    local out = {}
    if not build or not build.configTab then return out end
    for k, _ in pairs(CONFIG_KEYS) do
        out[k] = build.configTab.input[k]
    end
    return out
end

-- Action dispatch.
local actions = {}

actions.ping = function(_)
    ok_reply({ version = HARNESS_VERSION, game = game_tag })
end

actions.version = function(_)
    ok_reply({ version = HARNESS_VERSION, game = game_tag })
end

actions.quit = function(_)
    ok_reply({})
    os.exit(0)
end

actions.load_build_xml = function(req)
    local xml = req.xml
    if type(xml) ~= "string" or xml == "" then
        return err_reply("load_build_xml requires non-empty xml string")
    end
    local _, lerr = safe(loadBuildFromXML, xml, "bestel-build")
    if lerr then return err_reply(lerr) end
    -- The `build` global is set by HeadlessWrapper at boot to point at
    -- mainObject.main.modes["BUILD"]. loadBuildFromXML mutates that mode
    -- in place, so `build` keeps a valid reference. No re-bind needed.
    recompute_build()
    ok_reply({})
end

actions.set_config = function(req)
    if not build or not build.configTab then
        return err_reply("no build loaded")
    end
    local rejected = {}
    for k, v in pairs(req) do
        if k ~= "action" then
            local _, kerr = set_one_config(k, v)
            if kerr then table.insert(rejected, kerr) end
        end
    end
    if #rejected > 0 then
        return err_reply(table.concat(rejected, "; "))
    end
    recompute_build()
    ok_reply({})
end

actions.get_config = function(_)
    ok_reply({ config = snapshot_config() })
end

actions.set_main_selection = function(req)
    if not build then return err_reply("no build loaded") end
    -- Always assign when the caller provides a value. The upstream
    -- behaviour of guarding with `~= nil` swallowed our request when the
    -- target field hadn't been initialised yet (e.g. fresh build, no
    -- skill group selected) and we ended up computing stats for the
    -- WRONG skill group silently.
    if req.mainSocketGroup ~= nil then
        build.mainSocketGroup = req.mainSocketGroup
    end
    if req.mainActiveSkill ~= nil then
        build.mainActiveSkill = req.mainActiveSkill
    end
    if req.skillPart ~= nil then
        build.skillPart = req.skillPart
    end
    recompute_build()
    ok_reply({})
end

-- Resolve metadata about the currently active skill group so the agent
-- can sanity-check that the calc actually used the build's main skill
-- and not an off-by-one fallback (e.g. Frostblink at index 1 instead of
-- Penance Brand at index 2 — silent wrong-skill bug fixed 2026-05-08).
local function active_skill_meta()
    local meta = {
        main_socket_group = build and build.mainSocketGroup or nil,
        main_active_skill = build and build.mainActiveSkill or nil,
        skill_part = build and build.skillPart or nil,
        active_skill_label = nil,
        active_skill_gem = nil,
    }
    if build and build.mainSocketGroup and build.skillsTab and build.skillsTab.socketGroupList then
        local sg = build.skillsTab.socketGroupList[build.mainSocketGroup]
        if sg then
            meta.active_skill_label = sg.label or sg.displayLabel
            if sg.gemList then
                for _, gem in ipairs(sg.gemList) do
                    if gem.skillId and not (gem.gemData and gem.gemData.tags and gem.gemData.tags.support) then
                        meta.active_skill_gem = gem.nameSpec or gem.skillId
                        break
                    end
                end
            end
        end
    end
    return meta
end

actions.get_stats = function(req)
    local cat = req.category or "all"
    local stats, gerr = get_stats(cat)
    if gerr then return err_reply(gerr) end
    -- Surface the active-skill metadata in EVERY get_stats reply. The
    -- agent must verify that `active_skill_label` matches what the user
    -- expects from the PoB XML's main_skill before quoting any number.
    -- If `main_socket_group` is nil, treat the numbers as unreliable.
    ok_reply({
        stats = stats,
        active_skill = active_skill_meta(),
    })
end

-- Main loop.
while true do
    local line = io.read("*l")
    if not line then break end
    if #line > 0 then
        local req, _, jerr = dkjson.decode(line, 1, nil)
        if not req or type(req) ~= "table" then
            err_reply("malformed json: " .. tostring(jerr))
        else
            local action = req.action
            local handler = action and actions[action]
            if not handler then
                err_reply("unknown action: " .. tostring(action))
            else
                local _, derr = safe(handler, req)
                if derr then err_reply("dispatch error: " .. derr) end
            end
        end
    end
end
