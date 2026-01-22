//! Design Tokens
//!
//! Core data structures for representing design system tokens.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete design token collection for a codebase
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignTokens {
    /// Color palette
    pub colors: ColorPalette,
    
    /// Typography settings
    pub typography: TypographySystem,
    
    /// Spacing scale
    pub spacing: SpacingScale,
    
    /// Component patterns discovered
    pub components: HashMap<String, ComponentPattern>,
    
    /// Metadata about extraction
    pub metadata: TokenMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// When tokens were extracted
    pub extracted_at: String,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Confidence in extraction (0-1)
    pub confidence: f64,
}

/// Color palette extracted from codebase
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Named colors with their values and usage counts
    pub colors: HashMap<String, ColorToken>,
    
    /// Semantic color mappings (e.g., "primary" -> "#2563eb")
    pub semantic: HashMap<String, String>,
}

/// A single color token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorToken {
    /// Hex value (normalized to lowercase)
    pub hex: String,
    
    /// RGB values
    pub rgb: (u8, u8, u8),
    
    /// Optional alpha
    pub alpha: Option<f64>,
    
    /// CSS variable name if used
    pub css_var: Option<String>,
    
    /// Tailwind class if applicable
    pub tailwind_class: Option<String>,
    
    /// Number of times this color appears
    pub usage_count: usize,
    
    /// Contexts where this color is used
    pub contexts: Vec<ColorContext>,
}

impl ColorToken {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let mut hex = hex.trim_start_matches('#').to_lowercase();
        
        // Expand 3-character hex to 6-character (e.g., #fff -> #ffffff)
        if hex.len() == 3 {
            let chars: Vec<char> = hex.chars().collect();
            hex = format!("{}{}{}{}{}{}", 
                chars[0], chars[0], 
                chars[1], chars[1], 
                chars[2], chars[2]
            );
        }
        
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let alpha = if hex.len() == 8 {
            Some(u8::from_str_radix(&hex[6..8], 16).ok()? as f64 / 255.0)
        } else {
            None
        };
        
        Some(Self {
            hex: format!("#{}", &hex[0..6]),
            rgb: (r, g, b),
            alpha,
            css_var: None,
            tailwind_class: None,
            usage_count: 1,
            contexts: vec![],
        })
    }
    
    /// Calculate color distance (simple Euclidean in RGB space)
    pub fn distance(&self, other: &ColorToken) -> f64 {
        let dr = self.rgb.0 as f64 - other.rgb.0 as f64;
        let dg = self.rgb.1 as f64 - other.rgb.1 as f64;
        let db = self.rgb.2 as f64 - other.rgb.2 as f64;
        (dr * dr + dg * dg + db * db).sqrt()
    }
    
    /// Check if colors are similar (within threshold)
    pub fn is_similar(&self, other: &ColorToken, threshold: f64) -> bool {
        self.distance(other) < threshold
    }
}

/// Context where a color is used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorContext {
    Background,
    Text,
    Border,
    Shadow,
    Icon,
    Unknown,
}

/// Typography system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypographySystem {
    /// Font families used
    pub font_families: Vec<FontFamily>,
    
    /// Heading styles
    pub headings: HashMap<String, TypographyToken>,
    
    /// Body text styles
    pub body: Option<TypographyToken>,
    
    /// Font size scale
    pub size_scale: Vec<String>,
    
    /// Font weight scale
    pub weight_scale: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    pub name: String,
    pub category: FontCategory,
    pub usage_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FontCategory {
    Sans,
    Serif,
    Mono,
    Display,
    Unknown,
}

/// Typography token for a specific text style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyToken {
    /// Font size (with unit)
    pub size: String,
    
    /// Font weight
    pub weight: Option<u16>,
    
    /// Line height
    pub line_height: Option<String>,
    
    /// Letter spacing
    pub letter_spacing: Option<String>,
    
    /// Text color reference
    pub color: Option<String>,
    
    /// Usage count
    pub usage_count: usize,
}

/// Spacing scale
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpacingScale {
    /// Base unit (e.g., "0.25rem", "4px")
    pub base_unit: Option<String>,
    
    /// Scale values used
    pub scale: Vec<SpacingToken>,
    
    /// Common spacing values found
    pub common_values: HashMap<String, usize>,
}

/// A spacing token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingToken {
    /// The spacing value
    pub value: String,
    
    /// Normalized to pixels (if possible)
    pub pixels: Option<f64>,
    
    /// Tailwind class equivalent
    pub tailwind_class: Option<String>,
    
    /// Usage count
    pub usage_count: usize,
}

impl SpacingToken {
    pub fn from_value(value: &str) -> Self {
        let pixels = Self::parse_to_pixels(value);
        Self {
            value: value.to_string(),
            pixels,
            tailwind_class: None,
            usage_count: 1,
        }
    }
    
    fn parse_to_pixels(value: &str) -> Option<f64> {
        let value = value.trim();
        if value.ends_with("px") {
            value.trim_end_matches("px").parse().ok()
        } else if value.ends_with("rem") {
            value.trim_end_matches("rem").parse::<f64>().ok().map(|v| v * 16.0)
        } else if value.ends_with("em") {
            value.trim_end_matches("em").parse::<f64>().ok().map(|v| v * 16.0)
        } else {
            value.parse().ok()
        }
    }
}

/// Component pattern discovered in codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPattern {
    /// Component name
    pub name: String,
    
    /// Variants found
    pub variants: Vec<String>,
    
    /// Sizes found
    pub sizes: Vec<String>,
    
    /// Common props/attributes
    pub common_props: Vec<String>,
    
    /// Style patterns (classes, inline styles)
    pub style_patterns: Vec<String>,
    
    /// Usage count
    pub usage_count: usize,
}

impl DesignTokens {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Find the closest color in the palette to a given hex
    pub fn find_closest_color(&self, hex: &str) -> Option<(String, f64)> {
        let target = ColorToken::from_hex(hex)?;
        
        let mut closest: Option<(String, f64)> = None;
        
        for (name, token) in &self.colors.colors {
            let distance = target.distance(token);
            if closest.is_none() || distance < closest.as_ref().unwrap().1 {
                closest = Some((name.clone(), distance));
            }
        }
        
        closest
    }
    
    /// Check if a color is in the palette
    pub fn has_color(&self, hex: &str) -> bool {
        let normalized = hex.trim_start_matches('#').to_lowercase();
        self.colors.colors.values().any(|c| {
            c.hex.trim_start_matches('#').to_lowercase() == normalized
        })
    }
    
    /// Get suggested color for an unknown color
    pub fn suggest_color(&self, hex: &str, threshold: f64) -> Option<String> {
        let (name, distance) = self.find_closest_color(hex)?;
        if distance < threshold {
            Some(name)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_from_hex() {
        let color = ColorToken::from_hex("#2563eb").unwrap();
        assert_eq!(color.rgb, (37, 99, 235));
        assert_eq!(color.hex, "#2563eb");
    }
    
    #[test]
    fn test_color_distance() {
        let white = ColorToken::from_hex("#ffffff").unwrap();
        let black = ColorToken::from_hex("#000000").unwrap();
        let gray = ColorToken::from_hex("#808080").unwrap();
        
        // White to black should be max distance
        assert!(white.distance(&black) > white.distance(&gray));
    }
    
    #[test]
    fn test_spacing_parse() {
        let px = SpacingToken::from_value("16px");
        assert_eq!(px.pixels, Some(16.0));
        
        let rem = SpacingToken::from_value("1rem");
        assert_eq!(rem.pixels, Some(16.0));
    }
}
