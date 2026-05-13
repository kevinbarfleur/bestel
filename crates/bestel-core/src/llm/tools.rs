use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Context, Result};
use bestel_rag::SharedKbEngine;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::llm::sheet_tools::{
    self, GET_ACTIVE_BUILD_SHEET, SHEET_ASK, SHEET_FINALIZE_REQUEST, SHEET_OPEN_INTERVIEW,
    SHEET_PROPOSE_SECTION,
};
use crate::llm::LlmDelta;
use crate::pob::semantic::BuildIdentity;
use crate::pob::{PobBuild, PoeVersion};
use crate::sources::poedb::PoedbClient;
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
pub const REPOE_MODS_FOR_BASE: &str = "repoe_mods_for_base";
pub const POEDB_LOOKUP: &str = "poedb_lookup";
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
    pub poedb_poe1: PoedbClient,
    pub poedb_poe2: PoedbClient,
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
    /// Delta sender plumbed from the provider so sheet tools can emit
    /// `SheetDraftUpdate` / `SheetAskUser` directly to the UI. `None` for
    /// MCP-serve and CLI entry points where no UI is listening.
    pub deltas: Option<mpsc::UnboundedSender<LlmDelta>>,
}

impl ToolCtx {
    pub fn new(build: BuildContext) -> Result<Self> {
        let http = PoeHttpClient::new().context("init HTTP client")?;
        let cache = FileCache::new(FileCache::default_dir());
        let wiki_poe1 = WikiClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
        let wiki_poe2 = WikiClient::new(http.clone(), cache.clone(), PoeVersion::Poe2);
        let trade_poe1 = TradeClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
        let trade_poe2 = TradeClient::new(http.clone(), cache.clone(), PoeVersion::Poe2);
        let poedb_poe1 = PoedbClient::new(http.clone(), cache.clone(), PoeVersion::Poe1);
        let poedb_poe2 = PoedbClient::new(http.clone(), cache, PoeVersion::Poe2);
        let pob_engine = crate::llm::pob_engine::global();
        let kb = crate::llm::kb::global();
        Ok(Self {
            build,
            wiki_poe1,
            wiki_poe2,
            trade_poe1,
            trade_poe2,
            poedb_poe1,
            poedb_poe2,
            http,
            pob_engine,
            kb,
            kb_calls_this_turn: Arc::new(AtomicU32::new(0)),
            skill_calls_this_turn: Arc::new(AtomicU32::new(0)),
            user_question: None,
            deltas: None,
        })
    }

    /// Attach the per-turn `LlmDelta` sender. Called by the provider so
    /// sheet tools can emit `SheetDraftUpdate` / `SheetAskUser` events
    /// directly while a tool is running.
    pub fn with_deltas(mut self, deltas: mpsc::UnboundedSender<LlmDelta>) -> Self {
        self.deltas = Some(deltas);
        self
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

    fn poedb(&self, game: &str) -> &PoedbClient {
        match parse_game(game) {
            PoeVersion::Poe2 => &self.poedb_poe2,
            _ => &self.poedb_poe1,
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
    let mut schemas = vec![
        json!({
            "name": GET_ACTIVE_BUILD,
            "description": "Returns the exile's currently loaded Path of Building build: game (PoE1/PoE2), class, ascendancy, level, main skill, full skill groups with linked gems, every item with its full text, and a `stats` map with ~60 high-value cached PoB stats (pools + ES + EHP + unreserved; headline DPS + ailment DoT DPS + with-* aggregates; resistances + over-caps + max-resists; armour/evasion/PDR; block/spell-block/suppression/dodge/evade; life/mana/ES regen + leech-gain; movement speed, attack/cast speed, crit chance + multi + pre-effective; hit chance; stun + ailment avoid; max charges; Str/Dex/Int). Plus per-element max-hit values, charges (power/frenzy/endurance current+max), active buffs (combat/buff/curse lists), config (boss profile, enemy resists, flask uptimes, custom mods), passive tree summary, the FULL `allocated_nodes` list, all chosen `mastery_picks`, `jewel_placements` on tree sockets, raised `spectres`, PoE1 `tattoos`, the `pantheon` choice (major + minor + bandit), the active item-set `slot_map`, and the pobb.in `import_link` if present. The response also includes SEMANTIC FACTS computed from the parsed build: `archetype` (defense/hit_model/mechanic tags — e.g. {defense:[\"life\",\"MoM\"], hit_model:[\"non-crit-EO\"], mechanic:[\"self-cast\"]}), `defining_uniques` (uniques present, each tagged engine|defining|amplifier with an identity hint), and `conversion_chain` (verbatim damage-conversion steps when applicable). Surface archetype tags FIRST when commenting on the build — do NOT guess the archetype from class+ascendancy alone. Never recommend selling an item flagged `category: \"engine\"` without explicit user instruction; engine items collapse the build if removed. Always call this BEFORE making any claim about the exile's character. No arguments.",
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
            "name": REPOE_MODS_FOR_BASE,
            "description": "Enumerate every explicit mod that can spawn on a given base item, with effective spawn weight + stat ranges. Use this for `what can roll on a Shaper Reaver Sword`, `what's the tier 1 prefix on a Vaal Regalia`, `what fractured mods exist for Cluster Jewels`, etc. — the join (mod.spawn_weights × base_item.tags) is computed offline from the bundled RePoE snapshot, so the result is the authoritative GGG mod pool (not a wiki article that may be patches out of date). `base_id` is the metadata path returned by `repoe_lookup category=base_items` (e.g. `Metadata/Items/Weapons/.../ReaverSword`) — call `repoe_lookup` first if you only have the human name. `influence` is optional and one of `shaper`, `elder`, `crusader`, `hunter`, `redeemer`, `warlord`, `exarch`, `eater`; when set, the join also synthesises `<influence>_<base_class>` virtual tags so influence mods are included. Returns mods sorted by group then descending required_level (highest tier first). Result is capped at `limit` (default 60, max 200); `total_matched` shows the pre-truncation count so you can tell when to refine.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "game": {"type": "string", "enum": ["poe1", "poe2"]},
                    "base_id": {
                        "type": "string",
                        "description": "Exact metadata id for the base item, e.g. 'Metadata/Items/Weapons/TwoHandWeapons/TwoHandSwords/TwoHandSword15'. Get it via repoe_lookup category=base_items first."
                    },
                    "influence": {
                        "type": "string",
                        "enum": ["shaper", "elder", "crusader", "hunter", "redeemer", "warlord", "exarch", "eater"],
                        "description": "Optional. When set, expands the base tag set with `{influence}_{class_tag}` synthetic tags so influence-gated mods get matched."
                    },
                    "limit": {"type": "integer", "default": 60, "minimum": 1, "maximum": 200}
                },
                "required": ["game", "base_id"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": POEDB_LOOKUP,
            "description": "Fetch a derived view from PoEDB that RePoE doesn't pre-compute. Three operations: (a) `craft_bench` — the FULL crafting bench mod table (all classes), with `[mod_text, require, item_classes, unlock]` per row; (b) `craft_bench_for_class` — same schema, filtered to one item class (pass `class` as the plural URL slug PoEDB uses, e.g. `Rings`, `Two_Hand_Swords`, `Body_Armours`); (c) `skill_gem` — per-skill level progression table + implicit mods + version-history changelog (pass `gem` as the gem's PoEDB slug, e.g. `Molten_Strike`, `Spark_of_Cataclysm`). Use this **AFTER** trying `repoe_lookup` / `repoe_mods_for_base` — those are instant and offline. Reach for PoEDB when you need the curated joined view: 'what does the bench offer for Body Armour right now?', 'what stats does Molten Strike have at level 21?'. The page is HTTP-fetched and 24h-cached. Returns the URL it pulled in `source_url` so the agent can cite PoEDB in `Sources:`.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "game": {"type": "string", "enum": ["poe1", "poe2"]},
                    "operation": {
                        "type": "string",
                        "enum": ["craft_bench", "craft_bench_for_class", "skill_gem"]
                    },
                    "class": {
                        "type": "string",
                        "description": "Required for `craft_bench_for_class`. The plural URL slug PoEDB uses for the item class (e.g. `Rings`, `Two_Hand_Swords`, `Body_Armours`, `Helmets`, `Amulets`)."
                    },
                    "gem": {
                        "type": "string",
                        "description": "Required for `skill_gem`. The skill gem's PoEDB slug (e.g. `Molten_Strike`, `Spark_of_Cataclysm`). Spaces are auto-converted to underscores."
                    }
                },
                "required": ["game", "operation"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": POB_CALC,
            "description": "Run the bundled headless PoB engine against the active build and return canonical output stats. Categories: offence (DPS, hit chance, crit, ailment DPS), defence (EHP, max-hit by element, block, suppression, resists, leech, regen, recovery, movement, ailment avoid), charges (max counts), reservation (life/mana/spirit %), ailments (shock/freeze/ignite chance + avoidance), all (entire output table). Optional `skill_index` selects a non-default skill group (1-indexed, matching PoB's mainSocketGroup). Optional `skill_part` picks a sub-mode of a multi-part skill (Lightning Strike melee vs projectile, Pulverise main vs aoe, etc.). Optional `calcs` overrides the PoB Calcs config: combat toggles (enemyIsBoss, usePowerCharges/useFrenzyCharges/useEnduranceCharges, forceBuffOnslaught, multiplierImpaleStacks), flask uptimes (useFlask1..5), PoE1 character choices (bandit one of 'Alira'/'Kraityn'/'Oak'/'None', pantheonMajorGod, pantheonMinorGod), boss math overrides (enemyLevel — default 84, push to 85-87 to simulate uber pinnacle), and arbitrary condition toggles for runtime states (any key starting with `condition*` accepts a boolean: `conditionFullLife`, `conditionLowLife`, `conditionAtMaxFrenzyCharges`, `conditionEnemyChilled`, `conditionUsingFlask`, etc.). Bestel's wrong-skill gate AUTO-RETRIES once when active_skill_label disagrees with build.main_skill — if it still mismatches you receive an `error: \"wrong_skill\"` envelope with `available_skill_indices`; pick one and retry pob_calc with that skill_index. The response ALWAYS echoes (1) the effective Calcs config and (2) `active_skill` metadata. Surface the Calcs assumptions in your answer (`bandit=Alira`, `enemyIsBoss=Pinnacle`, `useFlask3=true`, etc.) — two PoBs with identical gear can show 10× DPS swings purely from Calcs config; never quote a number without naming the assumptions. The response also includes a `warnings` array (e.g. `flask_slot_missing` when a requested flask slot is empty); if non-empty, surface those warnings to the user.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "enum": ["offence", "defence", "charges", "reservation", "ailments", "all"]
                    },
                    "skill_index": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "1-indexed mainSocketGroup. Pick from available_skill_indices when wrong_skill fires."
                    },
                    "skill_part": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "Sub-mode of a multi-part skill (Lightning Strike: 1=melee, 2=projectile; Pulverise: 1=main, 2=aoe; etc.)."
                    },
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
                            "useFlask5": {"type": "boolean"},
                            "bandit": {
                                "type": "string",
                                "description": "PoE1 reward choice. One of 'Alira' / 'Kraityn' / 'Oak' / 'None'."
                            },
                            "pantheonMajorGod": {
                                "type": "string",
                                "description": "PoE1 major pantheon (e.g. 'TheBrineKing', 'Solaris', 'Lunaris', 'Arakaali')."
                            },
                            "pantheonMinorGod": {
                                "type": "string",
                                "description": "PoE1 minor pantheon (e.g. 'Ralakesh', 'Yugul', 'Garukhan')."
                            },
                            "enemyLevel": {
                                "type": "integer",
                                "minimum": 1,
                                "description": "Boss math level; default 84 (pinnacle), 85-87 simulates uber."
                            },
                            "conditions": {
                                "type": "object",
                                "description": "Free-form `conditionXxx` boolean map (conditionFullLife, conditionAtMaxFrenzyCharges, conditionEnemyChilled, …).",
                                "additionalProperties": {"type": "boolean"}
                            }
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
    ];
    schemas.extend(sheet_tools::schemas());
    schemas
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
        REPOE_MODS_FOR_BASE => {
            let game = input
                .get("game")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'game' is required ('poe1' or 'poe2')"))?;
            let game = RepoeGame::parse(game)
                .ok_or_else(|| anyhow!("'game' must be 'poe1' or 'poe2', got '{game}'"))?;
            let base_id = input
                .get("base_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'base_id' is required"))?;
            let influence = input.get("influence").and_then(|v| v.as_str());
            let limit = input
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n.clamp(1, 200) as usize)
                .unwrap_or(60);
            let client = repoe::global();
            let result = client.mods_for_base(game, base_id, influence, limit)?;
            let value = serde_json::to_value(&result).context("serialize mods_for_base")?;
            Ok(truncate(&serde_json::to_string(&value).unwrap_or_default(), 30_000))
        }
        POEDB_LOOKUP => {
            let game = input.get("game").and_then(|v| v.as_str()).unwrap_or("poe1");
            let operation = input
                .get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("'operation' is required (craft_bench|craft_bench_for_class|skill_gem)"))?;
            let client = ctx.poedb(game);
            let value = match operation {
                "craft_bench" => {
                    let r = client.craft_bench().await?;
                    serde_json::to_value(&r).context("serialize craft_bench")?
                }
                "craft_bench_for_class" => {
                    let class = input
                        .get("class")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("'class' is required for craft_bench_for_class"))?;
                    let r = client.craft_bench_for_class(class).await?;
                    serde_json::to_value(&r).context("serialize craft_bench_for_class")?
                }
                "skill_gem" => {
                    let gem = input
                        .get("gem")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("'gem' is required for skill_gem"))?;
                    let r = client.skill_gem(gem).await?;
                    serde_json::to_value(&r).context("serialize skill_gem")?
                }
                other => {
                    return Err(anyhow!(
                        "'operation' must be one of craft_bench/craft_bench_for_class/skill_gem, got '{other}'"
                    ))
                }
            };
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
        SHEET_PROPOSE_SECTION => sheet_tools::dispatch_sheet_propose_section(input, &ctx.deltas).await,
        SHEET_ASK => sheet_tools::dispatch_sheet_ask(input, &ctx.deltas).await,
        SHEET_OPEN_INTERVIEW => sheet_tools::dispatch_sheet_open_interview(input, &ctx.deltas).await,
        SHEET_FINALIZE_REQUEST => sheet_tools::dispatch_sheet_finalize_request(input, &ctx.deltas).await,
        GET_ACTIVE_BUILD_SHEET => sheet_tools::dispatch_get_active_build_sheet(input, &ctx.deltas).await,
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

    // Compute the sheet-status directive BEFORE we touch the JSON, so we
    // can prepend it as a text block. A directive buried inside the JSON
    // (alphabetical key order, ~30k-token payload) was empirically
    // skimmed by DeepSeek-V4-Flash — it called get_active_build_sheet at
    // position 6 instead of the prescribed 2 on a stale sheet test.
    // Surfacing it as a leading PLAIN-TEXT block in front of the JSON
    // payload makes it unmissable. Anthropic / DeepSeek tool-results are
    // strings ; the model parses the whole content textually, so a
    // pre-JSON banner is naturally read first.
    //
    // NOTE: the anti-hallucination directive (anchor claims to this JSON,
    // do not extrapolate) lives in the runtime tag injected by
    // `crates/bestel-core/src/llm/anthropic.rs::run` — NOT here. Prepending
    // it to the tool result text breaks `ArtPoBImport.vue`'s
    // JSON.parse(segment.output), which falls back to a raw text dump
    // visible to the user (the entire banner + 30k-token JSON). The
    // runtime tag is in the system prompt context, never shown to the
    // exile, and is read by the model at every turn start — same
    // attentional position with zero UI side-effects.
    let priority_directive = compute_sheet_priority_directive(active_build.as_ref());

    let mut value: Value = match serde_json::from_str(&base) {
        Ok(v) => v,
        Err(_) => {
            // If the base failed to parse as JSON, still prepend the
            // directive as a banner so the agent sees it.
            return Ok(match &priority_directive {
                Some(d) => format!("{d}\n\n{base}"),
                None => base,
            });
        }
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
        None => {
            return Ok(match &priority_directive {
                Some(d) => format!("{d}\n\n{base}"),
                None => base,
            });
        }
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
    if let Some(build) = active_build.as_ref() {
        let session_id = derive_build_session_id(build);
        let notes = crate::llm::session_notes::read_notes(&session_id);
        if notes.current_diagnosis.is_some() {
            if let Ok(notes_json) = serde_json::to_value(&notes) {
                summary.insert("session_notes".into(), notes_json);
            }
        }
    }

    // (c) Sprint K — sheet status hint. The actual computation lives in
    // `compute_sheet_priority_directive` so we can use it both as a
    // pre-JSON banner (high-salience for models that skim) AND as a
    // structured `next_required_action` field inside the summary (for
    // any tooling that parses the JSON shape downstream). Both paths
    // carry the same string.
    if let Some(text) = priority_directive.as_deref() {
        summary.insert("next_required_action".into(), json!(text));
    }

    // Sprint v5: raised from 30_000 to 80_000. The structured item-mod
    // surface added in this sprint roughly doubles the items-array size,
    // and at 30 KB the binding cap dropped skill_groups / signatures /
    // stats / tattoos for full-gear builds (alphabetical key order puts
    // them after `items`). 80 KB sits comfortably under Anthropic /
    // DeepSeek tool-result limits while preserving the full surface.
    let body = truncate(
        &serde_json::to_string(&value).unwrap_or_else(|_| base.clone()),
        80_000,
    );
    // Prepend the directive as a plain-text banner so the model reads it
    // BEFORE the JSON payload. Empirically a directive buried in the
    // body was skimmed by DeepSeek-V4-Flash on long build dumps.
    Ok(match priority_directive {
        Some(d) => format!("{d}\n\n--- BUILD CONTEXT (JSON) ---\n\n{body}"),
        None => body,
    })
}

/// Compute the priority directive that nudges the agent toward
/// `get_active_build_sheet` as its IMMEDIATE next tool call when a sheet
/// exists for the loaded build. Returns `None` when no sheet is
/// applicable (no build, no fingerprint, no DB, or no sheet found).
///
/// Mirrors the lookup logic in `crate::llm::anthropic::run`'s tag
/// injector — keep them in sync. TODO(refactor): extract to
/// `crate::sheets::lookup_status`.
fn compute_sheet_priority_directive(build: Option<&PobBuild>) -> Option<String> {
    let build = build?;
    let fingerprint = crate::sheets::compute_fingerprint_from_pob(build)?;
    let current_hash = crate::sheets::compute_pob_hash_from_build(build);
    let row = crate::persistence::global_db().and_then(|db| {
        crate::sheets::store::find_by_fingerprint(&db, &fingerprint)
            .ok()
            .flatten()
    })?;

    if row.pob_hash == current_hash {
        Some(format!(
            "[PRIORITY DIRECTIVE — STEP 2 / READ SHEET FIRST]\n\
             A validated, fresh Build Sheet (id={id}) exists for this build. Your \
             IMMEDIATE next tool call MUST be get_active_build_sheet with \
             fingerprint='{fp}'. DO NOT call pob_calc / wiki_search / wiki_parse / \
             kb_search / load_skill / read_internal_reference before reading the \
             sheet — its sections intent / archetype / damage / defense / items \
             were authored from a deep PoB analysis at finalize time and often \
             already contain the pob_calc numbers, threshold lookups, and wiki \
             facts that bear on the user's question. Only do extra research for \
             things the sheet does not cover. End the answer with `read_from_sheet \
             · key1 · key2` italic caption.",
            id = row.id,
            fp = fingerprint,
        ))
    } else {
        Some(format!(
            "[PRIORITY DIRECTIVE — STEP 2 / READ STALE SHEET FIRST]\n\
             A Build Sheet (id={id}) exists for this build, but the PoB hash \
             differs since authoring (gear / gem swaps). Your IMMEDIATE next tool \
             call MUST be get_active_build_sheet with fingerprint='{fp}'. \
             Intent / archetype / defining items / known gaps are authored \
             decisions that don't go stale when gear shifts — only the numbers \
             age. After reading: cite sections.intent / archetype / items for \
             context, then re-derive numerical claims (DPS, EHP, max-hit, resists, \
             regen) from the live PoB via pob_calc. Open the answer with one \
             short sentence acknowledging the drift. End with `read_from_sheet · \
             keys` italic caption listing the sections you cited (never numbers). \
             DO NOT call sheet_open_interview to refresh — the user has a refresh \
             button in the UI and triggers it themselves.",
            id = row.id,
            fp = fingerprint,
        ))
    }
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
    let skill_part = input
        .get("skill_part")
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
        if let Some(s) = c.get("bandit").and_then(|v| v.as_str()) {
            calcs.bandit = Some(s.to_string());
        }
        if let Some(s) = c.get("pantheonMajorGod").and_then(|v| v.as_str()) {
            calcs.pantheon_major_god = Some(s.to_string());
        }
        if let Some(s) = c.get("pantheonMinorGod").and_then(|v| v.as_str()) {
            calcs.pantheon_minor_god = Some(s.to_string());
        }
        if let Some(n) = c.get("enemyLevel").and_then(|v| v.as_u64()) {
            calcs.enemy_level = Some(n as u32);
        }
        if let Some(conds) = c.get("conditions").and_then(|v| v.as_object()) {
            for (k, v) in conds {
                if let Some(b) = v.as_bool() {
                    // Forward any condition* key as-is. Lua side rejects
                    // unknown non-condition keys with a clear error.
                    if k.starts_with("condition") {
                        calcs.conditions.insert(k.clone(), b);
                    }
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
            build_xml: xml.clone(),
            category,
            skill_index,
            skill_part,
            calcs: calcs.clone(),
        })
        .await
        .map_err(|e| anyhow!("pob engine: {e}"))?;

    // Wrong-skill gate (Sprint v5). Compare the engine's active_skill_label
    // against the build's main_skill (case-insensitive trim, transfigure-
    // tolerant). On mismatch, locate the correct skill_index from
    // build.skill_groups and retry once. If retry also mismatches, return
    // a structured `wrong_skill` error so the LLM re-plans rather than
    // quoting numbers for the wrong skill.
    if let Some(out) = wrong_skill_recover(
        engine,
        &build,
        skill_index,
        skill_part,
        &response,
        game,
        &xml,
        category,
        &calcs,
    )
    .await?
    {
        return Ok(out);
    }

    let payload = serde_json::to_value(&response).context("serialize pob_calc response")?;
    // Sprint v5: 30 KB cap was too tight for `category=all` payloads
    // after Step 7's CATEGORY_KEYS expansion. 60 KB matches the
    // dispatch_get_active_build budget.
    Ok(truncate(&serde_json::to_string(&payload).unwrap_or_default(), 60_000))
}

/// Returns `Some(tool_result_json)` when the engine returned numbers for a
/// skill other than the build's main one and we either successfully
/// auto-retried or need to surface a structured error. `None` means the
/// active skill matches expectations and the caller should serialise the
/// original response.
#[allow(clippy::too_many_arguments)]
async fn wrong_skill_recover(
    engine: &std::sync::Arc<bestel_pob_engine::PobEngineHandle>,
    build: &PobBuild,
    attempted_skill_index: Option<u32>,
    skill_part: Option<u32>,
    response: &bestel_pob_engine::CalcResponse,
    game: bestel_pob_engine::lifecycle::Game,
    xml: &str,
    category: bestel_pob_engine::Category,
    calcs: &bestel_pob_engine::EngineCalcs,
) -> Result<Option<String>> {
    use bestel_pob_engine::CalcRequest;

    let expected = build
        .main_skill
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let got = response
        .active_skill
        .get("active_skill_label")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let (Some(expected), Some(got)) = (expected, got) else {
        return Ok(None);
    };
    if skill_labels_match(&expected, &got) {
        return Ok(None);
    }

    // Sprint v5: only auto-correct when the LLM did NOT specify a
    // skill_index. If the LLM explicitly requested skill_index=N it is
    // asking for a non-default skill on purpose (e.g. "compute Hatred
    // DPS" on a Penance Brand build). Overriding back to main_skill
    // would silently ignore the user's intent. The verification step
    // in the schema documentation still tells the LLM to recheck
    // active_skill_label against its target; we only intervene on
    // the silent-default-fallback class of failure.
    if attempted_skill_index.is_some() {
        return Ok(None);
    }

    let target = find_skill_index_for(build, &expected);
    let available = available_indices_for(build, &expected);

    // Retry once when we have a different index to try.
    if let Some(idx) = target {
        if Some(idx) != attempted_skill_index {
            let retry = engine
                .calc(CalcRequest {
                    game,
                    build_xml: xml.to_string(),
                    category,
                    skill_index: Some(idx),
                    skill_part,
                    calcs: calcs.clone(),
                })
                .await
                .map_err(|e| anyhow!("pob engine retry: {e}"))?;
            let retry_label = retry
                .active_skill
                .get("active_skill_label")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if skill_labels_match(&expected, retry_label) {
                let mut annotated = retry;
                annotated.warnings.push(format!(
                    "auto-retry: engine reported '{}' for skill_index={} but build.main_skill is '{}'; re-queried with skill_index={} and the engine confirmed",
                    got,
                    attempted_skill_index
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "default".to_string()),
                    expected,
                    idx
                ));
                let payload = serde_json::to_value(&annotated)
                    .context("serialize pob_calc retry response")?;
                return Ok(Some(truncate(
                    &serde_json::to_string(&payload).unwrap_or_default(),
                    30_000,
                )));
            }
        }
    }

    Ok(Some(
        json!({
            "error": "wrong_skill",
            "expected": expected,
            "got": got,
            "attempted_skill_index": attempted_skill_index,
            "retry_skill_index": target,
            "available_skill_indices": available,
            "message": format!(
                "pob_calc returned numbers for '{got}' but the build's main_skill is '{expected}'. Pick one of available_skill_indices and retry pob_calc with that skill_index. If the list is empty, the build's main_skill may be stale — verify by re-reading get_active_build's skill_groups and main_skill fields."
            )
        })
        .to_string(),
    ))
}

/// Two skill labels match when they are equal (case-insensitive trim) or
/// one is a transfigure suffix of the other. PoB sometimes stores the
/// base name ("Penance Brand") in main_skill while the engine reports
/// the transfigure ("Penance Brand of Dissipation"), or vice versa.
fn skill_labels_match(a: &str, b: &str) -> bool {
    let aa = a.trim().to_ascii_lowercase();
    let bb = b.trim().to_ascii_lowercase();
    if aa.is_empty() || bb.is_empty() {
        return false;
    }
    aa == bb || aa.starts_with(&bb) || bb.starts_with(&aa)
}

/// Scan skill_groups for the FIRST group whose enabled gems or label match
/// `expected`. Returns the 1-indexed group number (PoB's mainSocketGroup
/// convention), or None when nothing matches.
fn find_skill_index_for(build: &PobBuild, expected: &str) -> Option<u32> {
    available_indices_for(build, expected).into_iter().next()
}

/// All 1-indexed skill_group positions whose label or any enabled gem name
/// is a transfigure-tolerant match for `expected`.
fn available_indices_for(build: &PobBuild, expected: &str) -> Vec<u32> {
    let needle = expected.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (i, group) in build.skill_groups.iter().enumerate() {
        let label_match = !group.label.is_empty()
            && skill_labels_match(&group.label, expected);
        let gem_match = group
            .gems
            .iter()
            .any(|gem| gem.enabled && skill_labels_match(&gem.name, expected));
        if label_match || gem_match {
            out.push(i as u32 + 1);
        }
    }
    out
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

    // Selected high-value stats verbatim. Names match PoB's `<PlayerStat>`
    // keys as written by Path of Building (verified against vendored test
    // builds + CalcDefence/CalcOffence outputs). Unknown keys silently
    // skip — PoB doesn't always cache every entry, and the LLM can fall
    // back to `pob_calc category=all`. Grouped thematically.
    let key_stats = [
        // Pools and EHP.
        "Life",
        "Mana",
        "EnergyShield",
        "Spirit",
        "TotalEHP",
        "LifeUnreserved",
        "ManaUnreserved",
        // Headline DPS.
        "CombinedDPS",
        "TotalDPS",
        "FullDPS",
        "SkillDPS",
        "AverageHit",
        "AverageDamage",
        // Ailment / DoT DPS.
        "IgniteDPS",
        "BleedDPS",
        "PoisonDPS",
        "TotalDot",
        "TotalDotDPS",
        "ImpaleDPS",
        "WithBleedDPS",
        "WithPoisonDPS",
        "WithIgniteDPS",
        // Resistances (cached + over-cap).
        "FireResist",
        "ColdResist",
        "LightningResist",
        "ChaosResist",
        "FireResistOverCap",
        "ColdResistOverCap",
        "LightningResistOverCap",
        "ChaosResistOverCap",
        // Max resists (often live-engine only, kept for graceful fallback).
        "FireResistMax",
        "ColdResistMax",
        "LightningResistMax",
        "ChaosResistMax",
        // Mitigation.
        "Armour",
        "Evasion",
        "PhysicalDamageReduction",
        "BlockChance",
        "SpellBlockChance",
        "EffectiveSpellSuppressionChance",
        "EffectiveBlockChance",
        "AttackDodgeChance",
        "SpellDodgeChance",
        "MeleeEvadeChance",
        "ProjectileEvadeChance",
        // Sustain (regen + leech-gain).
        "LifeRegen",
        "ManaRegen",
        "EnergyShieldRegen",
        "LifeLeechGainRate",
        "ManaLeechGainRate",
        "EnergyShieldLeechGainRate",
        // Speed and accuracy and crit.
        "EffectiveMovementSpeedMod",
        "Speed",
        "HitChance",
        "CritChance",
        "PreEffectiveCritChance",
        "CritMultiplier",
        // Ailment avoidance and stun.
        "StunAvoidChance",
        "IgniteAvoidChance",
        "PoisonAvoidChance",
        // Charges.
        "PowerChargesMax",
        "FrenzyChargesMax",
        "EnduranceChargesMax",
        // Attributes.
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
                    "variant_id": gem.variant_id,
                    "gem_id": gem.gem_id,
                    "stat_set_index": gem.stat_set_index,
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
            let mut entry = serde_json::Map::new();
            entry.insert("id".into(), json!(it.id));
            entry.insert("rarity".into(), json!(it.rarity));
            entry.insert("name".into(), json!(it.name));
            entry.insert("base".into(), json!(it.base));
            // Sprint v5: drop the raw blob when structured item mods are
            // populated — the structured fields already carry every line,
            // and keeping a 1500-char raw per item pushed the overall
            // render past the 60 KB truncate cap on full-gear builds,
            // dropping skill_groups / signatures / stats silently. Keep
            // a short raw fallback for items the parser couldn't split
            // (mostly non-named gear or one-line entries).
            let has_structured = !it.implicit_mods.is_empty()
                || !it.explicit_mods.is_empty()
                || !it.enchant_mods.is_empty()
                || !it.runic_mods.is_empty()
                || !it.sockets.is_empty();
            if !has_structured {
                entry.insert("raw".into(), json!(truncate(&it.raw, 600)));
            }
            // Sprint v5 — structured item fields. Skip entries whose
            // structured field is empty so the JSON stays tight on
            // common (e.g. low-mod) items.
            if let Some(v) = it.item_level {
                entry.insert("item_level".into(), json!(v));
            }
            if let Some(v) = &it.variant {
                entry.insert("variant".into(), json!(v));
            }
            if let Some(v) = &it.unique_id {
                entry.insert("unique_id".into(), json!(v));
            }
            if let Some(v) = &it.anointment {
                entry.insert("anointment".into(), json!(v));
            }
            if let Some(v) = &it.catalyst {
                entry.insert("catalyst".into(), json!(v));
            }
            if !it.sockets.is_empty() {
                entry.insert("sockets".into(), json!(it.sockets));
            }
            if !it.influences.is_empty() {
                entry.insert("influences".into(), json!(it.influences));
            }
            if it.corrupted {
                entry.insert("corrupted".into(), json!(true));
            }
            if it.mirrored {
                entry.insert("mirrored".into(), json!(true));
            }
            if it.split {
                entry.insert("split".into(), json!(true));
            }
            if !it.enchant_mods.is_empty() {
                entry.insert("enchant_mods".into(), json!(it.enchant_mods));
            }
            if !it.implicit_mods.is_empty() {
                entry.insert("implicit_mods".into(), json!(it.implicit_mods));
            }
            if !it.explicit_mods.is_empty() {
                entry.insert("explicit_mods".into(), json!(it.explicit_mods));
            }
            if !it.runic_mods.is_empty() {
                entry.insert("runic_mods".into(), json!(it.runic_mods));
            }
            serde_json::Value::Object(entry)
        })
        .collect();
    summary.insert("items".into(), json!(items));

    // Sprint v5 — surface PoB fields the parser captures but the LLM was
    // not seeing. Pantheon + bandit, Karui tattoos, the full allocated
    // node list, mastery picks, jewel placements, active spectres, the
    // active item-set slot map, and the pobb.in import link.
    if !b.pantheon.is_empty() {
        summary.insert("pantheon".into(), json!(b.pantheon));
    }
    if !b.tattoos.is_empty() {
        summary.insert("tattoos".into(), json!(b.tattoos));
    }
    if !b.allocated_nodes.is_empty() {
        summary.insert("allocated_nodes".into(), json!(b.allocated_nodes));
    }
    if !b.mastery_picks.is_empty() {
        summary.insert("mastery_picks".into(), json!(b.mastery_picks));
    }
    if !b.jewel_placements.is_empty() {
        summary.insert("jewel_placements".into(), json!(b.jewel_placements));
    }
    if !b.spectres.is_empty() {
        summary.insert("spectres".into(), json!(b.spectres));
    }
    if !b.slot_map.is_empty() {
        summary.insert("slot_map".into(), json!(b.slot_map));
    }
    if let Some(url) = &b.import_link {
        summary.insert("import_link".into(), json!(url));
    }
    if !b.skill_sets.is_empty() {
        summary.insert("skill_sets".into(), json!(b.skill_sets));
    }
    if !b.tree_specs.is_empty() {
        summary.insert("tree_specs".into(), json!(b.tree_specs));
    }

    // Sprint v5 — surface the 5 orthogonal signatures + canonical pob_hash
    // so the agent can describe drift in concrete terms ("your gear_sig
    // flipped since the last sheet, but tree_sig and skill_sig are
    // unchanged — purely a gear change") without a sheet round-trip.
    let sigs = crate::pob::signatures::BuildSignatures::from_build(b);
    let pob_hash = crate::sheets::compute_pob_hash_from_build(b);
    summary.insert(
        "signatures".into(),
        json!({
            "identity": sigs.identity,
            "tree": sigs.tree,
            "gear": sigs.gear,
            "skill": sigs.skill,
            "config": sigs.config,
            "pob_hash": pob_hash,
        }),
    );

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
    // Sprint v5: raised from 60_000 to 120_000 to absorb the structured
    // item-mod surface without dropping alphabetically-late fields
    // (signatures, skill_groups, stats, tattoos, tree_specs) when the
    // alphabetically-early `items` array is rich. Claude / DeepSeek-V4
    // tokens easily fit 120 KB into context.
    truncate(&s, 120_000)
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
        assert!(names.contains(&REPOE_MODS_FOR_BASE));
        assert!(names.contains(&POEDB_LOOKUP));
        assert!(names.contains(&POB_CALC));
        assert!(names.contains(&KB_SEARCH));
        assert!(names.contains(&LOAD_SKILL));
        assert!(names.contains(&SHEET_PROPOSE_SECTION));
        assert!(names.contains(&SHEET_ASK));
        assert!(names.contains(&SHEET_FINALIZE_REQUEST));
        assert!(names.contains(&GET_ACTIVE_BUILD_SHEET));
        assert_eq!(schemas.len(), 20);
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

    #[test]
    fn skill_labels_match_is_transfigure_tolerant() {
        assert!(skill_labels_match("Penance Brand", "Penance Brand"));
        assert!(skill_labels_match(
            "Penance Brand",
            "Penance Brand of Dissipation"
        ));
        assert!(skill_labels_match(
            "Penance Brand of Conduction",
            "penance brand"
        ));
        assert!(!skill_labels_match("Penance Brand", "Hatred"));
        assert!(!skill_labels_match("", "Penance Brand"));
        assert!(!skill_labels_match("Penance Brand", ""));
    }

    #[test]
    fn available_indices_for_finds_active_skill_in_groups() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Witch" mainSocketGroup="1"/>
  <Skills>
    <SkillSet id="1">
      <Skill mainActiveSkill="1" label="">
        <Gem nameSpec="Hatred" enabled="true"/>
      </Skill>
      <Skill mainActiveSkill="1" label="Mapping">
        <Gem nameSpec="Penance Brand of Dissipation" enabled="true"/>
      </Skill>
      <Skill mainActiveSkill="1" label="Boss">
        <Gem nameSpec="Penance Brand of Conduction" enabled="true"/>
      </Skill>
    </SkillSet>
  </Skills>
</PathOfBuilding>"#;
        let build =
            crate::pob::parser::parse_bytes(xml, std::path::PathBuf::from("t.xml")).unwrap();
        let indices = available_indices_for(&build, "Penance Brand");
        assert_eq!(indices, vec![2, 3]);
        assert_eq!(find_skill_index_for(&build, "Penance Brand"), Some(2));
        assert_eq!(find_skill_index_for(&build, "Hatred"), Some(1));
        assert_eq!(find_skill_index_for(&build, "Cyclone"), None);
    }

    #[test]
    fn render_surfaces_pantheon_tattoos_tree_jewels_spectres_slot_map_import() {
        let xml = br#"<?xml version="1.0"?>
<PathOfBuilding>
  <Build level="92" className="Templar" ascendClassName="Inquisitor"
         pantheonMajorGod="Solaris" pantheonMinorGod="Ralakesh" bandit="Alira"
         mainSocketGroup="1">
    <Spectre id="Metadata/Monsters/X/Y"/>
    <Spectre id="Metadata/Monsters/Z"/>
  </Build>
  <Import importLink="https://pobb.in/pob/abc123"/>
  <Tree activeSpec="1">
    <Spec nodes="100,200,300" masteryEffects="{100,5},{200,7}" treeVersion="3_25" classId="5">
      <Sockets>
        <Socket nodeId="2311" itemId="13"/>
        <Socket nodeId="9408" itemId="1"/>
      </Sockets>
      <Overrides>
        <Override dn="Tattoo of the Tukohama Warrior" nodeId="55555">+4% to Fire Resistance</Override>
      </Overrides>
    </Spec>
  </Tree>
  <Skills>
    <SkillSet id="1">
      <Skill mainActiveSkill="1" slot="Helmet" label="">
        <Gem nameSpec="Penance Brand of Dissipation" enabled="true"
             skillId="PenanceBrandOfDissipationPlayer"
             variantId="PenanceBrandOfDissipation"
             gemId="Metadata/Items/Gems/SkillGemPenanceBrandOfDissipation"/>
      </Skill>
    </SkillSet>
  </Skills>
  <Items activeItemSet="1">
    <Item id="1">Rarity: RARE
Mind Star
Praetor Crown</Item>
    <ItemSet id="1">
      <Slot name="Helmet" itemId="1"/>
    </ItemSet>
  </Items>
</PathOfBuilding>"#;
        let build = crate::pob::parser::parse_bytes(xml, std::path::PathBuf::from("t.xml"))
            .expect("parse fixture");
        let rendered = render_build_for_llm(&build);
        let v: serde_json::Value =
            serde_json::from_str(&rendered).expect("render output is valid JSON");
        let obj = v.as_object().expect("render output is a JSON object");

        // Pantheon carries major / minor / bandit.
        let pantheon = obj.get("pantheon").expect("pantheon present");
        assert_eq!(pantheon.get("major").and_then(|v| v.as_str()), Some("Solaris"));
        assert_eq!(pantheon.get("minor").and_then(|v| v.as_str()), Some("Ralakesh"));
        assert_eq!(pantheon.get("bandit").and_then(|v| v.as_str()), Some("Alira"));

        // Tattoos with their node id and display name.
        let tattoos = obj.get("tattoos").and_then(|v| v.as_array()).expect("tattoos array");
        assert_eq!(tattoos.len(), 1);
        assert_eq!(
            tattoos[0].get("display_name").and_then(|v| v.as_str()),
            Some("Tattoo of the Tukohama Warrior")
        );

        // Full allocated-node and mastery-pick lists.
        let nodes = obj.get("allocated_nodes").and_then(|v| v.as_array()).unwrap();
        assert_eq!(nodes.len(), 3);
        let masteries = obj.get("mastery_picks").and_then(|v| v.as_array()).unwrap();
        assert_eq!(masteries.len(), 2);

        // Jewel placements from the Sockets block.
        let jewels = obj.get("jewel_placements").and_then(|v| v.as_array()).unwrap();
        assert_eq!(jewels.len(), 2);

        // Spectres list.
        let spectres = obj.get("spectres").and_then(|v| v.as_array()).unwrap();
        assert_eq!(spectres.len(), 2);

        // Slot map from the active item set.
        let slot_map = obj.get("slot_map").and_then(|v| v.as_object()).unwrap();
        assert_eq!(slot_map.get("Helmet").and_then(|v| v.as_str()), Some("1"));

        // pobb.in import link.
        assert_eq!(
            obj.get("import_link").and_then(|v| v.as_str()),
            Some("https://pobb.in/pob/abc123")
        );

        // Signatures + pob_hash flow through for drift reasoning.
        let signatures = obj.get("signatures").and_then(|v| v.as_object()).unwrap();
        for key in ["identity", "tree", "gear", "skill", "config", "pob_hash"] {
            let hex = signatures.get(key).and_then(|v| v.as_str()).expect(key);
            assert_eq!(hex.len(), 64, "{} is not a 64-char sha256 hex", key);
        }

        // Gem variant_id + gem_id flow through to disambiguate transfigure
        // variants (Penance Brand of Dissipation vs base).
        let skill_groups = obj
            .get("skill_groups")
            .and_then(|v| v.as_array())
            .expect("skill_groups array");
        assert!(!skill_groups.is_empty());
        let gem = skill_groups[0]
            .get("gems")
            .and_then(|v| v.as_array())
            .and_then(|g| g.first())
            .expect("first gem present");
        assert_eq!(
            gem.get("variant_id").and_then(|v| v.as_str()),
            Some("PenanceBrandOfDissipation")
        );
        assert!(gem
            .get("gem_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("PenanceBrandOfDissipation"));
    }
}
