//! Chat interaction commands. All driven via the `window.__bestel`
//! bridge exposed by the frontend when launched in debug mode.

use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use serde_json::Value;
use tokio::time::sleep;

use crate::cdp;
use crate::session;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// `bestel-driver new-chat [--build <path>]` — reset the chat store and
/// optionally attach a PoB file.
pub async fn new_chat(build_path: Option<String>) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;

    let js = match build_path.as_deref() {
        Some(path) => format!(
            "window.__bestel.newChat({})",
            serde_json::to_string(path).expect("string serialize")
        ),
        None => "window.__bestel.newChat()".to_string(),
    };

    let value = run_js_async(&attached.page, &js).await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "status": "ok",
            "build": build_path,
            "result": value,
        }))?
    );
    Ok(())
}

/// `bestel-driver send "<msg>" [--wait-completion --timeout N]` — enqueue
/// a user message via the bridge. With `--wait-completion`, polls
/// `isStreaming()` until it returns false or the timeout fires.
pub async fn send(text: String, wait: bool, timeout: Duration) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;

    let escaped = serde_json::to_string(&text).expect("string serialize");
    let js = format!("window.__bestel.sendMessage({escaped})");
    run_js_async(&attached.page, &js).await?;

    if !wait {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "status": "sent",
                "wait": false,
            }))?
        );
        return Ok(());
    }

    let result = wait_for_completion(&attached.page, timeout).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// `bestel-driver wait-completion [--timeout N]` — standalone wait, useful
/// after a previous `send --no-wait` or for debugging mid-flight streams.
pub async fn wait_completion(timeout: Duration) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;
    let result = wait_for_completion(&attached.page, timeout).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// `bestel-driver chat-state` — dump the current chat store as JSON.
pub async fn chat_state() -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;
    let value = run_js(&attached.page, "window.__bestel.getChatState()").await?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

/// `bestel-driver cancel` — abort the in-flight stream.
pub async fn cancel() -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;
    run_js_async(&attached.page, "window.__bestel.cancelStreaming()").await?;
    println!(r#"{{"status":"cancelled"}}"#);
    Ok(())
}

async fn wait_for_completion(page: &Page, timeout: Duration) -> Result<Value> {
    let started = Instant::now();
    let mut iterations = 0u64;
    loop {
        if started.elapsed() > timeout {
            return Ok(serde_json::json!({
                "status": "timeout",
                "elapsed_ms": started.elapsed().as_millis(),
                "iterations": iterations,
            }));
        }

        // Combine the streaming check and a memory probe so the caller
        // sees how heap evolves while waiting — useful for the OOM repro
        // scenario.
        let probe = run_js(
            page,
            "({streaming: window.__bestel.isStreaming(), mem: window.__bestel.getMemoryStats()})",
        )
        .await?;

        let streaming = probe
            .get("streaming")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !streaming {
            return Ok(serde_json::json!({
                "status": "completed",
                "elapsed_ms": started.elapsed().as_millis(),
                "iterations": iterations,
                "final_mem": probe.get("mem"),
            }));
        }

        iterations += 1;
        sleep(POLL_INTERVAL).await;
    }
}

async fn run_js(page: &Page, expr: &str) -> Result<Value> {
    run_js_with(page, expr, false).await
}

async fn run_js_async(page: &Page, expr: &str) -> Result<Value> {
    run_js_with(page, expr, true).await
}

async fn run_js_with(page: &Page, expr: &str, await_promise: bool) -> Result<Value> {
    let params = EvaluateParams::builder()
        .expression(expr.to_string())
        .return_by_value(true)
        .await_promise(await_promise)
        .build()
        .map_err(|e| anyhow!("build EvaluateParams: {e}"))?;

    let resp = page
        .execute(params)
        .await
        .with_context(|| format!("CDP Runtime.evaluate `{}`", short(expr)))?
        .result;

    if let Some(exc) = resp.exception_details {
        return Err(anyhow!("JS exception: {}", exc.text));
    }
    Ok(resp.result.value.unwrap_or(Value::Null))
}

fn short(s: &str) -> String {
    if s.len() <= 80 {
        s.to_string()
    } else {
        format!("{}…", &s[..80])
    }
}
