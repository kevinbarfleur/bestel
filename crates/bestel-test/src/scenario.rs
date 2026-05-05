use std::path::Path;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderChoice {
    Codex,
    Claude,
    Anthropic,
    Any,
}

impl ProviderChoice {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "anthropic" => Some(Self::Anthropic),
            "any" | "auto" | "" => Some(Self::Any),
            _ => None,
        }
    }

    pub fn as_cli_flag(&self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Anthropic => "anthropic",
            Self::Any => "auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cost {
    Low,
    High,
}

impl Cost {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "low" | "cheap" => Some(Self::Low),
            "high" | "expensive" => Some(Self::High),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub provider: ProviderChoice,
    pub build_fixture: Option<String>,
    pub prompt: String,
    pub timeout_secs: u64,
    pub cost: Cost,
    pub expectations: Vec<Expectation>,
}

#[derive(Debug, Clone)]
pub struct Expectation {
    pub must_match: Vec<Regex>,
    pub must_not_match: Vec<Regex>,
    pub must_cite_domain: Vec<String>,
    pub must_call_tool: Option<String>,
    pub forbid_tool: Vec<String>,
    /// Minimum length of the final answer (in characters). Catches lazy
    /// one-liner replies that pass a regex check but offer no real depth.
    pub min_final_text_len: usize,
    /// Minimum number of `web_search` (or any) tool invocations during the
    /// turn. Forces the agent to do real research instead of answering
    /// from memory alone.
    pub min_tool_calls: usize,
}

#[derive(Debug, Deserialize)]
struct RawScenario {
    name: Option<String>,
    provider: Option<String>,
    build_fixture: Option<String>,
    prompt: String,
    #[serde(default)]
    timeout_secs: Option<u64>,
    #[serde(default)]
    cost: Option<String>,
    #[serde(default)]
    expectations: Vec<RawExpectation>,
}

#[derive(Debug, Deserialize, Default)]
struct RawExpectation {
    #[serde(default)]
    must_match: Vec<String>,
    #[serde(default)]
    must_not_match: Vec<String>,
    #[serde(default)]
    must_cite_domain: Vec<String>,
    #[serde(default)]
    must_call_tool: Option<String>,
    #[serde(default)]
    forbid_tool: Vec<String>,
    #[serde(default)]
    min_final_text_len: Option<usize>,
    #[serde(default)]
    min_tool_calls: Option<usize>,
}

pub fn load_scenario(path: &Path) -> Result<Scenario> {
    let bytes = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let raw: RawScenario = toml::from_str(&bytes)
        .with_context(|| format!("parse TOML in {}", path.display()))?;

    let name = raw.name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("scenario")
            .to_string()
    });

    let provider = ProviderChoice::parse(raw.provider.as_deref().unwrap_or("any"))
        .ok_or_else(|| {
            anyhow!(
                "scenario {}: unknown provider '{}'",
                name,
                raw.provider.as_deref().unwrap_or("?")
            )
        })?;

    let cost = Cost::parse(raw.cost.as_deref().unwrap_or("low"))
        .ok_or_else(|| anyhow!("scenario {}: unknown cost", name))?;

    let timeout_secs = raw.timeout_secs.unwrap_or(180);

    let mut expectations = Vec::new();
    for raw_exp in raw.expectations {
        let must_match = compile_regexes(&raw_exp.must_match)?;
        let must_not_match = compile_regexes(&raw_exp.must_not_match)?;
        expectations.push(Expectation {
            must_match,
            must_not_match,
            must_cite_domain: raw_exp.must_cite_domain,
            must_call_tool: raw_exp.must_call_tool,
            forbid_tool: raw_exp.forbid_tool,
            min_final_text_len: raw_exp.min_final_text_len.unwrap_or(0),
            min_tool_calls: raw_exp.min_tool_calls.unwrap_or(0),
        });
    }

    Ok(Scenario {
        name,
        provider,
        build_fixture: raw.build_fixture,
        prompt: raw.prompt,
        timeout_secs,
        cost,
        expectations,
    })
}

fn compile_regexes(patterns: &[String]) -> Result<Vec<Regex>> {
    let mut out = Vec::with_capacity(patterns.len());
    for p in patterns {
        let re = Regex::new(p).with_context(|| format!("invalid regex: {p}"))?;
        out.push(re);
    }
    Ok(out)
}

/// Walk a directory tree (non-recursive — flat assumed) and return every
/// `*.toml` scenario file path. Sorted by name for stable runs.
pub fn discover_scenarios(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    if !dir.is_dir() {
        return Err(anyhow!("scenarios dir not found: {}", dir.display()));
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
