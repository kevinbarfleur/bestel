// Each provider has its own native event schema; see `PROVIDER_NOTES.md`
// in this directory for the quirks we've documented (Anthropic's
// structured thinking blocks, Ollama's NDJSON tool-call accumulation).
// The parsers translate native events into the shared `LlmDelta` enum
// below — every downstream consumer (TUI rendering, devlog, focus,
// scroll) is provider-agnostic and only deals with `LlmDelta`.

pub mod anthropic;
pub mod detect;
pub mod keys;
pub mod models;
pub mod ollama;
pub mod recorder;
pub mod tools;
pub mod wiki;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use anthropic::AnthropicClient;
use ollama::OllamaClient;
use tools::BuildContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// A single attachment carried by a user message: image (sent as base64
/// content block to vision-capable providers) or text-like document
/// (PDF / TXT / MD — inlined as text with a header preamble).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
}

impl Attachment {
    pub fn is_image(&self) -> bool {
        matches!(
            self.mime.as_str(),
            "image/png" | "image/jpeg" | "image/jpg" | "image/webp" | "image/gif"
        )
    }

    pub fn is_text_doc(&self) -> bool {
        matches!(
            self.mime.as_str(),
            "text/plain" | "text/markdown" | "application/pdf"
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub enum LlmDelta {
    TextDelta(String),
    ReasoningBegin,
    ReasoningDelta(String),
    ReasoningEnd,
    ToolBegin {
        id: String,
        name: String,
        detail: Option<String>,
    },
    ToolOutput {
        id: String,
        chunk: String,
    },
    ToolEnd {
        id: String,
        status: ToolStatus,
        summary: Option<String>,
    },
    /// Token accounting for the full agent run, emitted once before
    /// `MessageEnd`. Subscription / local providers may skip this entirely.
    Usage(UsageStats),
    MessageEnd,
    Error(String),
}

/// Per-run token accounting, summed across all loop iterations. Cache
/// fields are populated only by providers that report them (Anthropic /
/// DeepSeek via Anthropic-compat).
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    /// Fresh input tokens billed at the base rate.
    pub input_tokens: u64,
    /// Cached input tokens billed at the cache-read rate (~10% of base).
    pub cached_input_tokens: u64,
    /// Cache-creation tokens billed at the cache-write rate (~125% of base).
    pub cache_creation_tokens: u64,
    /// Output tokens billed at the output rate.
    pub output_tokens: u64,
    /// Computed cost in USD when the active profile has `cost_per_mtok`
    /// set; `None` for subscription / local profiles where billing is opaque.
    pub cost_usd: Option<f64>,
}

pub enum Provider {
    Anthropic(AnthropicClient),
    Ollama(OllamaClient),
}

impl Provider {
    pub fn label(&self) -> String {
        match self {
            Provider::Anthropic(c) => format!("anthropic api · {}", c.model()),
            Provider::Ollama(c) => format!("ollama · {}", c.model()),
        }
    }

    pub fn auth_label(&self) -> &'static str {
        match self {
            Provider::Anthropic(_) => "API key",
            Provider::Ollama(_) => "local",
        }
    }

    pub async fn run(
        &self,
        history: Vec<ChatMessage>,
        ctx: BuildContext,
        deltas: mpsc::UnboundedSender<LlmDelta>,
    ) -> Result<String> {
        match self {
            Provider::Anthropic(c) => c.run(history, ctx, deltas).await,
            Provider::Ollama(c) => c.run(history, ctx, deltas).await,
        }
    }
}
