//! Input validation utilities

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Validate and sanitize file path to prevent path traversal attacks
pub fn validate_file_path(path: &Path) -> Result<PathBuf> {
    let canonical = path.canonicalize()
        .context("Failed to canonicalize path")?;
    
    // Check for path traversal attempts
    if canonical.to_str().map_or(false, |s| s.contains("..")) {
        anyhow::bail!("Path traversal detected");
    }
    
    Ok(canonical)
}

/// Validate file size is within limits
pub fn validate_file_size(path: &Path, max_size: usize) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .context("Failed to read file metadata")?;
    
    let size = metadata.len() as usize;
    if size > max_size {
        anyhow::bail!("File too large: {} bytes (max: {} bytes)", size, max_size);
    }
    
    Ok(())
}

/// Validate policy YAML content before parsing
pub fn validate_policy_yaml(content: &str) -> Result<()> {
    if content.is_empty() {
        anyhow::bail!("Policy file is empty");
    }
    
    if content.len() > 1_000_000 {
        anyhow::bail!("Policy file too large");
    }
    
    // Basic YAML structure check
    if !content.contains("ctp_version") {
        anyhow::bail!("Invalid policy: missing ctp_version");
    }
    
    if !content.contains("policy:") {
        anyhow::bail!("Invalid policy: missing policy section");
    }
    
    Ok(())
}

/// Sanitize user input for display
pub fn sanitize_for_display(input: &str, max_len: usize) -> String {
    let truncated = if input.len() > max_len {
        format!("{}...", &input[..max_len])
    } else {
        input.to_string()
    };
    
    // Remove control characters
    truncated.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();
        
        assert!(validate_file_size(&file_path, 100).is_ok());
        assert!(validate_file_size(&file_path, 3).is_err());
    }

    #[test]
    fn test_validate_policy_yaml() {
        let valid = r#"
ctp_version: "1.0.0"
policy:
  id: "test"
  name: "Test Policy"
"#;
        assert!(validate_policy_yaml(valid).is_ok());
        
        assert!(validate_policy_yaml("").is_err());
        assert!(validate_policy_yaml("invalid: yaml").is_err());
    }

    #[test]
    fn test_sanitize_for_display() {
        assert_eq!(sanitize_for_display("hello", 10), "hello");
        assert_eq!(sanitize_for_display("hello world", 5), "hello...");
        
        let with_control = "hello\x00world\x01";
        let sanitized = sanitize_for_display(with_control, 100);
        assert!(!sanitized.contains('\x00'));
        assert!(!sanitized.contains('\x01'));
    }
}
