//! Fetch fresh repoe-fork + trade-stats snapshots and write zstd-19
//! compressed blobs into `crates/bestel-core/src/sources/snapshots/`.
//!
//! Run from the workspace root:
//!
//! ```powershell
//! cargo run --release -p bestel-core --example refresh_snapshots
//! ```
//!
//! The script writes a `version.txt` manifest next to the blobs that lists
//! every fetched / skipped pair plus the total compressed footprint.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

const REPOE_BASE_POE1: &str = "https://repoe-fork.github.io";
const REPOE_BASE_POE2: &str = "https://repoe-fork.github.io/poe2";
const TRADE_STATS_POE1: &str = "https://www.pathofexile.com/api/trade/data/stats";
const TRADE_STATS_POE2: &str = "https://www.pathofexile.com/api/trade2/data/stats";

const POE1_CATS: &[&str] = &[
    "mods",
    "base_items",
    "gems",
    "uniques",
    "cluster_jewels",
    "essences",
    "fossils",
    "stat_translations",
];

const POE2_CATS: &[&str] = &[
    "mods",
    "base_items",
    "gems",
    "uniques",
    "stat_translations",
];

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("bestel-snapshot-refresher/0.1 (contact: hi@kevinbarfleur.dev)")
        .build()?;

    let root = workspace_root()?;
    let snapshots_dir = root
        .join("crates")
        .join("bestel-core")
        .join("src")
        .join("sources")
        .join("snapshots");
    fs::create_dir_all(snapshots_dir.join("poe1"))?;
    fs::create_dir_all(snapshots_dir.join("poe2"))?;
    fs::create_dir_all(snapshots_dir.join("trade_stats"))?;

    let mut report: Vec<String> = Vec::new();
    let mut total_compressed: usize = 0;

    for cat in POE1_CATS {
        let url = format!("{REPOE_BASE_POE1}/{cat}.min.json");
        let dest = snapshots_dir.join("poe1").join(format!("{cat}.json.zst"));
        match fetch_and_write(&client, &url, &dest).await {
            Ok(stats) => {
                report.push(format!(
                    "poe1/{cat}: {raw} raw -> {comp} zstd",
                    raw = human(stats.raw),
                    comp = human(stats.compressed)
                ));
                total_compressed += stats.compressed;
            }
            Err(e) => {
                report.push(format!("poe1/{cat}: SKIPPED ({e})"));
                eprintln!("[warn] poe1/{cat} skipped: {e}");
            }
        }
    }

    for cat in POE2_CATS {
        let url = format!("{REPOE_BASE_POE2}/{cat}.min.json");
        let dest = snapshots_dir.join("poe2").join(format!("{cat}.json.zst"));
        match fetch_and_write(&client, &url, &dest).await {
            Ok(stats) => {
                report.push(format!(
                    "poe2/{cat}: {raw} raw -> {comp} zstd",
                    raw = human(stats.raw),
                    comp = human(stats.compressed)
                ));
                total_compressed += stats.compressed;
            }
            Err(e) => {
                report.push(format!("poe2/{cat}: SKIPPED ({e})"));
                eprintln!("[warn] poe2/{cat} skipped: {e}");
            }
        }
    }

    // Trade stats — official endpoint, slightly different shape (envelope
    // with `result` array). Stored verbatim; consumer parses on read.
    {
        let dest = snapshots_dir.join("trade_stats").join("poe1.json.zst");
        match fetch_and_write(&client, TRADE_STATS_POE1, &dest).await {
            Ok(stats) => {
                report.push(format!(
                    "trade_stats/poe1: {raw} raw -> {comp} zstd",
                    raw = human(stats.raw),
                    comp = human(stats.compressed)
                ));
                total_compressed += stats.compressed;
            }
            Err(e) => {
                report.push(format!("trade_stats/poe1: SKIPPED ({e})"));
                eprintln!("[warn] trade_stats/poe1 skipped: {e}");
            }
        }
    }
    {
        let dest = snapshots_dir.join("trade_stats").join("poe2.json.zst");
        match fetch_and_write(&client, TRADE_STATS_POE2, &dest).await {
            Ok(stats) => {
                report.push(format!(
                    "trade_stats/poe2: {raw} raw -> {comp} zstd",
                    raw = human(stats.raw),
                    comp = human(stats.compressed)
                ));
                total_compressed += stats.compressed;
            }
            Err(e) => {
                report.push(format!("trade_stats/poe2: SKIPPED ({e})"));
                eprintln!("[warn] trade_stats/poe2 skipped: {e}");
            }
        }
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut version_txt = format!(
        "# repoe-fork + trade-stats snapshot manifest\n# fetched_at: {timestamp}\n# total_compressed: {total} ({mb:.2} MB)\n\n",
        total = total_compressed,
        mb = total_compressed as f64 / 1_048_576.0
    );
    for line in &report {
        version_txt.push_str(line);
        version_txt.push('\n');
    }
    fs::write(snapshots_dir.join("version.txt"), version_txt)?;

    println!("\n--- summary ---");
    for line in &report {
        println!("  {line}");
    }
    println!(
        "\ntotal compressed: {} bytes ({:.2} MB)",
        total_compressed,
        total_compressed as f64 / 1_048_576.0
    );
    if total_compressed > 12 * 1_048_576 {
        eprintln!(
            "[warn] bundle exceeds 12 MB cap — consider dropping cluster_jewels or essences"
        );
    }
    Ok(())
}

struct FetchStats {
    raw: usize,
    compressed: usize,
}

async fn fetch_and_write(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
) -> Result<FetchStats> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow!("HTTP {status} on {url}"));
    }
    let body = resp.text().await?;
    let _: Value = serde_json::from_str(&body).with_context(|| format!("parse {url}"))?;
    let bytes = body.into_bytes();
    let compressed = zstd::encode_all(&bytes[..], 19)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("zst.tmp");
    fs::write(&tmp, &compressed)?;
    fs::rename(&tmp, dest)?;
    Ok(FetchStats {
        raw: bytes.len(),
        compressed: compressed.len(),
    })
}

fn workspace_root() -> Result<PathBuf> {
    let mut here = std::env::current_dir()?;
    loop {
        if here.join("Cargo.toml").exists() && here.join("crates").exists() {
            return Ok(here);
        }
        if !here.pop() {
            return Err(anyhow!("could not locate workspace root from cwd"));
        }
    }
}

fn human(bytes: usize) -> String {
    let mb = bytes as f64 / 1_048_576.0;
    if mb >= 0.5 {
        format!("{mb:.2} MB")
    } else {
        let kb = bytes as f64 / 1024.0;
        format!("{kb:.1} kB")
    }
}
