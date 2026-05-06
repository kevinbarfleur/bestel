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
            LlmDelta::Error(msg) => {
                self.error = Some(msg.clone());
            }
        }
    }

    pub fn finish(mut self, final_text: String) -> PersistedRun {
        let now = Utc::now();
        self.stats.elapsed_ms = (now - self.started).num_milliseconds().max(0) as u64;
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
            final_text,
            stats: self.stats,
            error: self.error,
        }
    }
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
    Ok(path)
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
