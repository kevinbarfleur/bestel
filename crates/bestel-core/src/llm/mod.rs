// Each provider has its own native event schema; see `PROVIDER_NOTES.md`
// in this directory for the quirks we've documented (Anthropic's
// structured thinking blocks, Ollama's NDJSON tool-call accumulation).
// The parsers translate native events into the shared `LlmDelta` enum
// below — every downstream consumer (TUI rendering, devlog, focus,
// scroll) is provider-agnostic and only deals with `LlmDelta`.

pub mod anthropic;
pub mod detect;
pub mod kb;
pub mod keys;
pub mod models;
pub mod ollama;
pub mod pob_engine;
pub mod post_stream;
pub mod recorder;
pub mod session_notes;
pub mod sheet_tools;
pub mod tools;
pub mod turn_classifier;
pub mod util;
pub mod verifier;
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
    /// Late-arriving formatted view of the tool call's input arguments.
    /// Anthropic streams the input JSON across multiple `input_json_delta`
    /// chunks, so this is emitted at `content_block_stop` once the input
    /// is fully assembled. Ollama emits it alongside `ToolBegin` for
    /// shape parity. The string is pre-formatted by `util::summarize_tool_args`
    /// so the UI can render it inline without per-tool parsing.
    ToolDetailUpdate {
        id: String,
        summary_input: String,
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
    /// Sprint G — verifier verdict (one per turn) emitted after the
    /// streaming pump finishes and before `MessageEnd`. Carries
    /// `pass | revise | fail` plus structured findings; the dev panel
    /// surfaces the badge and the recorder persists it in `ChatStats`.
    Verifier(verifier::VerifierVerdict),
    /// Sprint v6 Phase 2 — live response-lint findings, emitted once per
    /// turn after `Usage` and before the verifier pass. **Warn-only**: the
    /// findings are surfaced to the dev panel and persisted in `ChatStats`
    /// for data collection, but the draft is NOT modified at this stage.
    /// Phase 3 will activate strategy A/B/C per rule based on observed
    /// fire rates. The delta is omitted entirely when the report is empty
    /// so a clean run produces no extra payload.
    LintFindings {
        findings: Vec<crate::test_runner::response_lint::LintFinding>,
    },
    /// Build Sheets feature — emitted when the agent calls
    /// `sheet_propose_section` to draft (or re-draft) a section. The UI
    /// patches the matching `BSDraftedCard` in place. Multiple updates may
    /// land for the same `section_id` over the course of an interview as
    /// the user corrects and the agent re-drafts.
    SheetDraftUpdate {
        section_id: String,
        title: String,
        body: String,
        confirmed: bool,
    },
    /// Build Sheets feature — emitted when the agent calls `sheet_ask` to
    /// surface a `questions_v2`-style picker for a personality question
    /// (purpose of an aura, role of a unique). The UI renders a
    /// `BSAskCard`; the user's answer comes back through the normal
    /// composer / chip-click flow as a regular ChatMessage.
    SheetAskUser {
        question_id: String,
        title: String,
        subtitle: Option<String>,
        options: Vec<String>,
        multi: bool,
        has_other: bool,
    },
    /// Build Sheets feature — emitted when the agent calls
    /// `sheet_open_interview` after deep PoB analysis. The payload contains
    /// every section's draft body PLUS every leverage-purpose question
    /// across sections PLUS a notes_prompt, all in one delta. The UI
    /// renders a single `BSInterviewPanel` and the user fills the entire
    /// form in one round before submitting; the submission round-trips
    /// back to the agent as a structured user message (see ref 32 §
    /// `[INTERVIEW SUBMISSION]` contract). The payload is opaque to the
    /// runtime — it's a JSON `Value` forwarded verbatim from the model's
    /// tool input. Schema validation happens at the dispatcher.
    SheetInterviewOpen {
        payload: serde_json::Value,
    },
    /// Build Sheets feature — emitted when `sheet_finalize_request` has
    /// successfully persisted a validated sheet to the `build_sheets`
    /// table. The frontend renders a one-line `BSSheetSavedBanner` segment
    /// in the chat timeline so the moment of validation is visible and
    /// permanent (the banner persists in the conversation log just like
    /// any other segment). The sidebar `BSLinkedSheetCard` is populated by
    /// the companion `SheetLoaded` delta the same dispatcher emits.
    SheetFinalized {
        sheet_id: String,
        name: String,
    },
    /// Build Sheets feature — emitted whenever a validated sheet becomes
    /// the active sheet for the current chat. Fired from BOTH
    /// `dispatch_sheet_finalize_request` (right after the row is
    /// persisted, so the sidebar populates the same turn) and
    /// `dispatch_get_active_build_sheet` (when the agent looks up an
    /// existing sheet by fingerprint). The frontend `BSLinkedSheetCard`
    /// reads from `sheet.activeSheet`, which this delta populates via
    /// `sheet.loadActiveSheet(...)`. No banner is dropped — that's
    /// `SheetFinalized`'s job, and only fires for fresh finalizations.
    SheetLoaded {
        sheet_id: String,
        fingerprint: String,
        name: String,
        pob_hash: String,
        stale: bool,
        authored_at: String,
        updated_at: String,
        schema_version: i64,
        payload: serde_json::Value,
    },
    /// Sprint v3 — fires once at the very start of an agent turn after the
    /// deterministic `TurnClassifier` decides which conversational mode
    /// applies. The frontend renders a `ModeChip` above the assistant
    /// message for non-default modes; persisted on the assistant message
    /// so the chip survives a reload. Values: "brief-mechanic",
    /// "deep-audit", "legacy-diagnostic", "refusal", "default".
    ModeAssigned {
        mode: String,
    },
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
