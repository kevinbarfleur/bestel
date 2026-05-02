pub mod anthropic;
pub mod claude_cli;
pub mod codex_cli;
pub mod detect;
pub mod spawn;
pub mod tools;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use anthropic::AnthropicClient;
use claude_cli::ClaudeCliClient;
use codex_cli::CodexCliClient;
use tools::BuildContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum LlmDelta {
    Text(String),
    ToolCall { name: String },
    ToolResult { name: String },
    End,
    Error(String),
}

pub enum Provider {
    Anthropic(AnthropicClient),
    CodexCli(CodexCliClient),
    ClaudeCli(ClaudeCliClient),
}

impl Provider {
    pub fn label(&self) -> String {
        match self {
            Provider::Anthropic(c) => format!("anthropic api · {}", c.model()),
            Provider::CodexCli(c) => format!("codex cli · {}", c.version_label()),
            Provider::ClaudeCli(c) => format!("claude cli · {}", c.version_label()),
        }
    }

    pub fn auth_label(&self) -> &'static str {
        match self {
            Provider::Anthropic(_) => "API key",
            Provider::CodexCli(_) => "subscription",
            Provider::ClaudeCli(_) => "subscription",
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
            Provider::CodexCli(c) => c.run(history, ctx, deltas).await,
            Provider::ClaudeCli(c) => c.run(history, ctx, deltas).await,
        }
    }
}
