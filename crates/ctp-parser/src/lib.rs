//! # CTP Parser
//!
//! AST parsing layer for CodeTruth Protocol using tree-sitter.
//!
//! This crate provides language-agnostic parsing capabilities for
//! extracting structural information from source code.

use std::collections::HashMap;

use anyhow::{Context, Result};
use thiserror::Error;
use tree_sitter::{Parser, Tree};
use tracing::debug;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Parse failed: {0}")]
    ParseFailed(String),

    #[error("Language '{language}' not enabled. Enable it with: cargo install ctp-cli --features {feature}")]
    LanguageNotEnabled {
        language: String,
        feature: String,
    },
}

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Java,
}

impl SupportedLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "py" => Some(Self::Python),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Java => "java",
        }
    }

    pub fn feature_name(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Rust => "rust-lang",
            Self::Go => "go",
            Self::Java => "java",
        }
    }
}

/// Parsed AST information
#[derive(Debug, Clone)]
pub struct ParsedAST {
    pub language: SupportedLanguage,
    pub tree: Tree,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
    pub comments: Vec<CommentInfo>,
    pub complexity: ComplexityMetrics,
}

/// Code complexity metrics
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity (number of decision points + 1)
    pub cyclomatic: usize,
    /// Lines of code (excluding blanks and comments)
    pub loc: usize,
    /// Number of functions
    pub function_count: usize,
    /// Maximum nesting depth
    pub max_nesting: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub parameters: Vec<String>,
    pub docstring: Option<String>,
    pub is_async: bool,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub methods: Vec<FunctionInfo>,
    pub docstring: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module: String,
    pub items: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CommentInfo {
    pub text: String,
    pub line: usize,
    pub is_doc_comment: bool,
}

/// Multi-language parser using tree-sitter
pub struct CTPParser {
    parsers: HashMap<SupportedLanguage, Parser>,
}

impl CTPParser {
    /// Create a new parser with default language support
    pub fn new() -> Result<Self> {
        Self::with_languages(&[])
    }

    /// Check if a language is supported (feature enabled)
    pub fn is_language_supported(&self, language: SupportedLanguage) -> bool {
        self.parsers.contains_key(&language)
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<SupportedLanguage> {
        self.parsers.keys().copied().collect()
    }

    /// Create parser with specific languages (for testing)
    fn with_languages(_languages: &[SupportedLanguage]) -> Result<Self> {
        let mut parsers = HashMap::new();

        // Initialize parsers for enabled languages
        #[cfg(feature = "python")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_python::language())
                .context("Failed to set Python language")?;
            parsers.insert(SupportedLanguage::Python, parser);
        }

        #[cfg(feature = "javascript")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_javascript::language())
                .context("Failed to set JavaScript language")?;
            parsers.insert(SupportedLanguage::JavaScript, parser);
        }

        #[cfg(feature = "typescript")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_typescript::language_typescript())
                .context("Failed to set TypeScript language")?;
            parsers.insert(SupportedLanguage::TypeScript, parser);
        }

        #[cfg(feature = "rust-lang")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_rust::language())
                .context("Failed to set Rust language")?;
            parsers.insert(SupportedLanguage::Rust, parser);
        }

        #[cfg(feature = "go")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_go::language())
                .context("Failed to set Go language")?;
            parsers.insert(SupportedLanguage::Go, parser);
        }

        #[cfg(feature = "java")]
        {
            let mut parser = Parser::new();
            parser
                .set_language(tree_sitter_java::language())
                .context("Failed to set Java language")?;
            parsers.insert(SupportedLanguage::Java, parser);
        }

        Ok(Self { parsers })
    }

    /// Parse source code for a given language
    pub fn parse(&mut self, source: &str, language: SupportedLanguage) -> Result<ParsedAST> {
        let parser = self
            .parsers
            .get_mut(&language)
            .ok_or_else(|| ParseError::LanguageNotEnabled {
                language: language.name().into(),
                feature: language.feature_name().into(),
            })?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| ParseError::ParseFailed("Parser returned None".into()))?;

        debug!("Parsed {} lines of {} code", source.lines().count(), language.name());

        // Extract structural information
        let functions = self.extract_functions(&tree, source, language);
        let classes = self.extract_classes(&tree, source, language);
        let imports = self.extract_imports(&tree, source, language);
        let comments = self.extract_comments(&tree, source, language);
        let complexity = self.calculate_complexity(&tree, source, language, &functions);

        Ok(ParsedAST {
            language,
            tree,
            functions,
            classes,
            imports,
            comments,
            complexity,
        })
    }

    /// Extract function definitions from AST
    fn extract_functions(
        &self,
        tree: &Tree,
        source: &str,
        language: SupportedLanguage,
    ) -> Vec<FunctionInfo> {
        let mut functions = vec![];
        let root = tree.root_node();

        // Node types vary by language
        let func_types = match language {
            SupportedLanguage::Python => vec!["function_definition"],
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                vec!["function_declaration", "arrow_function", "method_definition"]
            }
            SupportedLanguage::Rust => vec!["function_item"],
            SupportedLanguage::Go => vec!["function_declaration", "method_declaration"],
            SupportedLanguage::Java => vec!["method_declaration"],
        };

        self.traverse_for_nodes(&root, source, &func_types, &mut |node, src| {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node.utf8_text(src.as_bytes()).unwrap_or("").to_string();
                let line_start = node.start_position().row + 1;
                let line_end = node.end_position().row + 1;

                // Extract parameters
                let parameters = self.extract_parameters(node, src, language);

                // Extract docstring (check previous sibling or first child)
                let docstring = self.extract_docstring(node, src, language);

                // Check for async
                let is_async = self.check_is_async(node, src, language);

                // Check visibility
                let is_public = self.check_is_public(node, src, language);

                functions.push(FunctionInfo {
                    name,
                    line_start,
                    line_end,
                    parameters,
                    docstring,
                    is_async,
                    is_public,
                });
            }
        });

        functions
    }

    /// Extract function parameters
    fn extract_parameters(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: SupportedLanguage,
    ) -> Vec<String> {
        let mut params = vec![];

        let param_field = match language {
            SupportedLanguage::Python => "parameters",
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => "parameters",
            SupportedLanguage::Rust => "parameters",
            SupportedLanguage::Go => "parameters",
            SupportedLanguage::Java => "parameters",
        };

        if let Some(params_node) = node.child_by_field_name(param_field) {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                // Look for parameter/identifier nodes
                if child.kind().contains("parameter") || child.kind() == "identifier" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            params.push(name.to_string());
                        }
                    } else if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        // For simple identifiers
                        let text = text.trim();
                        if !text.is_empty() && !text.contains('(') && !text.contains(')') {
                            params.push(text.to_string());
                        }
                    }
                }
            }
        }

        params
    }

    /// Extract docstring from function
    fn extract_docstring(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: SupportedLanguage,
    ) -> Option<String> {
        match language {
            SupportedLanguage::Python => {
                // Python: docstring is first statement in body
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        if child.kind() == "expression_statement" {
                            if let Some(string_node) = child.child(0) {
                                if string_node.kind() == "string" {
                                    if let Ok(text) = string_node.utf8_text(source.as_bytes()) {
                                        let cleaned = text
                                            .trim_start_matches("\"\"\"")
                                            .trim_end_matches("\"\"\"")
                                            .trim_start_matches("'''")
                                            .trim_end_matches("'''")
                                            .trim();
                                        return Some(cleaned.to_string());
                                    }
                                }
                            }
                        }
                        break; // Only check first statement
                    }
                }
            }
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                // JS/TS: JSDoc comment before function
                if let Some(prev) = node.prev_sibling() {
                    if prev.kind() == "comment" {
                        if let Ok(text) = prev.utf8_text(source.as_bytes()) {
                            if text.starts_with("/**") {
                                let cleaned = text
                                    .trim_start_matches("/**")
                                    .trim_end_matches("*/")
                                    .lines()
                                    .map(|l| l.trim().trim_start_matches('*').trim())
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                return Some(cleaned);
                            }
                        }
                    }
                }
            }
            SupportedLanguage::Rust => {
                // Rust: doc comments before function
                let mut doc_lines = vec![];
                let mut current = node.prev_sibling();
                while let Some(prev) = current {
                    if prev.kind() == "line_comment" {
                        if let Ok(text) = prev.utf8_text(source.as_bytes()) {
                            if text.starts_with("///") || text.starts_with("//!") {
                                doc_lines.insert(0, text.trim_start_matches("///").trim_start_matches("//!").trim());
                            } else {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                    current = prev.prev_sibling();
                }
                if !doc_lines.is_empty() {
                    return Some(doc_lines.join(" "));
                }
            }
            _ => {}
        }
        None
    }

    /// Check if function is async
    fn check_is_async(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: SupportedLanguage,
    ) -> bool {
        match language {
            SupportedLanguage::Python => {
                // Check if parent or node text starts with "async"
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    return text.trim_start().starts_with("async");
                }
            }
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                // Check for async keyword
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "async" {
                        return true;
                    }
                }
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    return text.trim_start().starts_with("async");
                }
            }
            SupportedLanguage::Rust => {
                // Check for async keyword
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "async" {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Check if function is public
    fn check_is_public(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: SupportedLanguage,
    ) -> bool {
        match language {
            SupportedLanguage::Python => {
                // Python: functions starting with _ are private
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                        return !name.starts_with('_');
                    }
                }
                true
            }
            SupportedLanguage::Rust => {
                // Rust: check for pub keyword
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "visibility_modifier" {
                        return true;
                    }
                }
                false
            }
            SupportedLanguage::Java => {
                // Java: check for public/protected/private
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "modifiers" {
                        if let Ok(text) = child.utf8_text(source.as_bytes()) {
                            return text.contains("public");
                        }
                    }
                }
                false
            }
            _ => true, // Default to public for JS/TS/Go
        }
    }

    /// Extract class definitions from AST
    fn extract_classes(
        &self,
        tree: &Tree,
        source: &str,
        language: SupportedLanguage,
    ) -> Vec<ClassInfo> {
        let mut classes = vec![];
        let root = tree.root_node();

        let class_types = match language {
            SupportedLanguage::Python => vec!["class_definition"],
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                vec!["class_declaration"]
            }
            SupportedLanguage::Rust => vec!["struct_item", "impl_item"],
            SupportedLanguage::Go => vec!["type_declaration"],
            SupportedLanguage::Java => vec!["class_declaration"],
        };

        self.traverse_for_nodes(&root, source, &class_types, &mut |node, src| {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node.utf8_text(src.as_bytes()).unwrap_or("").to_string();
                let line_start = node.start_position().row + 1;
                let line_end = node.end_position().row + 1;

                // Extract methods from class body
                let methods = self.extract_class_methods(*node, src, language);

                classes.push(ClassInfo {
                    name,
                    line_start,
                    line_end,
                    methods,
                    docstring: None,
                });
            }
        });

        classes
    }

    /// Extract methods from a class/struct node
    fn extract_class_methods(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
        language: SupportedLanguage,
    ) -> Vec<FunctionInfo> {
        let mut methods = vec![];

        let method_types = match language {
            SupportedLanguage::Python => vec!["function_definition"],
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                vec!["method_definition"]
            }
            SupportedLanguage::Rust => vec!["function_item"],
            SupportedLanguage::Go => vec!["method_declaration"],
            SupportedLanguage::Java => vec!["method_declaration"],
        };

        // Traverse children of class node
        let mut cursor = class_node.walk();
        for child in class_node.children(&mut cursor) {
            if method_types.contains(&child.kind()) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                    let line_start = child.start_position().row + 1;
                    let line_end = child.end_position().row + 1;
                    let parameters = self.extract_parameters(&child, source, language);

                    methods.push(FunctionInfo {
                        name,
                        line_start,
                        line_end,
                        parameters,
                        docstring: None,
                        is_async: false,
                        is_public: true,
                    });
                }
            }
        }

        methods
    }

    /// Extract import statements from AST
    fn extract_imports(
        &self,
        tree: &Tree,
        source: &str,
        language: SupportedLanguage,
    ) -> Vec<ImportInfo> {
        let mut imports = vec![];
        let root = tree.root_node();

        let import_types = match language {
            SupportedLanguage::Python => vec!["import_statement", "import_from_statement"],
            SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                vec!["import_statement"]
            }
            SupportedLanguage::Rust => vec!["use_declaration"],
            SupportedLanguage::Go => vec!["import_declaration"],
            SupportedLanguage::Java => vec!["import_declaration"],
        };

        self.traverse_for_nodes(&root, source, &import_types, &mut |node, src| {
            let text = node.utf8_text(src.as_bytes()).unwrap_or("").to_string();
            let line = node.start_position().row + 1;

            imports.push(ImportInfo {
                module: text,
                items: vec![],
                line,
            });
        });

        imports
    }

    /// Extract comments from AST
    fn extract_comments(
        &self,
        tree: &Tree,
        source: &str,
        _language: SupportedLanguage,
    ) -> Vec<CommentInfo> {
        let mut comments = vec![];
        let root = tree.root_node();

        let comment_types = vec!["comment", "line_comment", "block_comment"];

        self.traverse_for_nodes(&root, source, &comment_types, &mut |node, src| {
            let text = node.utf8_text(src.as_bytes()).unwrap_or("").to_string();
            let line = node.start_position().row + 1;

            // Check if it's a doc comment
            let is_doc = text.starts_with("///")
                || text.starts_with("//!")
                || text.starts_with("/**")
                || text.starts_with("\"\"\"");

            comments.push(CommentInfo {
                text,
                line,
                is_doc_comment: is_doc,
            });
        });

        comments
    }

    /// Traverse AST and collect nodes of specific types
    fn traverse_for_nodes<F>(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        types: &[&str],
        callback: &mut F,
    ) where
        F: FnMut(&tree_sitter::Node, &str),
    {
        if types.contains(&node.kind()) {
            callback(node, source);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_nodes(&child, source, types, callback);
        }
    }

    /// Calculate code complexity metrics
    fn calculate_complexity(
        &self,
        tree: &Tree,
        source: &str,
        _language: SupportedLanguage,
        functions: &[FunctionInfo],
    ) -> ComplexityMetrics {
        let root = tree.root_node();

        // Decision point node types that increase cyclomatic complexity
        let decision_types = vec![
            "if_statement",
            "if_expression",
            "elif_clause",
            "else_clause",
            "for_statement",
            "for_expression",
            "while_statement",
            "while_expression",
            "match_expression",
            "match_arm",
            "switch_statement",
            "case_clause",
            "catch_clause",
            "conditional_expression",
            "ternary_expression",
            "and_expression",
            "or_expression",
            "binary_expression", // Will filter for && and ||
        ];

        let mut cyclomatic = 1; // Base complexity
        let mut max_nesting = 0;

        self.traverse_for_complexity(&root, source, &decision_types, 0, &mut cyclomatic, &mut max_nesting);

        // Calculate LOC (excluding blank lines and comments)
        let loc = source
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with('#')
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with('*')
                    && !trimmed.starts_with("/*")
            })
            .count();

        ComplexityMetrics {
            cyclomatic,
            loc,
            function_count: functions.len(),
            max_nesting,
        }
    }

    /// Traverse AST for complexity calculation
    fn traverse_for_complexity(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        decision_types: &[&str],
        depth: usize,
        cyclomatic: &mut usize,
        max_nesting: &mut usize,
    ) {
        let kind = node.kind();

        // Track nesting depth for control structures
        let is_nesting = matches!(
            kind,
            "if_statement"
                | "for_statement"
                | "while_statement"
                | "match_expression"
                | "switch_statement"
                | "try_statement"
        );

        let new_depth = if is_nesting { depth + 1 } else { depth };
        *max_nesting = (*max_nesting).max(new_depth);

        // Count decision points
        if decision_types.contains(&kind) {
            // Special handling for binary expressions - only count && and ||
            if kind == "binary_expression" {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    if text.contains("&&") || text.contains("||") || text.contains(" and ") || text.contains(" or ") {
                        *cyclomatic += 1;
                    }
                }
            } else {
                *cyclomatic += 1;
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_complexity(&child, source, decision_types, new_depth, cyclomatic, max_nesting);
        }
    }
}

impl Default for CTPParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            SupportedLanguage::from_extension("py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            SupportedLanguage::from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("rs"),
            Some(SupportedLanguage::Rust)
        );
    }

    #[cfg(feature = "python")]
    #[test]
    fn test_parse_python() {
        let mut parser = CTPParser::new().unwrap();
        let source = r#"
def hello(name):
    """Say hello to someone."""
    print(f"Hello, {name}!")

class Greeter:
    def greet(self):
        pass
"#;

        let result = parser.parse(source, SupportedLanguage::Python).unwrap();
        assert!(!result.functions.is_empty());
    }
}
