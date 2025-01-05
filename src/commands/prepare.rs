use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;
use crate::types::{DependencyGraph, Node};

pub fn create_graph_from_dir(root_dir: PathBuf, dependency_toml_name: Option<String>) -> Result<DependencyGraph, Box<dyn std::error::Error>> {
    // Recursively walk directory and collect all dependency.toml files
    let mut nodes = Vec::new();
    
    for entry in WalkDir::new(root_dir) {
        let entry = entry?;
        if entry.file_name().to_string_lossy() == dependency_toml_name.as_deref().unwrap_or("dependencies.toml") {
            let content = fs::read_to_string(entry.path())?;
            let node = Node::from_toml_str(&content, entry.path().parent().unwrap().to_path_buf())?;
            nodes.push(node);
        }
    }

    // Create dependency graph from nodes
    let graph = DependencyGraph::new(nodes)?;

    Ok(graph)
}
