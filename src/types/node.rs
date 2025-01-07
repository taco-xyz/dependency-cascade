use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// The name of the node. Must be unique among all nodes.
    pub name: String,
    /// Arbitrary JSON metadata (loaded from e.g. dependencies.toml).
    pub metadata: Option<serde_json::Value>,
    /// The path of the node.
    pub path: PathBuf,
    /// The included paths for the node.
    pub included_paths: Vec<PathBuf>,
    /// The excluded paths for the node.
    pub excluded_paths: Vec<PathBuf>,
    /// The names of the nodes this node depends on.
    pub dependencies: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum NodeCreationError {
    #[error("No included paths found for node {0}")]
    NoIncludedPaths(String),
    #[error("Unable to read TOML file: {0}")]
    TomlReadError(#[from] std::io::Error),
    #[error("Failed to parse TOML content: {0}")]
    TomlParseError(#[from] toml::de::Error),
    #[error("Failed to convert metadata to JSON: {0}")]
    MetadataConversionError(#[from] serde_json::Error),
}


// These structs define the shape of the TOML. Adjust as needed.
#[derive(Debug, Deserialize)]
struct TomlRoot {
    module: TomlModule,
    #[serde(default)]
    metadata: Option<toml::Table>,
    #[serde(default)]
    dependencies: HashMap<String, TomlDependency>,
    #[serde(rename = "file_paths", default)]
    file_paths: TomlFilePaths,
}

#[derive(Debug, Deserialize)]
struct TomlModule {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TomlDependency {
    name: String,
}

#[derive(Debug, Deserialize, Default)]
struct TomlFilePaths {
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
}


impl Node {
    pub fn new(name: String, path: PathBuf, included_paths: Vec<PathBuf>, excluded_paths: Vec<PathBuf>, dependencies: Vec<String>, metadata: Option<serde_json::Value>) -> Result<Self, NodeCreationError> {
        
        // Throw an error if there are no included paths
        if included_paths.is_empty() {
            return Err(NodeCreationError::NoIncludedPaths(name));
        }

        Ok(Self { name, path, included_paths, excluded_paths, dependencies, metadata })
    }

    /// Constructs a `Node` by reading and parsing a TOML file.
    ///
    /// # Arguments
    /// * `toml_file_path` - Path to the TOML file to read.
    /// * `node_path` - The path you want to assign to the created `Node`.
    ///
    /// # Returns
    /// A `Result<Node, NodeCreationError>` which, on success, contains a new `Node`
    /// configured by the TOML file.
    pub fn from_toml_str(
        content: &str,
        node_path: PathBuf,
    ) -> Result<Self, NodeCreationError> {
        let parsed: TomlRoot = toml::from_str(content)?;

        let metadata_json = parsed.metadata.map(|m| {
            serde_json::to_value(m).unwrap_or_default()
        });

        // Gather dependency names from the [dependencies] table
        let dependencies = parsed
            .dependencies
            .values()
            .map(|dep| dep.name.clone())
            .collect::<Vec<_>>();

        // Create the node via the existing ::new method
        Node::new(
            parsed.module.name,
            node_path,
            parsed.file_paths.include.iter().map(|s| PathBuf::from(s)).collect(),
            parsed.file_paths.exclude.iter().map(|s| PathBuf::from(s)).collect(),
            dependencies,
            metadata_json,
        )
    }

    /// Returns true if the given path matches any of the included paths and none of the excluded paths.
    /// Paths are checked relative to the node's base path.
    /// 
    /// # Arguments
    /// * `path` - The path to check.
    ///
    /// # Returns
    /// A boolean indicating whether the path is included.
    pub fn includes_path(&self, path: &PathBuf) -> bool {
        // First check if path matches any include pattern
        let matches_include = self.included_paths.iter()
            .any(|pattern| {
                let full_pattern = self.path.join(pattern);
                // println!("full_pattern: {}", full_pattern.to_str().unwrap());
                glob::Pattern::new(full_pattern.to_str().unwrap())
                    .map(|p| p.matches_path(path))
                    .unwrap_or(false)
            });
        
        // println!("matches_include: {}", matches_include);

        // Then check it's not explicitly excluded
        let matches_exclude = self.excluded_paths.iter()
            .any(|pattern| {
                let full_pattern = self.path.join(pattern);
                glob::Pattern::new(full_pattern.to_str().unwrap())
                    .map(|p| p.matches_path(path))
                    .unwrap_or(false)
            });

        matches_include && !matches_exclude
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Node Creation

    #[test]
    fn test_node_creation_success() {
        let node = Node::new(
            "test-node".to_string(),
            PathBuf::from("/path/to/node"),
            vec![PathBuf::from("src/**/*.rs")],
            vec![PathBuf::from("src/excluded")],
            vec!["dep1".to_string()],
            Some(serde_json::json!({"key": "value"}))
        ).unwrap();

        assert_eq!(node.name, "test-node");
        assert_eq!(node.path, PathBuf::from("/path/to/node"));
        assert_eq!(node.included_paths, vec![PathBuf::from("src/**/*.rs")]);
        assert_eq!(node.excluded_paths, vec![PathBuf::from("src/excluded")]);
        assert_eq!(node.dependencies, vec!["dep1"]);
    }

    #[test]
    fn test_node_creation_no_included_paths() {
        let result = Node::new(
            "test-node".to_string(),
            PathBuf::from("/path/to/node"),
            vec![],
            vec![PathBuf::from("src/excluded")],
            vec!["dep1".to_string()],
            None
        );

        assert!(matches!(result, Err(NodeCreationError::NoIncludedPaths(name)) if name == "test-node"));
    }

    // TOML Parsing

    #[test]
    fn test_from_toml_success() {
        let toml = r#"
            [module]
            name = "test-module"

            [dependencies]
            dep1 = { name = "dependency-1" }
            dep2 = { name = "dependency-2" }

            [file_paths]
            include = ["src/**/*.rs", "tests/**/*.rs"]
            exclude = ["target/**"]
        "#;

        let node = Node::from_toml_str(toml, PathBuf::from("/test/path")).unwrap();

        assert_eq!(node.name, "test-module");
        assert_eq!(node.path, PathBuf::from("/test/path"));
        assert_eq!(node.included_paths, vec![PathBuf::from("src/**/*.rs"), PathBuf::from("tests/**/*.rs")]);
        assert_eq!(node.excluded_paths, vec![PathBuf::from("target/**")]);
        assert_eq!(node.dependencies.len(), 2);
        assert!(node.dependencies.contains(&"dependency-1".to_string()));
        assert!(node.dependencies.contains(&"dependency-2".to_string()));
    }

    #[test]
    fn test_from_toml_minimal() {
        let toml = r#"
            [module]
            name = "minimal"

            [file_paths]
            include = ["src/**"]
        "#;

        let node = Node::from_toml_str(toml, PathBuf::from("/test")).unwrap();

        assert_eq!(node.name, "minimal");
        assert_eq!(node.included_paths, vec![PathBuf::from("src/**")]);
        assert!(node.excluded_paths.is_empty());
        assert!(node.dependencies.is_empty());
        assert!(node.metadata.is_none());
    }

    #[test]
    fn test_from_toml_invalid_syntax() {
        let invalid_toml = r#"
            [module
            name = test"
        "#;

        let result = Node::from_toml_str(invalid_toml, PathBuf::from("/test"));
        assert!(matches!(result, Err(NodeCreationError::TomlParseError(_))));
    }

    #[test]
    fn test_from_toml_missing_required() {
        let missing_module = r#"
            [file_paths]
            include = ["src/**"]
        "#;

        let result = Node::from_toml_str(missing_module, PathBuf::from("/test"));
        assert!(matches!(result, Err(NodeCreationError::TomlParseError(_))));
    }

    #[test]
    fn test_from_toml_no_includes() {
        let no_includes = r#"
            [module]
            name = "test"
            
            [file_paths]
            exclude = ["test/**"]
        "#;

        let result = Node::from_toml_str(no_includes, PathBuf::from("/test"));
        assert!(matches!(result, Err(NodeCreationError::NoIncludedPaths(_))));
    }

    #[test]
    fn test_from_toml_complex_metadata() {
        let complex_toml = r#"
            [module]
            name = "complex"

            [metadata]
            nested = { key = "value", num = 42 }
            array = [1, 2, 3]
            string = "test"
            bool = true

            [file_paths]
            include = ["src/**"]
        "#;

        let node = Node::from_toml_str(complex_toml, PathBuf::from("/test")).unwrap();
        let metadata = node.metadata.unwrap();

        assert_eq!(metadata["nested"]["key"], "value");
        assert_eq!(metadata["nested"]["num"], 42);
        assert_eq!(metadata["array"], serde_json::json!([1, 2, 3]));
        assert_eq!(metadata["string"], "test");
        assert_eq!(metadata["bool"], true);
    }

    #[test]
    fn test_includes_path() {
        let node = Node::new(
            "test".to_string(),
            PathBuf::from("test"),
            vec![PathBuf::from("src/**"), PathBuf::from("test/*.rs")],
            vec![PathBuf::from("src/excluded/**")],
            vec![],
            None
        ).unwrap();

        // Should match include pattern
        assert!(node.includes_path(&PathBuf::from("test/src/file.rs")));
        assert!(node.includes_path(&PathBuf::from("test/test/test.rs")));

        // Should not match due to exclude pattern
        assert!(!node.includes_path(&PathBuf::from("test/src/excluded/file.rs")));

        // Should not match any patterns
        assert!(!node.includes_path(&PathBuf::from("test/other/file.rs")));
    }

    #[test]
    fn test_includes_path_no_excludes() {
        let node = Node::new(
            "test".to_string(), 
            PathBuf::from("test"),
            vec![PathBuf::from("src/**")],
            vec![],
            vec![],
            None
        ).unwrap();

        assert!(node.includes_path(&PathBuf::from("test/src/any/path.rs")));
        assert!(!node.includes_path(&PathBuf::from("test/other/path.rs")));
    }

    #[test]
    fn test_includes_path_invalid_pattern() {
        let node = Node::new(
            "test".to_string(),
            PathBuf::from("test"), 
            vec![PathBuf::from("[invalid")],
            vec![],
            vec![],
            None
        ).unwrap();

        assert!(!node.includes_path(&PathBuf::from("test/anything.rs")));
    }
}

