use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::pob::PobBuild;

pub const GET_ACTIVE_BUILD: &str = "get_active_build";
pub const FIND_SYNERGIES: &str = "find_synergies";

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

/// Tool context hands every dispatched tool the things it needs. For now,
/// only `get_active_build` is registered, so the context is just the build.
/// Future tools (none planned in Phase 5) would extend this.
#[derive(Clone)]
pub struct ToolCtx {
    pub build: BuildContext,
}

impl ToolCtx {
    pub fn new(build: BuildContext) -> Result<Self> {
        Ok(Self { build })
    }
}

pub fn tool_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": GET_ACTIVE_BUILD,
            "description": "Returns the exile's currently loaded Path of Building build: game (PoE1/PoE2), class, ascendancy, level, main skill, full skill groups with linked gems, every item with its full text, key defensive stats (life, mana, ES, EHP, armour, evasion, suppression, block, dodge), per-element resistances and max-hit values, charges (power/frenzy/endurance current+max), active buffs (combat/buff/curse lists), config (boss profile, enemy resists, flask uptimes, custom mods), and passive tree summary (class/ascend IDs, version, node and mastery counts, weapon-set node split). Always call this BEFORE making any claim about the exile's character. No arguments."
            ,
            "input_schema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": FIND_SYNERGIES,
            "description": "Queries the Path of Exile wiki's reverse-link index (Special:WhatLinksHere) for a given topic. Returns the list of pages that link TO the topic — uniques, passive skills, cluster jewel notables, ascendancy nodes, modifiers, mechanics pages — even when the user did not name them. Use this AFTER understanding the core mechanic to surface synergies a creator-grade answer should mention (e.g. find_synergies(topic='Divine Flesh') reveals The Fourth Vow, Mahuxotl's Machination, Born of Chaos). One call replaces a manual web_fetch of '/index.php?title=Special:WhatLinksHere/<topic>&limit=500'. Filtered to drop wiki meta pages (User:/Talk:/File:/Template:/Category:/Help:/Special:/Module:/MediaWiki:).",
            "input_schema": {
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "Canonical wiki page name. Spaces are accepted and are converted to underscores. Examples: 'Divine Flesh', 'Mind Over Matter', 'The Fourth Vow', 'Resolute Technique', 'Mageblood'."
                    },
                    "game": {
                        "type": "string",
                        "enum": ["poe1", "poe2"],
                        "description": "Which wiki to query. 'poe1' uses poewiki.net, 'poe2' uses poe2wiki.net. Defaults to 'poe1' when omitted.",
                        "default": "poe1"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of entries to return (after filtering). Defaults to 80; clamped to [1, 200].",
                        "default": 80,
                        "minimum": 1,
                        "maximum": 200
                    }
                },
                "required": ["topic"],
                "additionalProperties": false
            }
        }),
    ]
}

pub async fn dispatch(name: &str, input: &Value, ctx: &ToolCtx) -> Result<String> {
    match name {
        GET_ACTIVE_BUILD => Ok(ctx.build.render_tool_result()),
        FIND_SYNERGIES => {
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
        other => Err(anyhow!("unknown tool '{other}'")),
    }
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

    let value = serde_json::Value::Object(summary);
    let s = serde_json::to_string(&value).unwrap_or_default();
    truncate(&s, 60_000)
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

    #[test]
    fn schemas_advertise_both_tools() {
        let schemas = tool_schemas();
        let names: Vec<&str> = schemas
            .iter()
            .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&GET_ACTIVE_BUILD));
        assert!(names.contains(&FIND_SYNERGIES));
        assert_eq!(schemas.len(), 2);
    }

    #[test]
    fn no_build_returns_no_build_status() {
        let ctx = BuildContext::new();
        let s = ctx.render_tool_result();
        assert!(s.contains("\"status\":\"no_build\""));
    }
}
