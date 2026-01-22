//! Product Specification Management
//!
//! This crate provides intelligent product specification management that:
//! - Auto-generates specs from codebase analysis
//! - Enriches specs with LLM intelligence (Groq/OpenAI)
//! - Maintains bidirectional sync with code
//! - Validates specs against implementation

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

pub mod generator;
pub mod validator;
pub mod discovery;
pub mod keyword_dictionary;

// TODO: Implement LLM enrichment module
// #[cfg(feature = "llm")]
// pub mod llm_enrichment;

pub use generator::SpecGenerator;
pub use validator::SpecValidator;
pub use discovery::{DiscoveredSpec, PatternCluster};
pub use keyword_dictionary::{KeywordDictionary, DomainKeyword, CriticalDomain};

/// Product metadata specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMetadata {
    pub ctp_version: String,
    pub product: ProductInfo,
    pub demographics: Option<Demographics>,
    pub core_functionalities: Vec<CoreFunctionality>,
    pub technical_constraints: Option<TechnicalConstraints>,
    pub development_markers: Option<DevelopmentMarkers>,
    pub drift_thresholds: Option<DriftThresholds>,
    pub documentation: Option<DocumentationConfig>,
    pub git_analysis: Option<GitAnalysisConfig>,
    
    /// Confidence score for this spec (0.0-1.0)
    #[serde(default)]
    pub confidence: f64,
    
    /// How this spec was generated
    #[serde(default)]
    pub generation_method: GenerationMethod,
    
    /// When this spec was last updated
    #[serde(default)]
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub name: String,
    pub product_type: String,
    pub version: Option<String>,
    pub primary_language: String,
    pub natural_language: String,
    pub supported_languages: Vec<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographics {
    pub target_user_base: Option<UserBaseMetrics>,
    pub primary_markets: Vec<String>,
    pub user_segments: Vec<UserSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBaseMetrics {
    pub current_count: usize,
    pub one_year_target: usize,
    pub growth_rate_expected: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSegment {
    pub name: String,
    pub percentage: f64,
    pub critical_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreFunctionality {
    pub id: String,
    pub name: String,
    pub category: FunctionalityCategory,
    pub criticality: Criticality,
    pub description: String,
    pub entry_points: Vec<EntryPoint>,
    pub dependencies: Vec<String>,
    pub metrics: Option<FunctionalityMetrics>,
    pub status: FunctionalityStatus,
    pub last_verified: Option<String>,
    
    /// Business context for this functionality
    pub business_context: String,
    
    /// Technical rationale
    pub technical_rationale: String,
    
    /// Confidence in this functionality definition
    #[serde(default)]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalityCategory {
    Core,
    Essential,
    Enhancement,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criticality {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub file: String,
    pub function: Option<String>,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalityMetrics {
    pub usage_percentage: Option<f64>,
    pub performance_target: Option<String>,
    pub error_rate_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionalityStatus {
    Active,
    Deprecated,
    Planned,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalConstraints {
    pub max_response_time_ms: Option<u64>,
    pub max_memory_mb: Option<u64>,
    pub supported_platforms: Vec<String>,
    pub minimum_test_coverage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentMarkers {
    pub stub_patterns: Vec<StubPattern>,
    pub allowed_exceptions: Vec<AllowedException>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubPattern {
    pub pattern: String,
    pub language: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedException {
    pub pattern: String,
    pub file_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftThresholds {
    pub intent_similarity_threshold: f64,
    pub behavior_similarity_threshold: f64,
    pub documentation_staleness_days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub readme_files: Vec<String>,
    pub doc_directories: Vec<String>,
    pub required_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAnalysisConfig {
    pub analyze_recent_commits: usize,
    pub high_churn_threshold: usize,
    pub protected_files: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMethod {
    #[default]
    Manual,
    RuleBased,
    LlmEnriched,
    Hybrid,
}

/// Manager for product specifications
pub struct ProductSpecManager {
    spec: Arc<RwLock<ProductMetadata>>,
    spec_path: PathBuf,
    generator: SpecGenerator,
    validator: SpecValidator,
}

impl ProductSpecManager {
    /// Create new manager with existing spec
    pub fn new(spec: ProductMetadata, spec_path: PathBuf) -> Self {
        Self {
            spec: Arc::new(RwLock::new(spec)),
            spec_path,
            generator: SpecGenerator::new(),
            validator: SpecValidator::new(),
        }
    }
    
    /// Load spec from file or generate if missing
    pub async fn load_or_generate(
        repo_path: &Path,
        use_llm: bool,
    ) -> Result<Self> {
        let spec_path = repo_path.join("product-metadata.json");
        
        if spec_path.exists() {
            info!("Loading existing product spec from {}", spec_path.display());
            let content = tokio::fs::read_to_string(&spec_path).await?;
            let spec: ProductMetadata = serde_json::from_str(&content)?;
            Ok(Self::new(spec, spec_path))
        } else {
            info!("No product spec found, generating from codebase...");
            let generator = SpecGenerator::new();
            let spec = generator.generate_from_codebase(repo_path, use_llm).await?;
            
            // Save generated spec
            let content = serde_json::to_string_pretty(&spec)?;
            tokio::fs::write(&spec_path, content).await?;
            
            Ok(Self::new(spec, spec_path))
        }
    }
    
    /// Get current spec
    pub async fn get_spec(&self) -> ProductMetadata {
        self.spec.read().await.clone()
    }
    
    /// Update spec
    pub async fn update_spec(&self, spec: ProductMetadata) -> Result<()> {
        *self.spec.write().await = spec.clone();
        
        // Save to file
        let content = serde_json::to_string_pretty(&spec)?;
        tokio::fs::write(&self.spec_path, content).await?;
        
        Ok(())
    }
    
    /// Validate spec against codebase
    pub async fn validate(&self, analyses: &[ctp_core::ExplanationGraph]) -> Result<ValidationReport> {
        let spec = self.get_spec().await;
        self.validator.validate(&spec, analyses)
    }
    
    /// Find functionality related to a file
    pub async fn find_related_functionality(
        &self,
        file_path: &str,
        intent: &str,
    ) -> Option<CoreFunctionality> {
        let spec = self.spec.read().await;
        
        // Match by entry point
        for func in &spec.core_functionalities {
            for entry in &func.entry_points {
                if file_path.contains(&entry.file) {
                    return Some(func.clone());
                }
            }
        }
        
        // Match by intent similarity (simple keyword matching for now)
        for func in &spec.core_functionalities {
            if self.calculate_similarity(intent, &func.description) > 0.7 {
                return Some(func.clone());
            }
        }
        
        None
    }
    
    fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        use std::collections::HashSet;
        
        let a_lower = a.to_lowercase();
        let a_words: HashSet<_> = a_lower
            .split_whitespace()
            .collect();
        let b_lower = b.to_lowercase();
        let b_words: HashSet<_> = b_lower
            .split_whitespace()
            .collect();
        
        let intersection = a_words.intersection(&b_words).count();
        let union = a_words.union(&b_words).count();
        
        if union == 0 { 0.0 } else { intersection as f64 / union as f64 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub confidence: f64,
    pub missing_functionalities: Vec<String>,
    pub undocumented_code: Vec<String>,
    pub spec_drift: Vec<SpecDrift>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDrift {
    pub drift_type: SpecDriftType,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecDriftType {
    NewFunctionality,
    RemovedFunctionality,
    ChangedBehavior,
    InconsistentMetrics,
}

impl Default for ProductMetadata {
    fn default() -> Self {
        Self {
            ctp_version: "1.0.0".into(),
            product: ProductInfo {
                name: "Unknown".into(),
                product_type: "library".into(),
                version: None,
                primary_language: "rust".into(),
                natural_language: "en".into(),
                supported_languages: vec!["en".into()],
                currency: None,
            },
            demographics: None,
            core_functionalities: vec![],
            technical_constraints: None,
            development_markers: None,
            drift_thresholds: None,
            documentation: None,
            git_analysis: None,
            confidence: 0.0,
            generation_method: GenerationMethod::Manual,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}
