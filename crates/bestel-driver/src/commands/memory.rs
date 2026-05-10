//! `bestel-driver memory` — heap, DOM, localStorage probe via the JS bridge.

use anyhow::Result;

pub async fn run() -> Result<()> {
    // Reuse the eval command, but with the bridge call hardcoded so the
    // user doesn't have to remember the exact JS expression.
    super::eval::run("window.__bestel?.getMemoryStats() ?? null".to_string()).await
}
