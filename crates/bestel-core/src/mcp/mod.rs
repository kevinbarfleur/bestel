//! MCP (Model Context Protocol) server.
//!
//! When Bestel is launched with `mcp-serve` it speaks JSON-RPC 2.0 over stdio
//! and exposes the same toolbox that `llm/tools.rs::dispatch` provides on the
//! Anthropic API path. CLI providers (Codex, Claude Code, Gemini) that
//! support MCP can be configured to launch us as their MCP server, giving
//! them access to wiki/cargo/trade/fetch tools without an API key.
//!
//! Wire format reference: <https://spec.modelcontextprotocol.io/>
//! Methods we implement:
//!   - `initialize`               (request)
//!   - `notifications/initialized` (notification, no response)
//!   - `tools/list`               (request)
//!   - `tools/call`               (request)
//!   - `ping`                     (request, returns empty result)
//!   - `shutdown` / `exit`        (request / notification)
//!
//! The server runs single-threaded over stdin/stdout, parses one JSON-RPC
//! message per line (LSP-style framing is not used by MCP — it's plain
//! line-delimited JSON).

pub mod server;

pub use server::run_stdio_server;
