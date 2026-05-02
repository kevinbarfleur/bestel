//! Optional developer-only event logger.
//!
//! Enable by setting `BESTEL_DEV_LOG=1` (or any non-empty value) before launch.
//! Writes one JSONL file per process to `~/.bestel/logs/session-YYYYMMDD-HHMMSS.jsonl`
//! (or `BESTEL_DEV_LOG_DIR` if defined).
//!
//! Each line: `{"ts":"…","cat":"…","data":{…}}`.
//! Categories used by the codebase :
//! - `provider_raw`     : raw line/event from the provider stream (codex JSONL,
//!                        claude JSONL, anthropic SSE), with provider name.
//! - `delta`            : the `LlmDelta` we emit to the TUI.
//! - `user_input`       : the user message that started a turn.
//! - `assistant_final`  : the full assistant text once a turn ends.
//! - `pob_update`       : PoB build was reparsed.
//! - `provider_error`   : transport / parse / auth errors.
//! - `note`             : free-form developer note.
//!
//! Disabled at runtime if the env flag is not set — the helpers become no-ops
//! so we can sprinkle calls anywhere without performance impact.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;
use serde::Serialize;
use serde_json::{json, Value};

static LOGGER: std::sync::OnceLock<Option<DevLogger>> = std::sync::OnceLock::new();

struct DevLogger {
    path: PathBuf,
    file: Mutex<std::fs::File>,
}

fn enabled() -> bool {
    std::env::var("BESTEL_DEV_LOG")
        .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
        .unwrap_or(false)
}

fn logger() -> Option<&'static DevLogger> {
    LOGGER
        .get_or_init(|| {
            if !enabled() {
                return None;
            }
            let dir = std::env::var("BESTEL_DEV_LOG_DIR")
                .ok()
                .map(PathBuf::from)
                .or_else(|| dirs::home_dir().map(|h| h.join(".bestel").join("logs")))?;
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("[bestel devlog] create dir failed: {e}");
                return None;
            }
            let stamp = Local::now().format("session-%Y%m%d-%H%M%S").to_string();
            let path = dir.join(format!("{stamp}.jsonl"));
            let file = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("[bestel devlog] open {} failed: {e}", path.display());
                    return None;
                }
            };
            eprintln!("[bestel devlog] writing to {}", path.display());
            Some(DevLogger {
                path,
                file: Mutex::new(file),
            })
        })
        .as_ref()
}

pub fn is_enabled() -> bool {
    logger().is_some()
}

pub fn log_path() -> Option<&'static PathBuf> {
    logger().map(|l| &l.path)
}

pub fn log<S: Serialize>(category: &str, data: &S) {
    let Some(l) = logger() else { return };
    let value = match serde_json::to_value(data) {
        Ok(v) => v,
        Err(e) => json!({"_serialize_error": e.to_string()}),
    };
    write_line(l, category, value);
}

pub fn log_value(category: &str, data: Value) {
    let Some(l) = logger() else { return };
    write_line(l, category, data);
}

pub fn log_str(category: &str, key: &str, value: &str) {
    let Some(l) = logger() else { return };
    let v = json!({ key: value });
    write_line(l, category, v);
}

pub fn log_provider_raw(provider: &str, line: &str) {
    let Some(l) = logger() else { return };
    let v = json!({
        "provider": provider,
        "line": line,
    });
    write_line(l, "provider_raw", v);
}

pub fn log_delta(provider: &str, delta_kind: &str, payload: Value) {
    let Some(l) = logger() else { return };
    let v = json!({
        "provider": provider,
        "kind": delta_kind,
        "payload": payload,
    });
    write_line(l, "delta", v);
}

fn write_line(l: &DevLogger, category: &str, data: Value) {
    let entry = json!({
        "ts": Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, false),
        "cat": category,
        "data": data,
    });
    if let Ok(mut f) = l.file.lock() {
        if let Ok(line) = serde_json::to_string(&entry) {
            let _ = writeln!(f, "{}", line);
            let _ = f.flush();
        }
    }
}
