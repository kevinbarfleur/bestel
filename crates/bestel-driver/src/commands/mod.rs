//! CLI sub-command implementations. Each module exposes a single async
//! `run(args) -> Result<()>` that the top-level dispatcher invokes.

pub mod attach;
pub mod chat;
pub mod crash;
pub mod eval;
pub mod logs;
pub mod memory;
pub mod scenario;
pub mod screenshot;
