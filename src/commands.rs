use std::path::PathBuf;
use std::fs;

use clap::Subcommand;
use walkdir::WalkDir;

use crate::types::{DependencyGraph, Node};

/// Prepares an artifact of the dependency graph from the given directory.
/// JSON conversion is done in the CLI.
/// 
/// ### Arguments
/// * `dir` - The directory to start the recursive scan from
/// * `dependency_toml_name` - The name of the dependency toml file commmon to all the services. Defaults to `dependencies.toml`
/// 
/// ### Returns
/// * `DependencyGraph` - The dependency graph artifact
pub fn prepare(dir: PathBuf, dependency_toml_name: Option<String>, allow_cyclical: bool) -> Result<DependencyGraph, Box<dyn std::error::Error>> {
    // Recursively walk directory and collect all dependency.toml files as nodes of the graph
    let mut nodes: Vec<Node> = Vec::new();
    for entry in WalkDir::new(&dir) {
        let entry = entry?;
        if entry.file_name().to_string_lossy() == dependency_toml_name.as_deref().unwrap_or("dependencies.toml") {
            let path = entry.path().parent().unwrap().to_path_buf();
            let content = fs::read_to_string(entry.path())?;
            
            // Fix the path to be relative to the root directory
            // NOTE - Surely there is a better way to do this. IDK it's 5:10am
            let path = &path.strip_prefix("./").unwrap_or(&path);
            let path = &path.strip_prefix("/").unwrap_or(&path);
            let path = &path.strip_prefix(".\\").unwrap_or(&path);
            let path = &path.strip_prefix("\\").unwrap_or(&path);

            // Create the node
            let node = Node::from_toml_str(&content, path.to_path_buf())?;
            nodes.push(node);
        }
    }

    // Create dependency graph from nodes
    let graph = DependencyGraph::new(nodes, allow_cyclical)?;

    Ok(graph)
}

/// Queries the dependency graph for the given files.
/// 
/// ### Arguments
/// * `graph` - The dependency graph artifact
/// * `changed_files` - The list of files that have changed
/// 
/// ### Returns
/// * `Vec<Node>` - The list of nodes that are affected by the changes
pub fn query(graph: &DependencyGraph, changed_files: &Vec<PathBuf>) -> Vec<Node> {
    let affected_nodes = graph.get_affected_nodes(changed_files);
    affected_nodes.iter()
        .filter_map(|name| graph.get_node(name))
        .cloned()
        .collect()
}

/// The commands that can be executed by the Clap-based CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// Prepares a dependency graph using all the `dependency.toml` files, starting 
    /// recursively from the given directory. Store the resulting JSON in an 
    /// artifact to use it for other commands.
    Prepare {
        /// The directory to start the recursive scan from.
        #[arg(short, long, value_name = "DIR")]
        dir: PathBuf,
        /// The name of the dependency toml file commmon to all the services. 
        /// Defaults to `dependencies.toml`.
        #[arg(long, value_name = "NAME")]
        dependency_toml_name: Option<String>,
        /// Whether to allow the node dependency graph to be cyclical. Defaults to `false`.
        #[arg(long, value_name = "ALLOW_CYCLICAL")]
        allow_cyclical: bool,
    },
    /// Queries the dependency graph artifact for all the dependency nodes touched by 
    /// the given file changes. HINT: Combo it with `git diff --name-only` to know which 
    /// files have changed, and, consequently, which nodes are affected. Results include 
    /// all the metadata and file paths of the affected nodes.
    /// 
    /// This command only requires the artifact and changed file names, it doesn't need 
    /// to read any files or directories.
    Query {
        /// The JSON artifact file path containing the previously prepared dependency graph 
        /// from the `prepare` command
        #[arg(short, long, value_name = "FILE")]
        graph_artifact_path: PathBuf,
        /// A list of file paths to query.
        #[arg(short, long, value_name = "FILE")]
        files: Vec<PathBuf>,
    },
}
