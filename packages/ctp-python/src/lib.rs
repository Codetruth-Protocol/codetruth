//! PyO3 bindings for CodeTruth Protocol
//!
//! This module provides Python bindings for the CTP core analysis engine,
//! enabling high-performance code analysis from Python.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

/// Rust-powered analyzer for CodeTruth Protocol
#[pyclass]
pub struct RustAnalyzer {
    // Internal state
}

#[pymethods]
impl RustAnalyzer {
    #[new]
    fn new() -> Self {
        RustAnalyzer {}
    }

    /// Analyze code and return a minimal analysis result
    fn analyze_code(&self, code: &str, language: &str) -> PyResult<MinimalAnalysisResult> {
        let hash = self.hash_code(code);
        let intent = self.extract_intent(code, language);
        let behavior = self.analyze_behavior(code, language);
        let drift = self.detect_drift(&intent, &behavior);
        let confidence = if intent.is_empty() { 0.5 } else { 0.8 };

        Ok(MinimalAnalysisResult {
            ctp_version: "1.0.0".to_string(),
            file_hash: format!("sha256:{}", hash),
            intent: if intent.is_empty() { "No declared intent".to_string() } else { intent },
            behavior,
            drift,
            confidence,
        })
    }

    /// Analyze a file and return full explanation graph as dict
    fn analyze_file(&self, py: Python<'_>, path: &str) -> PyResult<PyObject> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        let language = self.detect_language(path);
        let analysis = self.analyze_code(&content, &language)?;

        // Build full explanation graph dict
        let dict = PyDict::new(py);
        dict.set_item("ctp_version", &analysis.ctp_version)?;
        dict.set_item("explanation_id", &analysis.file_hash)?;

        // Module info
        let module = PyDict::new(py);
        module.set_item("name", std::path::Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("unknown"))?;
        module.set_item("path", path)?;
        module.set_item("language", &language)?;
        module.set_item("lines_of_code", content.lines().count())?;
        module.set_item("complexity_score", self.calculate_complexity(&content, &language))?;
        dict.set_item("module", module)?;

        // Intent
        let intent_dict = PyDict::new(py);
        let declared = self.extract_intent(&content, &language);
        intent_dict.set_item("declared_intent", &declared)?;
        intent_dict.set_item("inferred_intent", &analysis.intent)?;
        intent_dict.set_item("confidence", analysis.confidence)?;
        intent_dict.set_item("business_context", "")?;
        intent_dict.set_item("technical_rationale", "")?;
        dict.set_item("intent", intent_dict)?;

        // Behavior
        let behavior_dict = PyDict::new(py);
        behavior_dict.set_item("actual_behavior", &analysis.behavior)?;
        behavior_dict.set_item("entry_points", Vec::<String>::new())?;
        behavior_dict.set_item("exit_points", Vec::<String>::new())?;
        behavior_dict.set_item("side_effects", self.get_side_effects(&content))?;
        behavior_dict.set_item("dependencies", Vec::<String>::new())?;
        dict.set_item("behavior", behavior_dict)?;

        // Drift
        let drift_dict = PyDict::new(py);
        drift_dict.set_item("drift_detected", analysis.drift != "NONE")?;
        drift_dict.set_item("drift_severity", &analysis.drift)?;
        drift_dict.set_item("drift_details", Vec::<String>::new())?;
        dict.set_item("drift", drift_dict)?;

        // Policies (empty for now)
        let policies_dict = PyDict::new(py);
        policies_dict.set_item("evaluated_at", "")?;
        policies_dict.set_item("policy_results", Vec::<String>::new())?;
        dict.set_item("policies", policies_dict)?;

        // History
        let history_dict = PyDict::new(py);
        history_dict.set_item("previous_versions", Vec::<String>::new())?;
        let evolution = PyDict::new(py);
        evolution.set_item("created_at", "")?;
        evolution.set_item("last_modified", "")?;
        evolution.set_item("modification_count", 0)?;
        evolution.set_item("stability_score", 1.0)?;
        history_dict.set_item("evolution", evolution)?;
        dict.set_item("history", history_dict)?;

        // Metadata
        let metadata_dict = PyDict::new(py);
        metadata_dict.set_item("generated_at", "")?;
        let generator = PyDict::new(py);
        generator.set_item("name", "CodeTruth")?;
        generator.set_item("version", "0.1.0")?;
        generator.set_item("llm_provider", pyo3::types::PyNone::get(py))?;
        generator.set_item("llm_model", pyo3::types::PyNone::get(py))?;
        metadata_dict.set_item("generator", generator)?;
        metadata_dict.set_item("extensions", PyDict::new(py))?;
        dict.set_item("metadata", metadata_dict)?;

        Ok(dict.into())
    }

    /// Calculate cyclomatic complexity
    fn calculate_complexity(&self, code: &str, language: &str) -> f64 {
        let mut complexity = 1.0;

        // Count decision points
        let decision_keywords = match language {
            "python" => vec!["if ", "elif ", "for ", "while ", "except ", "and ", "or "],
            "javascript" | "typescript" => vec!["if ", "else if ", "for ", "while ", "catch ", "&&", "||", "?"],
            "rust" => vec!["if ", "else if ", "for ", "while ", "match ", "&&", "||"],
            _ => vec!["if ", "for ", "while "],
        };

        for keyword in decision_keywords {
            complexity += code.matches(keyword).count() as f64;
        }

        // Normalize by function count
        let func_count = self.count_functions(code, language).max(1) as f64;
        (complexity / func_count).min(10.0)
    }

    fn count_functions(&self, code: &str, language: &str) -> usize {
        match language {
            "python" => code.matches("def ").count(),
            "javascript" | "typescript" => code.matches("function ").count() + code.matches("=> ").count(),
            "rust" => code.matches("fn ").count(),
            _ => 1,
        }
    }
}

impl RustAnalyzer {
    fn hash_code(&self, code: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn detect_language(&self, path: &str) -> String {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "py" => "python",
            "js" | "mjs" => "javascript",
            "ts" | "tsx" => "typescript",
            "rs" => "rust",
            "go" => "go",
            "java" => "java",
            _ => "unknown",
        }.to_string()
    }

    fn extract_intent(&self, code: &str, language: &str) -> String {
        match language {
            "python" => {
                if let Some(start) = code.find("\"\"\"") {
                    if let Some(end) = code[start + 3..].find("\"\"\"") {
                        return code[start + 3..start + 3 + end].trim().chars().take(280).collect();
                    }
                }
            }
            "javascript" | "typescript" => {
                if let Some(start) = code.find("/**") {
                    if let Some(end) = code[start..].find("*/") {
                        let comment = &code[start + 3..start + end];
                        return comment
                            .lines()
                            .map(|l| l.trim().trim_start_matches('*').trim())
                            .collect::<Vec<_>>()
                            .join(" ")
                            .chars()
                            .take(280)
                            .collect();
                    }
                }
            }
            "rust" => {
                let doc_lines: Vec<&str> = code
                    .lines()
                    .take_while(|l| l.trim().starts_with("///") || l.trim().starts_with("//!"))
                    .map(|l| l.trim().trim_start_matches("///").trim_start_matches("//!").trim())
                    .collect();
                if !doc_lines.is_empty() {
                    return doc_lines.join(" ").chars().take(280).collect();
                }
            }
            _ => {}
        }
        String::new()
    }

    fn analyze_behavior(&self, code: &str, language: &str) -> String {
        let mut parts = vec![];

        let func_count = self.count_functions(code, language);
        if func_count > 0 {
            parts.push(format!("{} function(s)", func_count));
        }

        let io_patterns = ["open(", "read(", "write(", "fetch(", "fs."];
        if io_patterns.iter().any(|p| code.contains(p)) {
            parts.push("file/network I/O".to_string());
        }

        let db_patterns = ["SELECT ", "INSERT ", "UPDATE ", "DELETE ", "mongodb", "prisma"];
        if db_patterns.iter().any(|p| code.to_uppercase().contains(&p.to_uppercase())) {
            parts.push("database operations".to_string());
        }

        if parts.is_empty() {
            "Simple logic".to_string()
        } else {
            format!("Performs {}", parts.join(", "))
        }
    }

    fn detect_drift(&self, intent: &str, behavior: &str) -> String {
        if intent.is_empty() {
            return "LOW".to_string();
        }

        let intent_words: std::collections::HashSet<&str> = intent
            .to_lowercase()
            .split_whitespace()
            .collect();
        let behavior_words: std::collections::HashSet<&str> = behavior
            .to_lowercase()
            .split_whitespace()
            .collect();

        let intersection = intent_words.intersection(&behavior_words).count();
        let union = intent_words.union(&behavior_words).count();

        let similarity = if union > 0 { intersection as f64 / union as f64 } else { 0.0 };

        match similarity {
            x if x >= 0.7 => "NONE",
            x if x >= 0.5 => "LOW",
            x if x >= 0.3 => "MEDIUM",
            _ => "HIGH",
        }.to_string()
    }

    fn get_side_effects(&self, code: &str) -> Vec<HashMap<String, String>> {
        let mut effects = vec![];

        let io_patterns = ["open(", "read(", "write(", "fetch(", "fs."];
        if io_patterns.iter().any(|p| code.contains(p)) {
            let mut effect = HashMap::new();
            effect.insert("effect_type".to_string(), "io".to_string());
            effect.insert("description".to_string(), "File/network operations".to_string());
            effect.insert("risk_level".to_string(), "MEDIUM".to_string());
            effects.push(effect);
        }

        let db_patterns = ["SELECT ", "INSERT ", "UPDATE ", "DELETE "];
        if db_patterns.iter().any(|p| code.to_uppercase().contains(p)) {
            let mut effect = HashMap::new();
            effect.insert("effect_type".to_string(), "database".to_string());
            effect.insert("description".to_string(), "Database operations".to_string());
            effect.insert("risk_level".to_string(), "HIGH".to_string());
            effects.push(effect);
        }

        effects
    }
}

/// Minimal analysis result returned from Rust
#[pyclass]
#[derive(Clone)]
pub struct MinimalAnalysisResult {
    #[pyo3(get)]
    pub ctp_version: String,
    #[pyo3(get)]
    pub file_hash: String,
    #[pyo3(get)]
    pub intent: String,
    #[pyo3(get)]
    pub behavior: String,
    #[pyo3(get)]
    pub drift: String,
    #[pyo3(get)]
    pub confidence: f64,
}

#[pymethods]
impl MinimalAnalysisResult {
    fn __repr__(&self) -> String {
        format!(
            "MinimalAnalysisResult(drift={}, confidence={:.2})",
            self.drift, self.confidence
        )
    }

    fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("ctp_version", &self.ctp_version)?;
        dict.set_item("file_hash", &self.file_hash)?;
        dict.set_item("intent", &self.intent)?;
        dict.set_item("behavior", &self.behavior)?;
        dict.set_item("drift", &self.drift)?;
        dict.set_item("confidence", self.confidence)?;
        Ok(dict.into())
    }
}

/// Python module definition
#[pymodule]
fn _core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<RustAnalyzer>()?;
    m.add_class::<MinimalAnalysisResult>()?;
    m.add("__version__", "0.1.0")?;
    m.add("CTP_VERSION", "1.0.0")?;
    Ok(())
}
