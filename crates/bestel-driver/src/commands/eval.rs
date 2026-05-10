//! `bestel-driver eval "<js>"` — runs a JS expression in the page context
//! and prints the result as JSON. Mostly a debug primitive; chat-state /
//! memory / chat ops layer on top of this via `window.__bestel.*` calls.

use anyhow::{Context, Result};
use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;

use crate::cdp;
use crate::session;

pub async fn run(expr: String) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;

    // returnByValue = true → CDP serializes the result as JSON instead of
    // returning a remote object handle we'd then need to round-trip.
    let params = EvaluateParams::builder()
        .expression(expr)
        .return_by_value(true)
        .await_promise(true)
        .build()
        .map_err(|e| anyhow::anyhow!("build EvaluateParams: {e}"))?;

    let result = attached
        .page
        .execute(params)
        .await
        .context("CDP Runtime.evaluate")?;
    let response = result.result;

    if let Some(exc) = response.exception_details {
        return Err(anyhow::anyhow!(
            "JS exception: {}",
            exc.text
        ));
    }

    let value = response
        .result
        .value
        .unwrap_or(serde_json::Value::Null);
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
