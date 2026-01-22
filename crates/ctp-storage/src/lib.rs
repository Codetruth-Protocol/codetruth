//! # CTP Storage Layer
//!
//! Persistent storage for CodeTruth Protocol explanation graphs.
//! Enables historical drift tracking and caching.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::{debug, info};

use ctp_core::ExplanationGraph;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
}

pub struct StorageBackend {
    conn: Connection,
}

impl StorageBackend {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database")?;
        
        let backend = Self { conn };
        backend.initialize_schema()?;
        
        info!("Storage backend initialized at {:?}", db_path);
        Ok(backend)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to create in-memory database")?;
        
        let backend = Self { conn };
        backend.initialize_schema()?;
        
        debug!("In-memory storage backend initialized");
        Ok(backend)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS explanation_graphs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                analyzed_at TEXT NOT NULL,
                graph_json TEXT NOT NULL,
                UNIQUE(file_path, content_hash)
            );

            CREATE INDEX IF NOT EXISTS idx_file_path ON explanation_graphs(file_path);
            CREATE INDEX IF NOT EXISTS idx_analyzed_at ON explanation_graphs(analyzed_at DESC);
            CREATE INDEX IF NOT EXISTS idx_file_hash ON explanation_graphs(file_hash);

            CREATE TABLE IF NOT EXISTS file_cache (
                file_path TEXT PRIMARY KEY,
                last_modified TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                latest_graph_id INTEGER,
                FOREIGN KEY(latest_graph_id) REFERENCES explanation_graphs(id)
            );
            "#,
        )?;
        
        Ok(())
    }

    pub fn store_graph(&self, file_path: &str, content: &str, graph: &ExplanationGraph) -> Result<i64> {
        let content_hash = Self::hash_content(content);
        let file_hash = graph.module.content_hash.clone();
        let analyzed_at = chrono::Utc::now().to_rfc3339();
        let graph_json = serde_json::to_string(graph)?;

        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO explanation_graphs 
             (file_path, file_hash, content_hash, analyzed_at, graph_json) 
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )?;

        stmt.execute(params![file_path, file_hash, content_hash, analyzed_at, graph_json])?;
        let id = self.conn.last_insert_rowid();

        self.conn.execute(
            "INSERT OR REPLACE INTO file_cache (file_path, last_modified, file_hash, latest_graph_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![file_path, analyzed_at, file_hash, id],
        )?;

        debug!("Stored explanation graph for {} (id: {})", file_path, id);
        Ok(id)
    }

    pub fn get_latest_graph(&self, file_path: &str) -> Result<Option<ExplanationGraph>> {
        let mut stmt = self.conn.prepare(
            "SELECT graph_json FROM explanation_graphs 
             WHERE file_path = ?1 
             ORDER BY analyzed_at DESC 
             LIMIT 1"
        )?;

        let mut rows = stmt.query(params![file_path])?;
        
        if let Some(row) = rows.next()? {
            let graph_json: String = row.get(0)?;
            let graph: ExplanationGraph = serde_json::from_str(&graph_json)?;
            Ok(Some(graph))
        } else {
            Ok(None)
        }
    }

    pub fn get_previous_version(&self, file_path: &str, before_hash: &str) -> Result<Option<ExplanationGraph>> {
        let mut stmt = self.conn.prepare(
            "SELECT graph_json FROM explanation_graphs 
             WHERE file_path = ?1 AND content_hash != ?2
             ORDER BY analyzed_at DESC 
             LIMIT 1"
        )?;

        let mut rows = stmt.query(params![file_path, before_hash])?;
        
        if let Some(row) = rows.next()? {
            let graph_json: String = row.get(0)?;
            let graph: ExplanationGraph = serde_json::from_str(&graph_json)?;
            Ok(Some(graph))
        } else {
            Ok(None)
        }
    }

    pub fn get_history(&self, file_path: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_hash, content_hash, analyzed_at 
             FROM explanation_graphs 
             WHERE file_path = ?1 
             ORDER BY analyzed_at DESC 
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![file_path, limit], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                file_hash: row.get(1)?,
                content_hash: row.get(2)?,
                analyzed_at: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }

        Ok(entries)
    }

    pub fn is_cached(&self, file_path: &str, content: &str) -> Result<bool> {
        let content_hash = Self::hash_content(content);
        
        let mut stmt = self.conn.prepare(
            "SELECT 1 FROM explanation_graphs 
             WHERE file_path = ?1 AND content_hash = ?2 
             LIMIT 1"
        )?;

        let exists = stmt.exists(params![file_path, content_hash])?;
        Ok(exists)
    }

    pub fn clear_file_history(&self, file_path: &str) -> Result<usize> {
        // Delete file_cache first to avoid foreign key constraint
        self.conn.execute(
            "DELETE FROM file_cache WHERE file_path = ?1",
            params![file_path],
        )?;
        
        let count = self.conn.execute(
            "DELETE FROM explanation_graphs WHERE file_path = ?1",
            params![file_path],
        )?;

        debug!("Cleared {} history entries for {}", count, file_path);
        Ok(count)
    }

    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute_batch("VACUUM")?;
        info!("Database vacuumed");
        Ok(())
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub file_hash: String,
    pub content_hash: String,
    pub analyzed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctp_core::{Module, Intent, Behavior, DriftAnalysis, PolicyResults, History, Metadata, Generator};

    fn create_test_graph(file_path: &str, content_hash: &str) -> ExplanationGraph {
        ExplanationGraph {
            ctp_version: "1.0.0".into(),
            explanation_id: "test-123".into(),
            module: Module {
                name: "test".into(),
                path: file_path.into(),
                language: "rust".into(),
                lines_of_code: 100,
                complexity_score: 5.0,
                content_hash: content_hash.into(),
            },
            intent: Intent {
                declared_intent: "Test function".into(),
                inferred_intent: "Test function".into(),
                confidence: 0.9,
                business_context: String::new(),
                technical_rationale: String::new(),
            },
            behavior: Behavior {
                actual_behavior: "Tests things".into(),
                entry_points: vec![],
                exit_points: vec![],
                side_effects: vec![],
                dependencies: vec![],
            },
            drift: DriftAnalysis {
                drift_detected: false,
                drift_severity: ctp_core::DriftSeverity::None,
                drift_details: vec![],
            },
            policies: PolicyResults {
                evaluated_at: chrono::Utc::now().to_rfc3339(),
                policy_results: vec![],
            },
            history: History {
                previous_versions: vec![],
                evolution: ctp_core::Evolution {
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_modified: chrono::Utc::now().to_rfc3339(),
                    modification_count: 0,
                    stability_score: 1.0,
                },
            },
            metadata: Metadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: Generator {
                    name: "ctp-cli".into(),
                    version: "0.1.0".into(),
                    llm_provider: None,
                    llm_model: None,
                },
                extensions: serde_json::Value::Null,
            },
        }
    }

    #[test]
    fn test_store_and_retrieve() {
        let storage = StorageBackend::in_memory().unwrap();
        let graph = create_test_graph("test.rs", "hash123");
        
        storage.store_graph("test.rs", "fn test() {}", &graph).unwrap();
        
        let retrieved = storage.get_latest_graph("test.rs").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().module.name, "test");
    }

    #[test]
    fn test_caching() {
        let storage = StorageBackend::in_memory().unwrap();
        let graph = create_test_graph("test.rs", "hash123");
        let content = "fn test() {}";
        
        assert!(!storage.is_cached("test.rs", content).unwrap());
        
        storage.store_graph("test.rs", content, &graph).unwrap();
        
        assert!(storage.is_cached("test.rs", content).unwrap());
        assert!(!storage.is_cached("test.rs", "fn test2() {}").unwrap());
    }

    #[test]
    fn test_history() {
        let storage = StorageBackend::in_memory().unwrap();
        
        let graph1 = create_test_graph("test.rs", "hash1");
        storage.store_graph("test.rs", "version 1", &graph1).unwrap();
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let graph2 = create_test_graph("test.rs", "hash2");
        storage.store_graph("test.rs", "version 2", &graph2).unwrap();
        
        let history = storage.get_history("test.rs", 10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].content_hash, StorageBackend::hash_content("version 2"));
    }

    #[test]
    fn test_clear_history() {
        let storage = StorageBackend::in_memory().unwrap();
        let graph = create_test_graph("test.rs", "hash123");
        
        storage.store_graph("test.rs", "content", &graph).unwrap();
        assert!(storage.get_latest_graph("test.rs").unwrap().is_some());
        
        let count = storage.clear_file_history("test.rs").unwrap();
        assert_eq!(count, 1);
        assert!(storage.get_latest_graph("test.rs").unwrap().is_none());
    }
}
