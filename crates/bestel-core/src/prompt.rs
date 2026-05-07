//! Backward-compatible re-exports for the bundled prompts. The runtime loader
//! and per-turn re-read live in `crate::prompts`. Most call sites should
//! invoke `crate::prompts::load_composed()` per turn instead of these
//! constants — they are kept around for tests and the MCP `bestel mcp-serve`
//! mode where rebuild-on-edit is not a concern.

pub use crate::prompts::BUNDLED_CORE_KNOWLEDGE as CORE_KNOWLEDGE;
pub use crate::prompts::BUNDLED_SYSTEM_PROMPT as SYSTEM_PROMPT;
