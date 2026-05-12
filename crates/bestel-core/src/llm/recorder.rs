//! Debug-chat persistence: record one user→assistant turn (text, reasoning,
//! tool calls, stats) to disk so a dev-only viewer can replay it later.
//!
//! Both the Tauri `chat_start` command path and the offline smoke examples
//! funnel through [`Recorder`] so a single inbox shows everything Bestel
//! has answered, regardless of whether the trigger was the in-app
//! composer or `cargo run --example ollama_smoke`.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{LlmDelta, ToolStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedSegment {
    Text {
        text: String,
    },
    Reasoning {
        text: String,
    },
    Tool {
        id: String,
        name: String,
        detail: Option<String>,
        outputs: Vec<String>,
        status: String,
        summary: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedAttachment {
    pub name: String,
    pub mime: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatStats {
    pub elapsed_ms: u64,
    pub text_bytes: usize,
    pub reasoning_bytes: usize,
    pub tool_calls: usize,
    pub tool_done: usize,
    pub tool_failed: usize,
    /// Sprint C cache telemetry. Populated from `LlmDelta::Usage` when the
    /// provider reports it (Anthropic + DeepSeek via Anthropic-compat). Older
    /// persisted runs without these keys deserialize to zero / `None`.
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub cached_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// Sprint G — conditional verifier verdict for the run. `None` when the
    /// answer didn't trigger verification (Brief mechanics, off-topic
    /// refusal). Persisted runs from before Sprint G deserialize as `None`.
    #[serde(default)]
    pub verifier_verdict: Option<crate::llm::verifier::VerifierVerdict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedRun {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    /// Origin tag, free-form. Today: `"ui"` for Tauri commands, `"smoke"`
    /// for `cargo run --example`-style runs.
    pub source: String,
    pub provider: String,
    pub model_id: String,
    pub model_display_name: String,
    pub user_text: String,
    #[serde(default)]
    pub user_attachments: Vec<PersistedAttachment>,
    #[serde(default)]
    pub assistant_segments: Vec<PersistedSegment>,
    pub final_text: String,
    pub stats: ChatStats,
    pub error: Option<String>,
}

pub struct Recorder {
    started: DateTime<Utc>,
    source: String,
    provider: String,
    model_id: String,
    model_display_name: String,
    user_text: String,
    user_attachments: Vec<PersistedAttachment>,
    segments: Vec<PersistedSegment>,
    open_text: Option<usize>,
    open_reasoning: Option<usize>,
    tools: HashMap<String, usize>,
    stats: ChatStats,
    error: Option<String>,
}

impl Recorder {
    pub fn new(
        source: impl Into<String>,
        provider: impl Into<String>,
        model_id: impl Into<String>,
        model_display_name: impl Into<String>,
        user_text: impl Into<String>,
        user_attachments: Vec<PersistedAttachment>,
    ) -> Self {
        Self {
            started: Utc::now(),
            source: source.into(),
            provider: provider.into(),
            model_id: model_id.into(),
            model_display_name: model_display_name.into(),
            user_text: user_text.into(),
            user_attachments,
            segments: Vec::new(),
            open_text: None,
            open_reasoning: None,
            tools: HashMap::new(),
            stats: ChatStats::default(),
            error: None,
        }
    }

    pub fn apply(&mut self, delta: &LlmDelta) {
        match delta {
            LlmDelta::TextDelta(t) => {
                self.stats.text_bytes += t.len();
                self.open_reasoning = None;
                if let Some(idx) = self.open_text {
                    if let PersistedSegment::Text { text } = &mut self.segments[idx] {
                        text.push_str(t);
                        return;
                    }
                }
                self.segments.push(PersistedSegment::Text { text: t.clone() });
                self.open_text = Some(self.segments.len() - 1);
            }
            LlmDelta::ReasoningBegin => {
                self.open_text = None;
                self.segments
                    .push(PersistedSegment::Reasoning { text: String::new() });
                self.open_reasoning = Some(self.segments.len() - 1);
            }
            LlmDelta::ReasoningDelta(t) => {
                self.stats.reasoning_bytes += t.len();
                if let Some(idx) = self.open_reasoning {
                    if let PersistedSegment::Reasoning { text } = &mut self.segments[idx] {
                        text.push_str(t);
                    }
                }
            }
            LlmDelta::ReasoningEnd => {
                self.open_reasoning = None;
            }
            LlmDelta::ToolBegin { id, name, detail } => {
                self.open_text = None;
                self.open_reasoning = None;
                self.stats.tool_calls += 1;
                self.segments.push(PersistedSegment::Tool {
                    id: id.clone(),
                    name: name.clone(),
                    detail: detail.clone(),
                    outputs: Vec::new(),
                    status: "running".to_string(),
                    summary: None,
                });
                self.tools.insert(id.clone(), self.segments.len() - 1);
            }
            LlmDelta::ToolDetailUpdate { id, summary_input } => {
                if let Some(&idx) = self.tools.get(id) {
                    if let PersistedSegment::Tool { detail, .. } = &mut self.segments[idx] {
                        *detail = Some(summary_input.clone());
                    }
                }
            }
            LlmDelta::ToolOutput { id, chunk } => {
                if let Some(&idx) = self.tools.get(id) {
                    if let PersistedSegment::Tool { outputs, .. } = &mut self.segments[idx] {
                        outputs.push(chunk.clone());
                    }
                }
            }
            LlmDelta::ToolEnd {
                id,
                status,
                summary,
            } => {
                if let Some(&idx) = self.tools.get(id) {
                    if let PersistedSegment::Tool {
                        status: s,
                        summary: sm,
                        ..
                    } = &mut self.segments[idx]
                    {
                        *s = match status {
                            ToolStatus::Running => "running",
                            ToolStatus::Done => "done",
                            ToolStatus::Failed => "failed",
                        }
                        .to_string();
                        *sm = summary.clone();
                    }
                }
                match status {
                    ToolStatus::Done => self.stats.tool_done += 1,
                    ToolStatus::Failed => self.stats.tool_failed += 1,
                    _ => {}
                }
            }
            LlmDelta::MessageEnd => {}
            LlmDelta::Usage(u) => {
                // Anthropic emits one final Usage event per turn carrying
                // cumulative-since-start counters. Capture them so the
                // PersistedRun preserves cache telemetry alongside the
                // existing tool / segment counts.
                self.stats.input_tokens = u.input_tokens;
                self.stats.cached_input_tokens = u.cached_input_tokens;
                self.stats.cache_creation_tokens = u.cache_creation_tokens;
                self.stats.output_tokens = u.output_tokens;
                self.stats.cost_usd = u.cost_usd;
            }
            LlmDelta::Verifier(v) => {
                self.stats.verifier_verdict = Some(v.clone());
            }
            LlmDelta::SheetDraftUpdate { .. }
            | LlmDelta::SheetAskUser { .. }
            | LlmDelta::SheetInterviewOpen { .. }
            | LlmDelta::SheetFinalized { .. }
            | LlmDelta::SheetLoaded { .. }
            | LlmDelta::ModeAssigned { .. } => {
                // UI-only events — not persisted in the run JSON. The
                // sheet lives in `build_sheets`; the turn mode is
                // recomputed deterministically from the user message on
                // replay so persisting it would be redundant.
            }
            LlmDelta::Error(msg) => {
                self.error = Some(msg.clone());
            }
        }
    }

    pub fn finish(mut self, final_text: String) -> PersistedRun {
        let now = Utc::now();
        self.stats.elapsed_ms = (now - self.started).num_milliseconds().max(0) as u64;
        let cleaned_final = strip_thinking_blocks(&final_text);
        // Mirror the strip into persisted text segments so a re-render of
        // the run from `assistant_segments` matches the cleaned final_text
        // and the linter does not re-discover a leak that the user-facing
        // path already handled.
        for seg in self.segments.iter_mut() {
            if let PersistedSegment::Text { text } = seg {
                *text = strip_thinking_blocks(text);
            }
        }
        if cleaned_final.len() != final_text.len() {
            tracing::warn!(
                stripped_bytes = final_text.len() - cleaned_final.len(),
                "stripped <thinking> blocks from persisted final_text — model leaked native thinking tag"
            );
            self.stats.text_bytes = cleaned_final.len();
        }
        PersistedRun {
            id: format!(
                "{}__{}",
                self.started.format("%Y%m%dT%H%M%S"),
                random_suffix()
            ),
            started_at: self.started.to_rfc3339(),
            ended_at: Some(now.to_rfc3339()),
            source: self.source,
            provider: self.provider,
            model_id: self.model_id,
            model_display_name: self.model_display_name,
            user_text: self.user_text,
            user_attachments: self.user_attachments,
            assistant_segments: self.segments,
            final_text: cleaned_final,
            stats: self.stats,
            error: self.error,
        }
    }
}

/// Strip any `<thinking>...</thinking>` and `<reflection>...</reflection>`
/// blocks the model leaked into visible text. Catches the Haiku-class
/// behaviour where the model emits the tag despite the SYSTEM_PROMPT
/// prohibition. Idempotent — running twice on already-clean input is a
/// no-op. Tags are matched case-insensitively but only at exact tag
/// boundaries; defensive against partial leaks where only the opening
/// tag landed (truncates at the open tag).
fn strip_thinking_blocks(input: &str) -> String {
    const TAGS: &[&str] = &["thinking", "reflection"];
    let mut out = input.to_string();
    for tag in TAGS {
        out = strip_one_tag(&out, tag);
    }
    out.trim().to_string()
}

fn strip_one_tag(input: &str, tag: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = String::with_capacity(input.len());
    let mut cursor = 0usize;
    while let Some(open_at) = lower[cursor..].find(&open) {
        let abs_open = cursor + open_at;
        out.push_str(&input[cursor..abs_open]);
        let after_open = abs_open + open.len();
        match lower[after_open..].find(&close) {
            Some(close_off) => {
                cursor = after_open + close_off + close.len();
            }
            None => {
                // Unbalanced — opening tag with no close. Drop the rest of
                // the input rather than ship a partial leaked block.
                cursor = input.len();
                break;
            }
        }
    }
    out.push_str(&input[cursor..]);
    out
}

fn random_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:06x}", nanos & 0x00ff_ffff)
}

pub fn debug_runs_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".bestel").join("runtime").join("debug-chats"))
}

pub fn save_run(run: &PersistedRun) -> Result<PathBuf> {
    let dir = debug_runs_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
    std::fs::create_dir_all(&dir).context("create debug-chats dir")?;
    let path = dir.join(format!("{}.json", run.id));
    let body = serde_json::to_vec_pretty(run).context("serialize run")?;
    std::fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    if let Some(db) = crate::persistence::global_db() {
        if let Err(e) = crate::persistence::insert_run(&db, run) {
            tracing::warn!(error = ?e, run_id = %run.id, "failed to mirror run to SQLite");
        }
    }
    Ok(path)
}

/// Mirror a Sprint A linter report to SQLite. JSON sidecar remains
/// authoritative; this only powers SQL queries like
/// `query_runs_failing_identity`.
///
/// Each finding is mapped to `(run_id, rule, severity, message)`. The
/// sidecar JSON's optional `evidence` is dropped — if needed it can be
/// re-derived by re-linting. Pre-existing rows for the same `run_id` are
/// replaced atomically so re-runs stay idempotent.
pub fn mirror_lint_findings(
    run_id: &str,
    rows: Vec<crate::persistence::LintFindingRow>,
) {
    if let Some(db) = crate::persistence::global_db() {
        if let Err(e) = crate::persistence::replace_lint_findings(&db, run_id, &rows) {
            tracing::warn!(error = ?e, run_id, "failed to mirror lint findings to SQLite");
        }
    }
}

pub fn list_runs() -> Result<Vec<PersistedRun>> {
    let Some(dir) = debug_runs_dir() else {
        return Ok(Vec::new());
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out: Vec<PersistedRun> = Vec::new();
    for entry in std::fs::read_dir(&dir).context("read debug-chats dir")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Ok(run) = serde_json::from_str::<PersistedRun>(&raw) {
            out.push(run);
        }
    }
    out.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(out)
}

pub fn get_run(id: &str) -> Result<Option<PersistedRun>> {
    let Some(dir) = debug_runs_dir() else {
        return Ok(None);
    };
    let path = dir.join(format!("{id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&raw)?))
}

pub fn delete_run(id: &str) -> Result<()> {
    let Some(dir) = debug_runs_dir() else {
        return Ok(());
    };
    let path = dir.join(format!("{id}.json"));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn delete_all_runs() -> Result<usize> {
    let Some(dir) = debug_runs_dir() else {
        return Ok(0);
    };
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            std::fs::remove_file(&path)?;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec() -> Recorder {
        Recorder::new(
            "test",
            "ollama",
            "qwen3:8b",
            "Ollama · Qwen 3 8B",
            "hi",
            vec![],
        )
    }

    #[test]
    fn text_deltas_concatenate_into_one_segment() {
        let mut r = rec();
        r.apply(&LlmDelta::TextDelta("Hello ".into()));
        r.apply(&LlmDelta::TextDelta("world".into()));
        let run = r.finish("Hello world".into());
        assert_eq!(run.assistant_segments.len(), 1);
        match &run.assistant_segments[0] {
            PersistedSegment::Text { text } => assert_eq!(text, "Hello world"),
            _ => panic!("expected text segment"),
        }
        assert_eq!(run.stats.text_bytes, 11);
    }

    #[test]
    fn tool_call_lifecycle_recorded() {
        let mut r = rec();
        r.apply(&LlmDelta::ToolBegin {
            id: "t1".into(),
            name: "wiki_parse".into(),
            detail: Some("title=Resolute Technique".into()),
        });
        r.apply(&LlmDelta::ToolOutput {
            id: "t1".into(),
            chunk: "<page body>".into(),
        });
        r.apply(&LlmDelta::ToolEnd {
            id: "t1".into(),
            status: ToolStatus::Done,
            summary: Some("ok".into()),
        });
        r.apply(&LlmDelta::TextDelta("done".into()));
        let run = r.finish("done".into());
        assert_eq!(run.stats.tool_calls, 1);
        assert_eq!(run.stats.tool_done, 1);
        match &run.assistant_segments[0] {
            PersistedSegment::Tool {
                name,
                status,
                outputs,
                summary,
                ..
            } => {
                assert_eq!(name, "wiki_parse");
                assert_eq!(status, "done");
                assert_eq!(outputs.len(), 1);
                assert_eq!(summary.as_deref(), Some("ok"));
            }
            _ => panic!("expected tool segment"),
        }
    }

    #[test]
    fn strip_thinking_blocks_removes_full_block() {
        let input = "<thinking>\nplanning the answer.\n</thinking>\nThe cap is 100%.";
        let cleaned = strip_thinking_blocks(input);
        assert_eq!(cleaned, "The cap is 100%.");
    }

    #[test]
    fn strip_thinking_blocks_handles_unbalanced_open_tag() {
        // Defensive: stream truncated mid-block. We drop the rest rather
        // than ship a leaked partial block.
        let input = "intro <thinking>\npartial reasoning";
        let cleaned = strip_thinking_blocks(input);
        assert_eq!(cleaned, "intro");
    }

    #[test]
    fn strip_thinking_blocks_no_op_on_clean_input() {
        let input = "Spell suppression caps at 100%.";
        assert_eq!(strip_thinking_blocks(input), input);
    }

    #[test]
    fn strip_thinking_blocks_also_strips_reflection() {
        let input = "<reflection>second-guessing</reflection>The answer is 100%.";
        let cleaned = strip_thinking_blocks(input);
        assert_eq!(cleaned, "The answer is 100%.");
    }

    #[test]
    fn finish_strips_leaked_thinking_from_final_text() {
        let mut r = rec();
        r.apply(&LlmDelta::TextDelta("<thinking>\nplanning\n</thinking>\nThe cap is 100%.".into()));
        let run = r.finish("<thinking>\nplanning\n</thinking>\nThe cap is 100%.".into());
        assert_eq!(run.final_text, "The cap is 100%.");
        if let PersistedSegment::Text { text } = &run.assistant_segments[0] {
            assert_eq!(text, "The cap is 100%.");
        } else {
            panic!("expected first segment to be Text");
        }
    }

    #[test]
    fn text_after_tool_starts_a_new_segment() {
        let mut r = rec();
        r.apply(&LlmDelta::TextDelta("intro ".into()));
        r.apply(&LlmDelta::ToolBegin {
            id: "t1".into(),
            name: "wiki_parse".into(),
            detail: None,
        });
        r.apply(&LlmDelta::ToolEnd {
            id: "t1".into(),
            status: ToolStatus::Done,
            summary: None,
        });
        r.apply(&LlmDelta::TextDelta("body".into()));
        let run = r.finish("intro body".into());
        assert_eq!(run.assistant_segments.len(), 3);
    }
}
