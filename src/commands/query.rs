use std::path::Path;
use std::path::PathBuf;
use crate::types::DependencyGraph;
use crate::types::Node;

pub fn get_affected_nodes(graph: &DependencyGraph, changed_files: &Vec<PathBuf>) -> Vec<Node> {
    let affected_nodes = graph.get_affected_nodes(changed_files);
    affected_nodes.iter()
        .filter_map(|name| graph.get_node(name))
        .cloned()
        .collect()
}

