//! Ollama embeddings (`bge-small-en-v1.5` default, `bge-m3` optional).
//!
//! Endpoint: `POST http://localhost:11434/api/embeddings` with one prompt
//! per call. Output dim depends on the model: 384 for `bge-small-en-v1.5`,
//! 1024 for `bge-m3`. We loop in user code rather than batch — Ollama
//! does not accept arrays for `/api/embeddings` (the newer
//! `/api/embed` does, but not all Ollama versions ship it; staying on the
//! v1 endpoint maximises compatibility).

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::traits::EmbeddingProvider;

pub const BGE_SMALL_EN_V1_5: &str = "bge-small-en-v1.5";
pub const BGE_M3: &str = "bge-m3";
pub const NOMIC_EMBED_TEXT: &str = "nomic-embed-text";
pub const MXBAI_EMBED_LARGE: &str = "mxbai-embed-large";

#[derive(Debug, Clone)]
pub struct OllamaEmbedderConfig {
    pub model: String,
    pub base_url: String,
    pub dim: usize,
}

impl Default for OllamaEmbedderConfig {
    /// Default = `nomic-embed-text` (768 dim). Reason: it is the
    /// canonical Ollama embedding model — `bge-small-en-v1.5` is not on
    /// Ollama's default hub. Use [`Self::bge_small_en_v1_5`] /
    /// [`Self::bge_m3`] / [`Self::mxbai_embed_large`] when the user has
    /// pulled those manually.
    fn default() -> Self {
        Self::nomic_embed_text()
    }
}

impl OllamaEmbedderConfig {
    pub fn nomic_embed_text() -> Self {
        Self {
            model: NOMIC_EMBED_TEXT.into(),
            base_url: "http://localhost:11434".into(),
            dim: 768,
        }
    }

    pub fn bge_small_en_v1_5() -> Self {
        Self {
            model: BGE_SMALL_EN_V1_5.into(),
            base_url: "http://localhost:11434".into(),
            dim: 384,
        }
    }

    pub fn bge_m3() -> Self {
        Self {
            model: BGE_M3.into(),
            base_url: "http://localhost:11434".into(),
            dim: 1024,
        }
    }

    pub fn mxbai_embed_large() -> Self {
        Self {
            model: MXBAI_EMBED_LARGE.into(),
            base_url: "http://localhost:11434".into(),
            dim: 1024,
        }
    }
}

pub struct OllamaEmbedder {
    http: reqwest::Client,
    config: OllamaEmbedderConfig,
}

impl OllamaEmbedder {
    pub fn new(http: reqwest::Client, config: OllamaEmbedderConfig) -> Self {
        Self { http, config }
    }

    pub fn model_name(&self) -> &str {
        &self.config.model
    }

    fn build_request_body(model: &str, prompt: &str) -> serde_json::Value {
        serde_json::json!({
            "model": model,
            "prompt": prompt,
        })
    }

    fn parse_response(body: &str) -> Result<Vec<f32>> {
        let parsed: EmbeddingResponse = serde_json::from_str(body)
            .with_context(|| format!("parse Ollama embeddings response: {}", truncate(body, 200)))?;
        Ok(parsed.embedding)
    }
}

impl EmbeddingProvider for OllamaEmbedder {
    fn dim(&self) -> usize {
        self.config.dim
    }

    fn embed_batch(
        &self,
        texts: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<Vec<f32>>>> + Send {
        async move {
            let url = format!("{}/api/embeddings", self.config.base_url);
            let mut out = Vec::with_capacity(texts.len());
            for text in texts {
                let body = Self::build_request_body(&self.config.model, text);
                let resp = self
                    .http
                    .post(&url)
                    .json(&body)
                    .send()
                    .await
                    .with_context(|| format!("POST {url}"))?;
                let status = resp.status();
                let raw = resp.text().await.context("read Ollama body")?;
                if !status.is_success() {
                    return Err(anyhow!(
                        "Ollama embeddings HTTP {status}: {}",
                        truncate(&raw, 300)
                    ));
                }
                let vector = Self::parse_response(&raw)?;
                if vector.len() != self.config.dim {
                    return Err(anyhow!(
                        "Ollama returned dim {}, configured dim {}",
                        vector.len(),
                        self.config.dim
                    ));
                }
                out.push(vector);
            }
            Ok(out)
        }
    }
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
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
    fn build_request_body_carries_model_and_prompt() {
        let body = OllamaEmbedder::build_request_body("bge-small-en-v1.5", "hello world");
        assert_eq!(body["model"], "bge-small-en-v1.5");
        assert_eq!(body["prompt"], "hello world");
    }

    #[test]
    fn parse_response_returns_embedding() {
        let json = r#"{"embedding": [0.1, -0.2, 0.3]}"#;
        let out = OllamaEmbedder::parse_response(json).unwrap();
        assert_eq!(out, vec![0.1, -0.2, 0.3]);
    }

    #[test]
    fn parse_response_rejects_invalid_json() {
        let err = OllamaEmbedder::parse_response("nope").unwrap_err();
        assert!(err.to_string().to_lowercase().contains("parse"));
    }

    #[test]
    fn config_defaults_to_nomic_embed_text() {
        let c = OllamaEmbedderConfig::default();
        assert_eq!(c.model, NOMIC_EMBED_TEXT);
        assert_eq!(c.dim, 768);
    }

    #[test]
    fn config_bge_small_helper_picks_384_dim() {
        let c = OllamaEmbedderConfig::bge_small_en_v1_5();
        assert_eq!(c.model, BGE_SMALL_EN_V1_5);
        assert_eq!(c.dim, 384);
    }

    #[test]
    fn config_bge_m3_helper_picks_1024_dim() {
        let c = OllamaEmbedderConfig::bge_m3();
        assert_eq!(c.model, BGE_M3);
        assert_eq!(c.dim, 1024);
    }

    #[test]
    fn config_mxbai_helper_picks_1024_dim() {
        let c = OllamaEmbedderConfig::mxbai_embed_large();
        assert_eq!(c.model, MXBAI_EMBED_LARGE);
        assert_eq!(c.dim, 1024);
    }
}
