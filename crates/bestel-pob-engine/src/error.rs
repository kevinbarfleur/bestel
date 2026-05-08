use thiserror::Error;

#[derive(Debug, Error)]
pub enum PobEngineError {
    #[error("failed to spawn LuaJIT sidecar: {0}")]
    Spawn(String),

    #[error("sidecar protocol broken: {0}")]
    ProtocolBroken(String),

    #[error("sidecar timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("sidecar crashed unexpectedly")]
    EngineCrashed,

    #[error("circuit-breaker tripped: {0} restarts in window")]
    CircuitBreaker(u32),

    #[error("build XML rejected by harness: {0}")]
    BuildLoadFailed(String),

    #[error("Calcs config rejected by harness: {0}")]
    ConfigRejected(String),

    #[error("engine not bundled for this platform — Sprint 2 ships Windows-only")]
    PlatformUnsupported,

    #[error("no active build loaded — call get_active_build first")]
    NoActiveBuild,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
