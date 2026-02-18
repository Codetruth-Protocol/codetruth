//! Incremental analysis cache for CodeTruth
//!
//! Stores content_hash -> ExplanationGraph mappings to skip re-analysis
//! of unchanged files. Uses a simple JSON file for persistence.
//!
//! Cache invalidation:
//! - Content hash mismatch → re-analyze
//! - Engine version mismatch → re-analyze all
//! - Policy change → re-analyze all (policies affect results)
//! - Cache age > max_age → re-analyze

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::models::ExplanationGraph;

/// Cache entry for a single file analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// SHA256 hash of the file content
    pub content_hash: String,
    /// The cached analysis result
    pub result: ExplanationGraph,
    /// When this entry was created (RFC3339)
    pub cached_at: String,
    /// Engine version that produced this result
    pub engine_version: String,
    /// Hash of the policy configuration when this was cached
    pub policy_hash: String,
}

/// Persistent analysis cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCache {
    /// Engine version — cache is invalidated if this changes
    pub engine_version: String,
    /// Policy config hash — cache is invalidated if policies change
    pub policy_hash: String,
    /// Map of file path → cache entry
    pub entries: HashMap<String, CacheEntry>,
    /// When the cache was last saved
    pub last_saved: String,
}

impl AnalysisCache {
    /// Create a new empty cache
    pub fn new(engine_version: &str, policy_hash: &str) -> Self {
        Self {
            engine_version: engine_version.to_string(),
            policy_hash: policy_hash.to_string(),
            entries: HashMap::new(),
            last_saved: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Load cache from disk. Returns empty cache if file doesn't exist or is corrupt.
    pub fn load(cache_path: &Path, engine_version: &str, policy_hash: &str) -> Self {
        match std::fs::read_to_string(cache_path) {
            Ok(content) => match serde_json::from_str::<AnalysisCache>(&content) {
                Ok(cache) => {
                    // Invalidate if engine version or policy hash changed
                    if cache.engine_version != engine_version {
                        info!(
                            "Cache invalidated: engine version changed ({} → {})",
                            cache.engine_version, engine_version
                        );
                        return Self::new(engine_version, policy_hash);
                    }
                    if cache.policy_hash != policy_hash {
                        info!("Cache invalidated: policy configuration changed");
                        return Self::new(engine_version, policy_hash);
                    }
                    debug!("Loaded analysis cache with {} entries", cache.entries.len());
                    cache
                }
                Err(e) => {
                    warn!("Cache file corrupt, starting fresh: {}", e);
                    Self::new(engine_version, policy_hash)
                }
            },
            Err(_) => {
                debug!("No cache file found, starting fresh");
                Self::new(engine_version, policy_hash)
            }
        }
    }

    /// Save cache to disk
    pub fn save(&mut self, cache_path: &Path) -> std::io::Result<()> {
        self.last_saved = chrono::Utc::now().to_rfc3339();

        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(cache_path, content)?;

        debug!("Saved analysis cache with {} entries", self.entries.len());
        Ok(())
    }

    /// Check if a file has a valid cached result
    pub fn get(&self, file_path: &str, content_hash: &str) -> Option<&ExplanationGraph> {
        let entry = self.entries.get(file_path)?;

        if entry.content_hash != content_hash {
            debug!("Cache miss (hash changed): {}", file_path);
            return None;
        }

        if entry.engine_version != self.engine_version {
            debug!("Cache miss (version mismatch): {}", file_path);
            return None;
        }

        debug!("Cache hit: {}", file_path);
        Some(&entry.result)
    }

    /// Store an analysis result in the cache
    pub fn put(&mut self, file_path: &str, content_hash: &str, result: ExplanationGraph) {
        let entry = CacheEntry {
            content_hash: content_hash.to_string(),
            result,
            cached_at: chrono::Utc::now().to_rfc3339(),
            engine_version: self.engine_version.clone(),
            policy_hash: self.policy_hash.clone(),
        };
        self.entries.insert(file_path.to_string(), entry);
    }

    /// Remove stale entries for files that no longer exist
    pub fn prune_missing_files(&mut self) {
        let before = self.entries.len();
        self.entries
            .retain(|path, _| Path::new(path).exists());
        let removed = before - self.entries.len();
        if removed > 0 {
            debug!("Pruned {} stale cache entries", removed);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            engine_version: self.engine_version.clone(),
            policy_hash: self.policy_hash.clone(),
            last_saved: self.last_saved.clone(),
        }
    }
}

/// Cache statistics for reporting
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub engine_version: String,
    pub policy_hash: String,
    pub last_saved: String,
}

/// Default cache file location relative to project root
pub fn default_cache_path(project_root: &Path) -> PathBuf {
    project_root.join(".codetruth").join("analysis_cache.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_test_graph(hash: &str) -> ExplanationGraph {
        ExplanationGraph {
            ctp_version: "1.0.0".into(),
            explanation_id: hash.into(),
            module: Module {
                name: "test.rs".into(),
                path: "src/test.rs".into(),
                language: "rust".into(),
                lines_of_code: 10,
                complexity_score: 1.0,
                content_hash: hash.into(),
            },
            intent: Intent {
                declared_intent: "Test".into(),
                inferred_intent: "Test".into(),
                confidence: 1.0,
                business_context: String::new(),
                technical_rationale: String::new(),
            },
            behavior: Behavior {
                actual_behavior: "Test behavior".into(),
                entry_points: vec![],
                exit_points: vec![],
                side_effects: vec![],
                dependencies: vec![],
            },
            drift: DriftAnalysis {
                drift_detected: false,
                drift_severity: DriftSeverity::None,
                drift_details: vec![],
            },
            policies: PolicyResults {
                evaluated_at: chrono::Utc::now().to_rfc3339(),
                policy_results: vec![],
            },
            history: History {
                previous_versions: vec![],
                evolution: Evolution {
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_modified: chrono::Utc::now().to_rfc3339(),
                    modification_count: 0,
                    stability_score: 1.0,
                },
            },
            metadata: Metadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: Generator {
                    name: "test".into(),
                    version: "1.0.0".into(),
                    llm_provider: None,
                    llm_model: None,
                },
                extensions: serde_json::json!({}),
            },
        }
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = AnalysisCache::new("1.0.0", "policy_abc");
        let graph = make_test_graph("hash123");
        cache.put("src/test.rs", "hash123", graph);

        assert!(cache.get("src/test.rs", "hash123").is_some());
    }

    #[test]
    fn test_cache_miss_hash_changed() {
        let mut cache = AnalysisCache::new("1.0.0", "policy_abc");
        let graph = make_test_graph("hash123");
        cache.put("src/test.rs", "hash123", graph);

        assert!(cache.get("src/test.rs", "hash456").is_none());
    }

    #[test]
    fn test_cache_invalidation_version() {
        let cache = AnalysisCache::load(
            Path::new("/nonexistent"),
            "2.0.0",
            "policy_abc",
        );
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_cache_roundtrip() {
        let dir = std::env::temp_dir().join("ctp_cache_test");
        let cache_path = dir.join("cache.json");

        let mut cache = AnalysisCache::new("1.0.0", "policy_abc");
        cache.put("src/test.rs", "hash123", make_test_graph("hash123"));
        cache.save(&cache_path).unwrap();

        let loaded = AnalysisCache::load(&cache_path, "1.0.0", "policy_abc");
        assert_eq!(loaded.entries.len(), 1);
        assert!(loaded.get("src/test.rs", "hash123").is_some());

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }
}
