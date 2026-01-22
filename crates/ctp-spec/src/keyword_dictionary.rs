//! Keyword Dictionary - Extensible keyword management for code analysis
//! 
//! Provides fast loading and lookup of keywords from CSV files for:
//! - Domain classification
//! - Criticality assessment  
//! - Text processing (stopwords)
//! - Project type detection

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

/// Keyword dictionary loaded from CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordDictionary {
    /// Domain keywords for intent classification
    pub domain_keywords: HashMap<String, DomainKeyword>,
    /// Critical domains with scores
    pub critical_domains: HashMap<String, CriticalDomain>,
    /// Stopwords to ignore
    pub stopwords: HashMap<String, f32>,
    /// Project type specific keywords
    pub project_type_keywords: HashMap<String, Vec<String>>,
    /// Dictionary version
    pub version: String,
    /// Last updated timestamp
    pub last_updated: String,
}

/// Domain keyword with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainKeyword {
    pub term: String,
    pub weight: f64,
    pub context: String,
    pub synonyms: Vec<String>,
    pub notes: String,
    pub confidence_boost: f64,
}

/// Critical domain with scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalDomain {
    pub term: String,
    pub criticality_score: u8, // 0-10
    pub reason: String,
}

impl KeywordDictionary {
    /// Load dictionary from CSV file
    pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read keyword dictionary: {}", path.as_ref().display()))?;

        let mut domain_keywords = HashMap::new();
        let mut critical_domains = HashMap::new();
        let mut stopwords = HashMap::new();
        let mut project_type_keywords = HashMap::new();
        let mut version = "1.0.0".to_string();
        let mut last_updated = chrono::Utc::now().to_rfc3339();

        for (line_num, line) in content.lines().enumerate() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() {
                // Extract version from comments
                if line.contains("Version:") {
                    if let Some(v) = line.split(':').nth(1) {
                        version = v.trim().to_string();
                    }
                }
                if line.contains("Last Updated:") {
                    if let Some(date) = line.split(':').nth(1) {
                        last_updated = date.trim().to_string();
                    }
                }
                continue;
            }

            // Parse CSV line
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 3 {
                eprintln!("Warning: Invalid line {} in keyword dictionary: {}", line_num + 1, line);
                continue;
            }

            let category = fields[0].trim();
            let term = fields[1].trim().to_lowercase();
            let weight_score = fields[2].trim();

            match category {
                "domain_keywords" => {
                    if fields.len() >= 6 {
                        let weight = weight_score.parse::<f64>().unwrap_or(1.0);
                        let context = fields[3].trim().to_string();
                        let synonyms = if fields[4].trim().is_empty() {
                            Vec::new()
                        } else {
                            fields[4].split_whitespace().map(|s| s.trim().to_string()).collect()
                        };
                        let notes = fields[5].trim().to_string();

                        domain_keywords.insert(term.clone(), DomainKeyword {
                            term: term.clone(),
                            weight,
                            context,
                            synonyms,
                            notes,
                            confidence_boost: weight * 0.1, // Default boost based on weight
                        });
                    }
                }
                "critical_domains" => {
                    if fields.len() >= 5 {
                        let score = weight_score.parse::<u8>().unwrap_or(5);
                        let reason = if fields.len() > 4 {
                            fields[4].trim().to_string()
                        } else {
                            "Critical domain".to_string()
                        };

                        critical_domains.insert(term.clone(), CriticalDomain {
                            term: term.clone(),
                            criticality_score: score,
                            reason,
                        });
                    }
                }
                "stopwords" => {
                    let weight = weight_score.parse::<f32>().unwrap_or(1.0);
                    stopwords.insert(term, weight);
                }
                "project_type_keywords" => {
                    let project_type = if fields.len() > 3 {
                        fields[3].trim().to_string()
                    } else {
                        "library".to_string()
                    };

                    project_type_keywords
                        .entry(project_type)
                        .or_insert_with(Vec::new)
                        .push(term);
                }
                _ => {
                    eprintln!("Warning: Unknown category '{}' on line {}", category, line_num + 1);
                }
            }
        }

        Ok(Self {
            domain_keywords,
            critical_domains,
            stopwords,
            project_type_keywords,
            version,
            last_updated,
        })
    }

    /// Find best matching domain keyword for given text
    pub fn find_domain_keyword(&self, text: &str) -> Option<&DomainKeyword> {
        let lower = text.to_lowercase();
        
        // Direct match first
        if let Some(keyword) = self.domain_keywords.get(&lower) {
            return Some(keyword);
        }

        // Partial match - find keyword that appears in text
        for (term, keyword) in &self.domain_keywords {
            if lower.contains(term) {
                return Some(keyword);
            }
        }

        None
    }

    /// Get criticality score for a domain
    pub fn get_criticality_score(&self, domain: &str) -> u8 {
        let lower = domain.to_lowercase();
        
        // Direct match
        if let Some(critical) = self.critical_domains.get(&lower) {
            return critical.criticality_score;
        }

        // Partial match
        for (term, critical) in &self.critical_domains {
            if lower.contains(term) {
                return critical.criticality_score;
            }
        }

        0 // No criticality found
    }

    /// Check if a word is a stopword
    pub fn is_stopword(&self, word: &str) -> bool {
        self.stopwords.contains_key(&word.to_lowercase())
    }

    /// Get keywords for a specific project type
    pub fn get_project_type_keywords(&self, project_type: &str) -> &[String] {
        self.project_type_keywords
            .get(project_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Filter stopwords from text
    pub fn filter_stopwords(&self, words: &[String]) -> Vec<String> {
        words.iter()
            .filter(|word| !self.is_stopword(word))
            .cloned()
            .collect()
    }

    /// Get dictionary statistics
    pub fn stats(&self) -> DictionaryStats {
        DictionaryStats {
            version: self.version.clone(),
            last_updated: self.last_updated.clone(),
            domain_keywords_count: self.domain_keywords.len(),
            critical_domains_count: self.critical_domains.len(),
            stopwords_count: self.stopwords.len(),
            project_types_count: self.project_type_keywords.len(),
        }
    }

    /// Validate dictionary integrity
    pub fn validate(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check for empty terms
        for (term, keyword) in &self.domain_keywords {
            if term.is_empty() {
                report.errors.push("Empty domain keyword term found".to_string());
            }
            if keyword.weight < 0.0 || keyword.weight > 1.0 {
                report.warnings.push(format!("Domain keyword '{}' has invalid weight: {}", term, keyword.weight));
            }
        }

        // Check critical domain scores
        for (term, critical) in &self.critical_domains {
            if critical.criticality_score > 10 {
                report.errors.push(format!("Critical domain '{}' has score > 10: {}", term, critical.criticality_score));
            }
        }

        Ok(report)
    }
}

/// Dictionary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryStats {
    pub version: String,
    pub last_updated: String,
    pub domain_keywords_count: usize,
    pub critical_domains_count: usize,
    pub stopwords_count: usize,
    pub project_types_count: usize,
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for KeywordDictionary {
    fn default() -> Self {
        // Try to load from default location, otherwise return empty
        let default_path = Path::new("spec/keywords.csv");
        if default_path.exists() {
            Self::load_from_csv(default_path).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load default keyword dictionary: {}", e);
                Self::empty()
            })
        } else {
            Self::empty()
        }
    }
}

impl KeywordDictionary {
    /// Create empty dictionary (for testing)
    pub fn empty() -> Self {
        Self {
            domain_keywords: HashMap::new(),
            critical_domains: HashMap::new(),
            stopwords: HashMap::new(),
            project_type_keywords: HashMap::new(),
            version: "1.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_csv_dictionary() {
        let csv_content = r#"# Test dictionary
domain_keywords,auth,1.0,security,authentication authorization,Authentication system
critical_domains,payment,10,financial,Core payment processing
stopwords,the,1.0,common stopword
project_type_keywords,cli,1.0,CLI tool keywords"#;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_keywords.csv");
        fs::write(&file_path, csv_content).unwrap();

        let dict = KeywordDictionary::load_from_csv(&file_path).unwrap();
        
        assert_eq!(dict.domain_keywords.len(), 1);
        assert_eq!(dict.critical_domains.len(), 1);
        assert_eq!(dict.stopwords.len(), 1);
        assert!(dict.is_stopword("the"));
        assert!(!dict.is_stopword("payment"));
    }

    #[test]
    fn test_domain_keyword_matching() {
        let mut dict = KeywordDictionary::empty();
        dict.domain_keywords.insert("auth".to_string(), DomainKeyword {
            term: "auth".to_string(),
            weight: 1.0,
            context: "security".to_string(),
            synonyms: vec!["authentication".to_string()],
            notes: "Auth system".to_string(),
            confidence_boost: 0.1,
        });

        let keyword = dict.find_domain_keyword("user authentication system");
        assert!(keyword.is_some());
        assert_eq!(keyword.unwrap().term, "auth");
    }

    #[test]
    fn test_criticality_scoring() {
        let mut dict = KeywordDictionary::empty();
        dict.critical_domains.insert("payment".to_string(), CriticalDomain {
            term: "payment".to_string(),
            criticality_score: 10,
            reason: "Core payment".to_string(),
        });

        assert_eq!(dict.get_criticality_score("payment processing"), 10);
        assert_eq!(dict.get_criticality_score("user management"), 0);
    }
}
