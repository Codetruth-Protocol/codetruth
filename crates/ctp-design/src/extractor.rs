//! Design System Extractor
//!
//! Extracts design tokens from source code files.

use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

use crate::tokens::*;

/// Extracts design tokens from source code
pub struct DesignSystemExtractor {
    /// Regex patterns for color extraction
    color_patterns: Vec<Regex>,
    /// Regex patterns for spacing extraction
    spacing_patterns: Vec<Regex>,
    /// Regex patterns for typography extraction
    typography_patterns: Vec<Regex>,
    /// Tailwind color mappings
    tailwind_colors: HashMap<String, String>,
}

impl DesignSystemExtractor {
    pub fn new() -> Self {
        let mut extractor = Self {
            color_patterns: vec![],
            spacing_patterns: vec![],
            typography_patterns: vec![],
            tailwind_colors: HashMap::new(),
        };
        extractor.init_patterns();
        extractor.init_tailwind_colors();
        extractor
    }

    fn init_patterns(&mut self) {
        // Hex colors
        self.color_patterns.push(
            Regex::new(r#"#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b"#).unwrap()
        );
        // RGB/RGBA
        self.color_patterns.push(
            Regex::new(r#"rgba?\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)"#).unwrap()
        );
        // CSS variables for colors
        self.color_patterns.push(
            Regex::new(r#"var\s*\(\s*--([a-zA-Z][\w-]*(?:color|bg|text|border)[\w-]*)\s*\)"#).unwrap()
        );
        // Tailwind color classes
        self.color_patterns.push(
            Regex::new(r#"(?:bg|text|border|ring|shadow)-([a-z]+-\d{2,3})"#).unwrap()
        );

        // Spacing patterns
        self.spacing_patterns.push(
            Regex::new(r#"(?:margin|padding|gap|space|top|right|bottom|left|m|p|mt|mr|mb|ml|mx|my|pt|pr|pb|pl|px|py)-\[?(\d+(?:\.\d+)?(?:px|rem|em)?)\]?"#).unwrap()
        );
        // Tailwind spacing
        self.spacing_patterns.push(
            Regex::new(r#"(?:m|p|gap|space)-([xy]?-)?(\d+(?:\.5)?)"#).unwrap()
        );

        // Typography patterns
        self.typography_patterns.push(
            Regex::new(r#"font-size:\s*([^;]+)"#).unwrap()
        );
        self.typography_patterns.push(
            Regex::new(r#"text-(xs|sm|base|lg|xl|2xl|3xl|4xl|5xl|6xl|7xl|8xl|9xl)"#).unwrap()
        );
        self.typography_patterns.push(
            Regex::new(r#"font-(thin|extralight|light|normal|medium|semibold|bold|extrabold|black)"#).unwrap()
        );
    }

    fn init_tailwind_colors(&mut self) {
        // Common Tailwind color mappings
        let colors = [
            ("slate-50", "#f8fafc"), ("slate-100", "#f1f5f9"), ("slate-500", "#64748b"), ("slate-900", "#0f172a"),
            ("gray-50", "#f9fafb"), ("gray-100", "#f3f4f6"), ("gray-500", "#6b7280"), ("gray-900", "#111827"),
            ("red-50", "#fef2f2"), ("red-500", "#ef4444"), ("red-600", "#dc2626"), ("red-700", "#b91c1c"),
            ("green-50", "#f0fdf4"), ("green-500", "#22c55e"), ("green-600", "#16a34a"), ("green-700", "#15803d"),
            ("blue-50", "#eff6ff"), ("blue-500", "#3b82f6"), ("blue-600", "#2563eb"), ("blue-700", "#1d4ed8"),
            ("yellow-50", "#fefce8"), ("yellow-500", "#eab308"), ("yellow-600", "#ca8a04"),
            ("purple-50", "#faf5ff"), ("purple-500", "#a855f7"), ("purple-600", "#9333ea"),
            ("pink-50", "#fdf2f8"), ("pink-500", "#ec4899"), ("pink-600", "#db2777"),
            ("indigo-50", "#eef2ff"), ("indigo-500", "#6366f1"), ("indigo-600", "#4f46e5"),
        ];
        
        for (name, hex) in colors {
            self.tailwind_colors.insert(name.to_string(), hex.to_string());
        }
    }

    /// Extract design tokens from a single file
    pub fn extract_from_file(&self, content: &str, file_path: &str) -> FileTokens {
        debug!("Extracting tokens from {}", file_path);
        
        let mut tokens = FileTokens::default();
        
        // Extract colors
        for pattern in &self.color_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(color_match) = cap.get(0) {
                    let color_str = color_match.as_str();
                    
                    // Handle hex colors
                    if color_str.starts_with('#') {
                        if let Some(token) = ColorToken::from_hex(color_str) {
                            let key = token.hex.clone();
                            tokens.colors.entry(key)
                                .and_modify(|c: &mut ColorToken| c.usage_count += 1)
                                .or_insert(token);
                        }
                    }
                    // Handle Tailwind classes
                    else if let Some(tw_match) = cap.get(1) {
                        let tw_class = tw_match.as_str();
                        if let Some(hex) = self.tailwind_colors.get(tw_class) {
                            if let Some(mut token) = ColorToken::from_hex(hex) {
                                token.tailwind_class = Some(tw_class.to_string());
                                let key = token.hex.clone();
                                tokens.colors.entry(key)
                                    .and_modify(|c: &mut ColorToken| c.usage_count += 1)
                                    .or_insert(token);
                            }
                        }
                    }
                }
            }
        }

        // Extract spacing
        for pattern in &self.spacing_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(spacing_match) = cap.get(1).or(cap.get(2)) {
                    let value = spacing_match.as_str();
                    let token = SpacingToken::from_value(value);
                    tokens.spacing.entry(value.to_string())
                        .and_modify(|s: &mut SpacingToken| s.usage_count += 1)
                        .or_insert(token);
                }
            }
        }

        // Extract typography
        for pattern in &self.typography_patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(typo_match) = cap.get(1) {
                    let value = typo_match.as_str().trim();
                    tokens.typography_values.push(value.to_string());
                }
            }
        }

        // Detect component patterns (simplified)
        self.extract_components(content, &mut tokens);

        tokens
    }

    fn extract_components(&self, content: &str, tokens: &mut FileTokens) {
        // Look for React/Vue component patterns
        let component_regex = Regex::new(r#"<(Button|Card|Input|Modal|Dialog|Alert|Badge|Avatar|Tooltip|Dropdown|Menu|Tab|Accordion|Checkbox|Radio|Switch|Select|Slider|Progress|Spinner|Toast|Notification)\s+([^>]*)"#).unwrap();
        
        for cap in component_regex.captures_iter(content) {
            if let (Some(name), Some(props)) = (cap.get(1), cap.get(2)) {
                let component_name = name.as_str().to_string();
                let props_str = props.as_str();
                
                // Extract variant and size props
                let variant_regex = Regex::new(r#"variant=["']([^"']+)["']"#).unwrap();
                let size_regex = Regex::new(r#"size=["']([^"']+)["']"#).unwrap();
                
                let entry = tokens.components.entry(component_name.clone())
                    .or_insert_with(|| ComponentPattern {
                        name: component_name,
                        variants: vec![],
                        sizes: vec![],
                        common_props: vec![],
                        style_patterns: vec![],
                        usage_count: 0,
                    });
                
                entry.usage_count += 1;
                
                if let Some(variant_cap) = variant_regex.captures(props_str) {
                    if let Some(v) = variant_cap.get(1) {
                        let variant = v.as_str().to_string();
                        if !entry.variants.contains(&variant) {
                            entry.variants.push(variant);
                        }
                    }
                }
                
                if let Some(size_cap) = size_regex.captures(props_str) {
                    if let Some(s) = size_cap.get(1) {
                        let size = s.as_str().to_string();
                        if !entry.sizes.contains(&size) {
                            entry.sizes.push(size);
                        }
                    }
                }
            }
        }
    }

    /// Merge tokens from multiple files into a complete design system
    pub fn merge_tokens(&self, file_tokens: Vec<FileTokens>) -> DesignTokens {
        let mut tokens = DesignTokens::new();
        
        for ft in file_tokens {
            // Merge colors
            for (hex, color) in ft.colors {
                tokens.colors.colors.entry(hex)
                    .and_modify(|c| c.usage_count += color.usage_count)
                    .or_insert(color);
            }
            
            // Merge spacing
            for (value, spacing) in ft.spacing {
                tokens.spacing.common_values.entry(value.clone())
                    .and_modify(|count| *count += spacing.usage_count)
                    .or_insert(spacing.usage_count);
                
                if !tokens.spacing.scale.iter().any(|s| s.value == value) {
                    tokens.spacing.scale.push(spacing);
                }
            }
            
            // Merge components
            for (name, component) in ft.components {
                tokens.components.entry(name)
                    .and_modify(|c| {
                        c.usage_count += component.usage_count;
                        for v in &component.variants {
                            if !c.variants.contains(v) {
                                c.variants.push(v.clone());
                            }
                        }
                        for s in &component.sizes {
                            if !c.sizes.contains(s) {
                                c.sizes.push(s.clone());
                            }
                        }
                    })
                    .or_insert(component);
            }
        }
        
        // Infer semantic colors from usage patterns
        self.infer_semantic_colors(&mut tokens);
        
        // Sort spacing scale
        tokens.spacing.scale.sort_by(|a, b| {
            a.pixels.partial_cmp(&b.pixels).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        tokens
    }

    fn infer_semantic_colors(&self, tokens: &mut DesignTokens) {
        // Find most used colors and assign semantic names
        let mut colors_by_usage: Vec<_> = tokens.colors.colors.iter().collect();
        colors_by_usage.sort_by(|a, b| b.1.usage_count.cmp(&a.1.usage_count));
        
        // Heuristics for semantic color assignment
        for (hex, color) in &colors_by_usage {
            let (r, g, b) = (color.rgb.0 as f64, color.rgb.1 as f64, color.rgb.2 as f64);
            
            // Check if grayscale (low saturation)
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let saturation = if max > 0.0 { (max - min) / max } else { 0.0 };
            
            // Blue-ish colors are often primary (exclude purple/cyan)
            if color.rgb.2 > color.rgb.0 + 30 && color.rgb.2 > color.rgb.1 + 30
                && saturation > 0.3
                && !tokens.colors.semantic.contains_key("primary") {
                tokens.colors.semantic.insert("primary".to_string(), hex.to_string());
            }
            // Red-ish colors are often error/danger (lowered threshold)
            else if color.rgb.0 > 150 && color.rgb.0 > color.rgb.1 + 50 && color.rgb.0 > color.rgb.2 + 50
                && !tokens.colors.semantic.contains_key("error") {
                tokens.colors.semantic.insert("error".to_string(), hex.to_string());
            }
            // Green-ish colors are often success (exclude yellow-green)
            else if color.rgb.1 > color.rgb.0 + 30 && color.rgb.1 > color.rgb.2 + 30
                && saturation > 0.3
                && !tokens.colors.semantic.contains_key("success") {
                tokens.colors.semantic.insert("success".to_string(), hex.to_string());
            }
            // Grayscale colors
            else if saturation < 0.1 {
                if max < 50.0 && !tokens.colors.semantic.contains_key("text-dark") {
                    tokens.colors.semantic.insert("text-dark".to_string(), hex.to_string());
                } else if max > 200.0 && !tokens.colors.semantic.contains_key("background-light") {
                    tokens.colors.semantic.insert("background-light".to_string(), hex.to_string());
                }
            }
        }
    }
}

impl Default for DesignSystemExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Tokens extracted from a single file
#[derive(Debug, Default)]
pub struct FileTokens {
    pub colors: HashMap<String, ColorToken>,
    pub spacing: HashMap<String, SpacingToken>,
    pub typography_values: Vec<String>,
    pub components: HashMap<String, ComponentPattern>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hex_colors() {
        let extractor = DesignSystemExtractor::new();
        let content = r#"
            .button { background-color: #2563eb; }
            .text { color: #1f2937; }
        "#;
        
        let tokens = extractor.extract_from_file(content, "test.css");
        assert!(tokens.colors.contains_key("#2563eb"));
        assert!(tokens.colors.contains_key("#1f2937"));
    }

    #[test]
    fn test_extract_tailwind_colors() {
        let extractor = DesignSystemExtractor::new();
        let content = r#"<div className="bg-blue-600 text-slate-900">"#;
        
        let tokens = extractor.extract_from_file(content, "test.tsx");
        // Should extract the hex values for these Tailwind classes
        assert!(!tokens.colors.is_empty());
    }

    #[test]
    fn test_extract_components() {
        let extractor = DesignSystemExtractor::new();
        let content = r#"
            <Button variant="primary" size="lg">Click</Button>
            <Button variant="secondary" size="sm">Cancel</Button>
            <Card>Content</Card>
        "#;
        
        let tokens = extractor.extract_from_file(content, "test.tsx");
        assert!(tokens.components.contains_key("Button"));
        assert_eq!(tokens.components.get("Button").unwrap().usage_count, 2);
    }
}
