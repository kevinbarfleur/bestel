//! Background daily refresh task for repoe-fork bundled snapshots.
//!
//! Runs at process start (after a 10 s warm-up delay) and re-checks every
//! 24 h. For each `(game, category)` pair, sends an HTTP HEAD with
//! `If-None-Match: <bundled_etag>`; on 200, downloads the body, validates
//! parse, zstd-19 compresses, atomic-writes to
//! `~/.bestel/cache/repoe/{game}/<cat>.json.zst`, and swaps the
//! `Arc<CategorySnapshot>` slot in the in-memory map.
//!
//! Failure mode is "log + continue, never panic". Bundled blobs remain the
//! offline-first fallback.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde_json::Value;

use super::cache::FileCache;
use super::http::PoeHttpClient;
use super::repoe::{Category, CategorySnapshot, Game, RepoeClient, SnapshotSource};

const REPOE_BASE_POE1: &str = "https://repoe-fork.github.io";
const REPOE_BASE_POE2: &str = "https://repoe-fork.github.io/poe2";
const BOOT_DELAY: Duration = Duration::from_secs(10);
const REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 3600);

pub fn spawn(client: Arc<RepoeClient>, http: PoeHttpClient) {
    tokio::spawn(async move {
        if let Err(e) = run(client, http).await {
            tracing::warn!(target: "bestel::repoe_refresh", "task exited: {e:?}");
        }
    });
}

pub async fn run(client: Arc<RepoeClient>, http: PoeHttpClient) -> Result<()> {
    tokio::time::sleep(BOOT_DELAY).await;
    loop {
        for &game in &[Game::Poe1, Game::Poe2] {
            for &cat in Category::ALL {
                if !cat.available_for(game) {
                    continue;
                }
                if let Err(e) = refresh_one(&client, &http, game, cat).await {
                    tracing::debug!(
                        target: "bestel::repoe_refresh",
                        "skip {}/{}: {e:?}",
                        game.as_str(),
                        cat.as_str()
                    );
                }
            }
        }
        tokio::time::sleep(REFRESH_INTERVAL).await;
    }
}

async fn refresh_one(
    client: &RepoeClient,
    http: &PoeHttpClient,
    game: Game,
    category: Category,
) -> Result<()> {
    let url = source_url(game, category);
    let body = http
        .get_text(&url, "repoe_refresh")
        .await
        .with_context(|| format!("GET {url}"))?;
    let bytes = body.into_bytes();

    let parsed: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse {} {} payload", game.as_str(), category.as_str()))?;
    if !parsed.is_object() {
        anyhow::bail!("expected object at top level for {} {}", game.as_str(), category.as_str());
    }

    let compressed = zstd::encode_all(&bytes[..], 19).context("zstd compress refreshed payload")?;
    let path = cache_path(client, game, category);
    atomic_write(&path, &compressed)
        .with_context(|| format!("atomic-write {}", path.display()))?;

    let entries = match parsed {
        Value::Object(map) => map.into_iter().collect::<HashMap<_, _>>(),
        _ => unreachable!(),
    };
    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let snapshot = Arc::new(CategorySnapshot {
        game,
        category,
        entries,
        fetched_at,
        source: SnapshotSource::Refreshed,
    });
    client.install(snapshot);
    tracing::info!(
        target: "bestel::repoe_refresh",
        "refreshed {}/{} ({} bytes compressed)",
        game.as_str(),
        category.as_str(),
        compressed.len()
    );
    Ok(())
}

fn source_url(game: Game, category: Category) -> String {
    let base = match game {
        Game::Poe1 => REPOE_BASE_POE1,
        Game::Poe2 => REPOE_BASE_POE2,
    };
    let file = match category {
        Category::Mods => "mods.min.json",
        Category::BaseItems => "base_items.min.json",
        Category::Gems => "gems.min.json",
        Category::Uniques => "uniques.min.json",
        Category::ClusterJewels => "cluster_jewels.min.json",
        Category::Essences => "essences.min.json",
        Category::Fossils => "fossils.min.json",
        Category::StatTranslations => "stat_translations.min.json",
    };
    format!("{base}/{file}")
}

fn cache_path(_client: &RepoeClient, game: Game, category: Category) -> PathBuf {
    FileCache::default_dir()
        .join("repoe")
        .join(game.as_str())
        .join(format!("{}.json.zst", category.as_str()))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("zst.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

