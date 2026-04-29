//! Security utilities for MCP server
//!
//! Provides path traversal protection, input validation,
//! and resource limiting for production-grade security.

use std::path::{Path, PathBuf};
use crate::error::{MCPError, MCPResult};

/// Maximum file size for analysis (10MB)
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of files per batch analysis
pub const MAX_FILES_PER_BATCH: usize = 100;

/// Maximum path length
pub const MAX_PATH_LENGTH: usize = 4096;

/// Sanitize and validate a path to prevent directory traversal attacks
///
/// # Arguments
/// * `input` - The user-provided path string
/// * `base_dir` - The allowed base directory (optional, for sandboxing)
///
/// # Returns
/// * `Ok(PathBuf)` - The sanitized, canonicalized path
/// * `Err(MCPError)` - If path is invalid or escapes base directory
pub fn sanitize_path(input: &str, base_dir: Option<&Path>) -> MCPResult<PathBuf> {
    // Check path length
    if input.len() > MAX_PATH_LENGTH {
        return Err(MCPError::InvalidPath(format!(
            "path exceeds maximum length of {} characters",
            MAX_PATH_LENGTH
        )));
    }

    // Reject null bytes
    if input.contains('\0') {
        return Err(MCPError::InvalidPath("path contains null bytes".into()));
    }

    // Parse the path
    let path = Path::new(input);

    // Check for path traversal attempts
    let path_str = input.replace('\\', "/");
    if path_str.contains("../") || path_str.contains("/..") || path_str.starts_with("..") {
        return Err(MCPError::InvalidPath(
            "path traversal detected - '..' not allowed".into()
        ));
    }

    // If base directory is specified, ensure path is within it
    if let Some(base) = base_dir {
        // Canonicalize both paths for comparison
        let canonical_base = base.canonicalize()
            .map_err(|_| MCPError::InvalidPath("invalid base directory".into()))?;
        
        // For relative paths, join with base
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            canonical_base.join(path)
        };

        // Canonicalize the full path
        let canonical_path = full_path.canonicalize()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => MCPError::FileNotFound(full_path.clone()),
                std::io::ErrorKind::PermissionDenied => MCPError::PermissionDenied(full_path.clone()),
                _ => MCPError::InvalidPath(format!("failed to canonicalize path: {}", e)),
            })?;

        // Verify path is within base directory
        if !canonical_path.starts_with(&canonical_base) {
            return Err(MCPError::InvalidPath(
                "path escapes allowed base directory".into()
            ));
        }

        Ok(canonical_path)
    } else {
        // No base directory restriction, just canonicalize
        path.canonicalize()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => MCPError::FileNotFound(path.to_path_buf()),
                std::io::ErrorKind::PermissionDenied => MCPError::PermissionDenied(path.to_path_buf()),
                _ => MCPError::InvalidPath(format!("invalid path: {}", e)),
            })
    }
}

/// Validate a glob pattern to prevent regex DoS attacks
///
/// # Arguments
/// * `pattern` - The glob pattern string
///
/// # Returns
/// * `Ok(())` - Pattern is valid
/// * `Err(MCPError)` - If pattern is invalid or potentially dangerous
pub fn validate_glob_pattern(pattern: &str) -> MCPResult<()> {
    // Check pattern length
    if pattern.len() > 256 {
        return Err(MCPError::InvalidInput(
            "glob pattern exceeds maximum length of 256 characters".into()
        ));
    }

    // Check for suspicious patterns that could cause excessive backtracking
    let suspicious = ["*******", "???????", "+++**", "**++"];
    for s in &suspicious {
        if pattern.contains(s) {
            return Err(MCPError::InvalidInput(format!(
                "glob pattern contains suspicious sequence: {}", s
            )));
        }
    }

    // Validate the pattern compiles
    glob::Pattern::new(pattern)
        .map_err(|e| MCPError::InvalidInput(format!("invalid glob pattern: {}", e)))?;

    Ok(())
}

/// Check if file size is within allowed limits
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// * `Ok(u64)` - The file size in bytes
/// * `Err(MCPError)` - If file is too large or cannot be accessed
pub fn check_file_size(path: &Path) -> MCPResult<u64> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => MCPError::FileNotFound(path.to_path_buf()),
            std::io::ErrorKind::PermissionDenied => MCPError::PermissionDenied(path.to_path_buf()),
            _ => MCPError::Io(e),
        })?;

    let size = metadata.len();
    
    if size > MAX_FILE_SIZE {
        return Err(MCPError::InvalidInput(format!(
            "file {} exceeds maximum size of {}MB ({} bytes)",
            path.display(),
            MAX_FILE_SIZE / (1024 * 1024),
            size
        )));
    }

    Ok(size)
}

/// Resource limits for batch operations
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    /// Maximum number of files to process in a batch
    pub max_files: usize,
    /// Maximum total size of all files in bytes
    pub max_total_size: u64,
    /// Maximum time to spend on analysis
    pub timeout_seconds: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_files: MAX_FILES_PER_BATCH,
            max_total_size: MAX_FILE_SIZE * 10, // 100MB total
            timeout_seconds: 300, // 5 minutes
        }
    }
}

impl ResourceLimits {
    /// Create limits for testing (more restrictive)
    pub fn test_limits() -> Self {
        Self {
            max_files: 10,
            max_total_size: MAX_FILE_SIZE,
            timeout_seconds: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_path_basic() {
        let dir = TempDir::new().unwrap();
        let base = dir.path();
        
        // Create a test file
        let test_file = base.join("test.txt");
        fs::write(&test_file, "test").unwrap();
        
        // Valid relative path
        let result = sanitize_path("test.txt", Some(base));
        assert!(result.is_ok());
        
        // Valid absolute path
        let result = sanitize_path(test_file.to_str().unwrap(), Some(base));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_path_traversal() {
        let dir = TempDir::new().unwrap();
        let base = dir.path();
        
        // Path traversal should be rejected
        let result = sanitize_path("../etc/passwd", Some(base));
        assert!(matches!(result, Err(MCPError::InvalidPath(_))));
        
        // Double dots in middle
        let result = sanitize_path("foo/../../etc/passwd", Some(base));
        assert!(matches!(result, Err(MCPError::InvalidPath(_))));
    }

    #[test]
    fn test_sanitize_path_escapes_base() {
        let dir1 = TempDir::new().unwrap();
        let dir2 = TempDir::new().unwrap();
        
        // Try to access file outside base directory
        let outside_file = dir2.path().join("outside.txt");
        fs::write(&outside_file, "test").unwrap();
        
        let result = sanitize_path(
            outside_file.to_str().unwrap(),
            Some(dir1.path())
        );
        assert!(matches!(result, Err(MCPError::InvalidPath(_))));
    }

    #[test]
    fn test_validate_glob_pattern() {
        // Valid patterns
        assert!(validate_glob_pattern("**/*.rs").is_ok());
        assert!(validate_glob_pattern("src/*.ts").is_ok());
        
        // Too long
        let long_pattern = "*".repeat(300);
        assert!(validate_glob_pattern(&long_pattern).is_err());
        
        // Suspicious pattern
        assert!(validate_glob_pattern("*******").is_err());
    }

    #[test]
    fn test_check_file_size() {
        let dir = TempDir::new().unwrap();
        let small_file = dir.path().join("small.txt");
        fs::write(&small_file, "small").unwrap();
        
        assert!(check_file_size(&small_file).is_ok());
        
        // Create a file larger than MAX_FILE_SIZE would require
        // actually creating a large file, so we skip that test
    }
}
