//! OpenAI Embeddings API (`text-embedding-3-small`).
//!
//! Endpoint: `POST https://api.openai.com/v1/embeddings`. Up to 96 inputs
//! per call (we batch in chunks of 96). Output is 1536-dim f32 unless
//! `dimensions` shrinks it server-side (we keep the full 1536).

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::traits::EmbeddingProvider;

const DEFAULT_MODEL: &str = "text-embedding-3-small";
const DEFAULT_DIM: usize = 1536;
const DEFAULT_BATCH: usize = 96;
const DEFAULT_BASE: &str = "https://api.openai.com";

#[derive(Debug, Clone)]
pub struct OpenAiEmbedderConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub batch_size: usize,
    pub dim: usize,
}

impl Default for OpenAiEmbedderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: DEFAULT_MODEL.into(),
            base_url: DEFAULT_BASE.into(),
            batch_size: DEFAULT_BATCH,
            dim: DEFAULT_DIM,
        }
    }
}

pub struct OpenAiEmbedder {
    http: reqwest::Client,
    config: OpenAiEmbedderConfig,
}

impl OpenAiEmbedder {
    pub fn new(http: reqwest::Client, config: OpenAiEmbedderConfig) -> Self {
        Self { http, config }
    }

    fn build_request_body(model: &str, inputs: &[String]) -> serde_json::Value {
        serde_json::json!({
            "model": model,
            "input": inputs,
            "encoding_format": "float",
        })
    }

    fn parse_response(body: &str) -> Result<Vec<Vec<f32>>> {
        let parsed: EmbeddingResponse = serde_json::from_str(body)
            .with_context(|| format!("parse OpenAI embeddings response: {}", truncate(body, 200)))?;
        let mut data = parsed.data;
        data.sort_by_key(|d| d.index);
        Ok(data.into_iter().map(|d| d.embedding).collect())
    }
}

impl EmbeddingProvider for OpenAiEmbedder {
    fn dim(&self) -> usize {
        self.config.dim
    }

    fn embed_batch(
        &self,
        texts: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<Vec<f32>>>> + Send {
        async move {
            if texts.is_empty() {
                return Ok(Vec::new());
            }
            let url = format!("{}/v1/embeddings", self.config.base_url);
            let mut out = Vec::with_capacity(texts.len());
            for batch in texts.chunks(self.config.batch_size) {
                let body = Self::build_request_body(&self.config.model, batch);
                let resp = self
                    .http
                    .post(&url)
                    .bearer_auth(&self.config.api_key)
                    .json(&body)
                    .send()
                    .await
                    .with_context(|| format!("POST {url}"))?;
                let status = resp.status();
                let text = resp.text().await.context("read OpenAI body")?;
                if !status.is_success() {
                    return Err(anyhow!(
                        "OpenAI embeddings HTTP {status}: {}",
                        truncate(&text, 300)
                    ));
                }
                let mut vectors = Self::parse_response(&text)?;
                if let Some(first) = vectors.first() {
                    if first.len() != self.config.dim {
                        return Err(anyhow!(
                            "OpenAI returned dim {}, configured dim {}",
                            first.len(),
                            self.config.dim
                        ));
                    }
                }
                out.append(&mut vectors);
            }
            Ok(out)
        }
    }
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingDatum>,
}

#[derive(Deserialize)]
struct EmbeddingDatum {
    index: usize,
    embedding: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
    encoding_format: &'a str,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_body_carries_model_and_inputs() {
        let body =
            OpenAiEmbedder::build_request_body("text-embedding-3-small", &["hi".into(), "yo".into()]);
        assert_eq!(body["model"], "text-embedding-3-small");
        assert_eq!(body["input"][0], "hi");
        assert_eq!(body["input"][1], "yo");
        assert_eq!(body["encoding_format"], "float");
    }

    #[test]
    fn parse_response_orders_by_index_and_returns_vectors() {
        let json = r#"{
            "data": [
                {"index": 1, "embedding": [0.4, 0.5, 0.6]},
                {"index": 0, "embedding": [0.1, 0.2, 0.3]}
            ],
            "object": "list",
            "model": "text-embedding-3-small"
        }"#;
        let out = OpenAiEmbedder::parse_response(json).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(out[1], vec![0.4, 0.5, 0.6]);
    }

    #[test]
    fn parse_response_rejects_invalid_json() {
        let err = OpenAiEmbedder::parse_response("not json").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("parse"));
    }
}
