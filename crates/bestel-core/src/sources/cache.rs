//! File-backed TTL cache for source responses.
//!
//! One JSON file per cache key under `~/.bestel/cache/<sha256>.json`. The
//! file stores `{"stamp": <unix_secs>, "data": <T>}`. `get` returns `None`
//! whenever the file is missing, unreadable, malformed, or older than the
//! caller-provided TTL — never propagates I/O errors to callers (cache miss
//! is the safe default).

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs;

#[derive(Clone)]
pub struct FileCache {
    dir: PathBuf,
}

impl FileCache {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> PathBuf {
        std::env::var("BESTEL_CACHE_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|h| h.join(".bestel").join("cache")))
            .unwrap_or_else(|| PathBuf::from(".bestel-cache"))
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    fn path_for(&self, key: &str) -> PathBuf {
        let digest = Sha256::digest(key.as_bytes());
        let hex = digest.iter().fold(String::with_capacity(64), |mut s, b| {
            use std::fmt::Write;
            let _ = write!(&mut s, "{:02x}", b);
            s
        });
        self.dir.join(format!("{hex}.json"))
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str, ttl: Duration) -> Option<T> {
        let path = self.path_for(key);
        let raw = fs::read(&path).await.ok()?;
        let entry: CacheEntry<T> = serde_json::from_slice(&raw).ok()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        if now.saturating_sub(entry.stamp) >= ttl.as_secs() {
            return None;
        }
        Some(entry.data)
    }

    pub async fn put<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        fs::create_dir_all(&self.dir)
            .await
            .with_context(|| format!("cache: create_dir_all {}", self.dir.display()))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry = CacheEntry { stamp, data };
        let bytes = serde_json::to_vec(&entry)?;
        let path = self.path_for(key);
        fs::write(&path, bytes)
            .await
            .with_context(|| format!("cache: write {}", path.display()))?;
        Ok(())
    }
}

#[derive(Serialize, serde::Deserialize)]
struct CacheEntry<T> {
    stamp: u64,
    data: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Probe {
        n: u32,
        s: String,
    }

    fn tmpdir() -> PathBuf {
        let id = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("bestel-cache-test-{id}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[tokio::test]
    async fn miss_then_hit() {
        let cache = FileCache::new(tmpdir());
        assert!(cache
            .get::<Probe>("k1", Duration::from_secs(60))
            .await
            .is_none());
        let v = Probe {
            n: 7,
            s: "wraeclast".into(),
        };
        cache.put("k1", &v).await.unwrap();
        let hit: Probe = cache.get("k1", Duration::from_secs(60)).await.unwrap();
        assert_eq!(hit, v);
    }

    #[tokio::test]
    async fn expired_returns_none() {
        let cache = FileCache::new(tmpdir());
        let v = Probe {
            n: 1,
            s: "x".into(),
        };
        cache.put("k2", &v).await.unwrap();
        assert!(cache
            .get::<Probe>("k2", Duration::from_secs(0))
            .await
            .is_none());
    }
}
