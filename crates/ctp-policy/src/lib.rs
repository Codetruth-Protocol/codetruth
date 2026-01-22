//! # CTP Policy Engine
//!
//! Policy evaluation engine for CodeTruth Protocol.
//!
//! This crate provides policy definition parsing and evaluation
//! for enforcing code governance rules.

use anyhow::Result;
use glob::Pattern;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Policy Definition Language (PDL) structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub ctp_version: String,
    pub policy_schema_version: String,
    pub policy: Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: PolicyScope,
    pub severity: PolicySeverity,
    pub rules: Vec<PolicyRule>,
    #[serde(default)]
    pub enforcement: Option<Enforcement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyScope {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub rule_id: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    #[serde(default)]
    pub requires: Option<Vec<RuleRequirement>>,
    #[serde(default)]
    pub violation_message: String,
    #[serde(default)]
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRequirement {
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub must_exist: Option<bool>,
    #[serde(default)]
    pub must_have_doc: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enforcement {
    #[serde(default)]
    pub block_merge: bool,
    #[serde(default)]
    pub notify: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub policy_id: String,
    pub policy_name: String,
    pub status: PolicyStatus,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyStatus {
    Pass,
    Fail,
    Warning,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub severity: PolicySeverity,
    pub message: String,
    pub file: String,
    pub line: Option<usize>,
    pub evidence: String,
}

/// Policy evaluation engine
pub struct PolicyEngine {
    policies: Vec<PolicyDefinition>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self { policies: vec![] }
    }

    /// Load a policy from YAML string
    pub fn load_policy_from_str(&mut self, content: &str) -> Result<()> {
        let policy: PolicyDefinition = serde_yaml::from_str(content)?;
        debug!("Loaded policy: {}", policy.policy.name);
        self.policies.push(policy);
        Ok(())
    }

    /// Get the number of loaded policies
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }

    /// Evaluate all policies against a file
    pub fn evaluate(&self, file_path: &str, content: &str) -> Vec<PolicyResult> {
        self.policies
            .iter()
            .map(|p| self.evaluate_policy(&p.policy, file_path, content))
            .collect()
    }

    fn evaluate_policy(&self, policy: &Policy, file_path: &str, content: &str) -> PolicyResult {
        // Check if file is in scope
        if !self.file_in_scope(file_path, &policy.scope) {
            return PolicyResult {
                policy_id: policy.id.clone(),
                policy_name: policy.name.clone(),
                status: PolicyStatus::Skip,
                violations: vec![],
            };
        }

        let mut violations = vec![];

        // Evaluate each rule
        for rule in &policy.rules {
            if let Some(rule_violations) = self.evaluate_rule(rule, policy, file_path, content) {
                violations.extend(rule_violations);
            }
        }

        let status = if violations.is_empty() {
            PolicyStatus::Pass
        } else if violations.iter().any(|v| v.severity == PolicySeverity::Critical || v.severity == PolicySeverity::Error) {
            PolicyStatus::Fail
        } else {
            PolicyStatus::Warning
        };

        PolicyResult {
            policy_id: policy.id.clone(),
            policy_name: policy.name.clone(),
            status,
            violations,
        }
    }

    fn evaluate_rule(
        &self,
        rule: &PolicyRule,
        policy: &Policy,
        file_path: &str,
        content: &str,
    ) -> Option<Vec<Violation>> {
        let mut violations = vec![];

        match rule.rule_type.as_str() {
            "documentation" => {
                violations.extend(self.check_documentation_rule(rule, policy, file_path, content));
            }
            "behavior_pattern" => {
                violations.extend(self.check_behavior_pattern_rule(rule, policy, file_path, content));
            }
            "security" => {
                violations.extend(self.check_security_rule(rule, policy, file_path, content));
            }
            _ => {
                // Unknown rule type, skip
                debug!("Unknown rule type: {}", rule.rule_type);
            }
        }

        if violations.is_empty() {
            None
        } else {
            Some(violations)
        }
    }

    /// Check documentation requirements
    fn check_documentation_rule(
        &self,
        rule: &PolicyRule,
        policy: &Policy,
        file_path: &str,
        content: &str,
    ) -> Vec<Violation> {
        let mut violations = vec![];
        let lines: Vec<&str> = content.lines().collect();

        if let Some(requirements) = &rule.requires {
            for req in requirements {
                if let Some(pattern) = &req.pattern {
                    // Find lines matching the pattern (e.g., function definitions)
                    for (line_num, line) in lines.iter().enumerate() {
                        if line.contains(pattern) {
                            // Check if documentation exists
                            if req.must_have_doc.unwrap_or(false) {
                                let has_doc = self.has_documentation(&lines, line_num);
                                if !has_doc {
                                    violations.push(Violation {
                                        rule_id: rule.rule_id.clone(),
                                        severity: policy.severity,
                                        message: rule.violation_message.clone(),
                                        file: file_path.to_string(),
                                        line: Some(line_num + 1),
                                        evidence: line.trim().to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        violations
    }

    /// Check behavior pattern requirements (e.g., idempotency for payment retries)
    fn check_behavior_pattern_rule(
        &self,
        rule: &PolicyRule,
        policy: &Policy,
        file_path: &str,
        content: &str,
    ) -> Vec<Violation> {
        let mut violations = vec![];
        let content_lower = content.to_lowercase();

        if let Some(requirements) = &rule.requires {
            let mut context_matched = false;
            let mut required_patterns_found = vec![];

            for req in requirements {
                if let Some(pattern) = &req.pattern {
                    // Check if this is a context pattern
                    if let Some(context) = &req.context {
                        // Check if context matches
                        let context_parts: Vec<&str> = context.split('|').collect();
                        context_matched = context_parts.iter().any(|c| content_lower.contains(c));
                        
                        if context_matched && content_lower.contains(pattern) {
                            // Context and pattern both match - this triggers the rule
                        }
                    } else if req.must_exist.unwrap_or(false) {
                        // This pattern must exist
                        let pattern_regex = Regex::new(pattern).ok();
                        let found = pattern_regex
                            .map(|re| re.is_match(&content_lower))
                            .unwrap_or_else(|| content_lower.contains(pattern));
                        required_patterns_found.push((pattern.clone(), found));
                    }
                }
            }

            // If context matched but required patterns are missing, it's a violation
            if context_matched {
                for (pattern, found) in &required_patterns_found {
                    if !found {
                        violations.push(Violation {
                            rule_id: rule.rule_id.clone(),
                            severity: policy.severity,
                            message: rule.violation_message.clone(),
                            file: file_path.to_string(),
                            line: None,
                            evidence: format!("Missing required pattern: {}", pattern),
                        });
                    }
                }
            }
        }

        violations
    }

    /// Check security-related rules
    fn check_security_rule(
        &self,
        rule: &PolicyRule,
        policy: &Policy,
        file_path: &str,
        content: &str,
    ) -> Vec<Violation> {
        let mut violations = vec![];
        let lines: Vec<&str> = content.lines().collect();

        // Common security patterns to check
        let dangerous_patterns = [
            ("eval(", "Use of eval() is dangerous"),
            ("exec(", "Use of exec() is dangerous"),
            ("shell=True", "shell=True in subprocess is dangerous"),
            ("dangerouslySetInnerHTML", "XSS vulnerability risk"),
            ("innerHTML", "Potential XSS vulnerability"),
            ("password", "Potential hardcoded credential"),
            ("api_key", "Potential hardcoded API key"),
            ("secret", "Potential hardcoded secret"),
        ];

        for (line_num, line) in lines.iter().enumerate() {
            let line_lower = line.to_lowercase();
            for (pattern, message) in &dangerous_patterns {
                if line_lower.contains(&pattern.to_lowercase()) {
                    // Skip if it's in a comment
                    let trimmed = line.trim();
                    if trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.starts_with('*') {
                        continue;
                    }

                    violations.push(Violation {
                        rule_id: rule.rule_id.clone(),
                        severity: policy.severity,
                        message: message.to_string(),
                        file: file_path.to_string(),
                        line: Some(line_num + 1),
                        evidence: line.trim().to_string(),
                    });
                }
            }
        }

        violations
    }

    /// Check if a line has documentation (docstring or comment above it)
    fn has_documentation(&self, lines: &[&str], line_num: usize) -> bool {
        if line_num == 0 {
            return false;
        }

        // Check the line immediately before
        let prev_line = lines[line_num - 1].trim();
        
        // Python docstring on same line or comment above
        if prev_line.starts_with('#') || prev_line.starts_with("\"\"\"") || prev_line.starts_with("'''") {
            return true;
        }

        // JSDoc or block comment
        if prev_line.ends_with("*/") || prev_line.starts_with("//") {
            return true;
        }

        // Rust doc comment
        if prev_line.starts_with("///") || prev_line.starts_with("//!") {
            return true;
        }

        // Check for docstring on the next line (Python style)
        if line_num + 1 < lines.len() {
            let next_line = lines[line_num + 1].trim();
            if next_line.starts_with("\"\"\"") || next_line.starts_with("'''") {
                return true;
            }
        }

        false
    }

    fn file_in_scope(&self, file_path: &str, scope: &PolicyScope) -> bool {
        // Check exclusions first
        for exclude in &scope.exclude {
            if let Ok(pattern) = Pattern::new(exclude) {
                if pattern.matches(file_path) {
                    return false;
                }
            }
        }

        // If no includes specified, everything is in scope
        if scope.include.is_empty() {
            return true;
        }

        // Check inclusions
        for include in &scope.include {
            if let Ok(pattern) = Pattern::new(include) {
                if pattern.matches(file_path) {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_policy() {
        let mut engine = PolicyEngine::new();
        
        let policy_yaml = r#"
ctp_version: "1.0.0"
policy_schema_version: "1.0.0"
policy:
  id: "doc-required"
  name: "Documentation Required"
  description: "Functions must have documentation"
  scope:
    include:
      - "**/*.py"
  severity: "WARNING"
  rules:
    - rule_id: "docstring-required"
      type: "documentation"
      requires:
        - pattern: "def "
          must_have_doc: true
      violation_message: "Function missing documentation"
"#;

        engine.load_policy_from_str(policy_yaml).unwrap();
        
        let code = r#"
def undocumented_function():
    pass

# This function has a comment
def documented_function():
    pass
"#;

        let results = engine.evaluate("test.py", code);
        assert_eq!(results.len(), 1);
        assert!(!results[0].violations.is_empty());
    }

    #[test]
    fn test_file_scope() {
        let engine = PolicyEngine::new();
        
        let scope = PolicyScope {
            include: vec!["**/*.py".to_string()],
            exclude: vec!["**/*_test.py".to_string()],
        };

        assert!(engine.file_in_scope("src/main.py", &scope));
        assert!(!engine.file_in_scope("src/main_test.py", &scope));
        assert!(!engine.file_in_scope("src/main.js", &scope));
    }
}
