//! TOML scenario parser shared by `bestel-test`, the dev panel, and the
//! headless `bestel run-battery` CLI orchestrator.
//!
//! Schema (file `tests/scenarios/<name>.toml`):
//!
//! ```toml
//! name = "active_build_poe2_with_analysis"
//! provider = "any"               # any | anthropic | ollama | codex | claude
//! build_fixture = "poe2_druid"   # optional, resolves to tests/fixtures/pob/<name>.xml
//! prompt = "Look at my loaded PoB build…"
//! timeout_secs = 420
//! cost = "low"
//!
//! [[expectations]]
//! must_match = ["Druid", "(?i)\\b(suggest|swap)\\b"]
//! must_not_match = ["fandom\\.com"]
//! must_cite_domain = ["poe2wiki.net"]
//! must_call_tool = "web_fetch"
//! forbid_tool = []
//! min_final_text_len = 600
//! min_tool_calls = 2
//! ```

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderChoice {
    Codex,
    Claude,
    Anthropic,
    Ollama,
    Any,
}

impl ProviderChoice {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            "anthropic" => Some(Self::Anthropic),
            "ollama" => Some(Self::Ollama),
            "any" | "auto" | "" => Some(Self::Any),
            _ => None,
        }
    }

    pub fn as_cli_flag(&self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Anthropic => "anthropic",
            Self::Ollama => "ollama",
            Self::Any => "auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Serialize)]
pub struct Scenario {
    pub name: String,
    pub provider: ProviderChoice,
    pub build_fixture: Option<String>,
    /// Sprint H — game scope tags (`["poe1"]`, `["poe2"]`, or both). Used
    /// by the judge step to filter the contamination-free PoE2 subset.
    /// Defaults to `["poe1", "poe2"]` if absent in the scenario file.
    pub applies_to: Vec<String>,
    pub prompt: String,
    pub timeout_secs: u64,
    pub cost: Cost,
    pub expectations: Vec<Expectation>,
    /// Multi-turn conversation steps. Empty when the scenario is the
    /// legacy single-prompt shape (in that case `prompt` is the only
    /// user message). When non-empty, the runner sends each turn in
    /// order, accumulating assistant responses into the chat history,
    /// so later turns can reference what the agent said in earlier
    /// turns ("that's wrong", "now what about X", etc.).
    pub turns: Vec<Turn>,
    /// Absolute path the scenario was loaded from. Useful for the dev-panel
    /// "open in editor" affordance.
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct Turn {
    /// User-side message for this turn.
    pub user: String,
    /// Optional pause (in seconds) after the assistant responds, before
    /// sending the next turn. Lets the runner space requests when the
    /// user wants the engine to settle.
    pub pause_secs: u64,
}

impl Scenario {
    /// Effective list of user turns: the explicit `turns` array if
    /// provided, otherwise `[Turn { user: prompt, pause: 0 }]`.
    pub fn effective_turns(&self) -> Vec<Turn> {
        if !self.turns.is_empty() {
            self.turns.clone()
        } else {
            vec![Turn {
                user: self.prompt.clone(),
                pause_secs: 0,
            }]
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Expectation {
    /// Regex sources, kept verbatim so the dev panel can render them.
    pub must_match: Vec<String>,
    pub must_not_match: Vec<String>,
    pub must_cite_domain: Vec<String>,
    pub must_call_tool: Option<String>,
    pub forbid_tool: Vec<String>,
    /// Minimum length of the final answer (in characters).
    pub min_final_text_len: usize,
    /// Minimum number of tool invocations during the turn.
    pub min_tool_calls: usize,

    // Sprint A — specialised assertions wired to `response_lint` rules. When
    // true, the scenario expects the corresponding lint not to FAIL on the
    // recorded run. All default to `false` so existing scenarios keep their
    // current semantics.
    /// `BUILD_IDENTITY_REQUIRED` — when `get_active_build` succeeds the answer
    /// must lead with the identity card.
    pub must_have_identity_card_if_build: bool,
    /// `NO_THINKING_TAGS` — disallow `<thinking>` / `</thinking>` in the
    /// user-facing answer.
    pub must_not_expose_reasoning: bool,
    /// `PANEL_DATA_FIRST` — when panel markers are emitted, the message must
    /// open with the `⟦panel-data⟧` sidecar.
    pub must_have_panel_data_first: bool,
    /// `SOURCES_FETCHED_ONLY` — every URL listed in a Sources section must
    /// resolve to a host actually fetched this turn.
    pub must_cite_only_fetched_urls: bool,
    /// `CALCS_ECHO_REQUIRED` — when `pob_calc` succeeds the answer must
    /// surface the Calcs echo (boss / Pinnacle / active skill / config).
    pub must_surface_calcs_echo_if_pob_calc: bool,
    /// `POB_CALC_FAILURE_NO_REAL_NUMBER` — when `pob_calc` fails the answer
    /// must not claim a "real DPS" number without a cache/stale disclaimer.
    pub must_not_claim_real_number_if_pob_calc_failed: bool,
    /// `FAILED_RAW_DATA_NO_TABLE` — when `repoe_lookup` / `wiki_cargo` /
    /// `wiki_parse` / `web_fetch` fails the answer must not print an exact
    /// tier/ilvl mod table without an uncertainty marker.
    pub must_not_fabricate_mod_table_after_raw_source_failure: bool,
}

impl Expectation {
    /// Compile the `must_match` patterns. Cached at evaluation time, not at
    /// parse time, so the parser stays cheap and the JSON-serialised form is
    /// regex-source-only.
    pub fn must_match_compiled(&self) -> Result<Vec<Regex>> {
        compile_regexes(&self.must_match)
    }

    pub fn must_not_match_compiled(&self) -> Result<Vec<Regex>> {
        compile_regexes(&self.must_not_match)
    }

    /// Map each Sprint A specialised assertion to the `response_lint` rule IDs
    /// it is wired to. Used by the battery harness to fail a scenario when an
    /// opt-in expectation correlates with a `Fail` lint finding on the run.
    pub fn required_lints(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = Vec::new();
        if self.must_have_identity_card_if_build {
            ids.push("BUILD_IDENTITY_REQUIRED");
        }
        if self.must_not_expose_reasoning {
            ids.push("NO_THINKING_TAGS");
        }
        if self.must_have_panel_data_first {
            ids.push("PANEL_DATA_FIRST");
            ids.push("PANEL_MARKER_PAYLOAD_MATCH");
        }
        if self.must_cite_only_fetched_urls {
            ids.push("SOURCES_FETCHED_ONLY");
            ids.push("NO_INTERNAL_DOC_AS_FINAL_SOURCE");
            ids.push("NO_BLOCKED_SOURCE");
        }
        if self.must_surface_calcs_echo_if_pob_calc {
            ids.push("CALCS_ECHO_REQUIRED");
        }
        if self.must_not_claim_real_number_if_pob_calc_failed {
            ids.push("POB_CALC_FAILURE_NO_REAL_NUMBER");
        }
        if self.must_not_fabricate_mod_table_after_raw_source_failure {
            ids.push("FAILED_RAW_DATA_NO_TABLE");
        }
        ids
    }
}

#[derive(Debug, Deserialize)]
struct RawScenario {
    name: Option<String>,
    provider: Option<String>,
    build_fixture: Option<String>,
    #[serde(default)]
    applies_to: Vec<String>,
    /// Single-prompt shape (legacy / simple scenarios). Either this or
    /// `[[turn]]` blocks must be present; if both are set, `[[turn]]`
    /// wins and `prompt` is ignored.
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    turn: Vec<RawTurn>,
    #[serde(default)]
    timeout_secs: Option<u64>,
    #[serde(default)]
    cost: Option<String>,
    #[serde(default)]
    expectations: Vec<RawExpectation>,
}

#[derive(Debug, Deserialize, Default)]
struct RawTurn {
    user: String,
    #[serde(default)]
    pause_secs: Option<u64>,
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

    #[serde(default)]
    must_have_identity_card_if_build: bool,
    #[serde(default)]
    must_not_expose_reasoning: bool,
    #[serde(default)]
    must_have_panel_data_first: bool,
    #[serde(default)]
    must_cite_only_fetched_urls: bool,
    #[serde(default)]
    must_surface_calcs_echo_if_pob_calc: bool,
    #[serde(default)]
    must_not_claim_real_number_if_pob_calc_failed: bool,
    #[serde(default)]
    must_not_fabricate_mod_table_after_raw_source_failure: bool,
}

pub fn load_scenario(path: &Path) -> Result<Scenario> {
    let bytes =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let raw: RawScenario =
        toml::from_str(&bytes).with_context(|| format!("parse TOML in {}", path.display()))?;

    let name = raw.name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("scenario")
            .to_string()
    });

    let provider =
        ProviderChoice::parse(raw.provider.as_deref().unwrap_or("any")).ok_or_else(|| {
            anyhow!(
                "scenario {}: unknown provider '{}'",
                name,
                raw.provider.as_deref().unwrap_or("?")
            )
        })?;

    let cost = Cost::parse(raw.cost.as_deref().unwrap_or("low"))
        .ok_or_else(|| anyhow!("scenario {}: unknown cost", name))?;

    let timeout_secs = raw.timeout_secs.unwrap_or(180);

    // Validate regex sources at load time without keeping the compiled forms
    // (so Scenario stays Serialize-friendly).
    let mut expectations = Vec::with_capacity(raw.expectations.len());
    for raw_exp in raw.expectations {
        compile_regexes(&raw_exp.must_match)?;
        compile_regexes(&raw_exp.must_not_match)?;
        expectations.push(Expectation {
            must_match: raw_exp.must_match,
            must_not_match: raw_exp.must_not_match,
            must_cite_domain: raw_exp.must_cite_domain,
            must_call_tool: raw_exp.must_call_tool,
            forbid_tool: raw_exp.forbid_tool,
            min_final_text_len: raw_exp.min_final_text_len.unwrap_or(0),
            min_tool_calls: raw_exp.min_tool_calls.unwrap_or(0),
            must_have_identity_card_if_build: raw_exp.must_have_identity_card_if_build,
            must_not_expose_reasoning: raw_exp.must_not_expose_reasoning,
            must_have_panel_data_first: raw_exp.must_have_panel_data_first,
            must_cite_only_fetched_urls: raw_exp.must_cite_only_fetched_urls,
            must_surface_calcs_echo_if_pob_calc: raw_exp.must_surface_calcs_echo_if_pob_calc,
            must_not_claim_real_number_if_pob_calc_failed: raw_exp
                .must_not_claim_real_number_if_pob_calc_failed,
            must_not_fabricate_mod_table_after_raw_source_failure: raw_exp
                .must_not_fabricate_mod_table_after_raw_source_failure,
        });
    }

    let turns: Vec<Turn> = raw
        .turn
        .into_iter()
        .map(|t| Turn {
            user: t.user,
            pause_secs: t.pause_secs.unwrap_or(0),
        })
        .collect();

    let prompt = match (raw.prompt, turns.is_empty()) {
        (Some(p), _) => p,
        (None, false) => turns[0].user.clone(),
        (None, true) => {
            return Err(anyhow!(
                "scenario {name}: requires either `prompt = \"...\"` or at least one `[[turn]]` block"
            ));
        }
    };

    let applies_to = if raw.applies_to.is_empty() {
        vec!["poe1".to_string(), "poe2".to_string()]
    } else {
        raw.applies_to
    };

    Ok(Scenario {
        name,
        provider,
        build_fixture: raw.build_fixture,
        applies_to,
        prompt,
        turns,
        timeout_secs,
        cost,
        expectations,
        source_path: path.to_path_buf(),
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
pub fn discover_scenarios(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Err(anyhow!("scenarios dir not found: {}", dir.display()));
    }
    let mut out: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Convenience: discover + load every scenario in a directory.
pub fn load_all(dir: &Path) -> Result<Vec<Scenario>> {
    let paths = discover_scenarios(dir)?;
    let mut out = Vec::with_capacity(paths.len());
    for p in paths {
        out.push(load_scenario(&p)?);
    }
    Ok(out)
}
