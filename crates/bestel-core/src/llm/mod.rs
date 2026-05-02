pub mod anthropic;
pub mod tools;

use serde::{Deserialize, Serialize};

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
