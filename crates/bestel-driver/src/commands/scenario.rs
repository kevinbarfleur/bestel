//! `bestel-driver scenario <toml-path>` — replays a scripted sequence of
//! steps against a running Bestel instance and collects the outputs into
//! a timestamped run directory.
//!
//! Format (TOML):
//!
//! ```toml
//! name = "repro-oom-whisperer"
//! description = "..."
//! out_dir = "tools/scenarios/runs"   # optional, default below
//!
//! [[step]]
//! type = "new-chat"
//! build = "C:/.../Penance Brand.xml"
//!
//! [[step]]
//! type = "send"
//! text = "why do I die in t16?"
//!
//! [[step]]
//! type = "memory-probe"
//! interval_ms = 1500
//! duration_ms = 60000
//!
//! [[step]]
//! type = "wait-completion-or-crash"
//! timeout_ms = 90000
//!
//! [[step]]
//! type = "screenshot"
//! ```

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use chromiumoxide::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::Page;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::sleep;

use crate::cdp;
use crate::session;

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Output directory root (timestamped subdir is appended). Defaults to
    /// `tools/scenarios/runs` relative to the CWD.
    #[serde(default)]
    pub out_dir: Option<String>,
    #[serde(default)]
    pub step: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Step {
    NewChat {
        #[serde(default)]
        build: Option<String>,
    },
    Send {
        text: String,
    },
    MemoryProbe {
        #[serde(default = "default_probe_interval")]
        interval_ms: u64,
        #[serde(default = "default_probe_duration")]
        duration_ms: u64,
    },
    WaitCompletionOrCrash {
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    Screenshot {
        #[serde(default)]
        out: Option<String>,
    },
    Sleep {
        ms: u64,
    },
    Eval {
        expr: String,
    },
    /// Switch the active LLM model profile by id (e.g. `anthropic-haiku-4-5`,
    /// `deepseek-v4-flash`, `deepseek-v4-pro`). Lets a single scenario
    /// run the same prompts against multiple models for quality comparison.
    SetModel {
        id: String,
    },
    /// Delete the persisted Build Sheet (if any) for the currently-active
    /// build. Forces the `Build sheet: absent` runtime tag on the next
    /// turn — useful to test the one-shot interview flow without going
    /// through the UI's sheet picker. Returns the deleted sheet's id, or
    /// `null` when no sheet existed. Fails if no build is attached.
    DeleteSheet,
    /// Clear the active build (force `Build state: detached` runtime tag
    /// on the next turn). Mirrors clicking the build detach button in the
    /// UI.
    ClearActiveBuild,
    /// Poll for an active one-shot interview (sheet_open_interview tool
    /// result rendered into a BSInterviewPanel). Returns the interview
    /// payload (sections, questions, notes prompt) once detected. Times
    /// out with an explicit error if no interview appears.
    WaitForInterview {
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    /// Auto-fill the active interview and submit it. The driver picks the
    /// first option for every leverage question by default ; overrides
    /// can be supplied per question via `answers`. Section bodies use the
    /// agent's drafts unless overridden via `sections`. The synthesized
    /// `[INTERVIEW SUBMISSION ...]` user message is sent through the
    /// chat pipeline, and the driver then waits for the assistant's
    /// post-submission turn (sheet_finalize_request + answer).
    SubmitInterview {
        #[serde(default)]
        answers: std::collections::HashMap<String, Vec<usize>>,
        #[serde(default)]
        sections: std::collections::HashMap<String, String>,
        #[serde(default)]
        notes: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
}

fn default_probe_interval() -> u64 {
    1500
}
fn default_probe_duration() -> u64 {
    60000
}
fn default_timeout() -> u64 {
    90000
}

pub async fn run(toml_path: PathBuf) -> Result<()> {
    let body = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("read scenario {}", toml_path.display()))?;
    let scenario: Scenario =
        toml::from_str(&body).with_context(|| format!("parse scenario {}", toml_path.display()))?;

    // Output dir = <out_dir | tools/scenarios/runs>/<scenario_name>-<ts>
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let root = scenario
        .out_dir
        .as_deref()
        .unwrap_or("tools/scenarios/runs");
    let run_dir = PathBuf::from(root).join(format!("{}-{stamp}", scenario.name));
    std::fs::create_dir_all(&run_dir)
        .with_context(|| format!("create run dir {}", run_dir.display()))?;

    eprintln!(
        "[scenario] {} — {} steps, output {}",
        scenario.name,
        scenario.step.len(),
        run_dir.display()
    );

    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;

    let started = Instant::now();
    let mut step_results: Vec<Value> = Vec::new();
    let mut crashed = false;

    for (i, step) in scenario.step.iter().enumerate() {
        let step_started = Instant::now();
        eprintln!("[scenario] step {}/{}: {step:?}", i + 1, scenario.step.len());

        let result =
            run_step(step, &attached.page, &run_dir, i + 1).await;
        let elapsed_ms = step_started.elapsed().as_millis();

        match result {
            Ok(value) => {
                step_results.push(json!({
                    "index": i + 1,
                    "type": step_type(step),
                    "elapsed_ms": elapsed_ms,
                    "ok": true,
                    "result": value,
                }));
                if value.get("crashed").and_then(|v| v.as_bool()) == Some(true) {
                    crashed = true;
                    eprintln!("[scenario] crash detected — stopping");
                    break;
                }
            }
            Err(e) => {
                step_results.push(json!({
                    "index": i + 1,
                    "type": step_type(step),
                    "elapsed_ms": elapsed_ms,
                    "ok": false,
                    "error": e.to_string(),
                }));
                eprintln!("[scenario] step {} failed: {e}", i + 1);
                break;
            }
        }
    }

    let summary = json!({
        "name": scenario.name,
        "description": scenario.description,
        "scenario_path": toml_path.display().to_string(),
        "started_at": chrono::Utc::now().to_rfc3339(),
        "total_elapsed_ms": started.elapsed().as_millis(),
        "step_count": scenario.step.len(),
        "executed_steps": step_results.len(),
        "crashed": crashed,
        "steps": step_results,
    });

    let summary_path = run_dir.join("summary.json");
    std::fs::write(&summary_path, serde_json::to_vec_pretty(&summary)?)
        .with_context(|| format!("write {}", summary_path.display()))?;

    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn step_type(step: &Step) -> &'static str {
    match step {
        Step::NewChat { .. } => "new-chat",
        Step::Send { .. } => "send",
        Step::MemoryProbe { .. } => "memory-probe",
        Step::WaitCompletionOrCrash { .. } => "wait-completion-or-crash",
        Step::Screenshot { .. } => "screenshot",
        Step::Sleep { .. } => "sleep",
        Step::Eval { .. } => "eval",
        Step::SetModel { .. } => "set-model",
        Step::DeleteSheet => "delete-sheet",
        Step::ClearActiveBuild => "clear-active-build",
        Step::WaitForInterview { .. } => "wait-for-interview",
        Step::SubmitInterview { .. } => "submit-interview",
    }
}

async fn run_step(
    step: &Step,
    page: &Page,
    run_dir: &Path,
    index: usize,
) -> Result<Value> {
    match step {
        Step::NewChat { build } => {
            let js = match build {
                Some(b) => format!(
                    "window.__bestel.newChat({})",
                    serde_json::to_string(b)?
                ),
                None => "window.__bestel.newChat()".to_string(),
            };
            run_js(page, &js, true).await?;
            Ok(json!({"action": "new-chat", "build": build}))
        }
        Step::Send { text } => {
            let js = format!(
                "window.__bestel.sendMessage({})",
                serde_json::to_string(text)?
            );
            run_js(page, &js, true).await?;
            Ok(json!({"action": "send", "text": text}))
        }
        Step::MemoryProbe {
            interval_ms,
            duration_ms,
        } => {
            let interval = Duration::from_millis(*interval_ms);
            let until = Instant::now() + Duration::from_millis(*duration_ms);
            let mut samples: Vec<Value> = Vec::new();
            while Instant::now() < until {
                let value = run_js(page, "window.__bestel.getMemoryStats()", false).await?;
                samples.push(json!({
                    "ts": chrono::Utc::now().to_rfc3339(),
                    "stats": value,
                }));
                sleep(interval).await;
            }
            // Persist the curve to a JSON file so the user can plot it.
            let path = run_dir.join(format!("step{index:02}-memory.json"));
            std::fs::write(&path, serde_json::to_vec_pretty(&samples)?)?;
            Ok(json!({
                "action": "memory-probe",
                "samples": samples.len(),
                "out": path.display().to_string(),
            }))
        }
        Step::WaitCompletionOrCrash { timeout_ms } => {
            let deadline = Duration::from_millis(*timeout_ms);
            wait_completion_or_crash(page, deadline).await
        }
        Step::Screenshot { out } => {
            let path = match out {
                Some(p) => PathBuf::from(p),
                None => run_dir.join(format!("step{index:02}-screenshot.png")),
            };
            let params = CaptureScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .build();
            let bytes = page.screenshot(params).await?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&path, &bytes)?;
            Ok(json!({"action": "screenshot", "out": path.display().to_string(), "bytes": bytes.len()}))
        }
        Step::Sleep { ms } => {
            sleep(Duration::from_millis(*ms)).await;
            Ok(json!({"action": "sleep", "ms": ms}))
        }
        Step::Eval { expr } => {
            let value = run_js(page, expr, true).await?;
            Ok(json!({"action": "eval", "expr": expr, "result": value}))
        }
        Step::SetModel { id } => {
            let js = format!(
                "window.__bestel.setActiveModel({})",
                serde_json::to_string(id)?
            );
            let value = run_js(page, &js, true).await?;
            Ok(json!({"action": "set-model", "id": id, "result": value}))
        }
        Step::DeleteSheet => {
            let js =
                "window.__bestel.invoke('delete_active_build_sheet')".to_string();
            let value = run_js(page, &js, true).await?;
            Ok(json!({"action": "delete-sheet", "result": value}))
        }
        Step::ClearActiveBuild => {
            let js = "window.__bestel.invoke('clear_active_build')".to_string();
            run_js(page, &js, true).await?;
            Ok(json!({"action": "clear-active-build"}))
        }
        Step::WaitForInterview { timeout_ms } => {
            let deadline = Duration::from_millis(*timeout_ms);
            wait_for_interview(page, deadline).await
        }
        Step::SubmitInterview {
            answers,
            sections,
            notes,
            timeout_ms,
        } => {
            let payload = json!({
                "answers": answers,
                "sections": sections,
                "notes": notes,
            });
            let js = format!(
                "window.__bestel.autoSubmitInterview({})",
                serde_json::to_string(&payload)?
            );
            let submission_text = run_js(page, &js, true).await?;
            // Now wait for the agent's post-submission turn to finish
            // (sheet_finalize_request + answer to the original question).
            let deadline = Duration::from_millis(*timeout_ms);
            let post_wait = wait_completion_or_crash(page, deadline).await?;
            Ok(json!({
                "action": "submit-interview",
                "submitted_message": submission_text,
                "post_wait": post_wait,
            }))
        }
    }
}

/// Poll `window.__bestel.getActiveInterview()` every 1s until non-null
/// or the deadline expires. Returns the interview payload on success.
async fn wait_for_interview(page: &Page, deadline: Duration) -> Result<Value> {
    let started = Instant::now();
    let interval = Duration::from_secs(1);
    loop {
        if started.elapsed() > deadline {
            return Err(anyhow!(
                "wait-for-interview timed out after {} ms — no interview panel detected",
                started.elapsed().as_millis()
            ));
        }
        let value = run_js(page, "window.__bestel.getActiveInterview()", false).await?;
        if !value.is_null() {
            return Ok(json!({
                "action": "wait-for-interview",
                "outcome": "interview-detected",
                "elapsed_ms": started.elapsed().as_millis(),
                "interview": value,
            }));
        }
        sleep(interval).await;
    }
}

/// Polls `window.__bestel.isStreaming()` every 5s until it returns false
/// or the deadline expires. Tolerates transient CDP timeouts (which
/// happen under heavy stream traffic) — single failures are logged but
/// don't abort the wait. The 5s interval is intentional: every CDP
/// `Runtime.evaluate` wakes the JS main thread and competes for cycles
/// with the streaming pipeline (delta events emitting from Tauri,
/// reactive updates, the snapshot deep-watch). Polling more aggressively
/// (2s, 500ms) creates measurable contention on long DeepSeek+verifier
/// turns and contributed to the system-wide RAM exhaustion observed
/// 2026-05-10 (see commit body).
///
/// The scenario `crashed` field stays `false` here ; use the standalone
/// `wait-crash` command for explicit crash listening (it uses a dedicated
/// event subscription that isn't time-shared with eval polling).
async fn wait_completion_or_crash(page: &Page, deadline: Duration) -> Result<Value> {
    let started = Instant::now();
    let mut consecutive_errors = 0u32;
    let mut last_error: Option<String> = None;

    loop {
        if started.elapsed() > deadline {
            return Ok(json!({
                "outcome": "timeout",
                "elapsed_ms": started.elapsed().as_millis(),
                "consecutive_errors": consecutive_errors,
                "last_error": last_error,
            }));
        }

        match run_js(page, "window.__bestel.isStreaming()", false).await {
            Ok(value) => {
                consecutive_errors = 0;
                let streaming = value.as_bool().unwrap_or(true);
                if !streaming {
                    let mem = run_js(page, "window.__bestel.getMemoryStats()", false)
                        .await
                        .ok();
                    return Ok(json!({
                        "outcome": "completed",
                        "elapsed_ms": started.elapsed().as_millis(),
                        "final_mem": mem,
                    }));
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                last_error = Some(e.to_string());
                // 5 errors in a row (~25s of failed polls at the 5s
                // interval) → bail. CDP went dead.
                if consecutive_errors >= 5 {
                    return Err(anyhow!(
                        "wait-completion: {} consecutive CDP errors; last: {e}",
                        consecutive_errors
                    ));
                }
                eprintln!(
                    "[scenario] wait-completion poll error ({consecutive_errors}/5): {e}"
                );
            }
        }

        sleep(Duration::from_millis(5000)).await;
    }
}

async fn run_js(page: &Page, expr: &str, await_promise: bool) -> Result<Value> {
    let params = EvaluateParams::builder()
        .expression(expr.to_string())
        .return_by_value(true)
        .await_promise(await_promise)
        .build()
        .map_err(|e| anyhow!("build EvaluateParams: {e}"))?;
    let resp = page.execute(params).await?.result;
    if let Some(exc) = resp.exception_details {
        return Err(anyhow!("JS exception: {}", exc.text));
    }
    Ok(resp.result.value.unwrap_or(Value::Null))
}
