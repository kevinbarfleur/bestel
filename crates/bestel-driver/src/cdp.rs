//! CDP attach helper. Connects chromiumoxide to a running Bestel instance
//! and surfaces the most relevant `Page` (the main app window).
//!
//! Bestel runs three Tauri windows: the main app (URL `tauri.localhost/#/`),
//! the prompt-editor (`prompt-editor.html`), and the dev-panel
//! (`dev-panel.html`). We pick the main window by URL: the only one
//! without `.html` in the path.
//!
//! Every command opens a fresh connection. CDP WebSocket sessions don't
//! survive between CLI invocations and that's intentional — keeps the
//! tool stateless and resilient to user-side restarts of the app.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chromiumoxide::{Browser, Page};
use futures_util::StreamExt;
use tokio::task::JoinHandle;

const HOST: &str = "127.0.0.1";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CdpTarget {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[allow(dead_code)]
    #[serde(rename = "webSocketDebuggerUrl")]
    pub websocket_debugger_url: Option<String>,
}

pub async fn list_targets(port: u16) -> Result<Vec<CdpTarget>> {
    let url = format!("http://{HOST}:{port}/json/list");
    let resp = reqwest::get(&url).await.with_context(|| {
        format!("HTTP GET {url} — is Bestel running with BESTEL_DEBUG_PORT={port}?")
    })?;
    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {}: {}", resp.status(), url));
    }
    let targets: Vec<CdpTarget> = resp
        .json()
        .await
        .with_context(|| format!("parse JSON from {url}"))?;
    Ok(targets)
}

/// Pick the main Bestel target from a `/json/list` snapshot. Heuristic:
/// type `page` AND URL does NOT contain `.html` (the sub-windows route to
/// `dev-panel.html` / `prompt-editor.html` ; the main app is at
/// `tauri.localhost/#/`).
pub fn pick_main_target(targets: &[CdpTarget]) -> Result<&CdpTarget> {
    targets
        .iter()
        .find(|t| t.kind == "page" && !t.url.contains(".html"))
        .ok_or_else(|| {
            let summary = targets
                .iter()
                .map(|t| format!("  [{}] {} — {}", t.kind, t.title, t.url))
                .collect::<Vec<_>>()
                .join("\n");
            anyhow!(
                "no main Bestel target found among {} CDP targets:\n{summary}",
                targets.len()
            )
        })
}

/// Fetch the browser-level WebSocket URL from `/json/version`. This is
/// what we feed to `Browser::connect` so it can enumerate existing pages.
async fn browser_ws_url(port: u16) -> Result<String> {
    let url = format!("http://{HOST}:{port}/json/version");
    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("HTTP GET {url}"))?;
    let v: serde_json::Value = resp.json().await.with_context(|| format!("parse {url}"))?;
    v.get("webSocketDebuggerUrl")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("no webSocketDebuggerUrl in /json/version"))
}

pub struct Attached {
    /// Held to keep the underlying CDP transport alive for the lifetime
    /// of `Attached`. We don't read it directly; calls go through `page`.
    #[allow(dead_code)]
    pub browser: Browser,
    pub page: Page,
    /// Background task that pumps the chromiumoxide handler. Must be kept
    /// alive for the duration of the command — dropping it kills the
    /// CDP message loop.
    pub _handler: JoinHandle<()>,
}

/// Open a CDP connection and return a `Page` handle on the main Bestel
/// window. The caller owns the returned `Attached` and must keep it
/// alive while issuing commands.
pub async fn connect_to_main(port: u16) -> Result<Attached> {
    // 1. Browser-level connect — this is what enables `Browser::pages()`
    // to enumerate the existing tabs. Connecting to a page-level URL
    // returns a Browser handle that doesn't track other targets.
    let ws = browser_ws_url(port).await?;
    let (browser, mut handler) = Browser::connect(&ws)
        .await
        .with_context(|| format!("CDP connect to {ws}"))?;

    let handler_task = tokio::spawn(async move {
        while let Some(_event) = handler.next().await {
            // Discard — per-page event subscriptions are made via
            // `page.event_listener::<E>()` and run on dedicated streams.
        }
    });

    // 2. Brief tick to let the handler flush incoming
    // `Target.targetCreated` events for already-existing tabs. Without
    // it, `Browser::pages()` is racy and sometimes returns empty.
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 3. Filter for the main Bestel page. The dev-panel and prompt-editor
    // both serve `*.html` URLs ; the main app routes to `/#/`.
    let pages = browser.pages().await.context("Browser::pages")?;

    let mut main_page: Option<Page> = None;
    for p in pages {
        if let Ok(url) = p.url().await {
            if let Some(u) = url {
                if !u.contains(".html") {
                    main_page = Some(p);
                    break;
                }
            }
        }
    }

    let page = main_page.ok_or_else(|| {
        anyhow!("no main page (without .html in URL) found among Browser::pages()")
    })?;

    Ok(Attached {
        browser,
        page,
        _handler: handler_task,
    })
}
