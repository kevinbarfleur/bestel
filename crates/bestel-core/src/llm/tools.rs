use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Context, Result};
use bestel_rag::SharedKbEngine;
use serde_json::{json, Value};

use crate::pob::semantic::BuildIdentity;
use crate::pob::{PobBuild, PoeVersion};
use crate::sources::repoe::{self, Category as RepoeCategory, Game as RepoeGame};
use crate::sources::{FileCache, PoeHttpClient, TradeClient, WikiClient};

pub const GET_ACTIVE_BUILD: &str = "get_active_build";
pub const WIKI_SYNERGIES: &str = "wiki_synergies";
pub const WIKI_SEARCH: &str = "wiki_search";
pub const WIKI_PARSE: &str = "wiki_parse";
pub const WIKI_CARGO: &str = "wiki_cargo";
pub const TRADE_RESOLVE_STATS: &str = "trade_resolve_stats";
pub const TRADE_SEARCH_URL: &str = "trade_search_url";
pub const WEB_FETCH: &str = "web_fetch";
pub const READ_INTERNAL_REFERENCE: &str = "read_internal_reference";
pub const REPOE_LOOKUP: &str = "repoe_lookup";
pub const POB_CALC: &str = "pob_calc";
pub const KB_SEARCH: &str = "kb_search";
pub const LOAD_SKILL: &str = "load_skill";

/// Maximum `kb_search` calls per turn before the dispatch returns the
/// structural fallback `"tool_storm_kb_search"` (Sprint E task 8 cap).
pub const KB_SEARCH_TURN_CAP: u32 = 3;
/// Maximum `load_skill` calls per turn (Sprint F cap). Default = 1; the
/// model can ask for 2 only if the user explicitly requested a "full
/// audit" or similar multi-skill workflow — this cap guards against
/// skill-load thrash.
pub const LOAD_SKILL_TURN_CAP: u32 = 2;

/// Hosts that the `web_fetch` tool **rejects outright** before consulting the
/// allowlist. Mirrors the tier-4 SEO blocklist in
/// `prompts/references/15_source_registry.md`. Reason: SEO-optimised PoE blog
/// farms recycle patch announcements with no editorial integrity, frequently
/// contradict each other, and harm answer quality. Subdomains match too.
const FETCH_BLOCKLIST: &[&str] = &[
    "aoeah.com",
    "mmogah.com",
    "iggm.com",
    "ggwtb.com",
    "boostmatch.com",
    "sportskeeda.com",
    "gamewatcher.com",
    "switchbladegaming.com",
    "dotesports.com",
    "gamerant.com",
];

/// Hosts the `web_fetch` tool will accept. Mirrors the tier-1–7 allowlist
/// in `prompts/references/15_source_registry.md`. Subdomains of any entry are
/// also accepted (e.g. `forum.pathofexile.com` matches `pathofexile.com`).
const FETCH_ALLOWLIST: &[&str] = &[
    // Tier 1 — canonical
    "pathofexile.com",
    "pathofexile2.com",
    // Tier 2 — official wikis
    "poewiki.net",
    "poe2wiki.net",
    // Tier 3 — datamined
    "poedb.tw",
    "poe2db.tw",
    "repoe-fork.github.io",
    // Tier 4 — calculators
    "pathofbuilding.community",
    "craftofexile.com",
    // Tier 5 — economy
    "poe.ninja",
    // Tier 6 — filters
    "filterblade.xyz",
    // Tier 7 — trusted creator guides
    "maxroll.gg",
    "mobalytics.gg",
    "pohx.net",
    "poe-vault.com",
    "poeplanner.com",
    "pathofpathing.com",
    "poelab.com",
    "heartofphos.github.io",
    "poe.re",
    "exile.re",
];

/// Active build context shared across the watcher, the TUI, and any provider
/// that wants to expose the loaded PoB XML to the LLM via the
/// `get_active_build` tool. Replace-on-write — readers see the latest build.
#[derive(Clone, Default)]
pub struct BuildContext {
    inner: Arc<RwLock<Option<PobBuild>>>,
}

impl BuildContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, build: PobBuild) {
        if let Ok(mut g) = self.inner.write() {
            *g = Some(build);
        }
    }

    /// Detach the active build. After this, `get_active_build` reports
    /// no build loaded and the LLM should answer in generalist mode.
    pub fn clear(&self) {
        if let Ok(mut g) = self.inner.write() {
            *g = None;
        }
    }

    pub fn get(&self) -> Option<PobBuild> {
        self.inner.read().ok().and_then(|g| g.clone())
    }

    /// Render the active build as a JSON string suitable as a tool result.
    /// Includes class, ascendancy, level, key stats, defenses, charges,
    /// buffs, config, tree summary, items, and main skill — everything
    /// `get_active_build` advertises in its description.
    pub fn render_tool_result(&self) -> String {
        match self.get() {
            None => json!({
                "status": "no_build",
                "message": "No Path of Building build is currently loaded. Ask the exile to save a build in PoB so the chronicler can read it."
            })
            .to_string(),
            Some(b) => render_build_for_llm(&b),
        }
    }
}

/// Tool context hands every dispatched tool the things it needs:
/// the active build, wiki clients (one per game), the trade client (we
/// pick the right game per call), and a shared HTTP client for the
/// generic `web_fetch` tool. The HTTP client + cache are constructed
/// once and shared across clients to keep rate-limit state coherent and
/// the on-disk cache unique.
#[derive(Clone)]
pub struct ToolCtx {
    pub build: BuildContext,
    pub wiki_poe1: WikiClient,
    pub wiki_poe2: WikiClient,
    pub trade_poe1: TradeClient,
    pub trade_poe2: TradeClient,
    pub http: PoeHttpClient,
    /// Lazy headless PoB engine sidecar. Populated by the Tauri layer once
    /// `bundle.externalBin` resource paths are resolved. Left `None` for
    /// the MCP-serve entry point (no Tauri AppHandle, no resource dir);
    /// `pob_calc` then returns a clear "engine not configured" error.
    pub pob_engine: Option<Arc<bestel_pob_engine::PobEngineHandle>>,
    /// Hybrid embedded KB. `None` until the runtime finishes the boot-time
    /// indexing pass; until then `kb_search` returns a clear
    /// "kb_not_ready" error and the model falls back to
    /// `read_internal_reference`.
    pub kb: Option<SharedKbEngine>,
    /// Per-turn counter for `kb_search` invocations. Reset implicitly by
    /// constructing a new `ToolCtx` per LLM iteration (see
    /// `crates/bestel-core/src/llm/anthropic.rs`).
    pub kb_calls_this_turn: Arc<AtomicU32>,
    /// Per-turn counter for `load_skill` invocations (Sprint F cap).
    pub skill_calls_this_turn: Arc<AtomicU32>,
    /// Most recent user message text. Set by the provider before dispatch so
    /// `get_active_build` (Sprint E P7) can run a server-side `kb_search`
    /// against the same question and pre-attach `relevant_kb` passages.
    pub user_question: Option<String>,
}

impl ToolCtx {
    pub fn new(build: BuildContext) -> Result<Self> {
        let http = PoeHttpClient::new().context("init HTTP client")?;
        let cache = FileCache::new(FileCache::default_dir());
        let wiki_poe1 = WikiClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
        let wiki_poe2 = WikiClient::new(http.clone(), cache.clone(), PoeVersion::Poe2);
        let trade_poe1 = TradeClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
        let trade_poe2 = TradeClient::new(http.clone(), cache, PoeVersion::Poe2);
        let pob_engine = crate::llm::pob_engine::global();
        let kb = crate::llm::kb::global();
        Ok(Self {
            build,
            wiki_poe1,
            wiki_poe2,
            trade_poe1,
            trade_poe2,
            http,
            pob_engine,
            kb,
            kb_calls_this_turn: Arc::new(AtomicU32::new(0)),
            skill_calls_this_turn: Arc::new(AtomicU32::new(0)),
            user_question: None,
        })
    }

    /// Attach the most recent user message text. The provider should call
    /// this before dispatch so that `get_active_build` can echo a few
    /// `relevant_kb` passages alongside the build identity card.
    pub fn with_user_question(mut self, q: impl Into<String>) -> Self {
        self.user_question = Some(q.into());
        self
    }

    /// Attach a lazy PoB engine handle. Called from the Tauri bootstrap
    /// once vendored LuaJIT + harness paths are resolved via
    /// `app.path().resource_dir()`.
    pub fn with_pob_engine(
        mut self,
        engine: Arc<bestel_pob_engine::PobEngineHandle>,
    ) -> Self {
        self.pob_engine = Some(engine);
        self
    }

    /// Attach a [`SharedKbEngine`]. Called from the runtime bootstrap once
    /// the LanceDB index has been opened and the corpus indexed.
    pub fn with_kb(mut self, kb: SharedKbEngine) -> Self {
        self.kb = Some(kb);
        self
    }

    fn wiki(&self, game: &str) -> &WikiClient {
        match parse_game(game) {
            PoeVersion::Poe2 => &self.wiki_poe2,
            _ => &self.wiki_poe1,
        }
    }

    fn trade(&self, game: &str) -> &TradeClient {
        match parse_game(game) {
            PoeVersion::Poe2 => &self.trade_poe2,
            _ => &self.trade_poe1,
        }
    }
}

fn parse_game(s: &str) -> PoeVersion {
    match s.trim().to_ascii_lowercase().as_str() {
        "poe2" | "poe 2" | "pathofexile2" | "path of exile 2" => PoeVersion::Poe2,
        _ => PoeVersion::Poe1,
    }
}

/// Validate a URL against the trusted-source allowlist. Accepts the
/// host itself or any subdomain. Returns an `Err` describing the rejection
/// reason — the message is fed back to the LLM as a tool-result error so
/// it can retry with a valid host.
pub fn host_allowed(url: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url)
        .with_context(|| format!("parse url '{url}'"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(anyhow!(
            "Bestel only fetches http(s) URLs (got scheme '{}').",
            parsed.scheme()
        ));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("URL has no host"))?
        .to_ascii_lowercase();
    let blocked = FETCH_BLOCKLIST.iter().any(|b| {
        host == *b || host.ends_with(&format!(".{b}"))
    });
    if blocked {
        return Err(anyhow!(
            "Host '{host}' is on Bestel's tier-4 SEO blocklist (no editorial integrity, frequently wrong on PoE mechanics). Find the same fact via wiki / patch notes / official source instead."
        ));
    }
    let ok = FETCH_ALLOWLIST.iter().any(|allowed| {
        host == *allowed || host.ends_with(&format!(".{allowed}"))
    });
    if ok {
        Ok(())
    } else {
        Err(anyhow!(
            "Host '{host}' is not on Bestel's trusted source allowlist. Use one of: {}",
            FETCH_ALLOWLIST.join(", ")
        ))
    }
}

/// Strip HTML tags from a fetched body and collapse whitespace. Mirrors
/// the heuristic used by `WikiClient::parse` so plaintext output is
/// consistent across `wiki_parse` and `web_fetch`.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut prev_space = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            continue;
        }
        if ch == '>' {
            in_tag = false;
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }
        if in_tag {
            continue;
        }
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

pub fn tool_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": GET_ACTIVE_BUILD,
            "description": "Returns the exile's currently loaded Path of Building build: game (PoE1/PoE2), class, ascendancy, level, main skill, full skill groups with linked gems, every item with its full text, key defensive stats (life, mana, ES, EHP, armour, evasion, suppression, block, dodge), per-element resistances and max-hit values, charges (power/frenzy/endurance current+max), active buffs (combat/buff/curse lists), config (boss profile, enemy resists, flask uptimes, custom mods), and passive tree summary (class/ascend IDs, version, node and mastery counts, weapon-set node split). The response also includes SEMANTIC FACTS computed from the parsed build: `archetype` (defense/hit_model/mechanic tags — e.g. {defense:[\"life\",\"MoM\"], hit_model:[\"non-crit-EO\"], mechanic:[\"self-cast\"]}), `defining_uniques` (uniques present, each tagged engine|defining|amplifier with an identity hint), and `conversion_chain` (verbatim damage-conversion steps when applicable). Surface archetype tags FIRST when commenting on the build — do NOT guess the archetype from class+ascendancy alone. Never recommend selling an item flagged `category: \"engine\"` without explicit user instruction; engine items collapse the build if removed. Always call this BEFORE making any claim about the exile's character. No arguments.",
            "input_schema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": WIKI_SEARCH,
            "description": "Search the official PoE wiki (poewiki.net for PoE1, poe2wiki.net for PoE2) for a query string. Returns up to N hits with title, snippet, and canonical URL. Use this when you don't know the exact page title — e.g. 'spell suppression cap', 'mageblood drop', 'Divine Flesh keystone'. Follow up with `wiki_parse(title=...)` on the most relevant hit to read the full page text.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Free-text query."},
                    "game": {"type": "string", "enum": ["poe1", "poe2"], "default": "poe1"},
                    "limit": {"type": "integer", "default": 6, "minimum": 1, "maximum": 20}
                },
                "required": ["query"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": WIKI_PARSE,
            "description": "Fetch the full text of a specific PoE wiki page by exact title. Returns title, URL, section anchors, and stripped plain text. This is your primary research tool for mechanics — once you know the page name (from `wiki_search` or because the exile named the entity), call this to read the Mechanics / Caps / Interactions sections rather than relying on memory. Cached for 12 hours per page.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "Exact wiki page title. Spaces are fine; redirects are followed automatically. Examples: 'Divine Flesh', 'Spell suppression', 'Mageblood'."},
                    "game": {"type": "string", "enum": ["poe1", "poe2"], "default": "poe1"}
                },
                "required": ["title"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": WIKI_SYNERGIES,
            "description": "Query the wiki's reverse-link index (Special:WhatLinksHere) for a topic. Returns the list of pages that link TO the topic — uniques, passive skills, cluster jewel notables, ascendancy nodes, mechanics — even when the exile did not name them. Use AFTER understanding the core mechanic to surface synergies a creator-grade answer should mention (e.g. wiki_synergies(topic='Divine Flesh') reveals The Fourth Vow, Mahuxotl's Machination, Born of Chaos). Filtered to drop wiki meta pages.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "topic": {"type": "string", "description": "Canonical wiki page name. Spaces are accepted and are converted to underscores."},
                    "game": {"type": "string", "enum": ["poe1", "poe2"], "default": "poe1"},
                    "limit": {"type": "integer", "default": 80, "minimum": 1, "maximum": 200}
                },
                "required": ["topic"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": WIKI_CARGO,
            "description": "Run a Cargo (structured-table) query against the wiki. Niche but powerful when you need exact data: mod tier weights, item base stats, gem tags, version history. Tables include 'items', 'item_stats', 'mods', 'skill', 'versions'. Read the wiki page 'Special:CargoTables' for schemas before calling.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "table": {"type": "string", "description": "Table name, e.g. 'items', 'mods', 'skill'."},
                    "fields": {"type": "array", "items": {"type": "string"}, "description": "Field names to return."},
                    "where": {"type": "string", "description": "Optional SQL-like WHERE clause."},
                    "game": {"type": "string", "enum": ["poe1", "poe2"], "default": "poe1"},
                    "limit": {"type": "integer", "default": 50, "minimum": 1, "maximum": 500}
                },
                "required": ["table", "fields"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": TRADE_RESOLVE_STATS,
            "description": "Map a human stat phrase to its trade-stat ID(s). The trade site filters by IDs like `pseudo.pseudo_total_life` or `explicit.stat_3299347043`, not by free text. Call this before constructing any trade search — pass the phrase the exile used ('+life', 'increased fire damage', 'fractured chaos res') and pick the matching ID(s) from the result. Prefers `pseudo.*` for ambient phrases; cap-aware for prefixed types ('explicit', 'fractured', 'crafted', 'enchant').",
            "input_schema": {
                "type": "object",
                "properties": {
                    "phrase": {"type": "string"},
                    "game": {"type": "string", "enum": ["poe1", "poe2"], "default": "poe1"},
                    "prefer_pseudo": {"type": "boolean", "default": true},
                    "limit": {"type": "integer", "default": 6, "minimum": 1, "maximum": 20}
                },
                "required": ["phrase"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": TRADE_SEARCH_URL,
            "description": "Build a shareable trade-site URL from a query body and league. Bestel does NOT execute the search live (to avoid spamming GGG rate limits); the exile clicks the URL to see real listings. Construct the JSON body with `query.stats[].filters` referencing the IDs you got from `trade_resolve_stats`. League is the user's current league name (e.g. 'Mirage', 'Standard'). Returns the URL the exile can open.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "league": {"type": "string", "description": "Current league name."},
                    "query_body": {"type": "object", "description": "Full trade-API JSON body. Include 'query', 'sort'."},
                    "game": {"type": "string", "enum": ["poe1", "poe2"], "default": "poe1"}
                },
                "required": ["league", "query_body"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": WEB_FETCH,
            "description": "Fetch the plain-text content of a URL on Bestel's trusted-source allowlist (pathofexile.com, poewiki.net, poe2wiki.net, poedb.tw, poe2db.tw, pathofbuilding.community, craftofexile.com, poe.ninja, filterblade.xyz, maxroll.gg, mobalytics.gg, pohx.net, poe-vault.com, poeplanner.com, pathofpathing.com, poelab.com, heartofphos.github.io, poe.re, exile.re). Use for patch notes, PoEDB pages, Maxroll articles, etc. Strips HTML; truncates large bodies. Off-allowlist hosts (Fandom, Fextralife, RMT, SEO blogs) return an explicit error — retry with an allowlisted source.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "Full http(s) URL on the allowlist."}
                },
                "required": ["url"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": REPOE_LOOKUP,
            "description": "ALWAYS try this BEFORE `wiki_search` / `wiki_parse` / `wiki_cargo` / `web_fetch` when the question is about a base item, a unique, a mod pool, a gem, a cluster-jewel notable, an essence, a fossil, or a stat translation — bundled offline catalogue, < 10 ms cold, no network round-trip. Categories: `mods`, `base_items`, `gems`, `uniques`, `cluster_jewels`, `essences`, `fossils`, `stat_translations` (PoE1) — PoE2 only ships `mods`, `base_items`, `uniques` and returns an explicit 'not available for poe2' error for the rest. Provide either `id` (exact metadata path, e.g. 'Metadata/Items/Amulets/AmuletStellar') or `name` (fuzzy token-overlap, e.g. 'Marble Amulet'). Returns the matching JSON object(s) plus snapshot metadata (`source: bundled|refreshed`, `fetched_at`). If `repoe_lookup` returns 0 entries the entity may not exist under that name — fall back to `wiki_search` to disambiguate, then retry. Do NOT default to wiki for itemisation lookups; the wiki has rate limits and HTML noise that the bundle does not.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "game": {"type": "string", "enum": ["poe1", "poe2"]},
                    "category": {
                        "type": "string",
                        "enum": [
                            "mods", "base_items", "gems", "uniques",
                            "cluster_jewels", "essences", "fossils", "stat_translations"
                        ]
                    },
                    "id": {"type": "string", "description": "Exact metadata id for a single-entry lookup."},
                    "name": {"type": "string", "description": "Free-text name for fuzzy lookup."},
                    "limit": {"type": "integer", "default": 5, "minimum": 1, "maximum": 20}
                },
                "required": ["game", "category"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": POB_CALC,
            "description": "Run the bundled headless PoB engine against the active build and return canonical output stats. Categories: offence (DPS, hit chance, crit, ailment DPS), defence (EHP, max-hit by element, block, suppression), charges (max counts), reservation (life/mana/spirit %), ailments (shock/freeze/ignite chance), all (entire output table). Optional `skill_index` selects a non-default skill group. Optional `calcs` overrides the PoB Calcs config (enemyIsBoss, usePowerCharges, useFrenzyCharges, useEnduranceCharges, forceBuffOnslaught, multiplierImpaleStacks, useFlask1..5). The response ALWAYS echoes (1) the effective Calcs config and (2) `active_skill` metadata identifying which skill group the engine actually used. **CRITICAL VERIFICATION STEP**: before quoting any number, compare `active_skill.active_skill_label` (or `active_skill_gem`) with the build's `main_skill` from `get_active_build`. If they don't match, the engine fell back to the wrong skill — DO NOT quote the number; instead report the cached `<PlayerStat>` value with an explicit staleness note, OR retry pob_calc with an explicit `skill_index`. Surface the Calcs assumptions in your answer (`enemyIsBoss=Pinnacle`, `useFlask3=true`, etc.) — two PoBs with identical gear can show 10× DPS swings purely from Calcs config; never quote a number without naming the assumptions. The response also includes a `warnings` array; if non-empty, surface those warnings to the user.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "enum": ["offence", "defence", "charges", "reservation", "ailments", "all"]
                    },
                    "skill_index": {"type": "integer", "minimum": 0},
                    "calcs": {
                        "type": "object",
                        "properties": {
                            "enemyIsBoss": {
                                "description": "true → 'Pinnacle' boss profile; false → 'None'. Pass a string for finer control: 'None' / 'Boss' / 'Pinnacle' / 'Uber'."
                            },
                            "usePowerCharges": {"type": "boolean"},
                            "useFrenzyCharges": {"type": "boolean"},
                            "useEnduranceCharges": {"type": "boolean"},
                            "forceBuffOnslaught": {"type": "boolean"},
                            "multiplierImpaleStacks": {"type": "integer"},
                            "useFlask1": {"type": "boolean"},
                            "useFlask2": {"type": "boolean"},
                            "useFlask3": {"type": "boolean"},
                            "useFlask4": {"type": "boolean"},
                            "useFlask5": {"type": "boolean"}
                        },
                        "additionalProperties": false
                    }
                },
                "required": ["category"],
                "additionalProperties": false
            }
        }),
        {
            // Build the rel_path `enum` from the bundled references list
            // so the schema ALWAYS matches what `read_reference()` actually
            // accepts. Hard-coding a list drifts; reading from
            // `crate::prompts::BUNDLED_REFERENCES` keeps them in lockstep.
            // Models that respect JSON Schema `enum` (Anthropic, DeepSeek
            // via Anthropic-compat) cannot invent a filename — the API
            // layer rejects unknown values upfront.
            let valid_paths: Vec<String> = crate::prompts::BUNDLED_REFERENCES
                .iter()
                .map(|(p, _)| (*p).to_string())
                .collect();
            json!({
                "name": READ_INTERNAL_REFERENCE,
                "description": "Read one of Bestel's internal reference files from `~/.bestel/prompts/references/`. **Use ONLY when you already know the exact filename** (e.g. you saw it cited in a prior turn or in the reference index). For semantic queries (`how do Pantheon interact with duration buffs?`, `what counts as life-on-block?`), prefer `kb_search` — it returns the relevant passages directly without you having to guess the right file. The `rel_path` MUST be one of the values in the `enum` list — never invent a filename, never re-number, never pluralise. Common mistake: 'build_archetypes' (plural) instead of 'build_archetype' (singular, file 17). Returns the file's full markdown text (truncated at 25 KB). Path traversal and absolute paths are rejected.",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "rel_path": {
                            "type": "string",
                            "enum": valid_paths,
                            "description": "MUST match one of the enum values. The list above is the complete reference library."
                        }
                    },
                    "required": ["rel_path"],
                    "additionalProperties": false
                }
            })
        },
        {
            let skill_names: Vec<String> = crate::skills::bundled_skill_names();
            json!({
                "name": LOAD_SKILL,
                "description": "Load the full body of one of Bestel's progressive-disclosure skills — repeatable workflows (build review, craft audit, mapping strategy) whose checklists are too verbose to keep in the system prompt. The system prompt advertises each skill's `description` + triggers (~100 words each). Call this when the user's question matches a skill's triggers and you need the full diagnostic flow / template / checklist. Returns the SKILL.md body plus the names of templates available under that skill. Cap: 1 skill per turn by default, 2 only when the user asked for a full audit. Skills are NOT a substitute for tool calls — pob_calc / wiki_parse / kb_search still run for live data; the skill provides STRUCTURE for the answer.",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "enum": skill_names,
                            "description": "Skill folder name. MUST match one of the enum values."
                        }
                    },
                    "required": ["name"],
                    "additionalProperties": false
                }
            })
        },
        json!({
            "name": KB_SEARCH,
            "description": "Hybrid (vector + BM25) search over Bestel's internal knowledge base — the same `~/.bestel/prompts/references/**` corpus exposed by `read_internal_reference`, but indexed semantically. Use for any question where you don't already know the exact source file: `how do Pantheon interact with duration buffs`, `what's the spell suppression cap`, `what defines a slow-projectile build`, etc. Returns up to `top_k` passages with their parent file, heading path, and a short body excerpt. Each passage cites the exact `~/.bestel/prompts/references/<rel_path>` it came from so you can chain into `read_internal_reference` if you need the full file. Cap: 3 calls per turn (after that, dispatch returns `tool_storm_kb_search` and you must answer with what you have or fall back to `wiki_search`). The corpus is limited to Bestel's internal references — for live game data (skills, uniques, bases) use `wiki_parse` / `repoe_lookup` instead.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural-language query. Phrase it close to the user's intent — embeddings handle paraphrase well."
                    },
                    "game": {
                        "type": "string",
                        "enum": ["poe1", "poe2"],
                        "description": "Filter to passages applicable to one game. Omit to search both."
                    },
                    "top_k": {
                        "type": "integer",
                        "default": 5,
                        "minimum": 1,
                        "maximum": 10,
                        "description": "Number of passages to return."
                    },
                    "tags": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Optional metadata tag filter (AND-joined). Currently unused for the bundled refs corpus; reserved for future skill / wiki-cache shards."
                    }
                },
                "required": ["query"],
                "additionalProperties": false
            }
        }),
    ]
}

pub async fn dispatch(name: &str, input: &Value, ctx: &ToolCtx) -> Result<String> {
    match name {
        GET_ACTIVE_BUILD => dispatch_get_active_build(ctx).await,
        // Backwards-compat alias for the renamed tool. Some external MCP
        // clients may still call the old name; route them to the new
        // implementation transparently.
        WIKI_SYNERGIES | "find_synergies" => {
            let topic = input
                .get("topic")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'topic' is required and must be a string"))?;
            let game = input
                .get("game")
                .and_then(|v| v.as_str())
                .unwrap_or("poe1");
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n.clamp(1, 200) as usize)
                .unwrap_or(80);
            crate::llm::wiki::find_synergies(topic, game, limit).await
        }
        WIKI_SEARCH => {
            let query = input
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'query' is required and must be a string"))?;
            let game = input.get("game").and_then(|v| v.as_str()).unwrap_or("poe1");
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n.clamp(1, 20) as u32)
                .unwrap_or(6);
            let hits = ctx.wiki(game).search(query, limit).await?;
            let value = serde_json::to_value(&hits).context("serialize wiki hits")?;
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        WIKI_PARSE => {
            let title = input
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'title' is required and must be a string"))?;
            let game = input.get("game").and_then(|v| v.as_str()).unwrap_or("poe1");
            let mut page = ctx.wiki(game).parse(title).await?;
            page.plain_text = truncate(&page.plain_text, 25_000);
            let value = serde_json::to_value(&page).context("serialize wiki page")?;
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        WIKI_CARGO => {
            let table = input
                .get("table")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'table' is required"))?;
            let fields_json = input
                .get("fields")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow!("'fields' must be an array of strings"))?;
            let fields: Vec<&str> = fields_json
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            if fields.is_empty() {
                return Err(anyhow!("'fields' must contain at least one string"));
            }
            let where_clause = input.get("where").and_then(|v| v.as_str());
            let game = input.get("game").and_then(|v| v.as_str()).unwrap_or("poe1");
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n.clamp(1, 500) as u32)
                .unwrap_or(50);
            let rows = ctx
                .wiki(game)
                .cargo(table, &fields, where_clause, limit)
                .await?;
            let value = json!({"rows": rows, "count": rows.len()});
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        TRADE_RESOLVE_STATS => {
            let phrase = input
                .get("phrase")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'phrase' is required"))?;
            let game = input.get("game").and_then(|v| v.as_str()).unwrap_or("poe1");
            let prefer_pseudo = input
                .get("prefer_pseudo")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n.clamp(1, 20) as usize)
                .unwrap_or(6);
            let hits = ctx
                .trade(game)
                .resolve(phrase, prefer_pseudo, limit)
                .await?;
            let value = serde_json::to_value(&hits).context("serialize stat refs")?;
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 8_000))
        }
        TRADE_SEARCH_URL => {
            let league = input
                .get("league")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'league' is required"))?;
            let query_body = input
                .get("query_body")
                .cloned()
                .ok_or_else(|| anyhow!("'query_body' is required"))?;
            let game = input.get("game").and_then(|v| v.as_str()).unwrap_or("poe1");
            let resp = ctx.trade(game).search(league, query_body).await?;
            let value = serde_json::to_value(&resp).context("serialize trade resp")?;
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 8_000))
        }
        WEB_FETCH => {
            let url = input
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'url' is required"))?;
            host_allowed(url)?;
            let body = ctx.http.get_text(url, "web_fetch").await?;
            let plaintext = strip_html(&body);
            let value = json!({
                "url": url,
                "content": truncate(&plaintext, 25_000),
            });
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        REPOE_LOOKUP => {
            let game = input
                .get("game")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'game' is required ('poe1' or 'poe2')"))?;
            let game = RepoeGame::parse(game)
                .ok_or_else(|| anyhow!("'game' must be 'poe1' or 'poe2', got '{game}'"))?;
            let category = input
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'category' is required"))?;
            let category = RepoeCategory::parse(category).ok_or_else(|| {
                anyhow!(
                    "'category' must be one of mods/base_items/gems/uniques/cluster_jewels/essences/fossils/stat_translations, got '{category}'"
                )
            })?;
            let id = input.get("id").and_then(|v| v.as_str());
            let name = input.get("name").and_then(|v| v.as_str());
            if id.is_some() && name.is_some() {
                return Err(anyhow!("provide either 'id' or 'name', not both"));
            }
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n.clamp(1, 20) as usize)
                .unwrap_or(5);
            let client = repoe::global();
            let result = if let Some(id) = id {
                client.lookup_by_id(game, category, id)?
            } else if let Some(name) = name {
                client.lookup_by_name(game, category, name, limit)?
            } else {
                return Err(anyhow!("provide either 'id' or 'name'"));
            };
            let value = serde_json::to_value(&result).context("serialize repoe lookup")?;
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        READ_INTERNAL_REFERENCE => {
            let rel = input
                .get("rel_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'rel_path' is required and must be a string"))?;
            let content = crate::prompts::read_reference(rel)?;
            let value = json!({
                "rel_path": rel,
                "content": truncate(&content, 25_000),
            });
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        POB_CALC => dispatch_pob_calc(input, ctx).await,
        KB_SEARCH => dispatch_kb_search(input, ctx).await,
        LOAD_SKILL => dispatch_load_skill(input, ctx).await,
        other => Err(anyhow!("unknown tool '{other}'")),
    }
}

async fn dispatch_load_skill(input: &Value, ctx: &ToolCtx) -> Result<String> {
    let prev = ctx.skill_calls_this_turn.fetch_add(1, Ordering::SeqCst);
    if prev >= LOAD_SKILL_TURN_CAP {
        return Ok(json!({
            "status": "tool_storm_load_skill",
            "message": format!(
                "load_skill has been called {} times this turn (cap = {}). Stop loading more workflows and answer with what you have.",
                prev + 1,
                LOAD_SKILL_TURN_CAP
            ),
            "cap": LOAD_SKILL_TURN_CAP,
        })
        .to_string());
    }
    let name = input
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'name' is required and must be a string"))?;
    let skill = crate::skills::load_skill(name)?;
    let value = json!({
        "status": "ok",
        "name": skill.frontmatter.name,
        "description": skill.frontmatter.description,
        "when_to_use": skill.frontmatter.when_to_use,
        "trigger_examples": skill.frontmatter.trigger_examples,
        "body": skill.body,
        "templates": skill.templates.keys().cloned().collect::<Vec<String>>(),
    });
    Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
}

/// Sprint E P7 / Sprint G P4 — wraps the Sprint D identity-card render with:
/// (a) a server-side `kb_search` over the user's most recent question
/// (`summary.relevant_kb`), and (b) the cached `current_diagnosis` from
/// `~/.bestel/notes/<build_session>.json` (`summary.session_notes`) so a
/// multi-turn diagnosis stays coherent across pivots. Falls back gracefully
/// when KB / notes / question are absent.
async fn dispatch_get_active_build(ctx: &ToolCtx) -> Result<String> {
    let base = ctx.build.render_tool_result();
    let active_build = ctx.build.get();
    let mut value: Value = match serde_json::from_str(&base) {
        Ok(v) => v,
        Err(_) => return Ok(base),
    };
    let summary_obj = match value.as_object_mut() {
        Some(o) if o.contains_key("summary") => {
            o.get_mut("summary").and_then(|v| v.as_object_mut())
        }
        Some(o) => Some(o),
        None => None,
    };
    let summary = match summary_obj {
        Some(s) => s,
        None => return Ok(base),
    };

    // (a) Sprint E P7 — relevant_kb passages.
    if let (Some(kb), Some(question)) = (
        ctx.kb.as_ref(),
        ctx.user_question
            .as_deref()
            .filter(|s| !s.trim().is_empty()),
    ) {
        match kb.search(question, 5, None, &[]).await {
            Ok(hits) => {
                let kb_passages: Vec<Value> = hits
                    .into_iter()
                    .map(|h| {
                        json!({
                            "rel_path": h.chunk.doc_path,
                            "heading_path": h.chunk.heading_path,
                            "score": h.score,
                            "excerpt": truncate(&h.chunk.body, 800),
                        })
                    })
                    .collect();
                summary.insert("relevant_kb".into(), Value::Array(kb_passages));
            }
            Err(err) => {
                tracing::warn!(error = ?err, "kb.search inside get_active_build failed");
            }
        }
    }

    // (b) Sprint G P4 — session_notes for multi-turn diagnosis carry-over.
    if let Some(build) = active_build {
        let session_id = derive_build_session_id(&build);
        let notes = crate::llm::session_notes::read_notes(&session_id);
        if notes.current_diagnosis.is_some() {
            if let Ok(notes_json) = serde_json::to_value(&notes) {
                summary.insert("session_notes".into(), notes_json);
            }
        }
    }

    Ok(truncate(&serde_json::to_string(&value).unwrap_or(base.clone()), 30_000))
}

/// Stable session identifier per loaded build. Currently a Blake3 of the
/// source file path so re-opening the same PoB resumes the diagnosis.
fn derive_build_session_id(build: &PobBuild) -> String {
    let path = build.source_file.display().to_string();
    let hash = blake3::hash(path.as_bytes());
    let hex = hash.to_hex();
    // Take a 16-char prefix — collision-safe for per-user notes folders.
    hex[..16.min(hex.len())].to_string()
}

async fn dispatch_kb_search(input: &Value, ctx: &ToolCtx) -> Result<String> {
    let prev = ctx.kb_calls_this_turn.fetch_add(1, Ordering::SeqCst);
    if prev >= KB_SEARCH_TURN_CAP {
        return Ok(json!({
            "status": "tool_storm_kb_search",
            "message": format!(
                "kb_search has been called {} times this turn (cap = {}). Stop retrieving and answer with what you have, or fall back to wiki_search if the question requires live game data.",
                prev + 1,
                KB_SEARCH_TURN_CAP
            ),
            "cap": KB_SEARCH_TURN_CAP,
        })
        .to_string());
    }
    let kb = match ctx.kb.as_ref() {
        Some(k) => k.clone(),
        None => {
            return Ok(json!({
                "status": "kb_not_ready",
                "message": "Knowledge base index has not finished bootstrapping. Use read_internal_reference if you know the file, or wiki_parse for live game mechanics.",
            })
            .to_string());
        }
    };
    let query = input
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'query' is required and must be a string"))?;
    let game = input
        .get("game")
        .and_then(|v| v.as_str());
    let top_k = input
        .get("top_k")
        .and_then(|v| v.as_u64())
        .map(|n| n.clamp(1, 10) as usize)
        .unwrap_or(5);
    let tags: Vec<String> = input
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let hits = kb.search(query, top_k, game, &tags).await?;
    let response_hits: Vec<Value> = hits
        .into_iter()
        .map(|h| {
            json!({
                "rel_path": h.chunk.doc_path,
                "heading_path": h.chunk.heading_path,
                "applies_to": h.chunk.applies_to,
                "score": h.score,
                "excerpt": truncate(&h.chunk.body, 1200),
            })
        })
        .collect();
    let value = json!({
        "status": "ok",
        "query": query,
        "game": game,
        "top_k": top_k,
        "hits": response_hits,
    });
    Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
}

async fn dispatch_pob_calc(input: &Value, ctx: &ToolCtx) -> Result<String> {
    use bestel_pob_engine::{CalcRequest, Category as PobCategory, EngineCalcs};
    use bestel_pob_engine::lifecycle::Game as PobGame;

    let engine = ctx
        .pob_engine
        .as_ref()
        .ok_or_else(|| anyhow!(
            "PoB engine not configured for this entry point. The headless calculator is wired in the desktop app; run Bestel via the Tauri window (not `bestel mcp-serve`) to use pob_calc."
        ))?;

    let build = ctx.build.get().ok_or_else(|| anyhow!(
        "No active Path of Building build. Ask the exile to save a build in PoB so the chronicler can read it before calling pob_calc."
    ))?;

    let category_str = input
        .get("category")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'category' is required"))?;
    let category = PobCategory::parse(category_str).ok_or_else(|| {
        anyhow!(
            "'category' must be one of offence/defence/charges/reservation/ailments/all, got '{category_str}'"
        )
    })?;

    let skill_index = input
        .get("skill_index")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32);

    let mut calcs = EngineCalcs::default();
    if let Some(c) = input.get("calcs").and_then(|v| v.as_object()) {
        if let Some(v) = c.get("enemyIsBoss") {
            // Accept bool or string; the harness normalises both.
            if let Some(b) = v.as_bool() {
                calcs.enemy_is_boss = Some(b);
            } else if let Some(s) = v.as_str() {
                // For string, encode the boolean equivalent: anything other
                // than "None" maps to true (interesting case).
                calcs.enemy_is_boss = Some(!s.eq_ignore_ascii_case("None"));
            }
        }
        if let Some(b) = c.get("usePowerCharges").and_then(|v| v.as_bool()) {
            calcs.use_power_charges = Some(b);
        }
        if let Some(b) = c.get("useFrenzyCharges").and_then(|v| v.as_bool()) {
            calcs.use_frenzy_charges = Some(b);
        }
        if let Some(b) = c.get("useEnduranceCharges").and_then(|v| v.as_bool()) {
            calcs.use_endurance_charges = Some(b);
        }
        if let Some(b) = c.get("forceBuffOnslaught").and_then(|v| v.as_bool()) {
            calcs.force_buff_onslaught = Some(b);
        }
        if let Some(n) = c.get("multiplierImpaleStacks").and_then(|v| v.as_i64()) {
            calcs.multiplier_impale_stacks = Some(n as i32);
        }
        for (i, key) in [
            "useFlask1", "useFlask2", "useFlask3", "useFlask4", "useFlask5",
        ]
        .iter()
        .enumerate()
        {
            if let Some(b) = c.get(*key).and_then(|v| v.as_bool()) {
                match i {
                    0 => calcs.use_flask1 = Some(b),
                    1 => calcs.use_flask2 = Some(b),
                    2 => calcs.use_flask3 = Some(b),
                    3 => calcs.use_flask4 = Some(b),
                    4 => calcs.use_flask5 = Some(b),
                    _ => unreachable!(),
                }
            }
        }
    }

    let xml = tokio::fs::read_to_string(&build.source_file)
        .await
        .with_context(|| format!(
            "read PoB XML at {}",
            build.source_file.display()
        ))?;

    let game = match build.game {
        PoeVersion::Poe2 => PobGame::Poe2,
        _ => PobGame::Poe1,
    };

    let response = engine
        .calc(CalcRequest {
            game,
            build_xml: xml,
            category,
            skill_index,
            calcs,
        })
        .await
        .map_err(|e| anyhow!("pob engine: {e}"))?;

    let payload = serde_json::to_value(&response).context("serialize pob_calc response")?;
    Ok(truncate(&serde_json::to_string(&payload).unwrap_or_default(), 30_000))
}

fn render_build_for_llm(b: &PobBuild) -> String {
    let mut summary = serde_json::Map::new();
    summary.insert("game".into(), json!(b.game.label()));
    summary.insert("class".into(), json!(b.class));
    summary.insert("ascendancy".into(), json!(b.ascendancy));
    summary.insert("level".into(), json!(b.level));
    summary.insert("target_version".into(), json!(b.target_version));
    summary.insert("notes".into(), json!(b.notes));
    if !b.notes_sections.is_empty() {
        summary.insert(
            "notes_sections".into(),
            json!(b
                .notes_sections
                .iter()
                .map(|s| json!({"heading": s.heading, "body": truncate(&s.body, 1500)}))
                .collect::<Vec<_>>()),
        );
    }
    summary.insert("main_skill".into(), json!(b.main_skill));
    summary.insert(
        "source_file".into(),
        json!(b.source_file.display().to_string()),
    );

    if let Some(url) = &b.passive_tree_url {
        summary.insert("passive_tree_url".into(), json!(url));
    }

    // Defenses lifted from the flat stats map.
    summary.insert("defenses".into(), json!(b.defenses));
    summary.insert("charges".into(), json!(b.charges));
    summary.insert("buffs".into(), json!(b.buffs));
    if !b.config.is_empty() {
        summary.insert("config".into(), json!(b.config));
    }
    summary.insert("tree".into(), json!(b.tree));

    // Headline DPS numbers — convenient for the LLM to quote without grepping
    // the stats map.
    let mut headline = serde_json::Map::new();
    if let Some(v) = b.dps() {
        headline.insert("combined_dps".into(), json!(v));
    }
    if let Some(v) = b.life() {
        headline.insert("life".into(), json!(v));
    }
    if let Some(v) = b.mana() {
        headline.insert("mana".into(), json!(v));
    }
    if let Some(v) = b.energy_shield() {
        headline.insert("energy_shield".into(), json!(v));
    }
    if let Some(v) = b.ehp() {
        headline.insert("total_ehp".into(), json!(v));
    }
    if !headline.is_empty() {
        summary.insert("headline".into(), serde_json::Value::Object(headline));
    }

    // Selected high-value stats verbatim. Anything not here is in `stats`.
    let key_stats = [
        "Life",
        "Mana",
        "EnergyShield",
        "Spirit",
        "TotalEHP",
        "CombinedDPS",
        "TotalDPS",
        "FullDPS",
        "FireResist",
        "ColdResist",
        "LightningResist",
        "ChaosResist",
        "Armour",
        "Evasion",
        "PhysicalDamageReduction",
        "EffectiveSpellSuppressionChance",
        "EffectiveBlockChance",
        "AttackDodgeChance",
        "SpellDodgeChance",
        "Str",
        "Dex",
        "Int",
    ];
    let mut stats = serde_json::Map::new();
    for k in key_stats {
        if let Some(v) = b.stats.get(k) {
            stats.insert(k.to_string(), json!(v));
        }
    }
    summary.insert("stats".into(), serde_json::Value::Object(stats));

    let skill_groups: Vec<_> = b
        .skill_groups
        .iter()
        .map(|g| {
            json!({
                "label": g.label,
                "is_main": g.is_main,
                "gems": g.gems.iter().map(|gem| json!({
                    "name": gem.name,
                    "level": gem.level,
                    "quality": gem.quality,
                    "enabled": gem.enabled,
                    "skill_id": gem.skill_id,
                    "is_minion": gem.is_minion,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    summary.insert("skill_groups".into(), json!(skill_groups));

    let items: Vec<_> = b
        .items
        .iter()
        .map(|it| {
            json!({
                "id": it.id,
                "rarity": it.rarity,
                "name": it.name,
                "base": it.base,
                "raw": truncate(&it.raw, 1500),
            })
        })
        .collect();
    summary.insert("items".into(), json!(items));

    // Sprint 3 — semantic build identity. Computed on every render
    // (cheap, ~1-5 ms). Surfaces archetype tags, defining uniques, and
    // the conversion chain so the agent quotes them instead of guessing
    // from class+ascendancy.
    let identity = BuildIdentity::from_build(b);
    summary.insert("archetype".into(), json!(identity.archetype));
    if !identity.defining_uniques.is_empty() {
        summary.insert("defining_uniques".into(), json!(identity.defining_uniques));
    }
    if let Some(chain) = identity.conversion_chain.as_ref() {
        summary.insert("conversion_chain".into(), json!(chain));
    }
    summary.insert(
        "identity_line".into(),
        json!(format_identity_line(&identity, identity.conversion_chain.as_ref())),
    );

    let value = serde_json::Value::Object(summary);
    let s = serde_json::to_string(&value).unwrap_or_default();
    truncate(&s, 60_000)
}

/// Compose the canonical `Identity:` card line that the agent must echo
/// verbatim when answering build-specific questions. Format:
///
/// `Identity: defense=<axis>, hit_model=<axis>, mechanic=<axis>. Defining
///  uniques: <name> (<category>), …. Conversion: <step>, …`
///
/// Multi-tag axes are joined with `+` (e.g. `defense=life+MoM`). Empty
/// tag lists render as `none`. Defining uniques and conversion clauses
/// are omitted entirely when absent.
fn format_identity_line(
    id: &BuildIdentity,
    chain: Option<&crate::pob::semantic::ConversionChain>,
) -> String {
    let join_axis = |axis: &[String]| -> String {
        if axis.is_empty() {
            "none".to_string()
        } else {
            axis.join("+")
        }
    };
    let mut line = format!(
        "Identity: defense={}, hit_model={}, mechanic={}.",
        join_axis(&id.archetype.defense),
        join_axis(&id.archetype.hit_model),
        join_axis(&id.archetype.mechanic),
    );
    if !id.defining_uniques.is_empty() {
        let uniques = id
            .defining_uniques
            .iter()
            .map(|u| format!("{} ({})", u.name, u.category))
            .collect::<Vec<_>>()
            .join(", ");
        line.push_str(&format!(" Defining uniques: {}.", uniques));
    }
    if let Some(c) = chain {
        if !c.steps.is_empty() {
            line.push_str(&format!(" Conversion: {}.", c.steps.join(", ")));
        }
    }
    line
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…[truncated]", &s[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pob::semantic::{
        ArchetypeTags, ConversionChain, DefiningUniqueMatch,
    };

    #[test]
    fn identity_line_full_grammar() {
        let id = BuildIdentity {
            archetype: ArchetypeTags {
                defense: vec!["life".into(), "MoM".into()],
                hit_model: vec!["crit".into()],
                mechanic: vec!["self-cast".into()],
            },
            defining_uniques: vec![
                DefiningUniqueMatch {
                    name: "Mageblood".into(),
                    category: "engine".into(),
                    identity_hint: "irrelevant".into(),
                },
                DefiningUniqueMatch {
                    name: "Watcher's Eye".into(),
                    category: "amplifier".into(),
                    identity_hint: "irrelevant".into(),
                },
            ],
            conversion_chain: Some(ConversionChain {
                steps: vec!["60% physical → cold".into()],
                final_type: "cold".into(),
            }),
        };
        let line = format_identity_line(&id, id.conversion_chain.as_ref());
        assert_eq!(
            line,
            "Identity: defense=life+MoM, hit_model=crit, mechanic=self-cast. \
             Defining uniques: Mageblood (engine), Watcher's Eye (amplifier). \
             Conversion: 60% physical → cold."
        );
    }

    #[test]
    fn identity_line_minimal_no_uniques_no_chain() {
        let id = BuildIdentity {
            archetype: ArchetypeTags {
                defense: vec!["CI".into()],
                hit_model: vec!["non-crit-EO".into()],
                mechanic: vec!["totem".into()],
            },
            defining_uniques: Vec::new(),
            conversion_chain: None,
        };
        let line = format_identity_line(&id, None);
        assert_eq!(
            line,
            "Identity: defense=CI, hit_model=non-crit-EO, mechanic=totem."
        );
    }

    #[test]
    fn identity_line_empty_axis_renders_none() {
        let id = BuildIdentity::default();
        let line = format_identity_line(&id, None);
        assert_eq!(
            line,
            "Identity: defense=none, hit_model=none, mechanic=none."
        );
    }

    #[test]
    fn schemas_advertise_full_in_app_surface() {
        let schemas = tool_schemas();
        let names: Vec<&str> = schemas
            .iter()
            .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&GET_ACTIVE_BUILD));
        assert!(names.contains(&WIKI_SYNERGIES));
        assert!(names.contains(&WIKI_SEARCH));
        assert!(names.contains(&WIKI_PARSE));
        assert!(names.contains(&WIKI_CARGO));
        assert!(names.contains(&TRADE_RESOLVE_STATS));
        assert!(names.contains(&TRADE_SEARCH_URL));
        assert!(names.contains(&WEB_FETCH));
        assert!(names.contains(&READ_INTERNAL_REFERENCE));
        assert!(names.contains(&REPOE_LOOKUP));
        assert!(names.contains(&POB_CALC));
        assert!(names.contains(&KB_SEARCH));
        assert!(names.contains(&LOAD_SKILL));
        assert_eq!(schemas.len(), 13);
    }

    #[test]
    fn no_build_returns_no_build_status() {
        let ctx = BuildContext::new();
        let s = ctx.render_tool_result();
        assert!(s.contains("\"status\":\"no_build\""));
    }

    #[test]
    fn host_allowed_accepts_poewiki() {
        assert!(host_allowed("https://www.poewiki.net/wiki/Divine_Flesh").is_ok());
        assert!(host_allowed("https://poewiki.net/wiki/Resistance").is_ok());
        assert!(host_allowed("https://www.poe2wiki.net/wiki/Spirit").is_ok());
        assert!(host_allowed("https://forum.pathofexile.com/forum/view-thread/x").is_ok());
        assert!(host_allowed("https://maxroll.gg/poe/getting-started").is_ok());
    }

    #[test]
    fn host_allowed_rejects_blocked_sources() {
        let err = host_allowed("https://pathofexile.fandom.com/wiki/Divine_Flesh").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not on Bestel's trusted source allowlist"));
        assert!(msg.contains("poewiki.net"));
        assert!(host_allowed("https://poe.fextralife.com/anything").is_err());
        assert!(host_allowed("https://random-seo-blog.example.com/build").is_err());
    }

    #[test]
    fn host_allowed_rejects_tier4_seo_blocklist() {
        let err = host_allowed("https://www.aoeah.com/poe-build-guide").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("tier-4 SEO blocklist"), "got: {msg}");
        assert!(host_allowed("https://mmogah.com/anything").is_err());
        assert!(host_allowed("https://news.dotesports.com/poe").is_err());
        assert!(host_allowed("https://www.sportskeeda.com/poe").is_err());
    }

    #[test]
    fn host_allowed_rejects_non_http_schemes() {
        assert!(host_allowed("file:///etc/passwd").is_err());
        assert!(host_allowed("ftp://poewiki.net/x").is_err());
    }

    #[test]
    fn strip_html_collapses_whitespace_and_drops_tags() {
        let html = "<p>Hello   <b>World</b></p>\n<div>foo</div>";
        let plain = strip_html(html);
        assert_eq!(plain, "Hello World foo");
    }
}
