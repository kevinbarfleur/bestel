//! Embedding providers.
//!
//! Two backends ship: OpenAI `text-embedding-3-small` (cloud, default) and
//! Ollama `bge-small-en-v1.5` / `bge-m3` (local, offline fallback). Both
//! implement [`crate::EmbeddingProvider`].
//!
//! Auto-detection logic in [`detect_default`] honours the ROADMAP § E-1
//! decision: online by default if `OPENAI_API_KEY` is set, fall back to a
//! probed Ollama instance, and surface an explicit error otherwise so
//! callers can degrade UX (warn user "no embedding backend reachable").

use anyhow::{anyhow, Result};

use crate::traits::EmbeddingProvider;

pub mod ollama;
pub mod openai;

pub use ollama::{OllamaEmbedder, OllamaEmbedderConfig};
pub use openai::{OpenAiEmbedder, OpenAiEmbedderConfig};

/// Concrete enum that delegates to whichever provider is configured.
///
/// Workaround for the AFIT object-safety limitation: callers that want to
/// hold a single typed handle behind an `Arc` (e.g. `Arc<AnyEmbedder>`
/// inside the runtime tool context) can use this enum instead of a
/// trait object.
pub enum AnyEmbedder {
    OpenAi(OpenAiEmbedder),
    Ollama(OllamaEmbedder),
}

impl AnyEmbedder {
    /// Provider:model label — the OpenAI variant currently always reports
    /// `text-embedding-3-small` because that is the only OpenAI model the
    /// default config wires up; the Ollama variant reads the configured
    /// model name dynamically so users who pulled a non-default embedding
    /// model see the truth.
    pub fn label(&self) -> String {
        match self {
            AnyEmbedder::OpenAi(_) => "openai:text-embedding-3-small".to_string(),
            AnyEmbedder::Ollama(e) => format!("ollama:{}", e.model_name()),
        }
    }
}

impl EmbeddingProvider for AnyEmbedder {
    fn dim(&self) -> usize {
        match self {
            AnyEmbedder::OpenAi(e) => e.dim(),
            AnyEmbedder::Ollama(e) => e.dim(),
        }
    }

    fn embed_batch(
        &self,
        texts: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<Vec<f32>>>> + Send {
        async move {
            match self {
                AnyEmbedder::OpenAi(e) => e.embed_batch(texts).await,
                AnyEmbedder::Ollama(e) => e.embed_batch(texts).await,
            }
        }
    }
}

/// Pick the default embedder for this environment.
///
/// Priority:
/// 1. `OPENAI_API_KEY` set → [`OpenAiEmbedder`] with `text-embedding-3-small`.
/// 2. Probe `http://localhost:11434/api/tags` (Ollama) → [`OllamaEmbedder`]
///    with `bge-small-en-v1.5`.
/// 3. Else: error with explicit guidance.
pub async fn detect_default(http: reqwest::Client) -> Result<AnyEmbedder> {
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        let embedder = OpenAiEmbedder::new(
            http.clone(),
            OpenAiEmbedderConfig {
                api_key: key,
                ..Default::default()
            },
        );
        return Ok(AnyEmbedder::OpenAi(embedder));
    }
    let probe = http
        .get("http://localhost:11434/api/tags")
        .send()
        .await;
    if let Ok(resp) = probe {
        if resp.status().is_success() {
            let embedder = OllamaEmbedder::new(http, OllamaEmbedderConfig::default());
            return Ok(AnyEmbedder::Ollama(embedder));
        }
    }
    Err(anyhow!(
        "no embedding backend available — set OPENAI_API_KEY or run Ollama at localhost:11434 with an embedding model pulled (e.g. `ollama pull nomic-embed-text`)"
    ))
}

