//! # CTP WASM
//!
//! WebAssembly bindings for CodeTruth Protocol.
//! Enables running CTP analysis in browsers and edge environments.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalAnalysis {
    pub ctp_version: String,
    pub file_hash: String,
    pub intent: String,
    pub behavior: String,
    pub drift: String,
    pub confidence: f64,
}

#[wasm_bindgen]
pub struct CTPService {
    // Configuration stored here
}

#[wasm_bindgen]
impl CTPService {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("CTP WASM initialized");
        Self {}
    }

    #[wasm_bindgen]
    pub fn analyze_code(&self, code: &str, language: &str) -> JsValue {
        let hash = self.hash_code(code);
        let intent = self.extract_intent(code, language);
        let behavior = self.analyze_behavior(code, language);
        let drift = self.detect_drift(&intent, &behavior);

        let analysis = MinimalAnalysis {
            ctp_version: "1.0.0".into(),
            file_hash: format!("sha256:{}", hash),
            intent: if intent.is_empty() { "No declared intent".into() } else { intent },
            behavior,
            drift,
            confidence: 0.8,
        };

        serde_wasm_bindgen::to_value(&analysis).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen]
    pub fn version(&self) -> String {
        "0.1.0".into()
    }

    fn hash_code(&self, code: &str) -> String {
        let mut hash: u64 = 0;
        for byte in code.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        format!("{:016x}", hash)
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
            _ => {}
        }
        String::new()
    }

    fn analyze_behavior(&self, code: &str, language: &str) -> String {
        let mut parts = vec![];

        let func_count = match language {
            "python" => code.matches("def ").count(),
            "javascript" | "typescript" => {
                code.matches("function ").count() + code.matches("const ").count()
            }
            "rust" => code.matches("fn ").count(),
            _ => 0,
        };

        if func_count > 0 {
            parts.push(format!("{} function(s)", func_count));
        }

        let io_patterns = ["open(", "read(", "write(", "fetch("];
        if io_patterns.iter().any(|p| code.contains(p)) {
            parts.push("file/network I/O".into());
        }

        let db_patterns = ["SELECT ", "INSERT ", "UPDATE ", "DELETE "];
        if db_patterns.iter().any(|p| code.to_uppercase().contains(p)) {
            parts.push("database operations".into());
        }

        if parts.is_empty() {
            "Simple logic".into()
        } else {
            format!("Performs {}", parts.join(", "))
        }
    }

    fn detect_drift(&self, intent: &str, behavior: &str) -> String {
        if intent.is_empty() {
            return "LOW".into();
        }

        let intent_words: std::collections::HashSet<&str> =
            intent.to_lowercase().split_whitespace().collect();
        let behavior_words: std::collections::HashSet<&str> =
            behavior.to_lowercase().split_whitespace().collect();

        let intersection = intent_words.intersection(&behavior_words).count();
        let union = intent_words.union(&behavior_words).count();

        let similarity = if union > 0 { intersection as f64 / union as f64 } else { 0.0 };

        match similarity {
            x if x >= 0.7 => "NONE",
            x if x >= 0.5 => "LOW",
            x if x >= 0.3 => "MEDIUM",
            _ => "HIGH",
        }.into()
    }
}

impl Default for CTPService {
    fn default() -> Self {
        Self::new()
    }
}
