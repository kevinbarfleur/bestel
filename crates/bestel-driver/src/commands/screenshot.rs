//! `bestel-driver screenshot --out <path>` — capture the current page as PNG.

use std::path::PathBuf;

use anyhow::{Context, Result};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};

use crate::cdp;
use crate::session;

pub async fn run(out: PathBuf) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;

    let params = CaptureScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png)
        .build();

    let bytes = attached
        .page
        .screenshot(params)
        .await
        .context("CDP Page.captureScreenshot")?;

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&out, &bytes).with_context(|| format!("write {}", out.display()))?;

    println!(
        "{}",
        serde_json::json!({
            "status": "ok",
            "out": out.display().to_string(),
            "bytes": bytes.len(),
        })
    );
    Ok(())
}
