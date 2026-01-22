//! LLM utility functions for key detection and provider inference

use anyhow::{Result, Context};

/// Detect LLM provider from API key prefix
pub fn detect_provider_from_key(key: &str) -> Option<&'static str> {
    if key.starts_with("gsk_") {
        Some("groq")
    } else if key.starts_with("sk-") {
        Some("openai")
    } else if key.starts_with("sk-ant-") {
        Some("anthropic")
    } else {
        None
    }
}

/// Get LLM provider, auto-detecting from key if not specified
pub fn get_llm_provider(key: Option<&str>, provider: Option<&str>) -> Option<String> {
    // If provider explicitly specified, use it
    if let Some(p) = provider {
        return Some(p.to_string());
    }
    
    // Otherwise try to detect from key
    if let Some(k) = key {
        if let Some(detected) = detect_provider_from_key(k) {
            return Some(detected.to_string());
        }
    }
    
    None
}

/// Validate LLM configuration
pub fn validate_llm_config(key: Option<&str>, provider: Option<&str>) -> Result<(String, String)> {
    let key = key.context("LLM key required. Set via --llm-key or CTP_LLM_KEY env var")?;
    
    let provider = get_llm_provider(Some(key), provider)
        .context("Could not detect LLM provider from key. Specify with --llm-provider")?;
    
    Ok((key.to_string(), provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_groq() {
        assert_eq!(detect_provider_from_key("gsk_abc123"), Some("groq"));
    }

    #[test]
    fn test_detect_openai() {
        assert_eq!(detect_provider_from_key("sk-abc123"), Some("openai"));
    }

    #[test]
    fn test_detect_anthropic() {
        assert_eq!(detect_provider_from_key("sk-ant-abc123"), Some("anthropic"));
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_provider_from_key("unknown_key"), None);
    }

    #[test]
    fn test_get_provider_explicit() {
        let provider = get_llm_provider(Some("gsk_123"), Some("openai"));
        assert_eq!(provider, Some("openai".to_string()));
    }

    #[test]
    fn test_get_provider_auto_detect() {
        let provider = get_llm_provider(Some("gsk_123"), None);
        assert_eq!(provider, Some("groq".to_string()));
    }
}
