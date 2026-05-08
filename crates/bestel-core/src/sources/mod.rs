//! Live external data sources for Bestel.
//!
//! Phase 3 wires Bestel into authoritative PoE data sources so it can answer
//! with cited, verifiable facts instead of LLM-memory guesses.
//!
//! Layers:
//! - `http`  : shared `reqwest` client with the OAuth-style User-Agent GGG
//!             requires, plus a dynamic rate-limit parser that honours
//!             `X-Rate-Limit-*` and `Retry-After` headers per rule.
//! - `cache` : disk-backed JSON TTL cache under `~/.bestel/cache/`.
//! - `wiki`  : MediaWiki API (opensearch, action=parse, action=cargoquery)
//!             for poewiki.net (PoE1) and poe2wiki.net (PoE2).
//! - `trade` : PoE Trade Data API (`/api/trade/data/{stats,items,...}`),
//!             stat resolver (human phrase → `pseudo.*` / `explicit.stat_*`),
//!             and shareable search-URL builder.
//! - `poedb` : Phase 3 stub — link-only redirector. Scrape lives in Phase 4.
//!
//! Every public client returns plain `serde_json::Value` or typed structs;
//! transport errors bubble up as `anyhow::Error`. Callers wire the results
//! into `crate::llm::tools::dispatch` so the Anthropic API tool loop can
//! surface them as `ToolBegin/Output/End` deltas in the TUI.
//!
//! CLI providers (Codex, Claude) cannot dispatch custom tools — they keep
//! their native web_search. The enriched `SYSTEM_PROMPT.md` is what guides
//! them toward the same authoritative URLs.

pub mod cache;
pub mod http;
pub mod poedb;
pub mod repoe;
pub mod repoe_refresh;
pub mod trade;
pub mod trade_catalogue;
pub mod wiki;

pub use cache::FileCache;
pub use http::{PoeHttpClient, RateLimitError};
pub use repoe::{Category, Game, LookupEntry, LookupResult, RepoeClient, SnapshotSource};
pub use trade::{StatRef, TradeClient, TradeSearchResp};
pub use wiki::{WikiClient, WikiHit, WikiPage};
