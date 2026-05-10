//! `bestel-driver logs` — combined stream of WebView console events
//! (`[wv]` prefix) and Rust tracing file logs (`[rs]` prefix), sorted
//! chronologically.
//!
//! Modes:
//! - `--tail` (default): follow the file log AND the live console; print
//!   forever until Ctrl-C.
//! - `--since <duration>`: snapshot of file log entries within the past
//!   duration ; ignores live console.
//! - `--filter <regex>`: applied to the prefix-stripped line.
//!
//! For the MVP we focus on the file-log side. Live console subscription
//! over CDP needs a separate event subscription which we'll add as the
//! second iteration of this command.

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use chromiumoxide::cdp::js_protocol::runtime::EventConsoleApiCalled;
use futures_util::StreamExt;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::sleep;

use crate::cdp;
use crate::session;

pub struct LogsArgs {
    pub tail: bool,
    pub since: Option<Duration>,
    pub filter: Option<String>,
}

pub async fn run(args: LogsArgs) -> Result<()> {
    let log_dir = bestel_log_dir().context("home dir resolution")?;
    let log_file = pick_today_log(&log_dir)?;

    let regex = args
        .filter
        .as_ref()
        .map(|s| Regex::new(s))
        .transpose()
        .context("compile --filter regex")?;

    if args.tail {
        tail_combined(&log_file, regex.clone()).await
    } else if let Some(window) = args.since {
        dump_since(&log_file, window, regex.as_ref()).await
    } else {
        // No flag → behave like --since 60s as a sane default.
        dump_since(&log_file, Duration::from_secs(60), regex.as_ref()).await
    }
}

/// Tail mode: file log + live WebView console events, merged. Each line
/// is tagged `[rs]` (Rust tracing) or `[wv]` (WebView console).
async fn tail_combined(log_file: &PathBuf, regex: Option<Regex>) -> Result<()> {
    let log_file = log_file.clone();
    let regex_file = regex.clone();
    let regex_console = regex.clone();

    // File tailer task — keeps running even if CDP session can't be
    // established (Bestel not running, BESTEL_DEBUG_PORT not set).
    let file_task = tokio::spawn(async move {
        if let Err(e) = tail_file(&log_file, regex_file.as_ref()).await {
            eprintln!("[bestel-driver] file tail stopped: {e}");
        }
    });

    // Console tailer — best-effort. If we can't attach (no session,
    // CDP unavailable), log a warning and let the file tail carry on.
    let console_task = tokio::spawn(async move {
        match tail_console(regex_console).await {
            Ok(()) => {}
            Err(e) => eprintln!("[bestel-driver] console tail unavailable: {e}"),
        }
    });

    // Whichever finishes first signals end of stream — usually a
    // Ctrl-C from the user that drops the file handle.
    tokio::select! {
        _ = file_task => {}
        _ = console_task => {}
    }
    Ok(())
}

async fn tail_console(regex: Option<Regex>) -> Result<()> {
    let s = session::load()?;
    let attached = cdp::connect_to_main(s.port).await?;
    let mut events = attached
        .page
        .event_listener::<EventConsoleApiCalled>()
        .await?;

    while let Some(event) = events.next().await {
        let kind = format!("{:?}", event.r#type).to_lowercase();
        // `args` is a Vec<RemoteObject>; serialize the literal/text form.
        let parts: Vec<String> = event
            .args
            .iter()
            .map(|arg| {
                arg.value
                    .as_ref()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .or_else(|| arg.description.clone())
                    .unwrap_or_default()
            })
            .collect();
        let line = format!("[wv:{kind}] {}", parts.join(" "));
        if matches_filter(&line, regex.as_ref()) {
            println!("{line}");
        }
    }
    Ok(())
}

fn bestel_log_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("no home dir")?;
    Ok(home.join(".bestel").join("runtime").join("logs"))
}

/// `tracing-appender` rolling-daily writes to `bestel.log.{YYYY-MM-DD}`.
/// We pick the most recently modified file in the directory so we
/// follow the active day even at midnight rollover.
fn pick_today_log(dir: &PathBuf) -> Result<PathBuf> {
    let entries = std::fs::read_dir(dir).with_context(|| {
        format!(
            "read log dir {} — was Bestel launched with BESTEL_FILE_LOG=1?",
            dir.display()
        )
    })?;
    let mut best: Option<(SystemTime, PathBuf)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("bestel.log") {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        match &best {
            Some((current_best, _)) if modified <= *current_best => {}
            _ => best = Some((modified, entry.path())),
        }
    }
    best.map(|(_, p)| p)
        .ok_or_else(|| anyhow::anyhow!("no bestel.log* files found in {}", dir.display()))
}

async fn dump_since(
    file: &PathBuf,
    window: Duration,
    filter: Option<&Regex>,
) -> Result<()> {
    let cutoff = SystemTime::now() - window;
    let cutoff_secs = cutoff
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let f = tokio::fs::File::open(file)
        .await
        .with_context(|| format!("open {}", file.display()))?;
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        // tracing default format: 2026-05-10T12:34:56.789Z LEVEL ...
        if let Some(ts) = parse_timestamp(&line) {
            if ts < cutoff_secs {
                line.clear();
                continue;
            }
        }
        let stripped = line.trim_end();
        if matches_filter(stripped, filter) {
            println!("[rs] {stripped}");
        }
        line.clear();
    }
    Ok(())
}

async fn tail_file(file: &PathBuf, filter: Option<&Regex>) -> Result<()> {
    let f = tokio::fs::File::open(file)
        .await
        .with_context(|| format!("open {}", file.display()))?;
    let metadata = f.metadata().await?;
    let mut position = metadata.len();
    drop(f);

    println!("[bestel-driver] tailing {} ...", file.display());
    loop {
        let mut f = tokio::fs::File::open(file).await?;
        let new_len = f.metadata().await?.len();
        if new_len > position {
            use tokio::io::AsyncSeekExt;
            f.seek(std::io::SeekFrom::Start(position)).await?;
            let mut reader = BufReader::new(f);
            let mut line = String::new();
            while reader.read_line(&mut line).await? > 0 {
                let stripped = line.trim_end();
                if matches_filter(stripped, filter) {
                    println!("[rs] {stripped}");
                }
                line.clear();
            }
            position = new_len;
        } else if new_len < position {
            // File rotated (midnight) — reset.
            position = 0;
        }
        sleep(Duration::from_millis(500)).await;
    }
}

fn parse_timestamp(line: &str) -> Option<u64> {
    // Best-effort: try to parse the leading RFC3339 timestamp emitted by
    // tracing-subscriber's default fmt layer.
    let ts_str = line.split_whitespace().next()?;
    chrono::DateTime::parse_from_rfc3339(ts_str)
        .ok()
        .map(|d| d.timestamp() as u64)
}

fn matches_filter(line: &str, filter: Option<&Regex>) -> bool {
    match filter {
        Some(re) => re.is_match(line),
        None => true,
    }
}
