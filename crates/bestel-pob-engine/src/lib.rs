//! Headless PoB engine sidecar.
//!
//! Spawns LuaJIT against the vendored `api-stdio-bestel/api-stdio.lua`
//! harness, talks ndJSON over stdio, and exposes a single high-level
//! [`PobEngineHandle::calc`] entry point used by the `pob_calc` tool.
//!
//! See `crates/bestel-pob-engine/vendor/api-stdio-bestel/api-stdio.lua` for the
//! protocol contract and `prompts/references/25_pob_engine_integration.md` for
//! the operating model.

pub mod calc;
pub mod error;
pub mod lifecycle;
pub mod protocol;

pub use calc::{CalcRequest, CalcResponse, Category, EngineCalcs};
pub use error::PobEngineError;
pub use lifecycle::{EngineConfig, PobEngineHandle};
