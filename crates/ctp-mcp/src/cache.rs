//! Analysis result caching for MCP server
//!
//! Implements in-memory caching with TTL for analysis results
//! to avoid redundant computation across multiple queries.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::debug;

use ctp_core::ExplanationGraph;

/// Cache key for file analysis
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct AnalysisCacheKey {
    file_path: String,
    content_hash: String,
}

/// Cached analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResult {
    graph: ExplanationGraph,
    timestamp: u64,
}

/// Analysis cache manager with key tracking for invalidation
pub struct AnalysisCache {
    cache: Cache<AnalysisCacheKey, CachedResult>,
    ttl_seconds: u64,
    /// Tracks all keys by file path for efficient invalidation
    path_index: Arc<RwLock<HashMap<String, Vec<AnalysisCacheKey>>>>,
}

impl AnalysisCache {
    /// Create a new cache with default TTL (5 minutes)
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(300))
    }

    /// Create a new cache with custom TTL
    pub fn with_ttl(ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(ttl)
            .build();

        Self {
            cache,
            ttl_seconds: ttl.as_secs(),
            path_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached result if available and valid
    pub async fn get(&self, file_path: &Path, content_hash: &str) -> Option<ExplanationGraph> {
        let key = AnalysisCacheKey {
            file_path: file_path.display().to_string(),
            content_hash: content_hash.to_string(),
        };

        let result = self.cache.get(&key).await;
        
        if result.is_some() {
            debug!("Cache hit for {}", file_path.display());
        }
        
        result.map(|r| r.graph)
    }

    /// Store analysis result in cache
    pub async fn put(&self, file_path: &Path, content_hash: &str, graph: ExplanationGraph) {
        let key = AnalysisCacheKey {
            file_path: file_path.display().to_string(),
            content_hash: content_hash.to_string(),
        };

        let cached = CachedResult {
            graph,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Track the key for this file path
        let path_str = file_path.display().to_string();
        {
            let mut index = self.path_index.write().await;
            index.entry(path_str.clone()).or_default().push(key.clone());
        }

        self.cache.insert(key, cached).await;
        debug!("Cached result for {}", path_str);
    }

    /// Invalidate all cache entries for a specific file
    pub async fn invalidate(&self, file_path: &Path) {
        let path_str = file_path.display().to_string();
        
        // Remove all keys associated with this file path
        let keys_to_remove = {
            let mut index = self.path_index.write().await;
            index.remove(&path_str)
        };
        
        if let Some(keys) = keys_to_remove {
            let count = keys.len();
            for key in &keys {
                self.cache.invalidate(key).await;
            }
            debug!("Invalidated {} cache entries for {}", count, path_str);
        } else {
            debug!("No cache entries found for {}", path_str);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            ttl_seconds: self.ttl_seconds,
        }
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        self.cache.invalidate_all();
        debug!("Cache cleared");
    }
}

impl Default for AnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AnalysisCache {
    fn clone(&self) -> Self {
        // Create a new cache with same TTL but empty state
        // This is needed because Cache doesn't implement Clone
        Self::with_ttl(Duration::from_secs(self.ttl_seconds))
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: u64,
    pub ttl_seconds: u64,
}
