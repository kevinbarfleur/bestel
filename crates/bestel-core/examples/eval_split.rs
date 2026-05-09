//! Split `tests/eval/eval_set.toml` into 40 standalone scenario files
//! compatible with `bestel run-battery`.
//!
//! Run from the repo root:
//!
//! ```
//! cargo run --release -p bestel-core --example eval_split -- tests/eval/eval_set.toml tests/eval/scenarios
//! ```
//!
//! Idempotent: existing files are overwritten so a re-split picks up
//! edits to the master file. The generated scenario files have the
//! Sprint A specialised expectations turned on conditionally:
//! - `must_have_identity_card_if_build` and `must_not_expose_reasoning`
//!   when `needs_identity = true`.
//! - `must_surface_calcs_echo_if_pob_calc` when `needs_pob_calc = true`.
//! - `must_cite_only_fetched_urls` and `must_have_panel_data_first` are
//!   always on for the eval set so the linter gates citation hygiene
//!   regardless of category.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EvalSet {
    entry: Vec<EvalEntry>,
}

#[derive(Debug, Deserialize)]
struct EvalEntry {
    id: String,
    category: String,
    prompt: String,
    #[serde(default)]
    build_fixture: Option<String>,
    #[serde(default)]
    needs_identity: bool,
    #[serde(default)]
    needs_pob_calc: bool,
    #[serde(default)]
    expected_signals: Vec<String>,
    #[serde(default)]
    forbidden_signals: Vec<String>,
    /// Sprint H — game scope tag. When set, scenarios are persisted with
    /// `applies_to = [...]` so the harness can grep PoE2-only entries
    /// (the contamination-free exit-criterion subset). Defaults to `["poe1",
    /// "poe2"]` if absent — backwards-compatible with the Sprint A entries.
    #[serde(default)]
    applies_to: Vec<String>,
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let src: PathBuf = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("usage: eval_split <eval_set.toml> <out_dir>"))?;
    let out_dir: PathBuf = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("usage: eval_split <eval_set.toml> <out_dir>"))?;

    let raw = std::fs::read_to_string(&src)
        .with_context(|| format!("read {}", src.display()))?;
    let set: EvalSet = toml::from_str(&raw).context("parse eval_set.toml")?;
    if set.entry.len() != 60 {
        eprintln!(
            "warning: expected 60 eval entries, found {}",
            set.entry.len()
        );
    }

    std::fs::create_dir_all(&out_dir)
        .with_context(|| format!("mkdir {}", out_dir.display()))?;

    for e in &set.entry {
        let path = out_dir.join(format!("{}.toml", e.id));
        let body = render_scenario(e);
        std::fs::write(&path, body)
            .with_context(|| format!("write {}", path.display()))?;
        println!("  wrote {}", path.display());
    }

    println!(
        "Split {} scenarios into {}",
        set.entry.len(),
        out_dir.display()
    );
    Ok(())
}

fn render_scenario(e: &EvalEntry) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Generated from tests/eval/eval_set.toml — do not hand-edit.\n"));
    out.push_str(&format!("# Category: {}\n", e.category));
    out.push_str("\n");
    out.push_str(&format!("name = \"{}\"\n", e.id));
    out.push_str("provider = \"any\"\n");
    if let Some(fix) = &e.build_fixture {
        out.push_str(&format!("build_fixture = \"{}\"\n", fix));
    }
    if !e.applies_to.is_empty() {
        let joined = e
            .applies_to
            .iter()
            .map(|s| toml_string(s))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("applies_to = [{}]\n", joined));
    }
    out.push_str(&format!("prompt = \"\"\"\n{}\n\"\"\"\n", e.prompt));
    out.push_str("timeout_secs = 240\n");
    out.push_str("cost = \"low\"\n");
    out.push_str("\n");
    out.push_str("[[expectations]]\n");
    if !e.expected_signals.is_empty() {
        out.push_str("must_match = [\n");
        for s in &e.expected_signals {
            out.push_str(&format!("  {},\n", toml_string(s)));
        }
        out.push_str("]\n");
    }
    if !e.forbidden_signals.is_empty() {
        out.push_str("must_not_match = [\n");
        for s in &e.forbidden_signals {
            out.push_str(&format!("  {},\n", toml_string(s)));
        }
        out.push_str("]\n");
    }
    out.push_str("must_not_expose_reasoning = true\n");
    out.push_str("must_have_panel_data_first = true\n");
    out.push_str("must_cite_only_fetched_urls = true\n");
    if e.needs_identity {
        out.push_str("must_have_identity_card_if_build = true\n");
    }
    if e.needs_pob_calc {
        out.push_str("must_surface_calcs_echo_if_pob_calc = true\n");
        out.push_str("must_not_claim_real_number_if_pob_calc_failed = true\n");
    }
    out
}

/// Emit a TOML-safe string literal. Falls back to a literal string with
/// escaped quotes/backslashes; the eval signals are short enough that
/// fancy multiline encoding is not needed.
fn toml_string(s: &str) -> String {
    let escaped: String = s
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            other => other.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}
