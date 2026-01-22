//! Design Baseline Management
//!
//! Git-like tracking of design system evolution over time.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::tokens::DesignTokens;

/// A snapshot of the design system at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignBaseline {
    /// Unique identifier for this baseline
    pub id: String,
    
    /// When this baseline was created
    pub created_at: String,
    
    /// Git commit hash (if available)
    pub commit_hash: Option<String>,
    
    /// The design tokens at this point
    pub tokens: DesignTokens,
    
    /// Hash of the tokens for comparison
    pub content_hash: String,
    
    /// Optional description
    pub description: Option<String>,
}

impl DesignBaseline {
    pub fn new(tokens: DesignTokens, description: Option<String>) -> Self {
        let content_hash = Self::hash_tokens(&tokens);
        let id = format!("baseline_{}", &content_hash[..8]);
        
        Self {
            id,
            created_at: chrono::Utc::now().to_rfc3339(),
            commit_hash: None,
            tokens,
            content_hash,
            description,
        }
    }

    pub fn with_commit(mut self, commit: &str) -> Self {
        self.commit_hash = Some(commit.to_string());
        self
    }

    fn hash_tokens(tokens: &DesignTokens) -> String {
        let json = serde_json::to_string(tokens).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if this baseline differs from another
    pub fn differs_from(&self, other: &DesignBaseline) -> bool {
        self.content_hash != other.content_hash
    }
}

/// Manages design system baselines
pub struct BaselineManager {
    /// Directory where baselines are stored
    baseline_dir: std::path::PathBuf,
    
    /// Current baseline (if loaded)
    current: Option<DesignBaseline>,
}

impl BaselineManager {
    pub fn new(baseline_dir: &Path) -> Self {
        Self {
            baseline_dir: baseline_dir.to_path_buf(),
            current: None,
        }
    }

    /// Initialize baseline directory
    pub fn init(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.baseline_dir)?;
        std::fs::create_dir_all(self.baseline_dir.join("history"))?;
        Ok(())
    }

    /// Save a new baseline
    pub fn save_baseline(&mut self, baseline: DesignBaseline) -> std::io::Result<()> {
        self.init()?;
        
        // Save as current
        let current_path = self.baseline_dir.join("current.json");
        let json = serde_json::to_string_pretty(&baseline)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&current_path, &json)?;
        
        // Save to history
        let history_path = self.baseline_dir.join("history")
            .join(format!("{}.json", baseline.created_at.replace(':', "-")));
        std::fs::write(&history_path, &json)?;
        
        self.current = Some(baseline);
        Ok(())
    }

    /// Load the current baseline
    pub fn load_current(&mut self) -> std::io::Result<Option<DesignBaseline>> {
        let current_path = self.baseline_dir.join("current.json");
        
        if !current_path.exists() {
            return Ok(None);
        }
        
        let json = std::fs::read_to_string(&current_path)?;
        let baseline: DesignBaseline = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        self.current = Some(baseline.clone());
        Ok(Some(baseline))
    }

    /// Get the current baseline
    pub fn current(&self) -> Option<&DesignBaseline> {
        self.current.as_ref()
    }

    /// List all historical baselines
    pub fn list_history(&self) -> std::io::Result<Vec<BaselineInfo>> {
        let history_dir = self.baseline_dir.join("history");
        
        if !history_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut baselines = vec![];
        
        for entry in std::fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(json) = std::fs::read_to_string(&path) {
                    if let Ok(baseline) = serde_json::from_str::<DesignBaseline>(&json) {
                        baselines.push(BaselineInfo {
                            id: baseline.id,
                            created_at: baseline.created_at,
                            commit_hash: baseline.commit_hash,
                            description: baseline.description,
                        });
                    }
                }
            }
        }
        
        // Sort by date (newest first)
        baselines.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(baselines)
    }

    /// Load a specific baseline by ID
    pub fn load_baseline(&self, id: &str) -> std::io::Result<Option<DesignBaseline>> {
        let history_dir = self.baseline_dir.join("history");
        
        for entry in std::fs::read_dir(&history_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(baseline) = serde_json::from_str::<DesignBaseline>(&json) {
                    if baseline.id == id {
                        return Ok(Some(baseline));
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Compare two baselines and return the differences
    pub fn compare(
        &self,
        old: &DesignBaseline,
        new: &DesignBaseline,
    ) -> BaselineDiff {
        let mut diff = BaselineDiff::default();
        
        // Compare colors
        for (hex, _) in &new.tokens.colors.colors {
            if !old.tokens.colors.colors.contains_key(hex) {
                diff.colors_added.push(hex.clone());
            }
        }
        for (hex, _) in &old.tokens.colors.colors {
            if !new.tokens.colors.colors.contains_key(hex) {
                diff.colors_removed.push(hex.clone());
            }
        }
        
        // Compare components
        for (name, component) in &new.tokens.components {
            if !old.tokens.components.contains_key(name) {
                diff.components_added.push(name.clone());
            } else {
                let old_component = &old.tokens.components[name];
                for variant in &component.variants {
                    if !old_component.variants.contains(variant) {
                        diff.variants_added.push(format!("{}:{}", name, variant));
                    }
                }
            }
        }
        for name in old.tokens.components.keys() {
            if !new.tokens.components.contains_key(name) {
                diff.components_removed.push(name.clone());
            }
        }
        
        diff
    }
}

/// Summary info about a baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineInfo {
    pub id: String,
    pub created_at: String,
    pub commit_hash: Option<String>,
    pub description: Option<String>,
}

/// Differences between two baselines
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BaselineDiff {
    pub colors_added: Vec<String>,
    pub colors_removed: Vec<String>,
    pub components_added: Vec<String>,
    pub components_removed: Vec<String>,
    pub variants_added: Vec<String>,
    pub variants_removed: Vec<String>,
}

impl BaselineDiff {
    pub fn is_empty(&self) -> bool {
        self.colors_added.is_empty()
            && self.colors_removed.is_empty()
            && self.components_added.is_empty()
            && self.components_removed.is_empty()
            && self.variants_added.is_empty()
            && self.variants_removed.is_empty()
    }

    pub fn summary(&self) -> String {
        let mut parts = vec![];
        
        if !self.colors_added.is_empty() {
            parts.push(format!("+{} colors", self.colors_added.len()));
        }
        if !self.colors_removed.is_empty() {
            parts.push(format!("-{} colors", self.colors_removed.len()));
        }
        if !self.components_added.is_empty() {
            parts.push(format!("+{} components", self.components_added.len()));
        }
        if !self.components_removed.is_empty() {
            parts.push(format!("-{} components", self.components_removed.len()));
        }
        if !self.variants_added.is_empty() {
            parts.push(format!("+{} variants", self.variants_added.len()));
        }
        
        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::ColorToken;

    #[test]
    fn test_baseline_creation() {
        let tokens = DesignTokens::new();
        let baseline = DesignBaseline::new(tokens, Some("Initial baseline".to_string()));
        
        assert!(baseline.id.starts_with("baseline_"));
        assert!(!baseline.content_hash.is_empty());
    }

    #[test]
    fn test_baseline_diff() {
        let mut old_tokens = DesignTokens::new();
        old_tokens.colors.colors.insert(
            "#2563eb".to_string(),
            ColorToken::from_hex("#2563eb").unwrap(),
        );
        
        let mut new_tokens = DesignTokens::new();
        new_tokens.colors.colors.insert(
            "#2563eb".to_string(),
            ColorToken::from_hex("#2563eb").unwrap(),
        );
        new_tokens.colors.colors.insert(
            "#ff0000".to_string(),
            ColorToken::from_hex("#ff0000").unwrap(),
        );
        
        let old_baseline = DesignBaseline::new(old_tokens, None);
        let new_baseline = DesignBaseline::new(new_tokens, None);
        
        let manager = BaselineManager::new(Path::new("/tmp/test"));
        let diff = manager.compare(&old_baseline, &new_baseline);
        
        assert_eq!(diff.colors_added.len(), 1);
        assert!(diff.colors_added.contains(&"#ff0000".to_string()));
    }
}
