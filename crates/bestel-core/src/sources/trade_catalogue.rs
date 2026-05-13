//! Bundled offline catalogue of `/api/trade/data/stats` for PoE1 and PoE2.
//!
//! Mirrors `repoe.rs` shape: `include_bytes!` zstd-compressed snapshot at
//! compile time, optional `~/.bestel/cache/trade_stats/{game}.json.zst`
//! refreshed payload, in-memory `Arc<StatsCatalogue>` slot per game.
//!
//! `TradeClient::raw_stats()` becomes a thin facade over `get(game)`. The
//! existing `resolve()` token-overlap algorithm is preserved verbatim and
//! consumes the same `serde_json::Value` shape returned by the live API.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::cache::FileCache;
use super::http::PoeHttpClient;
use super::repoe::Game;

const TRADE_STATS_POE1: &str = "https://www.pathofexile.com/api/trade/data/stats";
const TRADE_STATS_POE2: &str = "https://www.pathofexile.com/api/trade2/data/stats";
const REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 3600);
const BOOT_DELAY: Duration = Duration::from_secs(15);

/// Decoded `/api/trade/data/stats` payload — `result` array of stat
/// groups, each containing an `entries` list. Stored as raw `Value` so
/// the existing `TradeClient::resolve()` consumer is unchanged.
#[derive(Debug)]
pub struct StatsCatalogue {
    pub game: Game,
    pub raw: Value,
    pub fetched_at: u64,
    pub source: CatalogueSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogueSource {
    Bundled,
    Refreshed,
    Live,
}

type Slot = HashMap<Game, Arc<StatsCatalogue>>;

static SLOT: OnceLock<RwLock<Slot>> = OnceLock::new();

fn slot() -> &'static RwLock<Slot> {
    SLOT.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn get(game: Game) -> Result<Arc<StatsCatalogue>> {
    if let Some(existing) = slot().read().ok().and_then(|m| m.get(&game).cloned()) {
        return Ok(existing);
    }
    let catalogue = Arc::new(load_initial(game)?);
    if let Ok(mut guard) = slot().write() {
        guard.insert(game, catalogue.clone());
    }
    Ok(catalogue)
}

pub fn install(catalogue: Arc<StatsCatalogue>) {
    if let Ok(mut guard) = slot().write() {
        guard.insert(catalogue.game, catalogue);
    }
}

fn load_initial(game: Game) -> Result<StatsCatalogue> {
    if let Some(snap) = load_refreshed(game)? {
        return Ok(snap);
    }
    load_bundled(game)
}

fn refreshed_path(game: Game) -> PathBuf {
    FileCache::default_dir()
        .join("trade_stats")
        .join(format!("{}.json.zst", game.as_str()))
}

fn load_refreshed(game: Game) -> Result<Option<StatsCatalogue>> {
    let path = refreshed_path(game);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path)
        .with_context(|| format!("read refreshed trade stats {}", path.display()))?;
    let raw_bytes = zstd::decode_all(&bytes[..]).context("zstd decode refreshed trade stats")?;
    let raw: Value = serde_json::from_slice(&raw_bytes).context("parse refreshed trade stats")?;
    let fetched_at = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(Some(StatsCatalogue {
        game,
        raw,
        fetched_at,
        source: CatalogueSource::Refreshed,
    }))
}

fn load_bundled(game: Game) -> Result<StatsCatalogue> {
    let bytes = bundled_blob(game)
        .ok_or_else(|| anyhow!("no bundled trade-stats snapshot for {}", game.as_str()))?;
    let raw_bytes = zstd::decode_all(bytes).context("zstd decode bundled trade stats")?;
    let raw: Value = serde_json::from_slice(&raw_bytes).context("parse bundled trade stats")?;
    Ok(StatsCatalogue {
        game,
        raw,
        fetched_at: BUNDLE_FETCHED_AT,
        source: CatalogueSource::Bundled,
    })
}

fn bundled_blob(game: Game) -> Option<&'static [u8]> {
    match game {
        Game::Poe1 => Some(include_bytes!("snapshots/trade_stats/poe1.json.zst")),
        Game::Poe2 => Some(include_bytes!("snapshots/trade_stats/poe2.json.zst")),
    }
}

const BUNDLE_FETCHED_AT: u64 = 1_778_198_400; // 2026-05-08 00:00 UTC

pub fn spawn(http: PoeHttpClient, _cache: FileCache) {
    tokio::spawn(async move {
        if let Err(e) = run(http).await {
            tracing::warn!(target: "bestel::trade_catalogue", "task exited: {e:?}");
        }
    });
}

pub async fn run(http: PoeHttpClient) -> Result<()> {
    tokio::time::sleep(BOOT_DELAY).await;
    loop {
        for &game in &[Game::Poe1, Game::Poe2] {
            if let Err(e) = refresh_one(&http, game).await {
                tracing::debug!(
                    target: "bestel::trade_catalogue",
                    "trade-stats refresh {}: {e:?}",
                    game.as_str()
                );
            }
        }
        tokio::time::sleep(REFRESH_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_poe1_trade_stats_decompresses() {
        let cat = get(Game::Poe1).expect("poe1 trade catalogue");
        let result = cat
            .raw
            .get("result")
            .and_then(|r| r.as_array())
            .expect("trade stats payload must have a `result` array");
        assert!(
            !result.is_empty(),
            "expected non-empty trade stats catalogue"
        );
    }
}

async fn refresh_one(http: &PoeHttpClient, game: Game) -> Result<()> {
    let url = match game {
        Game::Poe1 => TRADE_STATS_POE1,
        Game::Poe2 => TRADE_STATS_POE2,
    };
    let body = http.get_text(url, "trade_catalogue").await?;
    let raw: Value = serde_json::from_str(&body).context("parse live trade stats")?;
    let compressed =
        zstd::encode_all(body.as_bytes(), 19).context("zstd compress live trade stats")?;
    let path = refreshed_path(game);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("zst.tmp");
    std::fs::write(&tmp, &compressed)?;
    std::fs::rename(&tmp, &path)?;
    let fetched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    install(Arc::new(StatsCatalogue {
        game,
        raw,
        fetched_at,
        source: CatalogueSource::Live,
    }));
    Ok(())
}
