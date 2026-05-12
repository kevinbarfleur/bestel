//! PoEDB / POE2DB HTML scraper.
//!
//! PoEDB has no public JSON API — every page is server-rendered HTML.
//! We narrow scope to the few pages that expose **curated derived
//! views** RePoE doesn't pre-compute: the global crafting-bench mod
//! table, per-class crafting-bench subsets, and per-skill-gem level
//! progression tables.
//!
//! v1 categories:
//!   - `craft_bench`               → `/us/Crafting_Bench`
//!   - `craft_bench_for_class`     → `/us/{Item_Class_Plural}`  (e.g. `Rings`)
//!   - `skill_gem`                 → `/us/{Gem_Name}`           (e.g. `Molten_Strike`)
//!
//! Influence-mod-pool views are NOT exposed by PoEDB as a single page;
//! that join is handled by `repoe::mods_for_base` instead.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::pob::PoeVersion;
use crate::sources::cache::FileCache;
use crate::sources::http::PoeHttpClient;

/// Pages on PoEDB are tied to patch releases; a daily cache window
/// keeps things fresh enough for week-of-league questions while
/// avoiding hot HTTP loops when the agent calls multiple times per
/// turn.
const POEDB_TTL: Duration = Duration::from_secs(24 * 3600);

#[derive(Clone)]
pub struct PoedbClient {
    http: PoeHttpClient,
    cache: FileCache,
    game: PoeVersion,
}

/// One bench row. `item_classes` is split on `·` (the bullet separator
/// PoEDB renders between class names) so callers get a clean list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftBenchRow {
    pub mod_text: String,
    pub require: String,
    pub item_classes: Vec<String>,
    pub unlock: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftBenchTable {
    pub source_url: String,
    pub rows: Vec<CraftBenchRow>,
}

/// One skill-gem level row. Optional columns survive missing values
/// (e.g. a gem with no Str requirement leaves `str` as None).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillGemLevel {
    pub level: u32,
    pub requires_level: Option<u32>,
    pub str_req: Option<i32>,
    pub dex_req: Option<i32>,
    pub int_req: Option<i32>,
    pub mana_cost: Option<String>,
    pub base_damage: Option<String>,
    pub experience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: String,
    pub changes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillGemDetail {
    pub gem_name: String,
    pub source_url: String,
    pub progression: Vec<SkillGemLevel>,
    pub implicit_mods: Vec<String>,
    pub version_history: Vec<VersionEntry>,
}

impl PoedbClient {
    pub fn new(http: PoeHttpClient, cache: FileCache, game: PoeVersion) -> Self {
        Self { http, cache, game }
    }

    fn host(&self) -> &'static str {
        match self.game {
            PoeVersion::Poe1 => "https://poedb.tw/us",
            PoeVersion::Poe2 => "https://poe2db.tw/us",
        }
    }

    /// URL-encode a page slug. PoEDB uses underscores between words and
    /// percent-encodes special chars; we mirror that by replacing spaces
    /// with `_` first, then percent-encoding the rest.
    fn slug(&self, raw: &str) -> String {
        let with_underscores = raw.trim().replace(' ', "_");
        urlencoding::encode(&with_underscores).into_owned()
    }

    /// Fetch `<host>/<slug>` with a 24 h on-disk cache. Returns the raw
    /// HTML. 4xx/5xx responses bubble up with the GGG-style chain.
    async fn fetch_cached(&self, slug: &str) -> Result<(String, String)> {
        let url = format!("{}/{}", self.host(), slug);
        let key = format!("poedb:{:?}:{}", self.game, slug);
        if let Some(body) = self.cache.get::<String>(&key, POEDB_TTL).await {
            return Ok((url, body));
        }
        let body = self
            .http
            .get_text(&url, "poedb")
            .await
            .with_context(|| format!("fetch {url}"))?;
        let _ = self.cache.put(&key, &body).await;
        Ok((url, body))
    }

    pub async fn craft_bench(&self) -> Result<CraftBenchTable> {
        let (url, html) = self.fetch_cached("Crafting_Bench").await?;
        let rows = parse_craft_bench_table(&html)
            .with_context(|| "parse Crafting_Bench page")?;
        Ok(CraftBenchTable {
            source_url: url,
            rows,
        })
    }

    pub async fn craft_bench_for_class(&self, item_class_plural: &str) -> Result<CraftBenchTable> {
        let slug = self.slug(item_class_plural);
        let (url, html) = self.fetch_cached(&slug).await?;
        let rows = parse_craft_bench_table(&html)
            .with_context(|| format!("parse {slug} page (no bench table found)"))?;
        Ok(CraftBenchTable {
            source_url: url,
            rows,
        })
    }

    pub async fn skill_gem(&self, gem_name: &str) -> Result<SkillGemDetail> {
        let slug = self.slug(gem_name);
        let (url, html) = self.fetch_cached(&slug).await?;
        let doc = Html::parse_document(&html);
        let progression = parse_gem_progression(&doc)
            .with_context(|| format!("parse skill gem progression for {gem_name}"))?;
        let implicit_mods = parse_gem_implicits(&doc);
        let version_history = parse_gem_version_history(&doc);
        Ok(SkillGemDetail {
            gem_name: gem_name.to_string(),
            source_url: url,
            progression,
            implicit_mods,
            version_history,
        })
    }
}

// ─── Parsers ─────────────────────────────────────────────────────────

/// Walk every `<table>` and return the first one whose `<thead>`
/// matches a crafting-bench schema. The global page `/us/Crafting_Bench`
/// uses `[Mod, Require, ItemClasses, Unlock]`; per-class pages like
/// `/us/Rings` drop the redundant `ItemClasses` column and ship
/// `[Mod, Require, Unlock]` instead. We accept both — when the
/// `ItemClasses` column is absent, `item_classes` is left empty (the
/// caller already knows the class from its query).
fn parse_craft_bench_table(html: &str) -> Result<Vec<CraftBenchRow>> {
    let doc = Html::parse_document(html);
    let table_sel = Selector::parse("table").unwrap();
    let head_th_sel = Selector::parse("thead th").unwrap();
    let body_row_sel = Selector::parse("tbody tr").unwrap();
    let cell_sel = Selector::parse("td").unwrap();

    for table in doc.select(&table_sel) {
        let headers: Vec<String> = table
            .select(&head_th_sel)
            .map(|th| normalize_text(&th))
            .collect();
        // Map header names to column indices so we don't break when
        // PoEDB shuffles or renames a column.
        let idx_mod = headers.iter().position(|h| h.eq_ignore_ascii_case("Mod"));
        let idx_req = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("Require"));
        let idx_unlock = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("Unlock"));
        let idx_classes = headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case("ItemClasses"));
        // Required columns. `ItemClasses` is optional.
        let (Some(i_mod), Some(i_req), Some(i_unlock)) = (idx_mod, idx_req, idx_unlock) else {
            continue;
        };
        let mut rows = Vec::new();
        for tr in table.select(&body_row_sel) {
            let cells: Vec<String> = tr.select(&cell_sel).map(|td| normalize_text(&td)).collect();
            let Some(mod_text) = cells.get(i_mod).cloned() else {
                continue;
            };
            if mod_text.is_empty() {
                continue;
            }
            let require = cells.get(i_req).cloned().unwrap_or_default();
            let unlock = cells.get(i_unlock).cloned().unwrap_or_default();
            let item_classes = idx_classes
                .and_then(|i| cells.get(i))
                .map(|s| split_bullet(s))
                .unwrap_or_default();
            rows.push(CraftBenchRow {
                mod_text,
                require,
                item_classes,
                unlock,
            });
        }
        return Ok(rows);
    }
    Err(anyhow!(
        "no table with crafting-bench headers (Mod, Require, Unlock) found"
    ))
}

/// Skill-gem level progression. The table headers PoEDB uses are:
/// `[Level, Requires Level, (Str|Dex|Int)*, Mana, Base Damage, Experience]`.
/// We match by recognising the first two columns and parsing the rest
/// best-effort.
fn parse_gem_progression(doc: &Html) -> Result<Vec<SkillGemLevel>> {
    let table_sel = Selector::parse("table").unwrap();
    let head_th_sel = Selector::parse("thead th").unwrap();
    let body_row_sel = Selector::parse("tbody tr").unwrap();
    let cell_sel = Selector::parse("td").unwrap();

    for table in doc.select(&table_sel) {
        let headers: Vec<String> = table
            .select(&head_th_sel)
            .map(|th| normalize_text(&th))
            .collect();
        if headers.len() < 3
            || !headers[0].eq_ignore_ascii_case("Level")
            || !headers
                .get(1)
                .map(|h| h.to_lowercase().contains("requires"))
                .unwrap_or(false)
        {
            continue;
        }
        let col_idx = |needle: &str| -> Option<usize> {
            headers
                .iter()
                .position(|h| h.eq_ignore_ascii_case(needle))
        };
        let i_str = col_idx("Str");
        let i_dex = col_idx("Dex");
        let i_int = col_idx("Int");
        let i_mana = col_idx("Mana")
            .or_else(|| col_idx("Mana Cost"))
            .or_else(|| col_idx("Cost"));
        let i_dmg = headers
            .iter()
            .position(|h| h.to_lowercase().contains("damage"));
        let i_exp = col_idx("Experience").or_else(|| col_idx("Exp"));

        let mut out = Vec::new();
        for tr in table.select(&body_row_sel) {
            let cells: Vec<String> = tr.select(&cell_sel).map(|td| normalize_text(&td)).collect();
            if cells.len() < 2 {
                continue;
            }
            let level = match cells[0].parse::<u32>() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let requires_level = cells[1].parse::<u32>().ok();
            out.push(SkillGemLevel {
                level,
                requires_level,
                str_req: i_str.and_then(|i| cells.get(i)).and_then(|s| s.parse().ok()),
                dex_req: i_dex.and_then(|i| cells.get(i)).and_then(|s| s.parse().ok()),
                int_req: i_int.and_then(|i| cells.get(i)).and_then(|s| s.parse().ok()),
                mana_cost: i_mana
                    .and_then(|i| cells.get(i))
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty()),
                base_damage: i_dmg
                    .and_then(|i| cells.get(i))
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty()),
                experience: i_exp
                    .and_then(|i| cells.get(i))
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty()),
            });
        }
        return Ok(out);
    }
    Err(anyhow!("no skill-gem progression table found"))
}

/// Gem implicit mods. PoEDB renders them in a tiny single-column table
/// captioned `Implicit`. Best-effort: empty vec if not present.
fn parse_gem_implicits(doc: &Html) -> Vec<String> {
    let table_sel = Selector::parse("table").unwrap();
    let head_th_sel = Selector::parse("thead th").unwrap();
    let body_row_sel = Selector::parse("tbody tr").unwrap();
    let cell_sel = Selector::parse("td").unwrap();
    for table in doc.select(&table_sel) {
        let headers: Vec<String> = table
            .select(&head_th_sel)
            .map(|th| normalize_text(&th))
            .collect();
        if headers.len() == 1 && headers[0].eq_ignore_ascii_case("Implicit") {
            return table
                .select(&body_row_sel)
                .flat_map(|tr| tr.select(&cell_sel).map(|td| normalize_text(&td)))
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

/// Patch-version changelog tables — headers `[Version, Changes]`.
/// Returns the first match's rows; PoEDB sometimes splits into two
/// (vanilla + transfigured), we surface the vanilla one.
fn parse_gem_version_history(doc: &Html) -> Vec<VersionEntry> {
    let table_sel = Selector::parse("table").unwrap();
    let head_th_sel = Selector::parse("thead th").unwrap();
    let body_row_sel = Selector::parse("tbody tr").unwrap();
    let cell_sel = Selector::parse("td").unwrap();
    for table in doc.select(&table_sel) {
        let headers: Vec<String> = table
            .select(&head_th_sel)
            .map(|th| normalize_text(&th))
            .collect();
        if headers != ["Version", "Changes"] {
            continue;
        }
        return table
            .select(&body_row_sel)
            .filter_map(|tr| {
                let cells: Vec<String> = tr.select(&cell_sel).map(|td| normalize_text(&td)).collect();
                if cells.len() < 2 {
                    return None;
                }
                Some(VersionEntry {
                    version: cells[0].clone(),
                    changes: cells[1].clone(),
                })
            })
            .collect();
    }
    Vec::new()
}

/// Strip HTML, collapse whitespace, trim. PoEDB cells embed anchors and
/// sprite icons; we only want the visible text.
fn normalize_text(el: &ElementRef<'_>) -> String {
    el.text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Split a bullet-separated list like `"One Hand Melee · Two Hand Melee · Body Armour"`
/// into individual entries. PoEDB uses U+00B7 (middle dot) but some
/// older pages use ASCII `·` byte-shifted; we accept both.
fn split_bullet(s: &str) -> Vec<String> {
    s.split(|c: char| c == '·' || c == '\u{B7}' || c == '|')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_craft_bench_minimal_table() {
        let html = r#"
            <html><body>
              <table>
                <thead><tr><th>Mod</th><th>Require</th><th>ItemClasses</th><th>Unlock</th></tr></thead>
                <tbody>
                  <tr>
                    <td>Two Sockets</td>
                    <td>1x <span>Jeweller's Orb</span></td>
                    <td>One Hand Melee · Two Hand Melee · Body Armour</td>
                    <td>The Grand Arena</td>
                  </tr>
                  <tr>
                    <td>+10 to Strength</td>
                    <td>2x Augmentation</td>
                    <td>Rings</td>
                    <td>Default</td>
                  </tr>
                </tbody>
              </table>
            </body></html>
        "#;
        let rows = parse_craft_bench_table(html).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].mod_text, "Two Sockets");
        assert_eq!(rows[0].item_classes, vec!["One Hand Melee", "Two Hand Melee", "Body Armour"]);
        assert_eq!(rows[0].unlock, "The Grand Arena");
        assert_eq!(rows[1].mod_text, "+10 to Strength");
        assert_eq!(rows[1].item_classes, vec!["Rings"]);
    }

    #[test]
    fn parse_craft_bench_accepts_per_class_three_column_schema() {
        // /us/Rings drops the ItemClasses column (the class is implicit).
        let html = r#"
            <html><body>
              <table>
                <thead><tr><th>Mod</th><th>Require</th><th>Unlock</th></tr></thead>
                <tbody>
                  <tr><td>+10 to Strength</td><td>2x Augmentation</td><td>Default</td></tr>
                  <tr><td>+5% Fire Resistance</td><td>1x Alteration</td><td>The Caverns</td></tr>
                </tbody>
              </table>
            </body></html>
        "#;
        let rows = parse_craft_bench_table(html).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].mod_text, "+10 to Strength");
        assert!(rows[0].item_classes.is_empty(), "implicit class -> empty");
        assert_eq!(rows[1].unlock, "The Caverns");
    }

    #[test]
    fn parse_craft_bench_returns_err_when_no_matching_table() {
        let html = r#"<html><body><table><thead><tr><th>foo</th></tr></thead></table></body></html>"#;
        assert!(parse_craft_bench_table(html).is_err());
    }

    #[test]
    fn parse_skill_gem_progression_extracts_levels() {
        let html = r#"
            <html><body>
              <table>
                <thead><tr>
                  <th>Level</th><th>Requires Level</th><th>Str</th>
                  <th>Mana</th><th>Base Damage</th><th>Experience</th>
                </tr></thead>
                <tbody>
                  <tr><td>1</td><td>12</td><td>33</td><td>6</td><td>120%</td><td>15,249</td></tr>
                  <tr><td>2</td><td>15</td><td>39</td><td>6</td><td>124%</td><td>24,165</td></tr>
                </tbody>
              </table>
            </body></html>
        "#;
        let doc = Html::parse_document(html);
        let prog = parse_gem_progression(&doc).unwrap();
        assert_eq!(prog.len(), 2);
        assert_eq!(prog[0].level, 1);
        assert_eq!(prog[0].requires_level, Some(12));
        assert_eq!(prog[0].str_req, Some(33));
        assert_eq!(prog[0].mana_cost.as_deref(), Some("6"));
        assert_eq!(prog[0].base_damage.as_deref(), Some("120%"));
        assert_eq!(prog[0].experience.as_deref(), Some("15,249"));
        assert_eq!(prog[1].level, 2);
    }

    #[test]
    fn parse_skill_gem_implicits() {
        let html = r#"
            <html><body>
              <table>
                <thead><tr><th>Implicit</th></tr></thead>
                <tbody>
                  <tr><td>Deals 120% of Base Attack Damage</td></tr>
                  <tr><td>30% less Attack Speed</td></tr>
                </tbody>
              </table>
            </body></html>
        "#;
        let doc = Html::parse_document(html);
        let im = parse_gem_implicits(&doc);
        assert_eq!(im, vec![
            "Deals 120% of Base Attack Damage",
            "30% less Attack Speed",
        ]);
    }

    #[test]
    fn parse_version_history_extracts_rows() {
        let html = r#"
            <html><body>
              <table>
                <thead><tr><th>Version</th><th>Changes</th></tr></thead>
                <tbody>
                  <tr><td>3.25.0</td><td>Base damage increased.</td></tr>
                  <tr><td>3.24.0</td><td>Renamed.</td></tr>
                </tbody>
              </table>
            </body></html>
        "#;
        let doc = Html::parse_document(html);
        let hist = parse_gem_version_history(&doc);
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].version, "3.25.0");
        assert_eq!(hist[0].changes, "Base damage increased.");
    }

    #[test]
    fn split_bullet_handles_middle_dot_and_pipe() {
        assert_eq!(split_bullet("a · b · c"), vec!["a", "b", "c"]);
        assert_eq!(split_bullet("a | b"), vec!["a", "b"]);
        assert_eq!(split_bullet("solo"), vec!["solo"]);
    }
}
