//! Parser for `docs/test_prompts/real_user_prompts.toml` — 30+ prompts mined
//! from real Reddit / official forum / Maxroll / Discord. Voice-preserved
//! (typos, slang, missing punctuation). The dev panel exposes them as a
//! free-exploration list, separate from the assertion-driven scenarios.
//!
//! Schema (one entry):
//!
//! ```toml
//! [[prompt]]
//! id = "vague_squishy_build"
//! category = "vague"
//! text = "my build feels like ass, what am i doing wrong"
//! intent = "User is unhappy with build performance, scope unclear…"
//! expected = "Ask which axis is bad: dying, low DPS, or just feels slow…"
//! source = "https://reddit.com/r/PathOfExileBuilds/…"   # optional
//! ```

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealPrompt {
    pub id: String,
    pub category: String,
    pub text: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub expected: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawFile {
    #[serde(default)]
    prompt: Vec<RealPrompt>,
}

pub fn load(path: &Path) -> Result<Vec<RealPrompt>> {
    let bytes =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let raw: RawFile =
        toml::from_str(&bytes).with_context(|| format!("parse TOML in {}", path.display()))?;
    Ok(raw.prompt)
}

/// Group helper: returns each unique category and the count of prompts in it,
/// sorted alphabetically. Useful for the dev panel filter chips.
pub fn category_counts(prompts: &[RealPrompt]) -> Vec<(String, usize)> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for p in prompts {
        *map.entry(p.category.clone()).or_insert(0) += 1;
    }
    map.into_iter().collect()
}
