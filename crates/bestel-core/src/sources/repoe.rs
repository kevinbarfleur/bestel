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

        if let Some(existing) = self.map().read().ok().and_then(|m| m.get(&(game, category)).cloned()) {
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

    pub fn lookup_by_id(
        &self,
        game: Game,
        category: Category,
        id: &str,
    ) -> Result<LookupResult> {
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
        (Game::Poe1, Category::Uniques) => {
            Some(include_bytes!("snapshots/poe1/uniques.json.zst"))
        }
        (Game::Poe1, Category::ClusterJewels) => {
            Some(include_bytes!("snapshots/poe1/cluster_jewels.json.zst"))
        }
        (Game::Poe1, Category::Essences) => {
            Some(include_bytes!("snapshots/poe1/essences.json.zst"))
        }
        (Game::Poe1, Category::Fossils) => {
            Some(include_bytes!("snapshots/poe1/fossils.json.zst"))
        }
        (Game::Poe1, Category::StatTranslations) => {
            Some(include_bytes!("snapshots/poe1/stat_translations.json.zst"))
        }
        (Game::Poe2, Category::Mods) => Some(include_bytes!("snapshots/poe2/mods.json.zst")),
        (Game::Poe2, Category::BaseItems) => {
            Some(include_bytes!("snapshots/poe2/base_items.json.zst"))
        }
        (Game::Poe2, Category::Gems) => Some(include_bytes!("snapshots/poe2/gems.json.zst")),
        (Game::Poe2, Category::Uniques) => {
            Some(include_bytes!("snapshots/poe2/uniques.json.zst"))
        }
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
            result
                .entries
                .iter()
                .map(|e| &e.id)
                .collect::<Vec<_>>()
        );
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
