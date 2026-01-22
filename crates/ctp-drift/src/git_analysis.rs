//! Git-based drift detection
//!
//! Analyzes recent commits to detect drift patterns

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitAnalysisConfig {
    /// Number of recent commits to analyze
    pub recent_commits: usize,
    /// Threshold for high churn (number of changes)
    pub high_churn_threshold: usize,
    /// Files that should rarely change
    pub protected_files: Vec<String>,
}

impl Default for GitAnalysisConfig {
    fn default() -> Self {
        Self {
            recent_commits: 2,
            high_churn_threshold: 10,
            protected_files: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDriftFinding {
    pub file_path: String,
    pub finding_type: GitDriftType,
    pub severity: GitDriftSeverity,
    pub description: String,
    pub commits: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GitDriftType {
    HighChurn,          // File changed too frequently
    ProtectedFileChange, // Protected file was modified
    LargeRefactor,      // Massive changes in one commit
    RecentChange,       // File changed in last N commits
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GitDriftSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

pub struct GitAnalyzer {
    config: GitAnalysisConfig,
    repo_path: PathBuf,
}

impl GitAnalyzer {
    pub fn new(repo_path: PathBuf, config: GitAnalysisConfig) -> Self {
        Self { repo_path, config }
    }

    /// Get files changed in last N commits
    pub fn get_recent_changes(&self) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args([
                "diff",
                "--name-only",
                &format!("HEAD~{}..HEAD", self.config.recent_commits),
            ])
            .output()
            .context("Failed to run git diff")?;

        if !output.status.success() {
            anyhow::bail!("Git command failed");
        }

        let files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| PathBuf::from(line.trim()))
            .collect();

        Ok(files)
    }

    /// Get commit history for a specific file
    pub fn get_file_history(&self, file_path: &Path, limit: usize) -> Result<Vec<String>> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args([
                "log",
                &format!("-{}", limit),
                "--pretty=format:%H",
                "--",
                file_path.to_str().unwrap_or(""),
            ])
            .output()
            .context("Failed to get git log")?;

        if !output.status.success() {
            anyhow::bail!("Git log failed");
        }

        let commits = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(commits)
    }

    /// Count changes to a file in recent history
    pub fn count_file_changes(&self, file_path: &Path, since_days: usize) -> Result<usize> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args([
                "log",
                &format!("--since={} days ago", since_days),
                "--oneline",
                "--",
                file_path.to_str().unwrap_or(""),
            ])
            .output()
            .context("Failed to count changes")?;

        if !output.status.success() {
            return Ok(0);
        }

        let count = String::from_utf8_lossy(&output.stdout).lines().count();
        Ok(count)
    }

    /// Analyze drift based on git history
    pub fn analyze_drift(&self, files: &[PathBuf]) -> Result<Vec<GitDriftFinding>> {
        let mut findings = vec![];

        for file in files {
            // Check if file changed recently
            let recent_commits = self.get_file_history(file, self.config.recent_commits)?;
            if !recent_commits.is_empty() {
                findings.push(GitDriftFinding {
                    file_path: file.display().to_string(),
                    finding_type: GitDriftType::RecentChange,
                    severity: GitDriftSeverity::Info,
                    description: format!("File changed in last {} commit(s)", recent_commits.len()),
                    commits: recent_commits.clone(),
                });
            }

            // Check for high churn
            let change_count = self.count_file_changes(file, 30)?;
            if change_count >= self.config.high_churn_threshold {
                findings.push(GitDriftFinding {
                    file_path: file.display().to_string(),
                    finding_type: GitDriftType::HighChurn,
                    severity: GitDriftSeverity::Medium,
                    description: format!("High churn: {} changes in last 30 days", change_count),
                    commits: recent_commits.clone(),
                });
            }

            // Check if protected file was changed
            let file_str = file.to_str().unwrap_or("");
            if self.config.protected_files.iter().any(|p| file_str.contains(p)) {
                findings.push(GitDriftFinding {
                    file_path: file.display().to_string(),
                    finding_type: GitDriftType::ProtectedFileChange,
                    severity: GitDriftSeverity::High,
                    description: "Protected file was modified".into(),
                    commits: recent_commits,
                });
            }
        }

        Ok(findings)
    }

    /// Get diff for specific files in last N commits
    pub fn get_recent_diff(&self, file_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args([
                "diff",
                &format!("HEAD~{}..HEAD", self.config.recent_commits),
                "--",
                file_path.to_str().unwrap_or(""),
            ])
            .output()
            .context("Failed to get diff")?;

        if !output.status.success() {
            anyhow::bail!("Git diff failed");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = GitAnalysisConfig::default();
        assert_eq!(config.recent_commits, 2);
        assert_eq!(config.high_churn_threshold, 10);
    }

    #[test]
    fn test_drift_severity_ordering() {
        assert!(GitDriftSeverity::Critical > GitDriftSeverity::High);
        assert!(GitDriftSeverity::High > GitDriftSeverity::Medium);
        assert!(GitDriftSeverity::Medium > GitDriftSeverity::Low);
    }
}
