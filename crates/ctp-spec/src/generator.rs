//! Product specification auto-generation from codebase analysis

use super::*;
use ctp_core::{CodeTruthEngine, EngineConfig, ExplanationGraph};
use crate::keyword_dictionary::KeywordDictionary;

pub struct SpecGenerator {
    engine: CodeTruthEngine,
    keyword_dict: KeywordDictionary,
}

impl SpecGenerator {
    pub fn new() -> Self {
        let config = EngineConfig {
            enable_llm: false, // Start without LLM for discovery
            ..Default::default()
        };
        
        Self {
            engine: CodeTruthEngine::new(config),
            keyword_dict: KeywordDictionary::default(),
        }
    }
    
    /// Create generator with custom keyword dictionary
    pub fn with_keyword_dict(keyword_dict: KeywordDictionary) -> Self {
        let config = EngineConfig {
            enable_llm: false,
            ..Default::default()
        };
        
        Self {
            engine: CodeTruthEngine::new(config),
            keyword_dict,
        }
    }
    
    /// Generate product spec from codebase scan
    pub async fn generate_from_codebase(
        &self,
        repo_path: &Path,
        use_llm: bool,
    ) -> Result<ProductMetadata> {
        info!("Scanning codebase at {}", repo_path.display());
        
        // 1. Scan all code files
        let files = self.collect_code_files(repo_path)?;
        info!("Found {} code files", files.len());
        
        // 2. Analyze all files
        let mut analyses = vec![];
        for file in &files {
            match self.engine.analyze_file(file).await {
                Ok(analysis) => analyses.push(analysis),
                Err(e) => warn!("Failed to analyze {}: {}", file.display(), e),
            }
        }
        
        info!("Successfully analyzed {} files", analyses.len());
        
        // 3. Discover patterns (rule-based)
        let discovered = self.discover_patterns(&analyses, repo_path)?;
        
        // 4. LLM enrichment if requested
        #[cfg(feature = "llm")]
        if use_llm {
            return self.llm_enrich_spec(discovered, &analyses).await;
        }
        
        // 5. Build spec from discovered patterns
        Ok(self.build_spec_from_discovery(discovered, repo_path))
    }
    
    fn collect_code_files(&self, repo_path: &Path) -> Result<Vec<PathBuf>> {
        use std::fs;
        
        let mut files = vec![];
        let extensions = ["rs", "py", "js", "ts", "go", "java"];
        
        fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>, extensions: &[&str]) -> Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    // Skip common ignore patterns (expanded to include test directories)
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.')
                            || name == "target"
                            || name == "node_modules"
                            || name == "tests"
                            || name == "test"
                            || name == "fixtures"
                            || name == "dist"
                            || name == "build"
                            || name == "__pycache__"
                            || name == "vendor"
                            || name == "coverage"
                            || name == ".git"
                            || name == "examples"
                            || name == "docs"
                        {
                            continue;
                        }
                    }
                    
                    if path.is_dir() {
                        visit_dirs(&path, files, extensions)?;
                    } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if extensions.contains(&ext) {
                            files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }
        
        visit_dirs(repo_path, &mut files, &extensions)?;
        Ok(files)
    }
    
    fn discover_patterns(
        &self,
        analyses: &[ExplanationGraph],
        repo_path: &Path,
    ) -> Result<discovery::DiscoveredSpec> {
        use crate::discovery::*;
        
        // Cluster by intent similarity
        let clusters = self.cluster_by_intent(analyses);
        
        // Extract functionalities from clusters
        let mut functionalities: Vec<CoreFunctionality> = clusters.iter()
            .map(|cluster| self.cluster_to_functionality(cluster, repo_path))
            .collect();
        
        // Post-process: deduplicate and validate
        functionalities = self.post_process_functionalities(functionalities);
        
        // Infer technical constraints
        let constraints = self.infer_constraints(analyses);
        
        // Detect primary language
        let primary_language = self.detect_primary_language(repo_path)?;
        
        // Calculate overall confidence based on functionalities
        let avg_confidence = if functionalities.is_empty() {
            0.0
        } else {
            functionalities.iter().map(|f| f.confidence).sum::<f64>() / functionalities.len() as f64
        };
        
        Ok(DiscoveredSpec {
            functionalities,
            constraints,
            primary_language,
            confidence: avg_confidence,
        })
    }
    
    fn cluster_by_intent(&self, analyses: &[ExplanationGraph]) -> Vec<discovery::PatternCluster> {
        use std::collections::HashMap;
        
        let mut clusters: HashMap<String, Vec<&ExplanationGraph>> = HashMap::new();
        
        for analysis in analyses {
            // Extract key terms from intent
            let key = self.extract_intent_key(&analysis.intent.inferred_intent);
            
            // Skip invalid keys
            if key == "general" || key.len() < 3 {
                continue;
            }
            
            clusters.entry(key).or_default().push(analysis);
        }
        
        // Convert to PatternCluster with evidence-based confidence
        clusters.into_iter()
            .filter(|(_, graphs)| graphs.len() >= 3) // Require 3+ files for higher confidence
            .map(|(key, graphs)| {
                let confidence = self.calculate_cluster_confidence(&graphs);
                discovery::PatternCluster {
                    key,
                    graphs: graphs.into_iter().cloned().collect(),
                    confidence,
                }
            })
            .collect()
    }
    
    fn extract_intent_key(&self, intent: &str) -> String {
        // Sanitize input
        let cleaned = intent.trim();
        if cleaned.is_empty() || cleaned.len() < 3 {
            return "general".to_string();
        }
        
        // Check if first character is alphabetic (prevent "#" and other symbols)
        if !cleaned.chars().next().unwrap_or(' ').is_alphabetic() {
            return "general".to_string();
        }
        
        // Use keyword dictionary for domain classification
        if let Some(keyword) = self.keyword_dict.find_domain_keyword(intent) {
            return keyword.term.clone();
        }
        
        let lower = intent.to_lowercase();
        
        // Fallback: extract first meaningful word (3+ chars, not stopword)
        for word in lower.split_whitespace() {
            if word.len() >= 3 && !self.keyword_dict.is_stopword(word) && word.chars().all(|c| c.is_alphabetic()) {
                return word.to_string();
            }
        }
        
        "general".to_string()
    }
    
    fn cluster_to_functionality(
        &self,
        cluster: &discovery::PatternCluster,
        repo_path: &Path,
    ) -> CoreFunctionality {
        let entry_points: Vec<EntryPoint> = cluster.graphs.iter()
            .map(|g| EntryPoint {
                file: g.module.path.strip_prefix(repo_path.to_str().unwrap_or(""))
                    .unwrap_or(&g.module.path)
                    .to_string(),
                function: g.behavior.entry_points.first()
                    .map(|ep| ep.function.clone()),
                module: Some(g.module.name.clone()),
            })
            .collect();
        
        CoreFunctionality {
            id: cluster.key.replace(' ', "_"),
            name: self.humanize_name(&cluster.key),
            category: self.classify_category(&cluster.key, repo_path),
            criticality: self.assess_criticality_advanced(cluster, repo_path),
            description: self.generate_description(&cluster.key, cluster.graphs.len()),
            entry_points,
            dependencies: vec![],
            metrics: None,
            status: FunctionalityStatus::Active,
            last_verified: Some(chrono::Utc::now().to_rfc3339()),
            business_context: String::new(), // Filled by LLM
            technical_rationale: format!("Discovered from {} related files", cluster.graphs.len()),
            confidence: cluster.confidence,
        }
    }
    
    fn humanize_name(&self, key: &str) -> String {
        key.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn classify_category(&self, key: &str, repo_path: &Path) -> FunctionalityCategory {
        // Context-aware classification based on project type
        let is_cli_tool = repo_path.join("Cargo.toml").exists() 
            && std::fs::read_to_string(repo_path.join("Cargo.toml"))
                .map(|c| c.contains("[[bin]]"))
                .unwrap_or(false);
        
        let is_web_app = repo_path.join("package.json").exists()
            && std::fs::read_to_string(repo_path.join("package.json"))
                .map(|c| c.contains("react") || c.contains("next"))
                .unwrap_or(false);
        
        // Core keywords vary by project type
        let core_keywords = self.keyword_dict.get_project_type_keywords(if is_cli_tool {
            "cli_tool"
        } else if is_web_app {
            "web_app"
        } else {
            "library"
        });
        
        let essential_keywords = ["handler", "middleware", "route", "endpoint", "service"];
        
        if core_keywords.iter().any(|k| key.contains(k)) {
            FunctionalityCategory::Core
        } else if essential_keywords.iter().any(|k| key.contains(k)) {
            FunctionalityCategory::Essential
        } else {
            FunctionalityCategory::Enhancement
        }
    }
    
    fn assess_criticality(&self, file_count: usize) -> Criticality {
        match file_count {
            10.. => Criticality::Critical,
            5..=9 => Criticality::High,
            3..=4 => Criticality::Medium,
            _ => Criticality::Low,
        }
    }
    
    fn assess_criticality_advanced(
        &self,
        cluster: &discovery::PatternCluster,
        repo_path: &Path,
    ) -> Criticality {
        let mut score = 0.0;
        
        // Signal 1: File count (0-2 points)
        score += (cluster.graphs.len() as f64 / 10.0).min(2.0);
        
        // Signal 2: Error handling presence (0-2 points)
        let has_error_handling = cluster.graphs.iter()
            .filter(|g| {
                if let Ok(content) = std::fs::read_to_string(&g.module.path) {
                    content.contains("Result<") || content.contains("Error") 
                        || content.contains("try") || content.contains("catch")
                        || content.contains("anyhow") || content.contains("thiserror")
                } else {
                    false
                }
            })
            .count() as f64 / cluster.graphs.len() as f64;
        score += has_error_handling * 2.0;
        
        // Signal 3: Public API surface (0-2 points)
        let is_public = cluster.graphs.iter()
            .filter(|g| {
                if let Ok(content) = std::fs::read_to_string(&g.module.path) {
                    content.contains("pub fn") || content.contains("pub struct")
                        || content.contains("export function") || content.contains("export class")
                        || content.contains("public class") || content.contains("public func")
                } else {
                    false
                }
            })
            .count() as f64 / cluster.graphs.len() as f64;
        score += is_public * 2.0;
        
        // Signal 4: Domain keywords (0-2 points)
        let criticality_score = self.keyword_dict.get_criticality_score(&cluster.key);
        score += (criticality_score as f64 / 10.0) * 2.0;
        
        // Signal 5: Not in utility/helper paths (0-2 points)
        let not_utility = cluster.graphs.iter()
            .filter(|g| {
                let path = g.module.path.to_lowercase();
                !path.contains("util") && !path.contains("helper") 
                    && !path.contains("common") && !path.contains("lib.rs")
            })
            .count() as f64 / cluster.graphs.len() as f64;
        score += not_utility * 2.0;
        
        // Convert score to criticality
        match score as i32 {
            8.. => Criticality::Critical,
            6..=7 => Criticality::High,
            4..=5 => Criticality::Medium,
            _ => Criticality::Low,
        }
    }
    
    fn generate_description(&self, key: &str, file_count: usize) -> String {
        format!("{} functionality ({} related files discovered)", 
            self.humanize_name(key), file_count)
    }
    
    fn infer_constraints(&self, analyses: &[ExplanationGraph]) -> TechnicalConstraints {
        // Detect platforms from code patterns
        let mut platforms = vec![];
        for analysis in analyses {
            if analysis.module.language == "rust" {
                platforms.extend(vec!["linux", "macos", "windows"]);
            }
        }
        platforms.sort();
        platforms.dedup();
        
        TechnicalConstraints {
            max_response_time_ms: None,
            max_memory_mb: None,
            supported_platforms: platforms.iter().map(|s| s.to_string()).collect(),
            minimum_test_coverage: None,
        }
    }
    
    fn detect_primary_language(&self, repo_path: &Path) -> Result<String> {
        // Check for common project files
        if repo_path.join("Cargo.toml").exists() {
            return Ok("rust".into());
        }
        if repo_path.join("package.json").exists() {
            return Ok("javascript".into());
        }
        if repo_path.join("go.mod").exists() {
            return Ok("go".into());
        }
        if repo_path.join("requirements.txt").exists() || repo_path.join("setup.py").exists() {
            return Ok("python".into());
        }
        
        Ok("unknown".into())
    }
    
    fn build_spec_from_discovery(
        &self,
        discovered: discovery::DiscoveredSpec,
        repo_path: &Path,
    ) -> ProductMetadata {
        let product_info = self.detect_product_info(repo_path);
        
        ProductMetadata {
            ctp_version: "1.0.0".into(),
            product: product_info,
            demographics: None,
            core_functionalities: discovered.functionalities,
            technical_constraints: Some(discovered.constraints),
            development_markers: None,
            drift_thresholds: Some(DriftThresholds {
                intent_similarity_threshold: 0.7,
                behavior_similarity_threshold: 0.7,
                documentation_staleness_days: 90,
            }),
            documentation: None,
            git_analysis: None,
            confidence: discovered.confidence,
            generation_method: GenerationMethod::RuleBased,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    #[cfg(feature = "llm")]
    async fn llm_enrich_spec(
        &self,
        discovered: discovery::DiscoveredSpec,
        _analyses: &[ExplanationGraph],
    ) -> Result<ProductMetadata> {
        // TODO: Implement LLM enrichment
        // For now, just return the discovered spec as-is
        warn!("LLM enrichment not yet implemented, returning rule-based spec");
        Ok(self.build_spec_from_discovery(discovered, &std::env::current_dir()?))
    }
    
    fn calculate_cluster_confidence(&self, graphs: &[&ExplanationGraph]) -> f64 {
        let mut score = 0.0;
        let mut weights_sum = 0.0;
        
        // Factor 1: File count (more files = higher confidence)
        let file_score = (graphs.len() as f64 / 10.0).min(1.0);
        score += file_score * 0.3;
        weights_sum += 0.3;
        
        // Factor 2: Intent clarity (check for TODO, stub, placeholder)
        let intent_clarity = graphs.iter()
            .filter(|g| {
                let intent = g.intent.inferred_intent.to_lowercase();
                !intent.contains("todo") && !intent.contains("stub") 
                    && !intent.contains("placeholder") && !intent.contains("unused")
                    && !intent.contains("fixme") && !intent.contains("hack")
            })
            .count() as f64 / graphs.len() as f64;
        score += intent_clarity * 0.25;
        weights_sum += 0.25;
        
        // Factor 3: Has entry points
        let has_entry_points = graphs.iter()
            .filter(|g| !g.behavior.entry_points.is_empty())
            .count() as f64 / graphs.len() as f64;
        score += has_entry_points * 0.25;
        weights_sum += 0.25;
        
        // Factor 4: Not in common generic paths
        let not_generic = graphs.iter()
            .filter(|g| {
                let path = g.module.path.to_lowercase();
                !path.contains("util") && !path.contains("helper") 
                    && !path.contains("common") && !path.contains("lib.rs")
            })
            .count() as f64 / graphs.len() as f64;
        score += not_generic * 0.2;
        weights_sum += 0.2;
        
        if weights_sum > 0.0 {
            score / weights_sum
        } else {
            0.5
        }
    }
    
    fn post_process_functionalities(
        &self,
        mut functionalities: Vec<CoreFunctionality>,
    ) -> Vec<CoreFunctionality> {
        use std::collections::HashSet;
        
        // Remove low-confidence entries
        functionalities.retain(|f| f.confidence >= 0.5);
        
        // Remove generic/invalid entries
        functionalities.retain(|f| {
            f.id.len() >= 3 
                && f.id != "#" 
                && f.id != "general"
                && !f.id.starts_with('_')
                && !f.description.contains("lib.rs")
                && f.name != "#"
                && f.name != "General"
        });
        
        // Deduplicate by file overlap
        let mut deduplicated = Vec::new();
        for func in functionalities {
            let func_files: HashSet<_> = func.entry_points.iter()
                .map(|ep| &ep.file)
                .collect();
            
            let is_duplicate = deduplicated.iter().any(|existing: &CoreFunctionality| {
                let existing_files: HashSet<_> = existing.entry_points.iter()
                    .map(|ep| &ep.file)
                    .collect();
                
                let intersection = func_files.intersection(&existing_files).count();
                let union = func_files.union(&existing_files).count();
                
                if union == 0 {
                    false
                } else {
                    let overlap = intersection as f64 / union as f64;
                    overlap > 0.7
                }
            });
            
            if !is_duplicate {
                deduplicated.push(func);
            }
        }
        
        deduplicated
    }
    
    fn detect_product_info(&self, repo_path: &Path) -> ProductInfo {
        // Try Cargo.toml first
        if let Ok(content) = std::fs::read_to_string(repo_path.join("Cargo.toml")) {
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                if let Some(package) = toml_value.get("package") {
                    let name = package.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    let version = package.get("version")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    
                    return ProductInfo {
                        name,
                        version,
                        product_type: self.infer_product_type(repo_path),
                        primary_language: "rust".into(),
                        natural_language: "en".into(),
                        supported_languages: vec!["en".into()],
                        currency: None,
                    };
                }
            }
        }
        
        // Try package.json
        if let Ok(content) = std::fs::read_to_string(repo_path.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let name = json.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let version = json.get("version")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                return ProductInfo {
                    name,
                    version,
                    product_type: self.infer_product_type(repo_path),
                    primary_language: "javascript".into(),
                    natural_language: "en".into(),
                    supported_languages: vec!["en".into()],
                    currency: None,
                };
            }
        }
        
        // Fallback to directory name
        let name = repo_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        ProductInfo {
            name,
            version: None,
            product_type: self.infer_product_type(repo_path),
            primary_language: self.detect_primary_language(repo_path).unwrap_or("unknown".into()),
            natural_language: "en".into(),
            supported_languages: vec!["en".into()],
            currency: None,
        }
    }
    
    fn infer_product_type(&self, repo_path: &Path) -> String {
        // Check for CLI indicators
        if repo_path.join("Cargo.toml").exists() {
            if let Ok(content) = std::fs::read_to_string(repo_path.join("Cargo.toml")) {
                if content.contains("[[bin]]") || content.contains("name = \"ctp-cli\"") {
                    return "cli_tool".into();
                }
            }
        }
        
        // Check for web indicators
        if repo_path.join("package.json").exists() {
            if let Ok(content) = std::fs::read_to_string(repo_path.join("package.json")) {
                if content.contains("\"react\"") || content.contains("\"next\"") 
                    || content.contains("\"vue\"") || content.contains("\"angular\"") {
                    return "web_app".into();
                }
                if content.contains("\"express\"") || content.contains("\"fastify\"") {
                    return "api_service".into();
                }
            }
        }
        
        // Check for mobile indicators
        if repo_path.join("android").exists() || repo_path.join("ios").exists() {
            return "mobile_app".into();
        }
        
        "library".into()
    }
}

impl Default for SpecGenerator {
    fn default() -> Self {
        Self::new()
    }
}
