//! Bundled offline catalogue of repoe-fork PoE1 / PoE2 game data.
//!
//! Sourced from `https://repoe-fork.github.io/` (PoE1) and
//! `https://repoe-fork.github.io/poe2/` (PoE2). Eight categories per game:
//! mods, base_items, gems, uniques, cluster_jewels, essences, fossils,
//! stat_translations.
//!
//! # Lookup precedence
//!
//! 1. In-memory `RwLock<HashMap<(Game, Category), Arc<CategorySnapshot>>>`
//!    (warm cache — populated on first access).
//! 2. On-disk refreshed snapshot at
//!    `~/.bestel/cache/repoe/{game}/<cat>.json.zst` (written by
//!    `repoe_refresh::run`).
//! 3. Bundled `include_bytes!` blob under `snapshots/{poe1,poe2}/`
//!    (compile-time fallback so the agent works offline at first launch).
//!
//! The refresh task swaps `Arc<CategorySnapshot>` atomically when it
//! validates a fresh payload; in-flight readers see a consistent snapshot
//! for the duration of their call.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::cache::FileCache;

/// Game variant — matches `crate::pob::PoeVersion` but stays decoupled so
/// the catalogue can ship without pulling pob types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Poe1,
    Poe2,
}

impl Game {
    pub fn as_str(self) -> &'static str {
        match self {
            Game::Poe1 => "poe1",
            Game::Poe2 => "poe2",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "poe1" => Some(Game::Poe1),
            "poe2" => Some(Game::Poe2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Mods,
    BaseItems,
    Gems,
    Uniques,
    ClusterJewels,
    Essences,
    Fossils,
    StatTranslations,
}

impl Category {
    pub const ALL: &'static [Category] = &[
        Category::Mods,
        Category::BaseItems,
        Category::Gems,
        Category::Uniques,
        Category::ClusterJewels,
        Category::Essences,
        Category::Fossils,
        Category::StatTranslations,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Category::Mods => "mods",
            Category::BaseItems => "base_items",
            Category::Gems => "gems",
            Category::Uniques => "uniques",
            Category::ClusterJewels => "cluster_jewels",
            Category::Essences => "essences",
            Category::Fossils => "fossils",
            Category::StatTranslations => "stat_translations",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mods" => Some(Category::Mods),
            "base_items" => Some(Category::BaseItems),
            "gems" => Some(Category::Gems),
            "uniques" => Some(Category::Uniques),
            "cluster_jewels" => Some(Category::ClusterJewels),
            "essences" => Some(Category::Essences),
            "fossils" => Some(Category::Fossils),
            "stat_translations" => Some(Category::StatTranslations),
            _ => None,
        }
    }

    /// Whether repoe-fork provides this category for the given game as of
    /// the current bundled snapshot. PoE2 lacks PoE1-only systems
    /// (cluster_jewels, essences, fossils) and — verified 2026-05-08 —
    /// also lacks `gems.min.json` and `stat_translations.min.json` on
    /// `https://repoe-fork.github.io/poe2/`. Update this matrix when the
    /// upstream export grows; the snapshot manifest at
    /// `src/sources/snapshots/version.txt` is the ground-truth check.
    pub fn available_for(self, game: Game) -> bool {
        match (game, self) {
            (Game::Poe1, _) => true,
            (Game::Poe2, Category::Mods) => true,
            (Game::Poe2, Category::BaseItems) => true,
            (Game::Poe2, Category::Uniques) => true,
            (Game::Poe2, _) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotSource {
    Bundled,
    Refreshed,
}

#[derive(Debug)]
pub struct CategorySnapshot {
    pub game: Game,
    pub category: Category,
    pub entries: HashMap<String, Value>,
    pub fetched_at: u64,
    pub source: SnapshotSource,
}

type SnapshotMap = HashMap<(Game, Category), Arc<CategorySnapshot>>;

static GLOBAL_CLIENT: OnceLock<Arc<RepoeClient>> = OnceLock::new();

/// Process-wide singleton. Both the tool dispatcher and the background
/// refresh task share this handle so cache flips are immediately visible
/// to in-flight tool calls.
pub fn global() -> Arc<RepoeClient> {
    GLOBAL_CLIENT
        .get_or_init(|| Arc::new(RepoeClient::new(FileCache::new(FileCache::default_dir()))))
        .clone()
}

pub struct RepoeClient {
    cache: FileCache,
    snapshots: OnceLock<RwLock<SnapshotMap>>,
}

impl RepoeClient {
    pub fn new(cache: FileCache) -> Self {
        Self {
            cache,
            snapshots: OnceLock::new(),
        }
    }

    fn map(&self) -> &RwLock<SnapshotMap> {
        self.snapshots.get_or_init(|| RwLock::new(HashMap::new()))
    }

    /// Fetch (or load) the snapshot for `(game, category)`. Returns an
    /// `Arc` so callers don't hold the `RwLock` across lookups.
    pub fn snapshot(&self, game: Game, category: Category) -> Result<Arc<CategorySnapshot>> {
        if !category.available_for(game) {
            return Err(anyhow!(
                "category {} not available for {} (not present in repoe-fork bundle)",
                category.as_str(),
                game.as_str()
            ));
        }

        if let Some(existing) = self
            .map()
            .read()
            .ok()
            .and_then(|m| m.get(&(game, category)).cloned())
        {
            return Ok(existing);
        }

        let snap = Arc::new(self.load_snapshot(game, category)?);
        if let Ok(mut guard) = self.map().write() {
            guard.insert((game, category), snap.clone());
        }
        Ok(snap)
    }

    /// Manually swap a refreshed snapshot in. Used by `repoe_refresh::run`.
    pub fn install(&self, snap: Arc<CategorySnapshot>) {
        if let Ok(mut guard) = self.map().write() {
            guard.insert((snap.game, snap.category), snap);
        }
    }

    fn load_snapshot(&self, game: Game, category: Category) -> Result<CategorySnapshot> {
        if let Some(snap) = self.load_refreshed(game, category)? {
            return Ok(snap);
        }
        self.load_bundled(game, category)
    }

    fn refreshed_path(&self, game: Game, category: Category) -> PathBuf {
        self.cache
            .dir()
            .join("repoe")
            .join(game.as_str())
            .join(format!("{}.json.zst", category.as_str()))
    }

    fn load_refreshed(&self, game: Game, category: Category) -> Result<Option<CategorySnapshot>> {
        let path = self.refreshed_path(game, category);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)
            .with_context(|| format!("read refreshed snapshot {}", path.display()))?;
        let entries = decode_zstd_json(&bytes)?;
        let fetched_at = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Ok(Some(CategorySnapshot {
            game,
            category,
            entries,
            fetched_at,
            source: SnapshotSource::Refreshed,
        }))
    }

    fn load_bundled(&self, game: Game, category: Category) -> Result<CategorySnapshot> {
        let bytes = bundled_blob(game, category).ok_or_else(|| {
            anyhow!(
                "no bundled snapshot for {}/{}",
                game.as_str(),
                category.as_str()
            )
        })?;
        let entries = decode_zstd_json(bytes)?;
        Ok(CategorySnapshot {
            game,
            category,
            entries,
            fetched_at: BUNDLE_FETCHED_AT,
            source: SnapshotSource::Bundled,
        })
    }

    pub fn lookup_by_id(&self, game: Game, category: Category, id: &str) -> Result<LookupResult> {
        let snap = self.snapshot(game, category)?;
        let entry = snap.entries.get(id).cloned();
        let entries = match entry {
            Some(value) => vec![LookupEntry {
                id: id.to_string(),
                score: 1.0,
                value,
            }],
            None => Vec::new(),
        };
        Ok(LookupResult::from_snap(&snap, entries))
    }

    /// Join `mods × base_items` to enumerate every explicit mod that can
    /// spawn on a given base item, with the effective spawn weight and
    /// stat ranges. Mimics GGG's roll algorithm: walk each mod's
    /// `spawn_weights` array in order, take the FIRST tag whose name
    /// appears in the base's `tags` set as the effective weight (0 → mod
    /// can't roll).
    ///
    /// `base_id` is the metadata path (e.g. `Metadata/Items/Weapons/.../ReaverSword`)
    /// — fuzzy name resolution is the caller's job (use `lookup_by_name`).
    /// `influence` is an optional lowercase string (`shaper`, `elder`,
    /// `crusader`, `hunter`, `redeemer`, `warlord`, `exarch`, `eater`)
    /// that synthesises matching virtual tags into the base tag set so
    /// influence mods (whose spawn_weights reference e.g. `shaper_2h_weapon`)
    /// get selected.
    pub fn mods_for_base(
        &self,
        game: Game,
        base_id: &str,
        influence: Option<&str>,
        limit: usize,
    ) -> Result<ModsForBaseResult> {
        let bases = self.snapshot(game, Category::BaseItems)?;
        let base_entry = bases
            .entries
            .get(base_id)
            .ok_or_else(|| anyhow!("base item id not found: {base_id}"))?;
        let base_tags = extract_base_tags(base_entry, influence);
        if base_tags.is_empty() {
            return Err(anyhow!(
                "base {base_id} has no usable tags — RePoE bundle may be malformed"
            ));
        }

        let mods = self.snapshot(game, Category::Mods)?;
        let mut out: Vec<ModForBaseEntry> = Vec::new();
        for (mod_id, mod_value) in mods.entries.iter() {
            let domain = mod_value
                .get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if domain != "item" {
                continue;
            }
            let weight = effective_spawn_weight(mod_value, &base_tags);
            if weight <= 0 {
                continue;
            }
            if mod_value
                .get("is_essence_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }
            let mut entry = ModForBaseEntry {
                id: mod_id.clone(),
                weight,
                name: mod_value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                generation_type: mod_value
                    .get("generation_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                required_level: mod_value
                    .get("required_level")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32)
                    .unwrap_or(0),
                group: mod_value
                    .get("groups")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                text: mod_value
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                mod_type: mod_value
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                stats: mod_value
                    .get("stats")
                    .cloned()
                    .unwrap_or(Value::Array(vec![])),
            };
            // Fall back to the `text` field when GGG doesn't ship a
            // human name (typical for unique-only / hidden mods).
            if entry.name.is_empty() {
                entry.name = entry.text.clone();
            }
            out.push(entry);
        }

        // Sort by group then descending required_level — same group
        // grouped together with highest tier first (typical "tier 1
        // before tier 2" display order).
        out.sort_by(|a, b| {
            a.group
                .cmp(&b.group)
                .then(b.required_level.cmp(&a.required_level))
                .then(a.id.cmp(&b.id))
        });

        let total = out.len();
        out.truncate(limit.clamp(1, 200));
        Ok(ModsForBaseResult {
            game,
            base_id: base_id.to_string(),
            influence: influence.map(|s| s.to_string()),
            base_tags,
            total_matched: total,
            mods: out,
        })
    }

    pub fn lookup_by_name(
        &self,
        game: Game,
        category: Category,
        name: &str,
        limit: usize,
    ) -> Result<LookupResult> {
        let snap = self.snapshot(game, category)?;
        let limit = limit.clamp(1, 20);
        let needle = tokenize(name);
        if needle.is_empty() {
            return Ok(LookupResult::from_snap(&snap, Vec::new()));
        }
        let mut scored: Vec<(f32, &String, &Value)> = snap
            .entries
            .iter()
            .filter_map(|(k, v)| {
                let score = score_entry(v, k, &needle);
                if score > 0.0 {
                    Some((score, k, v))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.cmp(b.1))
        });
        let entries = scored
            .into_iter()
            .take(limit)
            .map(|(score, id, value)| LookupEntry {
                id: id.clone(),
                score,
                value: value.clone(),
            })
            .collect();
        Ok(LookupResult::from_snap(&snap, entries))
    }
}

/// One mod row in a `mods_for_base` join. `weight` is the effective
/// spawn weight for THIS base (not the raw `default` weight from the
/// mod entry). `stats` keeps the raw RePoE stat array so callers can
/// read ranges and stat ids.
#[derive(Debug, Serialize)]
pub struct ModForBaseEntry {
    pub id: String,
    pub name: String,
    pub mod_type: String,
    pub group: String,
    pub generation_type: String,
    pub required_level: u32,
    pub weight: i64,
    pub text: String,
    pub stats: Value,
}

#[derive(Debug, Serialize)]
pub struct ModsForBaseResult {
    pub game: Game,
    pub base_id: String,
    pub influence: Option<String>,
    /// The expanded tag set used for matching (base tags plus any
    /// influence virtual tags). Exposed so the caller can confirm
    /// the join was set up the way they expected.
    pub base_tags: Vec<String>,
    pub total_matched: usize,
    pub mods: Vec<ModForBaseEntry>,
}

#[derive(Debug, Serialize)]
pub struct LookupEntry {
    pub id: String,
    pub score: f32,
    pub value: Value,
}

#[derive(Debug, Serialize)]
pub struct LookupResult {
    pub game: Game,
    pub category: Category,
    pub source: SnapshotSource,
    pub fetched_at: u64,
    pub entries: Vec<LookupEntry>,
    pub warnings: Vec<String>,
}

impl LookupResult {
    fn from_snap(snap: &CategorySnapshot, entries: Vec<LookupEntry>) -> Self {
        let mut warnings = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if snap.source == SnapshotSource::Bundled
            && now > snap.fetched_at
            && now - snap.fetched_at > 30 * 24 * 3600
        {
            warnings.push(format!(
                "bundled snapshot fetched at unix={} is older than 30 days; refresh task may be stale",
                snap.fetched_at
            ));
        }
        Self {
            game: snap.game,
            category: snap.category,
            source: snap.source,
            fetched_at: snap.fetched_at,
            entries,
            warnings,
        }
    }
}

pub fn decode_zstd_json(bytes: &[u8]) -> Result<HashMap<String, Value>> {
    let raw = zstd::decode_all(bytes).context("zstd decode repoe snapshot")?;
    let parsed: Value = serde_json::from_slice(&raw).context("parse repoe snapshot json")?;
    let map = match parsed {
        Value::Object(map) => map,
        _ => return Err(anyhow!("repoe snapshot top-level must be a JSON object")),
    };
    Ok(map.into_iter().collect())
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

fn score_entry(value: &Value, id: &str, needle: &[String]) -> f32 {
    let haystack = collect_searchable_strings(value, id);
    if haystack.is_empty() {
        return 0.0;
    }
    let hay_tokens: Vec<String> = haystack
        .iter()
        .flat_map(|s| tokenize(s).into_iter())
        .collect();
    if hay_tokens.is_empty() {
        return 0.0;
    }
    let mut hits = 0;
    for n in needle {
        if hay_tokens.iter().any(|h| h == n) {
            hits += 1;
        }
    }
    if hits == 0 {
        return 0.0;
    }
    let recall = hits as f32 / needle.len() as f32;
    let mut score = recall;
    let id_lower = id.to_lowercase();
    if needle.iter().any(|n| id_lower.contains(n)) {
        score += 0.05;
    }
    let joined = haystack.join(" ").to_lowercase();
    if needle.iter().all(|n| joined.contains(n)) {
        score += 0.10;
    }
    score
}

fn collect_searchable_strings(value: &Value, id: &str) -> Vec<String> {
    let mut out = vec![id.to_string()];
    push_strings(value, &mut out, 0);
    out
}

fn push_strings(value: &Value, out: &mut Vec<String>, depth: u8) {
    if depth > 4 {
        return;
    }
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(arr) => {
            for v in arr {
                push_strings(v, out, depth + 1);
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                if matches!(
                    k.as_str(),
                    "name" | "english" | "id" | "domain" | "type" | "implicit" | "description"
                ) {
                    if let Value::String(s) = v {
                        out.push(s.clone());
                        continue;
                    }
                }
                push_strings(v, out, depth + 1);
            }
        }
        _ => {}
    }
}

/// Read the `tags` array from a base-item RePoE entry, plus optional
/// influence virtual tags. Returns lowercase-trimmed strings.
///
/// Influence tags are synthesised based on the GGG mod-spawning model:
/// influence mods reference tags like `shaper_2h_weapon`, `elder_helmet`,
/// `crusader_amulet`, etc. The pattern is `{influence}_{class_tag}`
/// where `class_tag` is the item-class subset of the base's own tags
/// (e.g. `2h_weapon`, `helmet`, `ring`). We expand every reasonable
/// candidate; surplus tags are harmless because mods only check the
/// ones they reference.
fn extract_base_tags(base_value: &Value, influence: Option<&str>) -> Vec<String> {
    let mut tags: Vec<String> = base_value
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if let Some(inf) = influence
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
    {
        // Generic `<inf>_item` is referenced by some influence mods.
        tags.push(format!("{inf}_item"));
        // `<inf>_<class_tag>` for every existing tag — duplicates are
        // harmless and the cross-product covers GGG's varied naming.
        let snapshot = tags.clone();
        for t in snapshot {
            tags.push(format!("{inf}_{t}"));
        }
    }
    // Always include `default` — GGG falls back to the catch-all entry
    // when no class-specific tag matches.
    if !tags.iter().any(|t| t == "default") {
        tags.push("default".to_string());
    }
    tags.sort();
    tags.dedup();
    tags
}

/// Walk `spawn_weights` in array order — that's exactly how GGG resolves
/// a mod's effective weight on a given item. The FIRST matching tag
/// wins, regardless of subsequent entries. A weight of 0 explicitly
/// blocks the mod (used by entries like `{"tag":"two_hand_weapon","weight":0}`
/// to exclude two-handers from a one-hander-only mod).
fn effective_spawn_weight(mod_value: &Value, base_tags: &[String]) -> i64 {
    let weights = match mod_value.get("spawn_weights").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return 0,
    };
    let tag_set: std::collections::HashSet<&str> = base_tags.iter().map(|s| s.as_str()).collect();
    for entry in weights {
        let tag = entry.get("tag").and_then(|v| v.as_str()).unwrap_or("");
        if tag_set.contains(tag) {
            return entry.get("weight").and_then(|v| v.as_i64()).unwrap_or(0);
        }
    }
    0
}

const BUNDLE_FETCHED_AT: u64 = include_bundle_fetched_at();

const fn include_bundle_fetched_at() -> u64 {
    // 2026-05-08 00:00:00 UTC. Refreshed each time `scripts/refresh-snapshots.ps1`
    // bumps the bundled blobs.
    1_778_198_400
}

fn bundled_blob(game: Game, category: Category) -> Option<&'static [u8]> {
    match (game, category) {
        (Game::Poe1, Category::Mods) => Some(include_bytes!("snapshots/poe1/mods.json.zst")),
        (Game::Poe1, Category::BaseItems) => {
            Some(include_bytes!("snapshots/poe1/base_items.json.zst"))
        }
        (Game::Poe1, Category::Gems) => Some(include_bytes!("snapshots/poe1/gems.json.zst")),
        (Game::Poe1, Category::Uniques) => Some(include_bytes!("snapshots/poe1/uniques.json.zst")),
        (Game::Poe1, Category::ClusterJewels) => {
            Some(include_bytes!("snapshots/poe1/cluster_jewels.json.zst"))
        }
        (Game::Poe1, Category::Essences) => {
            Some(include_bytes!("snapshots/poe1/essences.json.zst"))
        }
        (Game::Poe1, Category::Fossils) => Some(include_bytes!("snapshots/poe1/fossils.json.zst")),
        (Game::Poe1, Category::StatTranslations) => {
            Some(include_bytes!("snapshots/poe1/stat_translations.json.zst"))
        }
        (Game::Poe2, Category::Mods) => Some(include_bytes!("snapshots/poe2/mods.json.zst")),
        (Game::Poe2, Category::BaseItems) => {
            Some(include_bytes!("snapshots/poe2/base_items.json.zst"))
        }
        (Game::Poe2, Category::Gems) => Some(include_bytes!("snapshots/poe2/gems.json.zst")),
        (Game::Poe2, Category::Uniques) => Some(include_bytes!("snapshots/poe2/uniques.json.zst")),
        (Game::Poe2, Category::StatTranslations) => {
            Some(include_bytes!("snapshots/poe2/stat_translations.json.zst"))
        }
        (Game::Poe2, _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_available_for_poe2_excludes_poe1_only() {
        assert!(Category::Mods.available_for(Game::Poe2));
        assert!(Category::BaseItems.available_for(Game::Poe2));
        assert!(Category::Uniques.available_for(Game::Poe2));
        assert!(!Category::ClusterJewels.available_for(Game::Poe2));
        assert!(!Category::Essences.available_for(Game::Poe2));
        assert!(!Category::Fossils.available_for(Game::Poe2));
        assert!(!Category::Gems.available_for(Game::Poe2));
        assert!(!Category::StatTranslations.available_for(Game::Poe2));
        for cat in Category::ALL {
            assert!(cat.available_for(Game::Poe1));
        }
    }

    #[test]
    fn tokenize_splits_punctuation() {
        let toks = tokenize("Adds 1 to (5-10) Lightning Damage");
        assert!(toks.contains(&"adds".to_string()));
        assert!(toks.contains(&"lightning".to_string()));
        assert!(toks.contains(&"damage".to_string()));
    }

    #[test]
    fn score_entry_ranks_full_match_above_partial() {
        let v_full = serde_json::json!({"name": "Stellar Amulet"});
        let v_part = serde_json::json!({"name": "Amber Amulet"});
        let needle = tokenize("stellar amulet");
        let s_full = score_entry(&v_full, "Metadata/Items/Amulets/AmuletStellar", &needle);
        let s_part = score_entry(&v_part, "Metadata/Items/Amulets/AmuletAmber", &needle);
        assert!(s_full > s_part);
    }

    fn test_client() -> RepoeClient {
        let tmp = std::env::temp_dir().join(format!(
            "bestel-repoe-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        RepoeClient::new(FileCache::new(tmp))
    }

    #[test]
    fn bundled_poe1_uniques_decompresses_with_entries() {
        let client = test_client();
        let snap = client.snapshot(Game::Poe1, Category::Uniques).unwrap();
        assert_eq!(snap.source, SnapshotSource::Bundled);
        assert!(
            snap.entries.len() > 100,
            "expected real uniques bundle, got {}",
            snap.entries.len()
        );
    }

    #[test]
    fn lookup_by_name_finds_known_unique() {
        let client = test_client();
        let result = client
            .lookup_by_name(Game::Poe1, Category::Uniques, "headhunter", 5)
            .unwrap();
        assert!(
            result.entries.iter().any(|e| {
                let id_lower = e.id.to_lowercase();
                let name_lower = e
                    .value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                id_lower.contains("headhunter") || name_lower.contains("headhunter")
            }),
            "expected to find Headhunter; got entries: {:?}",
            result.entries.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn extract_base_tags_no_influence_returns_tags_plus_default() {
        let v = serde_json::json!({"tags": ["sword", "weapon", "one_hand_sword"]});
        let tags = extract_base_tags(&v, None);
        assert!(tags.iter().any(|t| t == "sword"));
        assert!(tags.iter().any(|t| t == "one_hand_sword"));
        assert!(tags.iter().any(|t| t == "default"));
        // No influence tag synthesised.
        assert!(!tags.iter().any(|t| t.starts_with("shaper_")));
    }

    #[test]
    fn extract_base_tags_with_influence_synthesises_virtual_tags() {
        let v = serde_json::json!({"tags": ["helmet", "armour", "str_armour"]});
        let tags = extract_base_tags(&v, Some("shaper"));
        assert!(tags.iter().any(|t| t == "shaper_helmet"));
        assert!(tags.iter().any(|t| t == "shaper_armour"));
        assert!(tags.iter().any(|t| t == "shaper_item"));
        assert!(tags.iter().any(|t| t == "default"));
    }

    #[test]
    fn effective_spawn_weight_picks_first_matching_tag() {
        let v = serde_json::json!({
            "spawn_weights": [
                {"tag": "two_hand_weapon", "weight": 0},
                {"tag": "weapon", "weight": 1000},
                {"tag": "default", "weight": 500}
            ]
        });
        // Both `two_hand_weapon` and `weapon` are in the base tags; GGG
        // takes the first match, which is the 0-weight block. This is
        // the mechanism used to exclude two-handers from one-hand-only
        // mods.
        let w = effective_spawn_weight(
            &v,
            &["two_hand_weapon".into(), "weapon".into(), "default".into()],
        );
        assert_eq!(w, 0);

        // Without `two_hand_weapon` in the base tag set, the weapon
        // weight (1000) is picked.
        let w2 = effective_spawn_weight(&v, &["weapon".into(), "default".into()]);
        assert_eq!(w2, 1000);

        // No matching tag at all → 0 (mod can't roll on this base).
        let w3 = effective_spawn_weight(&v, &["jewel".into()]);
        assert_eq!(w3, 0);
    }

    #[test]
    fn mods_for_base_filters_by_spawn_weight_and_excludes_essence_only() {
        // Mini in-memory snapshot of one base + three mods. Use the
        // public APIs by inserting into a fresh RepoeClient's snapshot
        // map — saves us from depending on the actual bundle.
        use std::collections::HashMap;

        let base_id = "Metadata/Items/Weapons/OneHandWeapons/OneHandSwords/OneHandSwordTest";
        let mut bases = HashMap::new();
        bases.insert(
            base_id.to_string(),
            serde_json::json!({
                "tags": ["sword", "one_hand_sword", "weapon", "default"]
            }),
        );
        let mut mods = HashMap::new();
        mods.insert(
            "RollsOnSwords".to_string(),
            serde_json::json!({
                "domain": "item",
                "spawn_weights": [{"tag": "sword", "weight": 800}],
                "name": "of Swiftness",
                "generation_type": "suffix",
                "required_level": 20,
                "groups": ["AttackSpeed"],
                "type": "LocalAttackSpeed",
                "text": "+#% Attack Speed",
                "stats": [],
                "is_essence_only": false
            }),
        );
        mods.insert(
            "BlockedByZeroWeight".to_string(),
            serde_json::json!({
                "domain": "item",
                "spawn_weights": [{"tag": "two_hand_weapon", "weight": 0}],
                "name": "of Nothing",
                "groups": [], "generation_type": "suffix",
                "type": "X", "text": "X", "stats": [], "required_level": 1,
                "is_essence_only": false
            }),
        );
        mods.insert(
            "EssenceOnly".to_string(),
            serde_json::json!({
                "domain": "item",
                "spawn_weights": [{"tag": "sword", "weight": 500}],
                "name": "of Essence",
                "groups": [], "generation_type": "suffix",
                "type": "X", "text": "X", "stats": [], "required_level": 1,
                "is_essence_only": true
            }),
        );

        let client = test_client();
        client.install(Arc::new(CategorySnapshot {
            game: Game::Poe1,
            category: Category::BaseItems,
            entries: bases,
            fetched_at: 0,
            source: SnapshotSource::Refreshed,
        }));
        client.install(Arc::new(CategorySnapshot {
            game: Game::Poe1,
            category: Category::Mods,
            entries: mods,
            fetched_at: 0,
            source: SnapshotSource::Refreshed,
        }));

        let res = client.mods_for_base(Game::Poe1, base_id, None, 50).unwrap();
        let ids: Vec<&str> = res.mods.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"RollsOnSwords"), "got: {ids:?}");
        assert!(!ids.contains(&"BlockedByZeroWeight"), "0-weight blocked");
        assert!(!ids.contains(&"EssenceOnly"), "essence_only excluded");
        let m = res.mods.iter().find(|m| m.id == "RollsOnSwords").unwrap();
        assert_eq!(m.weight, 800);
        assert_eq!(m.required_level, 20);
    }

    #[test]
    fn lookup_unavailable_category_for_poe2_errors_explicitly() {
        let client = test_client();
        let err = client
            .snapshot(Game::Poe2, Category::Fossils)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not available for poe2"),
            "expected explicit guard, got: {err}"
        );
    }
}
