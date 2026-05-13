//! Trade Data API client + human-phrase → trade-stat-id resolver.
//!
//! GGG exposes the trade dataset at `/api/trade/data/*` (PoE1) and
//! `/api/trade2/data/*` (PoE2). The most important file is `/data/stats`,
//! which lists every searchable stat grouped by `pseudo`, `explicit`,
//! `implicit`, `fractured`, `crafted`, `enchant`, etc. We fuzzy-match a
//! human phrase against the `text` field, boosting `pseudo.*` for ambient
//! requests ("life") and exact stat groups for specific requests
//! ("fractured life").
//!
//! Search lifecycle:
//!   POST /api/trade/search/{league} JSON body
//!     → { id, total, result: [...] }
//!   GET  /api/trade/fetch/{ids}?query={id}    (NOT used in Phase 3)
//!
//! Phase 3 only returns the shareable URL `/trade/search/{league}/{id}`,
//! per the plan: cheap on rate limits, lets the user click through to the
//! real trade UI which is always more accurate than what we'd render.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::pob::PoeVersion;
use crate::sources::cache::FileCache;
use crate::sources::http::PoeHttpClient;
use crate::sources::repoe::Game;
use crate::sources::trade_catalogue;

const TRADE_TTL_DATA: Duration = Duration::from_secs(12 * 3600);
const TRADE_TTL_LEAGUES: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatRef {
    pub id: String,
    pub text: String,
    pub stat_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSearchResp {
    pub query_id: String,
    pub total: u64,
    pub share_url: String,
}

#[derive(Clone)]
pub struct TradeClient {
    http: PoeHttpClient,
    cache: FileCache,
    game: PoeVersion,
}

impl TradeClient {
    pub fn new(http: PoeHttpClient, cache: FileCache, game: PoeVersion) -> Self {
        Self { http, cache, game }
    }

    fn data_prefix(&self) -> &'static str {
        match self.game {
            PoeVersion::Poe1 => "https://www.pathofexile.com/api/trade/data",
            PoeVersion::Poe2 => "https://www.pathofexile.com/api/trade2/data",
        }
    }

    fn search_prefix(&self) -> &'static str {
        match self.game {
            PoeVersion::Poe1 => "https://www.pathofexile.com/api/trade/search",
            PoeVersion::Poe2 => "https://www.pathofexile.com/api/trade2/search/poe2",
        }
    }

    fn share_prefix(&self) -> &'static str {
        match self.game {
            PoeVersion::Poe1 => "https://www.pathofexile.com/trade/search",
            PoeVersion::Poe2 => "https://www.pathofexile.com/trade2/search/poe2",
        }
    }

    pub async fn leagues(&self) -> Result<Vec<String>> {
        let key = format!("trade:{:?}:leagues", self.game);
        if let Some(v) = self.cache.get::<Vec<String>>(&key, TRADE_TTL_LEAGUES).await {
            return Ok(v);
        }
        let url = format!("{}/leagues", self.data_prefix());
        let v: Value = self
            .http
            .get_json(&url, "trade-data")
            .await
            .context("trade leagues")?;
        let names: Vec<String> = v
            .get("result")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        e.get("id")
                            .and_then(|id| id.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let _ = self.cache.put(&key, &names).await;
        Ok(names)
    }

    /// Returns the raw `/data/stats` payload. Resolution order:
    /// 1. In-memory `trade_catalogue` (bundled snapshot or nightly-refreshed).
    /// 2. On-disk 12h TTL cache (legacy fallback for older installs).
    /// 3. Live HTTP fetch (writes both catalogue and TTL cache).
    pub async fn raw_stats(&self) -> Result<Value> {
        if let Ok(catalogue) = trade_catalogue::get(to_catalogue_game(self.game)) {
            return Ok(catalogue.raw.clone());
        }
        let key = format!("trade:{:?}:stats", self.game);
        if let Some(v) = self.cache.get::<Value>(&key, TRADE_TTL_DATA).await {
            return Ok(v);
        }
        let url = format!("{}/stats", self.data_prefix());
        let v: Value = self
            .http
            .get_json(&url, "trade-data")
            .await
            .context("trade stats")?;
        let _ = self.cache.put(&key, &v).await;
        Ok(v)
    }

    /// Fuzzy-resolve a human phrase against /data/stats. Returns top
    /// candidates ranked by token overlap. Pseudo stats get a small bonus
    /// when `prefer_pseudo` is true (the typical "life" / "resistance"
    /// case for an item to equip).
    pub async fn resolve(
        &self,
        phrase: &str,
        prefer_pseudo: bool,
        limit: usize,
    ) -> Result<Vec<StatRef>> {
        let stats = self.raw_stats().await?;
        let result = stats
            .get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| anyhow!("trade stats: missing 'result' array"))?;
        let needle_tokens = tokenize(phrase);
        if needle_tokens.is_empty() {
            return Ok(vec![]);
        }
        let mut scored: Vec<(f32, StatRef)> = Vec::new();
        for group in result {
            let group_id = group
                .get("id")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let entries = match group.get("entries").and_then(|e| e.as_array()) {
                Some(a) => a,
                None => continue,
            };
            for entry in entries {
                let id = entry
                    .get("id")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let text = entry
                    .get("text")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let stat_type = entry
                    .get("type")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&group_id)
                    .to_string();
                let score = score_phrase(&needle_tokens, &text, &stat_type, prefer_pseudo);
                if score > 0.0 {
                    scored.push((
                        score,
                        StatRef {
                            id,
                            text,
                            stat_type,
                        },
                    ));
                }
            }
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(limit).map(|(_, s)| s).collect())
    }

    /// POST a trade search and return the share URL plus result metadata.
    /// `query` is the full JSON body the trade API accepts; we don't try
    /// to validate it — callers (the LLM via `trade_build_query`) build
    /// it from `resolve` output and the optional filter helpers.
    pub async fn search(&self, league: &str, query: Value) -> Result<TradeSearchResp> {
        let url = format!("{}/{}", self.search_prefix(), urlencoding::encode(league));
        let v: Value = self
            .http
            .post_json(&url, &query, "trade-search")
            .await
            .context("trade search")?;
        let id = v
            .get("id")
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("trade search response missing 'id'"))?
            .to_string();
        let total = v.get("total").and_then(|t| t.as_u64()).unwrap_or(0);
        let share_url = format!(
            "{}/{}/{}",
            self.share_prefix(),
            urlencoding::encode(league),
            id
        );
        Ok(TradeSearchResp {
            query_id: id,
            total,
            share_url,
        })
    }

    /// Convenience: build a minimal, well-formed query body from constraints.
    /// The LLM can call this OR craft its own JSON. Defaults to online + any.
    pub fn build_query_body(
        category: Option<&str>,
        rarity: Option<&str>,
        stats_and: &[(String, Option<f64>, Option<f64>)],
        price_max_chaos: Option<f64>,
    ) -> Value {
        let mut filters_block = serde_json::Map::new();
        if category.is_some() || rarity.is_some() {
            let mut tf = serde_json::Map::new();
            if let Some(c) = category {
                tf.insert("category".into(), json!({"option": c}));
            }
            if let Some(r) = rarity {
                tf.insert("rarity".into(), json!({"option": r}));
            }
            filters_block.insert("type_filters".into(), json!({"filters": Value::Object(tf)}));
        }
        if let Some(p) = price_max_chaos {
            filters_block.insert(
                "trade_filters".into(),
                json!({
                    "filters": {
                        "price": {"option": "chaos", "max": p}
                    }
                }),
            );
        }
        let stat_filters: Vec<Value> = stats_and
            .iter()
            .map(|(id, min, max)| {
                let mut value = serde_json::Map::new();
                if let Some(m) = min {
                    value.insert("min".into(), json!(m));
                }
                if let Some(m) = max {
                    value.insert("max".into(), json!(m));
                }
                json!({"id": id, "value": Value::Object(value)})
            })
            .collect();
        json!({
            "query": {
                "status": {"option": "online"},
                "stats": [
                    {
                        "type": "and",
                        "filters": stat_filters
                    }
                ],
                "filters": Value::Object(filters_block)
            },
            "sort": {"price": "asc"}
        })
    }
}

fn to_catalogue_game(v: PoeVersion) -> Game {
    match v {
        PoeVersion::Poe1 => Game::Poe1,
        PoeVersion::Poe2 => Game::Poe2,
    }
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '#' && c != '+' && c != '-')
        .filter(|t| !t.is_empty() && t.len() > 1)
        .map(|t| t.to_string())
        .collect()
}

fn score_phrase(needle: &[String], hay_text: &str, stat_type: &str, prefer_pseudo: bool) -> f32 {
    let hay_tokens = tokenize(hay_text);
    if hay_tokens.is_empty() {
        return 0.0;
    }
    let mut hits = 0u32;
    for n in needle {
        if hay_tokens.iter().any(|h| h == n || h.contains(n.as_str())) {
            hits += 1;
        }
    }
    if hits == 0 {
        return 0.0;
    }
    let coverage = hits as f32 / needle.len() as f32;
    let mut score = coverage * 100.0;
    let len_penalty = (hay_tokens.len() as f32 - needle.len() as f32).abs() * 0.5;
    score -= len_penalty;
    let stat_lower = stat_type.to_ascii_lowercase();
    if prefer_pseudo {
        if stat_lower == "pseudo" {
            score += 25.0;
        } else if stat_lower == "explicit" {
            score += 5.0;
        }
    } else {
        for n in needle {
            if stat_lower.contains(n.as_str()) {
                score += 30.0;
            }
        }
    }
    if needle
        .iter()
        .all(|n| hay_text.to_lowercase().split_whitespace().any(|w| w == n))
    {
        score += 15.0;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_stats() -> Value {
        json!({
            "result": [
                {
                    "id": "pseudo",
                    "label": "Pseudo",
                    "entries": [
                        {"id": "pseudo.pseudo_total_life", "text": "+# to maximum Life", "type": "pseudo"},
                        {"id": "pseudo.pseudo_total_resistance", "text": "+#% total Elemental Resistance", "type": "pseudo"}
                    ]
                },
                {
                    "id": "explicit",
                    "label": "Explicit",
                    "entries": [
                        {"id": "explicit.stat_3299347043", "text": "+# to maximum Life", "type": "explicit"}
                    ]
                },
                {
                    "id": "fractured",
                    "label": "Fractured",
                    "entries": [
                        {"id": "fractured.stat_3299347043", "text": "+# to maximum Life", "type": "fractured"}
                    ]
                }
            ]
        })
    }

    fn temp_client() -> TradeClient {
        let http = PoeHttpClient::new().unwrap();
        let dir = std::env::temp_dir().join(format!(
            "bestel-trade-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        TradeClient::new(http, FileCache::new(dir), PoeVersion::Poe1)
    }

    fn resolve_with_stats(phrase: &str, prefer_pseudo: bool, stats: &Value) -> Vec<StatRef> {
        let needle = tokenize(phrase);
        let result = stats.get("result").unwrap().as_array().unwrap();
        let mut scored: Vec<(f32, StatRef)> = Vec::new();
        for group in result {
            let group_id = group
                .get("id")
                .and_then(|s| s.as_str())
                .unwrap()
                .to_string();
            for entry in group.get("entries").unwrap().as_array().unwrap() {
                let id = entry.get("id").unwrap().as_str().unwrap().to_string();
                let text = entry.get("text").unwrap().as_str().unwrap().to_string();
                let stat_type = entry
                    .get("type")
                    .and_then(|s| s.as_str())
                    .unwrap_or(&group_id)
                    .to_string();
                let score = score_phrase(&needle, &text, &stat_type, prefer_pseudo);
                if score > 0.0 {
                    scored.push((
                        score,
                        StatRef {
                            id,
                            text,
                            stat_type,
                        },
                    ));
                }
            }
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(_, s)| s).collect()
    }

    #[test]
    fn resolve_life_prefers_pseudo() {
        let stats = fake_stats();
        let res = resolve_with_stats("life", true, &stats);
        assert!(!res.is_empty());
        assert_eq!(res[0].id, "pseudo.pseudo_total_life");
    }

    #[test]
    fn resolve_fractured_life_prefers_fractured() {
        let stats = fake_stats();
        let res = resolve_with_stats("fractured life", false, &stats);
        assert!(!res.is_empty());
        assert_eq!(res[0].stat_type, "fractured");
    }

    #[test]
    fn build_query_body_has_required_shape() {
        let body = TradeClient::build_query_body(
            Some("accessory.ring"),
            Some("rare"),
            &[
                ("pseudo.pseudo_total_life".to_string(), Some(70.0), None),
                (
                    "pseudo.pseudo_total_resistance".to_string(),
                    Some(80.0),
                    None,
                ),
            ],
            Some(50.0),
        );
        assert_eq!(body["query"]["status"]["option"], "online");
        assert_eq!(body["query"]["stats"][0]["type"], "and");
        assert_eq!(
            body["query"]["filters"]["type_filters"]["filters"]["category"]["option"],
            "accessory.ring"
        );
        assert_eq!(
            body["query"]["filters"]["trade_filters"]["filters"]["price"]["max"],
            50.0
        );
    }

    #[tokio::test]
    async fn share_url_format() {
        let c = temp_client();
        // We don't hit the network here; just check the URL builder format
        // by reaching into search prefix logic.
        assert!(c.search_prefix().contains("/api/trade/search"));
        assert!(c.share_prefix().contains("/trade/search"));
    }
}
