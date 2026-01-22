use std::collections::HashSet;

/// Computes Jaccard similarity between two strings
/// using whitespace-delimited tokens
pub fn jaccard_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    
    let a_words: HashSet<_> = a_lower.split_whitespace().collect();
    let b_words: HashSet<_> = b_lower.split_whitespace().collect();
    
    let intersection = a_words.intersection(&b_words).count() as f64;
    let union = a_words.union(&b_words).count() as f64;
    
    if union == 0.0 { 0.0 } else { intersection / union }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_strings() {
        assert_eq!(jaccard_similarity("", ""), 1.0);
        assert_eq!(jaccard_similarity("", "test"), 0.0);
        assert_eq!(jaccard_similarity("test", ""), 0.0);
    }

    #[test]
    fn test_identical_strings() {
        assert_eq!(jaccard_similarity("hello world", "hello world"), 1.0);
    }

    #[test]
    fn test_partial_match() {
        let sim = jaccard_similarity("hello world", "hello there");
        assert!((sim - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(
            jaccard_similarity("Hello World", "hello world"), 
            1.0
        );
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(
            jaccard_similarity("user@example.com", "user@test.com"), 
            0.5
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(
            jaccard_similarity("version 1.2.3", "version 1.2.4"), 
            0.75
        );
    }

    #[test]
    fn test_completely_different() {
        assert_eq!(
            jaccard_similarity("apple banana", "cherry date"), 
            0.0
        );
    }
}
