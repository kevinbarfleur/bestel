//! `bestel-driver` — autonomous debug harness for a running Bestel
//! instance. Drives the WebView2 frontend via the Chrome DevTools
//! Protocol exposed by Bestel when launched with `BESTEL_DEBUG_PORT`.
//!
//! See `tools/launch-debuggable.ps1` for the canonical launch sequence
//! and `tools/scenarios/*.toml` for replayable scripted runs.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod cdp;
mod commands;
mod session;

#[derive(Parser, Debug)]
#[command(
    name = "bestel-driver",
    version,
    about = "Autonomous debug harness for a running Bestel instance",
    long_about = "Drives the Bestel desktop app over the Chrome DevTools Protocol \
                  exposed by WebView2 when launched with BESTEL_DEBUG_PORT. Useful \
                  for reproducible UI / streaming / memory tests that the headless \
                  `bestel run-battery` cannot cover."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Discover the running Bestel instance and persist the port.
    Attach {
        /// CDP port (must match the value set in BESTEL_DEBUG_PORT).
        #[arg(long, default_value_t = 9222)]
        port: u16,
    },
    /// Run a JS expression in the page context and print the result.
    Eval {
        /// JavaScript expression. Async expressions are awaited.
        expr: String,
    },
    /// Capture a PNG screenshot of the main window.
    Screenshot {
        /// Output path. Parent directories are created if needed.
        #[arg(long, default_value = "screenshot.png")]
        out: PathBuf,
    },
    /// Print heap, DOM, and localStorage stats (requires window.__bestel).
    Memory,
    /// Stream or dump Bestel logs.
    Logs {
        /// Follow the log file forever (Ctrl-C to stop).
        #[arg(long)]
        tail: bool,
        /// Dump entries from the past duration. Format: 30s / 5m / 1h.
        #[arg(long)]
        since: Option<String>,
        /// Filter regex applied to each line.
        #[arg(long)]
        filter: Option<String>,
    },
    /// Print the current persisted session.
    Session,
    /// Reset the chat store (and optionally attach a PoB build).
    NewChat {
        /// Absolute path to a Path of Building XML export.
        #[arg(long)]
        build: Option<String>,
    },
    /// Send a user message via the bridge.
    Send {
        /// The message text.
        text: String,
        /// Block until the assistant finishes streaming.
        #[arg(long)]
        wait_completion: bool,
        /// Timeout for --wait-completion. Format: 30s / 5m / 1h.
        #[arg(long, default_value = "90s")]
        timeout: String,
    },
    /// Wait until the assistant stops streaming, polling every 500ms.
    WaitCompletion {
        #[arg(long, default_value = "60s")]
        timeout: String,
    },
    /// Wait for an `Inspector.targetCrashed` event (used to verify OOM).
    WaitCrash {
        #[arg(long, default_value = "60s")]
        timeout: String,
    },
    /// Dump the current chat store as JSON.
    ChatState,
    /// Cancel any in-flight stream.
    Cancel,
    /// Run a TOML scenario (script of steps with timestamped output).
    Scenario {
        /// Path to the scenario TOML file.
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Attach { port } => commands::attach::run(port).await,
        Cmd::Eval { expr } => commands::eval::run(expr).await,
        Cmd::Screenshot { out } => commands::screenshot::run(out).await,
        Cmd::Memory => commands::memory::run().await,
        Cmd::Logs {
            tail,
            since,
            filter,
        } => {
            let since = since.as_deref().map(parse_duration).transpose()?;
            commands::logs::run(commands::logs::LogsArgs {
                tail,
                since,
                filter,
            })
            .await
        }
        Cmd::Session => {
            let s = session::load()?;
            println!("{}", serde_json::to_string_pretty(&s)?);
            Ok(())
        }
        Cmd::NewChat { build } => commands::chat::new_chat(build).await,
        Cmd::Send {
            text,
            wait_completion,
            timeout,
        } => {
            let timeout = parse_duration(&timeout)?;
            commands::chat::send(text, wait_completion, timeout).await
        }
        Cmd::WaitCompletion { timeout } => {
            let timeout = parse_duration(&timeout)?;
            commands::chat::wait_completion(timeout).await
        }
        Cmd::WaitCrash { timeout } => {
            let timeout = parse_duration(&timeout)?;
            commands::crash::run(timeout).await
        }
        Cmd::ChatState => commands::chat::chat_state().await,
        Cmd::Cancel => commands::chat::cancel().await,
        Cmd::Scenario { path } => commands::scenario::run(path).await,
    }
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .try_init();
}

fn parse_duration(s: &str) -> Result<Duration> {
    let (n, unit) = s.trim().split_at(s.trim().len().saturating_sub(1));
    let n: u64 = n.parse().map_err(|_| {
        anyhow::anyhow!("invalid duration `{s}` — expected like 30s / 5m / 1h")
    })?;
    let multiplier = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        other => {
            return Err(anyhow::anyhow!(
                "unknown duration unit `{other}` — use s / m / h"
            ));
        }
    };
    Ok(Duration::from_secs(n * multiplier))
}
