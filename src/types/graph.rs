use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use petgraph::prelude::*;
use petgraph::{Directed, Direction};
use petgraph::algo::toposort;

pub use super::node::Node;

/// A directed acyclic graph of dependencies, using petgraph.
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyGraph {
    graph: Graph<Node, (), Directed>,
    /// Maps a node's name to its petgraph index.
    name_to_index: HashMap<String, NodeIndex>,
}

#[derive(Debug, thiserror::Error)]
pub enum DependencyGraphCreationError {
    /// A node with the same name was found in the list of nodes.
    #[error("Duplicate node name found: {0}")]
    DuplicateNodeName(String),
    /// A dependency was found that is not in the graph.
    #[error("Dependency '{0}' not in the graph for '{1}' \
             Existing node names: {2}")]
    MissingDependency(String, String, String),
    /// A circular dependency was detected.
    #[error("Circular dependency detected: {0} -> {1}. \
             This means there is a cycle in the dependencies where a node depends on itself \
             either directly or through other nodes.")]
    CircularDependency(String, String),
}

impl DependencyGraph {
    /// Constructs a new `DependencyGraph` from a list of nodes.
    ///
    /// Errors/warnings:
    ///   - Logs an error if duplicate node names are found.
    ///   - Logs a warning if a dependency does not exist in the graph.
    ///   - Logs an error if a circular dependency is detected.
    pub fn new(nodes: Vec<Node>, allow_cyclical: bool) -> Result<Self, DependencyGraphCreationError> {
        let mut graph = Graph::<Node, (), Directed>::new();
        let mut name_to_index = HashMap::new();
        let mut seen_names = HashSet::new();

        
        // First pass: Add all nodes to the graph, check for duplicates.
        for node in &nodes {
            if !seen_names.insert(node.name.clone()) {
                return Err(DependencyGraphCreationError::DuplicateNodeName(node.name.clone()));
            }
        }

        // Second pass: insert them into the graph with an index map.
        for node in nodes.into_iter() {
            let idx = graph.add_node(node.clone());
            name_to_index.insert(node.name, idx);
        }

        // Add edges for dependencies (dep -> node).
        // Warn if a dependency is missing.
        for idx in graph.node_indices() {
            let node = graph[idx].clone();
            let deps = node.dependencies.clone(); // Clone to avoid borrow conflict
            for dep_name in deps {
                match name_to_index.get(&dep_name) {
                    Some(&dep_idx) => {
                        graph.add_edge(dep_idx, idx, ());
                    }
                    None => {
                        return Err(DependencyGraphCreationError::MissingDependency(
                            dep_name,
                            node.name,
                            name_to_index.keys().cloned().collect::<Vec<_>>().join(", ")
                        ));
                    }
                }
            }
        }

        // Check for cycles by trying a toposort.
        if !allow_cyclical {
            if let Err(cycle_err) = toposort(&graph, None) {
                // Find the cycle path by doing a DFS from the problematic node
                // this is important to help the user understand the cycle.
                let mut cycle_path = vec![cycle_err.node_id()];
                let mut current = cycle_err.node_id();
                let mut visited = HashSet::new();
            visited.insert(current);

            'outer: while let Some(neighbors) = graph.neighbors_directed(current, Direction::Outgoing).collect::<Vec<_>>().into_iter().next() {
                current = neighbors;
                if !visited.insert(current) {
                    // Found the cycle, trim the path to just the cycle
                    while cycle_path[0] != current {
                        cycle_path.remove(0);
                    }
                    break 'outer;
                }
                cycle_path.push(current);
            }

            let cycle_names: Vec<_> = cycle_path.iter().map(|&idx| graph[idx].name.as_str()).collect();

            return Err(DependencyGraphCreationError::CircularDependency(
                cycle_names.join(" -> "),
                    cycle_names[0].to_string() // Complete the cycle
                ));
            }
        }

        Ok(Self { graph, name_to_index })
    }
    
    /// Returns the list of nodes that are direct or indirect dependencies of the given node
    /// (i.e. upstream of `node_name`), using a reverse graph traversal.
    #[allow(dead_code)]
    pub fn get_dependencies(&self, node_name: &str) -> Vec<Node> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();

        if let Some(&start_idx) = self.name_to_index.get(node_name) {
            let mut stack = vec![start_idx];

            while let Some(idx) = stack.pop() {
                for neighbor in self
                    .graph
                    .neighbors_directed(idx, Direction::Incoming)
                {
                    if visited.insert(neighbor) {
                        results.push(self.graph[neighbor].clone());
                        stack.push(neighbor);
                    }
                }
            }
        }
        results
    }

    /// Returns the list of nodes that directly or indirectly depend on the given node
    /// (i.e. downstream of `node_name`), using a forward graph traversal.
    pub fn get_dependents(&self, node_name: &str) -> Vec<Node> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();

        if let Some(&start_idx) = self.name_to_index.get(node_name) {
            let mut stack = vec![start_idx];

            while let Some(idx) = stack.pop() {
                for neighbor in self
                    .graph
                    .neighbors_directed(idx, Direction::Outgoing)
                {
                    if visited.insert(neighbor) {
                        results.push(self.graph[neighbor].clone());
                        stack.push(neighbor);
                    }
                }
            }
        }
        results
    }

    /// Retrieves a reference to a node by name.
    pub fn get_node(&self, node_name: &str) -> Option<&Node> {
        self.name_to_index
            .get(node_name)
            .map(|&idx| &self.graph[idx])
    }

    /// Retrieves a list of all nodes in the graph.
    pub fn get_all_nodes(&self) -> Vec<&Node> {
        self.graph.node_indices().map(|idx| &self.graph[idx]).collect()
    }

    /// Returns a list of all affected nodes by a given file change.
    pub fn get_affected_nodes(&self, changed_files: &Vec<PathBuf>) -> Vec<String> {
        let mut affected_nodes = HashSet::new();
        let nodes = self.get_all_nodes();

        for node in nodes.iter() {
            // Check each path individually
            for path in changed_files {
                if node.includes_path(path) {
                    let dependents = self.get_dependents(&node.name);
                    affected_nodes.insert(node.name.clone());
                    for dependent in dependents {
                        affected_nodes.insert(dependent.name.clone());
                    }
                    break; // No need to check other paths for this node
                }
            }
        }

        affected_nodes.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn create_test_node(name: &str, deps: Vec<&str>) -> Node {
        Node::new(
            name.to_string(),
            PathBuf::from(format!("test/{}", name)),
            vec![PathBuf::from("src/**/*")],
            vec![PathBuf::from("test/**/*")],
            deps.into_iter().map(String::from).collect(),
            None
        ).unwrap()
    }

    #[test]
    fn test_graph_creation_success() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a"]),
            create_test_node("c", vec!["b"]),
        ];

        let graph = DependencyGraph::new(nodes, false).unwrap();
        
        assert!(graph.get_node("a").is_some());
        assert!(graph.get_node("b").is_some());
        assert!(graph.get_node("c").is_some());
        assert!(graph.get_node("d").is_none());
    }

    #[test]
    fn test_duplicate_node_name() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("a", vec![]),
        ];

        let err = DependencyGraph::new(nodes, false).unwrap_err();
        assert!(matches!(err, DependencyGraphCreationError::DuplicateNodeName(name) if name == "a"));
    }

    #[test]
    fn test_missing_dependency() {
        let nodes = vec![
            create_test_node("a", vec!["missing"]),
        ];

        let err = DependencyGraph::new(nodes, false).unwrap_err();
        assert!(matches!(err, 
            DependencyGraphCreationError::MissingDependency(dep, node, _) 
            if dep == "missing" && node == "a"
        ));
    }

    #[test]
    fn test_circular_dependency() {
        let nodes = vec![
            create_test_node("a", vec!["b"]),
            create_test_node("b", vec!["c"]),
            create_test_node("c", vec!["a"]),
        ];

        let err = DependencyGraph::new(nodes, false).unwrap_err();
        assert!(matches!(err, DependencyGraphCreationError::CircularDependency(_, _)));
    }

    #[test]
    fn test_cyclical_dependency_allowed() {
        let nodes = vec![
            create_test_node("a", vec!["b"]),
            create_test_node("b", vec!["c"]),
            create_test_node("c", vec!["a"]),
        ];

        let graph = DependencyGraph::new(nodes, true).unwrap();
        assert!(graph.get_node("a").is_some());
    }

    #[test]
    fn test_get_dependencies() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a"]),
            create_test_node("c", vec!["b"]),
            create_test_node("d", vec![]),
        ];

        let graph = DependencyGraph::new(nodes, false).unwrap();
        
        let c_deps: HashSet<_> = graph.get_dependencies("c")
            .into_iter()
            .map(|n| n.name)
            .collect();
        
        assert_eq!(c_deps, HashSet::from_iter(vec!["a".to_string(), "b".to_string()]));
        
        let a_deps: HashSet<_> = graph.get_dependencies("a")
            .into_iter()
            .map(|n| n.name)
            .collect();
        
        assert!(a_deps.is_empty());
    }

    #[test]
    fn test_get_dependents() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a"]),
            create_test_node("c", vec!["b"]),
            create_test_node("d", vec!["a"]),
        ];

        let graph = DependencyGraph::new(nodes, false).unwrap();
        
        let a_dependents: HashSet<_> = graph.get_dependents("a")
            .into_iter()
            .map(|n| n.name)
            .collect();
        
        assert_eq!(a_dependents, HashSet::from_iter(vec!["b".to_string(), "c".to_string(), "d".to_string()]));
        
        let c_dependents: HashSet<_> = graph.get_dependents("c")
            .into_iter()
            .map(|n| n.name)
            .collect();
        
        assert!(c_dependents.is_empty());
    }

    #[test]
    fn test_complex_dependency_chain() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a"]),
            create_test_node("c", vec!["b"]),
            create_test_node("d", vec!["b", "c"]),
            create_test_node("e", vec!["a", "d"]),
        ];

        let graph = DependencyGraph::new(nodes, false).unwrap();
        
        let e_deps: HashSet<_> = graph.get_dependencies("e")
            .into_iter()
            .map(|n| n.name)
            .collect();
        
        assert_eq!(e_deps, HashSet::from_iter(vec![
            "a".to_string(), 
            "b".to_string(), 
            "c".to_string(),
            "d".to_string()
        ]));
    }

    #[test]
    fn test_get_all_nodes() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a"]),
        ];

        let graph = DependencyGraph::new(nodes, false).unwrap();
        let all_nodes = graph.get_all_nodes();
        assert_eq!(all_nodes.len(), 2);
    }

    #[test]
    fn test_get_affected_nodes() {
        let nodes = vec![
            create_test_node("a", vec![]),
            create_test_node("b", vec!["a"]),
            create_test_node("c", vec!["b"]),
        ];

        let graph = DependencyGraph::new(nodes, false).unwrap();
        
        // Test single file change
        let affected = graph.get_affected_nodes(&vec![PathBuf::from("test/a/src/file.rs")]);
        assert_eq!(HashSet::<String>::from_iter(affected.clone()), 
            HashSet::from_iter(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        // Test multiple file changes
        let affected = graph.get_affected_nodes(&vec![
            PathBuf::from("test/a/src/file1.rs"),
        ]);
        assert_eq!(HashSet::<String>::from_iter(affected.clone()),
            HashSet::from_iter(vec!["a".to_string(), "b".to_string(), "c".to_string()]));

        // Test file that matches no nodes
        let affected = graph.get_affected_nodes(&vec![PathBuf::from("test/other/file.rs")]);
        assert!(affected.is_empty());
    }
}

