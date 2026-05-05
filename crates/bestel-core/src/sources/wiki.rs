//! MediaWiki + Cargo client for poewiki.net (PoE1) and poe2wiki.net (PoE2).
//!
//! Three operations:
//! - `search`     → opensearch + generator=search, returns title+snippet+url.
//! - `parse`      → action=parse with prop=sections|text, returns plain text
//!                  and section anchors. We strip HTML tags to keep the
//!                  payload small enough for an LLM tool result.
//! - `cargo`      → action=cargoquery for structured tables (`items`,
//!                  `item_stats`, `mods`, `skill`, `versions`, ...).
//!
//! TTL strategy: 12h cache for parse + cargo, 6h for search. Wikis update
//! a few times a day; this matches the slowest reasonable refresh.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::pob::PoeVersion;
use crate::sources::cache::FileCache;
use crate::sources::http::PoeHttpClient;

const WIKI_TTL_PARSE: Duration = Duration::from_secs(12 * 3600);
const WIKI_TTL_SEARCH: Duration = Duration::from_secs(6 * 3600);
const WIKI_TTL_CARGO: Duration = Duration::from_secs(12 * 3600);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiHit {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPage {
    pub title: String,
    pub url: String,
    pub sections: Vec<WikiSection>,
    pub plain_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSection {
    pub level: u32,
    pub heading: String,
    pub anchor: String,
}

#[derive(Clone)]
pub struct WikiClient {
    http: PoeHttpClient,
    cache: FileCache,
    game: PoeVersion,
}

impl WikiClient {
    pub fn new(http: PoeHttpClient, cache: FileCache, game: PoeVersion) -> Self {
        Self { http, cache, game }
    }

    fn host(&self) -> &'static str {
        match self.game {
            PoeVersion::Poe1 => "www.poewiki.net",
            PoeVersion::Poe2 => "www.poe2wiki.net",
        }
    }

    fn api(&self) -> String {
        format!("https://{}/w/api.php", self.host())
    }

    fn page_url(&self, title: &str) -> String {
        let underscored = title.replace(' ', "_");
        let encoded = urlencoding::encode(&underscored);
        format!("https://{}/wiki/{}", self.host(), encoded)
    }

    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<WikiHit>> {
        let key = format!("wiki:{:?}:search:{}:{}", self.game, query, limit);
        if let Some(v) = self.cache.get::<Vec<WikiHit>>(&key, WIKI_TTL_SEARCH).await {
            return Ok(v);
        }
        let url = format!(
            "{}?action=query&list=search&srsearch={}&srlimit={}&format=json",
            self.api(),
            urlencoding::encode(query),
            limit.clamp(1, 20)
        );
        let v: Value = self
            .http
            .get_json(&url, "wiki")
            .await
            .with_context(|| format!("wiki search '{query}'"))?;
        let hits = v
            .pointer("/query/search")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let title = item
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string();
                        let snippet = strip_html(
                            item.get("snippet").and_then(|t| t.as_str()).unwrap_or(""),
                        );
                        WikiHit {
                            url: self.page_url(&title),
                            title,
                            snippet,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let _ = self.cache.put(&key, &hits).await;
        Ok(hits)
    }

    pub async fn parse(&self, title: &str) -> Result<WikiPage> {
        let key = format!("wiki:{:?}:parse:{}", self.game, title);
        if let Some(v) = self.cache.get::<WikiPage>(&key, WIKI_TTL_PARSE).await {
            return Ok(v);
        }
        let url = format!(
            "{}?action=parse&page={}&prop=text|sections&format=json&redirects=1",
            self.api(),
            urlencoding::encode(title)
        );
        let v: Value = self
            .http
            .get_json(&url, "wiki")
            .await
            .with_context(|| format!("wiki parse '{title}'"))?;
        if let Some(err) = v.get("error") {
            return Err(anyhow!(
                "wiki parse error: {}",
                err.get("info")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown")
            ));
        }
        let parsed = v
            .get("parse")
            .ok_or_else(|| anyhow!("wiki parse: missing 'parse' field"))?;
        let resolved_title = parsed
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or(title)
            .to_string();
        let html = parsed
            .pointer("/text/*")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let sections = parsed
            .get("sections")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|sec| WikiSection {
                        level: sec
                            .get("toclevel")
                            .and_then(|l| l.as_u64())
                            .unwrap_or(1) as u32,
                        heading: sec
                            .get("line")
                            .and_then(|l| l.as_str())
                            .unwrap_or("")
                            .to_string(),
                        anchor: sec
                            .get("anchor")
                            .and_then(|a| a.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let plain_text = strip_html(html);
        let page = WikiPage {
            url: self.page_url(&resolved_title),
            title: resolved_title,
            sections,
            plain_text,
        };
        let _ = self.cache.put(&key, &page).await;
        Ok(page)
    }

    pub async fn cargo(
        &self,
        table: &str,
        fields: &[&str],
        where_clause: Option<&str>,
        limit: u32,
    ) -> Result<Vec<Value>> {
        let where_part = where_clause.unwrap_or("");
        let key = format!(
            "wiki:{:?}:cargo:{}:{}:{}:{}",
            self.game,
            table,
            fields.join(","),
            where_part,
            limit
        );
        if let Some(v) = self.cache.get::<Vec<Value>>(&key, WIKI_TTL_CARGO).await {
            return Ok(v);
        }
        let mut url = format!(
            "{}?action=cargoquery&format=json&tables={}&fields={}&limit={}",
            self.api(),
            urlencoding::encode(table),
            urlencoding::encode(&fields.join(",")),
            limit.clamp(1, 500),
        );
        if let Some(w) = where_clause {
            url.push_str("&where=");
            url.push_str(&urlencoding::encode(w));
        }
        let v: Value = self
            .http
            .get_json(&url, "wiki")
            .await
            .with_context(|| format!("cargo query table='{table}'"))?;
        if let Some(err) = v.get("error") {
            return Err(anyhow!(
                "cargo error: {}",
                err.get("info")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown")
            ));
        }
        let rows: Vec<Value> = v
            .get("cargoquery")
            .and_then(|q| q.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|w| w.get("title").cloned())
                    .collect()
            })
            .unwrap_or_default();
        let _ = self.cache.put(&key, &rows).await;
        Ok(rows)
    }
}

fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let collapsed = out
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    decode_entities(&collapsed)
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags_and_collapses() {
        let input = "<p>Hello <b>brave</b> &amp; <i>doomed</i> exile.</p>\n<p>Wraeclast.</p>";
        let out = strip_html(input);
        assert_eq!(out, "Hello brave & doomed exile. Wraeclast.");
    }

    #[test]
    fn page_url_handles_spaces() {
        let http = PoeHttpClient::new().unwrap();
        let cache = FileCache::new(std::env::temp_dir().join("bestel-wiki-test"));
        let c = WikiClient::new(http, cache, PoeVersion::Poe1);
        let url = c.page_url("Tabula Rasa");
        assert_eq!(url, "https://www.poewiki.net/wiki/Tabula_Rasa");
    }
}
