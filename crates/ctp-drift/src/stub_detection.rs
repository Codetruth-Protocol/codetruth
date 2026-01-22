//! Multilingual stub and placeholder detection
//!
//! Detects incomplete code markers across multiple natural languages

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubPattern {
    pub pattern: String,
    pub language: String,
    pub severity: StubSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StubSeverity {
    Critical,  // Must fix before production
    High,      // Should fix soon
    Medium,    // Should address
    Low,       // Nice to clean up
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubFinding {
    pub pattern_matched: String,
    pub severity: StubSeverity,
    pub language: String,
    pub line: usize,
    pub column: usize,
    pub context: String,
    pub suggestion: String,
}

pub struct StubDetector {
    patterns: Vec<CompiledPattern>,
}

struct CompiledPattern {
    regex: Regex,
    language: String,
    severity: StubSeverity,
    description: String,
}

impl StubDetector {
    /// Create detector with default multilingual patterns
    pub fn new() -> Self {
        Self::with_patterns(Self::default_patterns())
    }

    /// Create detector with custom patterns
    pub fn with_patterns(patterns: Vec<StubPattern>) -> Self {
        let compiled = patterns
            .into_iter()
            .filter_map(|p| {
                Regex::new(&p.pattern).ok().map(|regex| CompiledPattern {
                    regex,
                    language: p.language,
                    severity: p.severity,
                    description: p.description,
                })
            })
            .collect();

        Self { patterns: compiled }
    }

    /// Default multilingual stub patterns
    pub fn default_patterns() -> Vec<StubPattern> {
        vec![
            // English - Critical (production blockers)
            StubPattern {
                pattern: r"(?i)\b(implement.*in\s+production|replace.*before\s+prod|change.*before\s+deploy)\b".into(),
                language: "en".into(),
                severity: StubSeverity::Critical,
                description: "Production blocker comment".into(),
            },
            StubPattern {
                pattern: r"(?i)\b(not\s+for\s+production|do\s+not\s+use\s+in\s+prod|temporary\s+hack)\b".into(),
                language: "en".into(),
                severity: StubSeverity::Critical,
                description: "Temporary code warning".into(),
            },
            
            // English - High
            StubPattern {
                pattern: r"(?i)\b(placeholder|stub\s+implementation|mock\s+data)\b".into(),
                language: "en".into(),
                severity: StubSeverity::High,
                description: "Placeholder implementation".into(),
            },
            StubPattern {
                pattern: r"(?i)\b(replace\s+this|change\s+this|update\s+this)\b".into(),
                language: "en".into(),
                severity: StubSeverity::High,
                description: "Needs replacement".into(),
            },
            
            // English - Medium
            StubPattern {
                pattern: r"\b(TODO|FIXME|HACK|XXX|BUG|BROKEN)\b".into(),
                language: "en".into(),
                severity: StubSeverity::Medium,
                description: "Standard development marker".into(),
            },
            StubPattern {
                pattern: r"(?i)\b(temporary|temp\s+code|quick\s+fix)\b".into(),
                language: "en".into(),
                severity: StubSeverity::Medium,
                description: "Temporary solution".into(),
            },
            
            // Spanish
            StubPattern {
                pattern: r"(?i)\b(por\s+hacer|arreglar|pendiente|temporal)\b".into(),
                language: "es".into(),
                severity: StubSeverity::Medium,
                description: "Spanish TODO/temporary marker".into(),
            },
            StubPattern {
                pattern: r"(?i)\b(implementar.*producción|cambiar.*antes)\b".into(),
                language: "es".into(),
                severity: StubSeverity::Critical,
                description: "Spanish production blocker".into(),
            },
            
            // French
            StubPattern {
                pattern: r"(?i)\b(à\s+faire|corriger|en\s+attente|temporaire)\b".into(),
                language: "fr".into(),
                severity: StubSeverity::Medium,
                description: "French TODO/temporary marker".into(),
            },
            StubPattern {
                pattern: r"(?i)\b(implémenter.*production|changer.*avant)\b".into(),
                language: "fr".into(),
                severity: StubSeverity::Critical,
                description: "French production blocker".into(),
            },
            
            // German
            StubPattern {
                pattern: r"(?i)\b(zu\s+erledigen|beheben|ausstehend|temporär)\b".into(),
                language: "de".into(),
                severity: StubSeverity::Medium,
                description: "German TODO/temporary marker".into(),
            },
            StubPattern {
                pattern: r"(?i)\b(implementieren.*produktion|ändern.*vor)\b".into(),
                language: "de".into(),
                severity: StubSeverity::Critical,
                description: "German production blocker".into(),
            },
            
            // Chinese (Simplified)
            StubPattern {
                pattern: r"(待办|修复|临时|待定)".into(),
                language: "zh".into(),
                severity: StubSeverity::Medium,
                description: "Chinese TODO/temporary marker".into(),
            },
            StubPattern {
                pattern: r"(生产.*实现|部署.*更改)".into(),
                language: "zh".into(),
                severity: StubSeverity::Critical,
                description: "Chinese production blocker".into(),
            },
            
            // Japanese
            StubPattern {
                pattern: r"(やること|修正|一時的|保留)".into(),
                language: "ja".into(),
                severity: StubSeverity::Medium,
                description: "Japanese TODO/temporary marker".into(),
            },
            StubPattern {
                pattern: r"(本番.*実装|デプロイ.*変更)".into(),
                language: "ja".into(),
                severity: StubSeverity::Critical,
                description: "Japanese production blocker".into(),
            },
            
            // Portuguese
            StubPattern {
                pattern: r"(?i)\b(a\s+fazer|corrigir|pendente|temporário)\b".into(),
                language: "pt".into(),
                severity: StubSeverity::Medium,
                description: "Portuguese TODO/temporary marker".into(),
            },
            
            // Russian
            StubPattern {
                pattern: r"(сделать|исправить|временно|ожидание)".into(),
                language: "ru".into(),
                severity: StubSeverity::Medium,
                description: "Russian TODO/temporary marker".into(),
            },
            
            // Code-level patterns (language-agnostic)
            StubPattern {
                pattern: r"(?i)unimplemented!\(|todo!\(|unreachable!\(|panic!\(.*not\s+implemented".into(),
                language: "code".into(),
                severity: StubSeverity::High,
                description: "Rust unimplemented macro".into(),
            },
            StubPattern {
                pattern: r"(?i)raise\s+NotImplementedError|pass\s*#.*implement".into(),
                language: "code".into(),
                severity: StubSeverity::High,
                description: "Python not implemented".into(),
            },
            StubPattern {
                pattern: r#"(?i)throw\s+new\s+Error\(['"]not\s+implemented"#.into(),
                language: "code".into(),
                severity: StubSeverity::High,
                description: "JavaScript not implemented".into(),
            },
        ]
    }

    /// Detect stubs in source code
    pub fn detect(&self, content: &str, file_path: &str) -> Vec<StubFinding> {
        let mut findings = vec![];

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if let Some(mat) = pattern.regex.find(line) {
                    let context = self.extract_context(content, line_num, 2);
                    
                    findings.push(StubFinding {
                        pattern_matched: mat.as_str().to_string(),
                        severity: pattern.severity,
                        language: pattern.language.clone(),
                        line: line_num + 1,
                        column: mat.start() + 1,
                        context,
                        suggestion: self.generate_suggestion(&pattern.description, file_path),
                    });
                }
            }
        }

        findings
    }

    /// Extract context around a line
    fn extract_context(&self, content: &str, line_num: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = line_num.saturating_sub(context_lines);
        let end = (line_num + context_lines + 1).min(lines.len());
        
        lines[start..end].join("\n")
    }

    /// Generate suggestion based on finding
    fn generate_suggestion(&self, description: &str, file_path: &str) -> String {
        if description.contains("production blocker") {
            format!("CRITICAL: Complete implementation before deploying to production ({})", file_path)
        } else if description.contains("Placeholder") {
            format!("Replace placeholder with actual implementation in {}", file_path)
        } else if description.contains("TODO") {
            format!("Address TODO item in {}", file_path)
        } else {
            format!("Review and resolve marker in {}", file_path)
        }
    }

    /// Get statistics on stub findings
    pub fn analyze_findings(&self, findings: &[StubFinding]) -> StubStatistics {
        let mut by_severity = HashMap::new();
        let mut by_language = HashMap::new();

        for finding in findings {
            *by_severity.entry(finding.severity).or_insert(0) += 1;
            *by_language.entry(finding.language.clone()).or_insert(0) += 1;
        }

        StubStatistics {
            total_count: findings.len(),
            critical_count: *by_severity.get(&StubSeverity::Critical).unwrap_or(&0),
            high_count: *by_severity.get(&StubSeverity::High).unwrap_or(&0),
            medium_count: *by_severity.get(&StubSeverity::Medium).unwrap_or(&0),
            low_count: *by_severity.get(&StubSeverity::Low).unwrap_or(&0),
            by_language,
        }
    }
}

impl Default for StubDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubStatistics {
    pub total_count: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub by_language: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_todo() {
        let detector = StubDetector::new();
        let code = "// TODO: implement this function\nfn test() {}";
        let findings = detector.detect(code, "test.rs");
        
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, StubSeverity::Medium);
        assert_eq!(findings[0].language, "en");
    }

    #[test]
    fn test_production_blocker() {
        let detector = StubDetector::new();
        let code = "// Replace this before production deployment";
        let findings = detector.detect(code, "test.rs");
        
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, StubSeverity::Critical);
    }

    #[test]
    fn test_spanish_marker() {
        let detector = StubDetector::new();
        let code = "// Por hacer: implementar validación";
        let findings = detector.detect(code, "test.rs");
        
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].language, "es");
    }

    #[test]
    fn test_rust_unimplemented() {
        let detector = StubDetector::new();
        let code = "fn test() { unimplemented!() }";
        let findings = detector.detect(code, "test.rs");
        
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, StubSeverity::High);
    }

    #[test]
    fn test_statistics() {
        let detector = StubDetector::new();
        let code = r#"
// TODO: fix this
// FIXME: broken
// Replace before production
// Por hacer: implementar
"#;
        let findings = detector.detect(code, "test.rs");
        let stats = detector.analyze_findings(&findings);
        
        assert_eq!(stats.total_count, 4);
        assert_eq!(stats.critical_count, 1);
        assert!(stats.by_language.contains_key("en"));
        assert!(stats.by_language.contains_key("es"));
    }
}
