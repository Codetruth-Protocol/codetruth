//! Call graph analyzer for CodeTruth
//!
//! Builds a call graph from parsed AST data to enable:
//! - Hot path discovery (most-called functions for critical weight scoring)
//! - Dependency chain analysis (what breaks if X changes)
//! - Dead code detection (unreachable functions)
//! - Cross-file relationship building (for context_bridge integration)

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// A node in the call graph representing a function/method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallNode {
    /// Unique identifier: "file_path::function_name"
    pub id: String,
    /// File where the function is defined
    pub file: String,
    /// Function/method name
    pub name: String,
    /// Language of the source file
    pub language: String,
    /// Line number of definition
    pub line: usize,
    /// Whether this is a public/exported symbol
    pub is_public: bool,
    /// Whether this is an entry point (main, handler, test, etc.)
    pub is_entry_point: bool,
}

/// A directed edge in the call graph: caller → callee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: String,
    pub callee: String,
    /// Line number of the call site
    pub call_line: usize,
    /// Whether this is a direct call, indirect (callback), or dynamic dispatch
    pub call_type: CallType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallType {
    Direct,
    Indirect,
    Dynamic,
}

/// The complete call graph for a codebase
#[derive(Debug, Clone, Default)]
pub struct CallGraph {
    nodes: HashMap<String, CallNode>,
    /// Outgoing edges: caller_id → Vec<CallEdge>
    outgoing: HashMap<String, Vec<CallEdge>>,
    /// Incoming edges: callee_id → Vec<CallEdge>
    incoming: HashMap<String, Vec<CallEdge>>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a function node to the graph
    pub fn add_node(&mut self, node: CallNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add a call edge (caller → callee)
    pub fn add_edge(&mut self, edge: CallEdge) {
        self.incoming
            .entry(edge.callee.clone())
            .or_default()
            .push(edge.clone());
        self.outgoing
            .entry(edge.caller.clone())
            .or_default()
            .push(edge);
    }

    /// Get all nodes
    pub fn nodes(&self) -> &HashMap<String, CallNode> {
        &self.nodes
    }

    /// Get callers of a function (who calls this?)
    pub fn callers_of(&self, callee_id: &str) -> Vec<&CallEdge> {
        self.incoming
            .get(callee_id)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Get callees of a function (what does this call?)
    pub fn callees_of(&self, caller_id: &str) -> Vec<&CallEdge> {
        self.outgoing
            .get(caller_id)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Find entry points (functions with no callers, or marked as entry points)
    pub fn entry_points(&self) -> Vec<&CallNode> {
        self.nodes
            .values()
            .filter(|node| {
                node.is_entry_point
                    || !self.incoming.contains_key(&node.id)
                    || self.incoming.get(&node.id).map_or(true, |e| e.is_empty())
            })
            .collect()
    }

    /// Find dead code (non-entry-point functions with no callers)
    pub fn dead_functions(&self) -> Vec<&CallNode> {
        self.nodes
            .values()
            .filter(|node| {
                !node.is_entry_point
                    && !node.is_public
                    && (!self.incoming.contains_key(&node.id)
                        || self.incoming.get(&node.id).map_or(true, |e| e.is_empty()))
            })
            .collect()
    }

    /// Calculate "hotness" score for each function based on incoming call count
    /// Higher score = more callers = more critical
    pub fn hotness_scores(&self) -> HashMap<String, f64> {
        let max_callers = self
            .incoming
            .values()
            .map(|edges| edges.len())
            .max()
            .unwrap_or(1) as f64;

        self.nodes
            .keys()
            .map(|id| {
                let caller_count = self
                    .incoming
                    .get(id)
                    .map(|e| e.len())
                    .unwrap_or(0) as f64;
                let score = caller_count / max_callers;
                (id.clone(), score)
            })
            .collect()
    }

    /// Find all transitive dependencies of a function (what could break if this changes?)
    pub fn transitive_callers(&self, node_id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut stack = vec![node_id.to_string()];

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(edges) = self.incoming.get(&current) {
                for edge in edges {
                    if !visited.contains(&edge.caller) {
                        stack.push(edge.caller.clone());
                    }
                }
            }
        }

        visited.remove(node_id);
        visited
    }

    /// Get cross-file dependencies: which files depend on which other files
    pub fn file_dependencies(&self) -> HashMap<String, HashSet<String>> {
        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();

        for edges in self.outgoing.values() {
            for edge in edges {
                if let (Some(caller_node), Some(callee_node)) =
                    (self.nodes.get(&edge.caller), self.nodes.get(&edge.callee))
                {
                    if caller_node.file != callee_node.file {
                        deps.entry(caller_node.file.clone())
                            .or_default()
                            .insert(callee_node.file.clone());
                    }
                }
            }
        }

        deps
    }

    /// Get statistics about the call graph
    pub fn stats(&self) -> CallGraphStats {
        CallGraphStats {
            total_functions: self.nodes.len(),
            total_calls: self.outgoing.values().map(|e| e.len()).sum(),
            entry_points: self.entry_points().len(),
            dead_functions: self.dead_functions().len(),
            files: self.nodes.values().map(|n| &n.file).collect::<HashSet<_>>().len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallGraphStats {
    pub total_functions: usize,
    pub total_calls: usize,
    pub entry_points: usize,
    pub dead_functions: usize,
    pub files: usize,
}

// ---------------------------------------------------------------------------
// Builder — constructs call graph from parsed AST data
// ---------------------------------------------------------------------------

/// Builds a call graph from source files.
///
/// Currently a skeleton that will be filled in with tree-sitter AST walking
/// for each supported language.
pub struct CallGraphBuilder {
    graph: CallGraph,
}

impl CallGraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: CallGraph::new(),
        }
    }

    /// Add a function definition found via AST parsing
    pub fn add_function(
        &mut self,
        file: &str,
        name: &str,
        line: usize,
        language: &str,
        is_public: bool,
        is_entry_point: bool,
    ) {
        let id = format!("{}::{}", file, name);
        self.graph.add_node(CallNode {
            id,
            file: file.to_string(),
            name: name.to_string(),
            language: language.to_string(),
            line,
            is_public,
            is_entry_point,
        });
    }

    /// Add a call relationship found via AST parsing
    pub fn add_call(
        &mut self,
        caller_file: &str,
        caller_name: &str,
        callee_file: &str,
        callee_name: &str,
        call_line: usize,
        call_type: CallType,
    ) {
        let caller_id = format!("{}::{}", caller_file, caller_name);
        let callee_id = format!("{}::{}", callee_file, callee_name);
        self.graph.add_edge(CallEdge {
            caller: caller_id,
            callee: callee_id,
            call_line,
            call_type,
        });
    }

    /// Build and return the completed call graph
    pub fn build(self) -> CallGraph {
        self.graph
    }
}

impl Default for CallGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_graph() -> CallGraph {
        let mut builder = CallGraphBuilder::new();

        builder.add_function("src/main.rs", "main", 1, "rust", true, true);
        builder.add_function("src/auth.rs", "login", 10, "rust", true, false);
        builder.add_function("src/auth.rs", "hash_password", 30, "rust", false, false);
        builder.add_function("src/db.rs", "query_user", 5, "rust", true, false);
        builder.add_function("src/utils.rs", "unused_helper", 1, "rust", false, false);

        builder.add_call("src/main.rs", "main", "src/auth.rs", "login", 5, CallType::Direct);
        builder.add_call("src/auth.rs", "login", "src/auth.rs", "hash_password", 15, CallType::Direct);
        builder.add_call("src/auth.rs", "login", "src/db.rs", "query_user", 20, CallType::Direct);

        builder.build()
    }

    #[test]
    fn test_entry_points() {
        let graph = build_test_graph();
        let entries = graph.entry_points();
        assert!(entries.iter().any(|n| n.name == "main"));
        assert!(entries.iter().any(|n| n.name == "unused_helper"));
    }

    #[test]
    fn test_dead_functions() {
        let graph = build_test_graph();
        let dead = graph.dead_functions();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].name, "unused_helper");
    }

    #[test]
    fn test_callers_of() {
        let graph = build_test_graph();
        let callers = graph.callers_of("src/auth.rs::hash_password");
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].caller, "src/auth.rs::login");
    }

    #[test]
    fn test_hotness_scores() {
        let graph = build_test_graph();
        let scores = graph.hotness_scores();
        // login is called by main (1 caller)
        // hash_password is called by login (1 caller)
        // query_user is called by login (1 caller)
        // main and unused_helper have 0 callers
        assert_eq!(*scores.get("src/main.rs::main").unwrap(), 0.0);
        assert!(*scores.get("src/auth.rs::login").unwrap() > 0.0);
    }

    #[test]
    fn test_transitive_callers() {
        let graph = build_test_graph();
        let callers = graph.transitive_callers("src/db.rs::query_user");
        assert!(callers.contains("src/auth.rs::login"));
        assert!(callers.contains("src/main.rs::main"));
    }

    #[test]
    fn test_file_dependencies() {
        let graph = build_test_graph();
        let deps = graph.file_dependencies();
        let main_deps = deps.get("src/main.rs").unwrap();
        assert!(main_deps.contains("src/auth.rs"));

        let auth_deps = deps.get("src/auth.rs").unwrap();
        assert!(auth_deps.contains("src/db.rs"));
    }

    #[test]
    fn test_stats() {
        let graph = build_test_graph();
        let stats = graph.stats();
        assert_eq!(stats.total_functions, 5);
        assert_eq!(stats.total_calls, 3);
        assert_eq!(stats.dead_functions, 1);
        assert_eq!(stats.files, 4);
    }
}
