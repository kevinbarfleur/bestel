//! LLM-as-judge pipeline for the Sprint A baseline eval set.
//!
//! Given a directory of `PersistedRun` JSON files (produced by
//! `bestel run-battery --out`) and the master `eval_set.toml` that
//! defines per-prompt expected/forbidden signals, this module sends
//! each run to Claude Sonnet 4.5 along with the rubric in
//! `tests/eval/judge_prompt.md` and parses the strict-JSON score.
//!
//! Design notes:
//! - Single-file output keyed by run + judge-model. The output schema
//!   is stable enough that a baseline can be diffed against later.
//! - One-shot non-streaming Anthropic call per run. No tools, no
//!   retries-with-tool-loop, no cache breakpoints — judging is cheap
//!   relative to a full agent run, and we want bit-for-bit determinism
//!   from the judge's perspective. We do retry transient HTTP errors.
//! - The judge prompt itself is bundled at compile time so a release
//!   binary doesn't need the workspace `tests/` tree at runtime.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::llm::recorder::{PersistedRun, PersistedSegment};

/// Bundled rubric — the canonical scoring prompt. Editing the file at
/// `tests/eval/judge_prompt.md` requires a recompile to take effect in
/// release builds; the source-of-truth is `tests/eval/judge_prompt.md`.
pub const JUDGE_SYSTEM_PROMPT: &str = include_str!("../../../../tests/eval/judge_prompt.md");

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// One entry in `tests/eval/eval_set.toml`. Schema is intentionally
/// loose — expected/forbidden signals are short strings, not regex,
/// and the judge model interprets them semantically.
#[derive(Debug, Clone, Deserialize)]
pub struct EvalEntry {
    pub id: String,
    pub category: String,
    pub prompt: String,
    #[serde(default)]
    pub expected_signals: Vec<String>,
    #[serde(default)]
    pub forbidden_signals: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawEvalSet {
    #[serde(default)]
    entry: Vec<EvalEntry>,
}

/// Per-run judge output. Mirrors the schema in
/// `tests/eval/judge_prompt.md` plus a few bookkeeping fields so the
/// baseline file is self-contained.
#[derive(Debug, Clone, Serialize)]
pub struct JudgeRecord {
    pub id: String,
    pub category: String,
    pub score: u32,
    pub rubric: serde_json::Map<String, Value>,
    pub rationale: String,
    #[serde(default)]
    pub expected_signals_hit: Vec<String>,
    #[serde(default)]
    pub forbidden_signals_present: Vec<String>,
    /// PersistedRun id (filename stem when loaded from disk).
    pub run_id: String,
    /// Model that produced the answer being judged (display name + id).
    pub answered_by: String,
    /// End-to-end elapsed time of the answer-producing run, in ms.
    pub answer_elapsed_ms: u64,
    /// Distinct tool names invoked during the run.
    pub tools_called: Vec<String>,
    /// Tool names whose status was `failed`.
    pub tool_failures: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryStats {
    pub n: usize,
    pub avg_score: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BaselineSummary {
    pub total_runs: usize,
    pub avg_score: f64,
    pub by_category: BTreeMap<String, CategoryStats>,
    /// Number of runs the judge couldn't grade (HTTP error, malformed JSON…).
    pub judge_errors: usize,
}

/// Top-level baseline file format. Stored at `--out`.
#[derive(Debug, Clone, Serialize)]
pub struct Baseline {
    pub eval_set_path: String,
    pub runs_dir: String,
    pub judge_model: String,
    pub judged_at: String,
    pub summary: BaselineSummary,
    pub records: Vec<JudgeRecord>,
    /// Run files we found in the runs directory but couldn't grade —
    /// either no matching eval_set entry, or judge call failed.
    pub skipped: Vec<SkippedRun>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedRun {
    pub run_file: String,
    pub reason: String,
}

/// Read `eval_set.toml` and index by `id`. Duplicate ids are
/// rejected — the splitter assumes uniqueness.
pub fn load_eval_set(path: &Path) -> Result<HashMap<String, EvalEntry>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let raw: RawEvalSet =
        toml::from_str(&text).with_context(|| format!("parse TOML in {}", path.display()))?;
    let mut map = HashMap::with_capacity(raw.entry.len());
    for e in raw.entry {
        if map.insert(e.id.clone(), e).is_some() {
            return Err(anyhow!("eval_set: duplicate entry id"));
        }
    }
    Ok(map)
}

/// Walk a directory of PersistedRun JSON files (excludes `*.lint.json`
/// and `*.judge.json`). Returns `(filename_stem, parsed_run)` pairs.
pub fn load_persisted_runs(dir: &Path) -> Result<Vec<(String, PathBuf, PersistedRun)>> {
    if !dir.is_dir() {
        return Err(anyhow!("runs dir not found: {}", dir.display()));
    }
    let mut out: Vec<(String, PathBuf, PersistedRun)> = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.ends_with(".json") {
            continue;
        }
        if name.ends_with(".lint.json") || name.ends_with(".judge.json") {
            continue;
        }
        let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let run: PersistedRun = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse run {}", path.display()))?;
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("run")
            .to_string();
        out.push((stem, path, run));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Send one (entry, run) pair to the Anthropic judge model. The HTTP
/// call is one-shot non-streaming. Returns either the parsed
/// JudgeRecord or a wrapped error so the caller can decide whether to
/// abort the whole baseline or just record a skip.
pub async fn judge_one(
    http: &reqwest::Client,
    api_key: &str,
    judge_model: &str,
    entry: &EvalEntry,
    run: &PersistedRun,
) -> Result<JudgeRecord> {
    let mut tool_names: Vec<String> = Vec::new();
    let mut tool_failures: Vec<String> = Vec::new();
    for seg in &run.assistant_segments {
        if let PersistedSegment::Tool { name, status, .. } = seg {
            if !tool_names.iter().any(|n| n == name) {
                tool_names.push(name.clone());
            }
            if status == "failed" && !tool_failures.iter().any(|n| n == name) {
                tool_failures.push(name.clone());
            }
        }
    }

    let user_payload = json!({
        "id": entry.id,
        "category": entry.category,
        "question": entry.prompt,
        "expected_signals": entry.expected_signals,
        "forbidden_signals": entry.forbidden_signals,
        "final_text": run.final_text,
        "tools_called": tool_names,
        "tool_failures": tool_failures,
    });

    let body = json!({
        "model": judge_model,
        "max_tokens": 2048,
        "system": JUDGE_SYSTEM_PROMPT,
        "messages": [
            {
                "role": "user",
                "content": serde_json::to_string_pretty(&user_payload)
                    .unwrap_or_else(|_| user_payload.to_string()),
            }
        ],
    });

    let resp = http
        .post(ANTHROPIC_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_API_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .context("anthropic POST failed")?;

    let status = resp.status();
    let body_text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("anthropic HTTP {}: {}", status, body_text));
    }

    let resp_json: Value = serde_json::from_str(&body_text)
        .with_context(|| format!("parse anthropic response: {}", body_text))?;
    let text = resp_json
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
        })
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("anthropic response missing content[].text"))?
        .trim()
        .to_string();

    let raw_json = strip_code_fences(&text);
    let judge_value: Value =
        serde_json::from_str(raw_json).with_context(|| format!("judge JSON parse: {raw_json}"))?;

    let score = judge_value
        .get("score")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32;
    let category = judge_value
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or(&entry.category)
        .to_string();
    let rubric = judge_value
        .get("rubric")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let rationale = judge_value
        .get("rationale")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let expected_hit = string_list(&judge_value, "expected_signals_hit");
    let forbidden_present = string_list(&judge_value, "forbidden_signals_present");

    Ok(JudgeRecord {
        id: entry.id.clone(),
        category,
        score,
        rubric,
        rationale,
        expected_signals_hit: expected_hit,
        forbidden_signals_present: forbidden_present,
        run_id: run.id.clone(),
        answered_by: format!("{} ({})", run.model_display_name, run.model_id),
        answer_elapsed_ms: run.stats.elapsed_ms,
        tools_called: tool_names,
        tool_failures,
    })
}

fn string_list(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn strip_code_fences(s: &str) -> &str {
    let trimmed = s.trim();
    let no_open = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_start();
    no_open.strip_suffix("```").unwrap_or(no_open).trim()
}

/// Aggregate a slice of judge records into the summary that lives
/// alongside the records in the baseline JSON file.
pub fn summarise(records: &[JudgeRecord], judge_errors: usize) -> BaselineSummary {
    if records.is_empty() {
        return BaselineSummary {
            judge_errors,
            ..Default::default()
        };
    }
    let total_runs = records.len();
    let total_score: u64 = records.iter().map(|r| r.score as u64).sum();
    let avg_score = total_score as f64 / total_runs as f64;

    let mut by_cat: BTreeMap<String, (u64, usize)> = BTreeMap::new();
    for r in records {
        let entry = by_cat.entry(r.category.clone()).or_insert((0, 0));
        entry.0 += r.score as u64;
        entry.1 += 1;
    }
    let by_category: BTreeMap<String, CategoryStats> = by_cat
        .into_iter()
        .map(|(k, (sum, n))| {
            (
                k,
                CategoryStats {
                    n,
                    avg_score: sum as f64 / n as f64,
                },
            )
        })
        .collect();

    BaselineSummary {
        total_runs,
        avg_score,
        by_category,
        judge_errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_code_fences_handles_json_block() {
        let s = "```json\n{\"a\":1}\n```";
        assert_eq!(strip_code_fences(s), "{\"a\":1}");
    }

    #[test]
    fn strip_code_fences_handles_bare_block() {
        let s = "```\n{\"a\":1}\n```";
        assert_eq!(strip_code_fences(s), "{\"a\":1}");
    }

    #[test]
    fn strip_code_fences_passes_through_plain() {
        let s = "{\"a\":1}";
        assert_eq!(strip_code_fences(s), "{\"a\":1}");
    }

    #[test]
    fn summarise_produces_stable_averages() {
        let mk = |id: &str, cat: &str, score: u32| JudgeRecord {
            id: id.into(),
            category: cat.into(),
            score,
            rubric: serde_json::Map::new(),
            rationale: "".into(),
            expected_signals_hit: Vec::new(),
            forbidden_signals_present: Vec::new(),
            run_id: id.into(),
            answered_by: "test".into(),
            answer_elapsed_ms: 0,
            tools_called: Vec::new(),
            tool_failures: Vec::new(),
        };
        let recs = vec![
            mk("a", "mechanic", 80),
            mk("b", "mechanic", 60),
            mk("c", "build", 90),
        ];
        let s = summarise(&recs, 1);
        assert_eq!(s.total_runs, 3);
        assert!((s.avg_score - 76.666_666_666_666_67).abs() < 1e-6);
        assert_eq!(s.by_category.get("mechanic").unwrap().n, 2);
        assert!((s.by_category.get("mechanic").unwrap().avg_score - 70.0).abs() < 1e-6);
        assert_eq!(s.by_category.get("build").unwrap().n, 1);
        assert!((s.by_category.get("build").unwrap().avg_score - 90.0).abs() < 1e-6);
        assert_eq!(s.judge_errors, 1);
    }

    #[test]
    fn judge_system_prompt_includes_rubric_anchor() {
        // Sanity check that the bundled prompt isn't empty and mentions
        // a rubric anchor. Reduces the risk of a future include_str
        // path-rot landing silently.
        assert!(JUDGE_SYSTEM_PROMPT.contains("rubric"));
        assert!(JUDGE_SYSTEM_PROMPT.contains("score"));
    }
}
