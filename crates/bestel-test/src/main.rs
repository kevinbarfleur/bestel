//! Bestel scenario runner.
//!
//! Loads `tests/scenarios/*.toml`, runs each through `bestel chat --json`,
//! evaluates expectations, prints a pass/fail report. Designed for me to
//! regress-test the agent without touching the TUI.
//!
//!     cargo run -p bestel-test -- \
//!         --scenarios tests/scenarios \
//!         --provider codex \
//!         --cost-max low \
//!         --filter "max_resistance|tabula"
//!
//! Each scenario in `tests/scenarios/*.toml` describes a prompt, an optional
//! PoB build fixture, and assertions. Build fixtures live in
//! `tests/fixtures/pob/` and are referenced by name minus `.xml`.

mod expectations;
mod scenario;

use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use scenario::{Cost, ProviderChoice, Scenario};
use serde_json::Value;
use tokio::process::Command;

#[derive(Default)]
struct Args {
    scenarios_dir: Option<PathBuf>,
    fixtures_dir: Option<PathBuf>,
    bestel_bin: Option<PathBuf>,
    provider: Option<ProviderChoice>,
    cost_max: Option<Cost>,
    filter: Option<Regex>,
    json: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = parse_args()?;
    let workspace = workspace_root()?;
    let scenarios_dir = args
        .scenarios_dir
        .clone()
        .unwrap_or_else(|| workspace.join("tests/scenarios"));
    let fixtures_dir = args
        .fixtures_dir
        .clone()
        .unwrap_or_else(|| workspace.join("tests/fixtures/pob"));
    let bestel_bin = args
        .bestel_bin
        .clone()
        .unwrap_or_else(|| workspace.join("target/release/bestel.exe"));

    if !bestel_bin.is_file() {
        return Err(anyhow!(
            "bestel binary not found at {} — run `cargo build --release` first",
            bestel_bin.display()
        ));
    }

    let paths = scenario::discover_scenarios(&scenarios_dir)?;
    let mut total = 0;
    let mut passed = 0;
    let mut failed: Vec<(String, Vec<String>)> = Vec::new();
    let mut skipped: Vec<(String, &'static str)> = Vec::new();

    for path in paths {
        let scenario = match scenario::load_scenario(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("⚠ failed to load {}: {}", path.display(), e);
                continue;
            }
        };

        if let Some(re) = &args.filter {
            if !re.is_match(&scenario.name) {
                continue;
            }
        }

        if let Some(max) = args.cost_max {
            if scenario.cost > max {
                skipped.push((scenario.name.clone(), "cost > max"));
                continue;
            }
        }

        if let Some(forced) = args.provider {
            if !provider_matches(scenario.provider, forced) {
                skipped.push((scenario.name.clone(), "provider mismatch"));
                continue;
            }
        }

        total += 1;
        let banner_provider = args
            .provider
            .unwrap_or(if scenario.provider == ProviderChoice::Any {
                ProviderChoice::Codex
            } else {
                scenario.provider
            });
        println!(
            "▶ {} ({}) — {} expectations",
            scenario.name,
            banner_provider.as_cli_flag(),
            scenario.expectations.len()
        );

        let started = Instant::now();
        match run_one(&scenario, &bestel_bin, &fixtures_dir, banner_provider).await {
            Ok(result_json) => {
                let mut all_failures: Vec<String> = Vec::new();
                if let Some(err) = result_json.get("error").and_then(|v| v.as_str()) {
                    if !err.is_empty() {
                        all_failures.push(format!("provider error: {err}"));
                    }
                }
                for exp in &scenario.expectations {
                    let r = expectations::evaluate(exp, &result_json);
                    if !r.passed {
                        all_failures.extend(r.failures);
                    }
                }
                let elapsed = started.elapsed();
                if all_failures.is_empty() {
                    passed += 1;
                    println!("  ✓ pass · {} ms\n", elapsed.as_millis());
                } else {
                    println!("  ✗ fail · {} ms", elapsed.as_millis());
                    for f in &all_failures {
                        println!("      - {f}");
                    }
                    println!();
                    failed.push((scenario.name.clone(), all_failures));
                }
            }
            Err(e) => {
                println!("  ✗ runner error: {e}\n");
                failed.push((scenario.name.clone(), vec![e.to_string()]));
            }
        }
    }

    println!(
        "═══ summary ═══  {}/{} passed · {} failed · {} skipped",
        passed,
        total,
        failed.len(),
        skipped.len()
    );
    if !skipped.is_empty() {
        for (n, reason) in &skipped {
            println!("  · skipped {n}: {reason}");
        }
    }

    if !failed.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn provider_matches(scenario: ProviderChoice, forced: ProviderChoice) -> bool {
    matches!(scenario, ProviderChoice::Any) || scenario == forced
}

async fn run_one(
    scenario: &Scenario,
    bestel_bin: &std::path::Path,
    fixtures_dir: &std::path::Path,
    forced_provider: ProviderChoice,
) -> Result<Value> {
    let mut cmd = Command::new(bestel_bin);
    cmd.arg("chat")
        .arg("--provider")
        .arg(forced_provider.as_cli_flag())
        .arg("--json")
        .arg("--timeout-secs")
        .arg(scenario.timeout_secs.to_string());
    if let Some(fix) = &scenario.build_fixture {
        let p = fixtures_dir.join(format!("{fix}.xml"));
        if !p.is_file() {
            return Err(anyhow!("build fixture not found: {}", p.display()));
        }
        cmd.arg("--build").arg(p);
    }
    cmd.arg(&scenario.prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let timeout = Duration::from_secs(scenario.timeout_secs.saturating_add(30));
    let output = tokio::time::timeout(timeout, cmd.output())
        .await
        .map_err(|_| anyhow!("scenario hard timeout"))??;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if stdout.trim().is_empty() {
        return Err(anyhow!(
            "bestel chat produced no stdout (stderr: {})",
            truncate(&stderr, 200)
        ));
    }
    let v: Value = serde_json::from_str(&stdout)
        .with_context(|| format!("parse JSON output (stderr: {})", truncate(&stderr, 200)))?;
    Ok(v)
}

fn parse_args() -> Result<Args> {
    let mut out = Args::default();
    let mut iter = std::env::args().skip(1);
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--scenarios" => {
                out.scenarios_dir = Some(PathBuf::from(
                    iter.next().ok_or_else(|| anyhow!("--scenarios needs a path"))?,
                ));
            }
            "--fixtures" => {
                out.fixtures_dir = Some(PathBuf::from(
                    iter.next().ok_or_else(|| anyhow!("--fixtures needs a path"))?,
                ));
            }
            "--bestel-bin" => {
                out.bestel_bin = Some(PathBuf::from(
                    iter.next().ok_or_else(|| anyhow!("--bestel-bin needs a path"))?,
                ));
            }
            "--provider" => {
                let v = iter
                    .next()
                    .ok_or_else(|| anyhow!("--provider needs a value"))?;
                out.provider = Some(
                    ProviderChoice::parse(&v)
                        .ok_or_else(|| anyhow!("unknown provider '{v}'"))?,
                );
            }
            "--cost-max" => {
                let v = iter
                    .next()
                    .ok_or_else(|| anyhow!("--cost-max needs a value"))?;
                out.cost_max = Some(match v.as_str() {
                    "low" => Cost::Low,
                    "high" => Cost::High,
                    _ => return Err(anyhow!("--cost-max must be 'low' or 'high'")),
                });
            }
            "--filter" => {
                let v = iter
                    .next()
                    .ok_or_else(|| anyhow!("--filter needs a regex"))?;
                out.filter = Some(Regex::new(&v).context("invalid filter regex")?);
            }
            "--json" => out.json = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(anyhow!("unknown argument '{other}'")),
        }
    }
    Ok(out)
}

fn print_help() {
    println!("bestel-test — scenario runner for the Bestel agent\n");
    println!("USAGE:");
    println!("  bestel-test [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  --scenarios <dir>   tests/scenarios (default)");
    println!("  --fixtures <dir>    tests/fixtures/pob (default)");
    println!("  --bestel-bin <path> target/release/bestel.exe (default)");
    println!("  --provider <auto|codex|claude|anthropic>");
    println!("  --cost-max <low|high>");
    println!("  --filter <regex>    run only scenarios whose name matches");
}

fn workspace_root() -> Result<PathBuf> {
    let mut here = std::env::current_dir()?;
    loop {
        if here.join("Cargo.toml").is_file() && here.join("crates").is_dir() {
            return Ok(here);
        }
        if !here.pop() {
            return Err(anyhow!(
                "could not locate Bestel workspace root from {:?}",
                std::env::current_dir()
            ));
        }
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        &s[..end]
    }
}
