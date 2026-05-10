//! `bestel-driver wait-crash [--timeout N]` — listens for an
//! `Inspector.targetCrashed` event and exits 0 when one fires.
//!
//! WebView2 OOM kills the renderer process and surfaces this event
//! before showing the "Aw, snap!" page. Capturing it gives us the exact
//! timestamp of the crash, plus enough info to correlate with the
//! preceding memory probe history.

use std::time::{Duration, Instant};

use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::inspector::EventTargetCrashed;
use chromiumoxide::Page;
use futures_util::StreamExt;
use tokio::time::timeout as tokio_timeout;

use crate::cdp;
use crate::session;

pub async fn run(deadline: Duration) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;

    let started = Instant::now();
    let result = listen_for_crash(&attached.page, deadline).await?;
    let elapsed_ms = started.elapsed().as_millis();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": result,
            "deadline_ms": deadline.as_millis(),
            "elapsed_ms": elapsed_ms,
        }))?
    );
    if result == "crashed" {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

async fn listen_for_crash(page: &Page, deadline: Duration) -> Result<&'static str> {
    let mut events = page.event_listener::<EventTargetCrashed>().await?;
    match tokio_timeout(deadline, events.next()).await {
        Ok(Some(_event)) => Ok("crashed"),
        Ok(None) => Ok("event_stream_closed"),
        Err(_) => Ok("timeout_no_crash"),
    }
}
