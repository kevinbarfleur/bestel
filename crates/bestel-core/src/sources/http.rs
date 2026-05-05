//! Shared HTTP client for PoE data sources.
//!
//! GGG's developer docs require an OAuth-style User-Agent of the form
//! `OAuth <clientId>/<version> (contact: <email>)` even on unauthenticated
//! endpoints. The contact email comes from `BESTEL_CONTACT_EMAIL` (default
//! `hi@kevinbarfleur.dev`) so forks can override without editing source.
//!
//! Rate limits are dynamic per-rule. GGG returns:
//! - `X-Rate-Limit-<rule>: hits:period:timeout, hits:period:timeout, ...`
//! - `X-Rate-Limit-<rule>-State: hits:period:timeout, ...` (current usage)
//! - `Retry-After: <secs>` on 429.
//! We park future calls on the same rule until the window clears.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::devlog;

const DEFAULT_CONTACT: &str = "hi@kevinbarfleur.dev";
const CLIENT_ID: &str = "bestel";

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("rate limit hit on rule '{rule}', retry after {retry_after_secs}s")]
    Throttled { rule: String, retry_after_secs: u64 },
}

#[derive(Clone)]
pub struct PoeHttpClient {
    inner: reqwest::Client,
    user_agent: String,
    rules: Arc<Mutex<HashMap<String, RuleState>>>,
}

#[derive(Debug, Default, Clone)]
struct RuleState {
    /// Earliest Instant at which a call against this rule should be tried.
    paused_until: Option<Instant>,
}

impl PoeHttpClient {
    pub fn new() -> Result<Self> {
        let version = env!("CARGO_PKG_VERSION");
        let contact = std::env::var("BESTEL_CONTACT_EMAIL")
            .unwrap_or_else(|_| DEFAULT_CONTACT.to_string());
        let ua = format!("OAuth {CLIENT_ID}/{version} (contact: {contact})");
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .context("build reqwest client")?;
        Ok(Self {
            inner,
            user_agent: ua,
            rules: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str, rule: &str) -> Result<T> {
        let resp = self.send(Method::GET, url, None, rule).await?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .with_context(|| format!("read body from {url}"))?;
        if !status.is_success() {
            return Err(anyhow!("HTTP {status} on {url}: {}", truncate(&text, 400)));
        }
        serde_json::from_str(&text)
            .with_context(|| format!("decode JSON from {url} (got {} bytes)", text.len()))
    }

    pub async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
        rule: &str,
    ) -> Result<T> {
        let resp = self.send(Method::POST, url, Some(body.clone()), rule).await?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .with_context(|| format!("read body from {url}"))?;
        if !status.is_success() {
            return Err(anyhow!("HTTP {status} on {url}: {}", truncate(&text, 400)));
        }
        serde_json::from_str(&text)
            .with_context(|| format!("decode JSON from {url} (got {} bytes)", text.len()))
    }

    pub async fn get_text(&self, url: &str, rule: &str) -> Result<String> {
        let resp = self.send(Method::GET, url, None, rule).await?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .with_context(|| format!("read body from {url}"))?;
        if !status.is_success() {
            return Err(anyhow!("HTTP {status} on {url}: {}", truncate(&text, 400)));
        }
        Ok(text)
    }

    async fn send(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
        rule: &str,
    ) -> Result<Response> {
        self.wait_if_needed(rule).await;
        let mut req = self.inner.request(method, url);
        req = req.header(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);
        if let Some(b) = &body {
            req = req.json(b);
        }
        let resp = req
            .send()
            .await
            .with_context(|| format!("HTTP send to {url}"))?;
        self.absorb_rate_headers(rule, resp.headers()).await;
        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            let retry = retry_after_secs(resp.headers()).unwrap_or(30);
            self.set_pause(rule, Duration::from_secs(retry)).await;
            devlog::log_value(
                "source_rate_limit",
                serde_json::json!({"rule": rule, "url": url, "retry_after": retry}),
            );
            return Err(RateLimitError::Throttled {
                rule: rule.to_string(),
                retry_after_secs: retry,
            }
            .into());
        }
        Ok(resp)
    }

    async fn wait_if_needed(&self, rule: &str) {
        let pause = {
            let map = self.rules.lock().await;
            map.get(rule).and_then(|s| s.paused_until)
        };
        if let Some(until) = pause {
            let now = Instant::now();
            if until > now {
                tokio::time::sleep(until - now).await;
            }
        }
    }

    async fn set_pause(&self, rule: &str, dur: Duration) {
        let mut map = self.rules.lock().await;
        let entry = map.entry(rule.to_string()).or_default();
        entry.paused_until = Some(Instant::now() + dur);
    }

    /// Parse the dynamic rate-limit headers GGG returns. Format
    /// `X-Rate-Limit-<rule>: hits:period:timeout` and a parallel
    /// `-State` header showing current usage. We do a soft-pause when
    /// the worst window is ≥80% saturated.
    async fn absorb_rate_headers(&self, rule: &str, headers: &HeaderMap) {
        let mut worst_pause: Option<Duration> = None;
        for (name, value) in headers.iter() {
            let n = name.as_str().to_lowercase();
            if !n.starts_with("x-rate-limit-") || !n.ends_with("-state") {
                continue;
            }
            let policy_name = n
                .trim_start_matches("x-rate-limit-")
                .trim_end_matches("-state");
            let policy_header = format!("x-rate-limit-{}", policy_name);
            let policy_val = headers
                .get(policy_header.as_str())
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let state_val = value.to_str().unwrap_or("");
            for (state, policy) in state_val.split(',').zip(policy_val.split(',')) {
                let s = parse_triplet(state.trim());
                let p = parse_triplet(policy.trim());
                if let (Some((hits, _, timeout)), Some((max, period, _))) = (s, p) {
                    if max > 0 && hits >= max.saturating_sub(1) {
                        let pause = Duration::from_secs(timeout.max(period));
                        worst_pause = Some(worst_pause.map_or(pause, |w| w.max(pause)));
                    }
                }
            }
        }
        if let Some(p) = worst_pause {
            self.set_pause(rule, p).await;
        }
    }
}

fn parse_triplet(s: &str) -> Option<(u64, u64, u64)> {
    let mut it = s.split(':');
    let a: u64 = it.next()?.parse().ok()?;
    let b: u64 = it.next()?.parse().ok()?;
    let c: u64 = it.next()?.parse().ok()?;
    Some((a, b, c))
}

fn retry_after_secs(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()
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
