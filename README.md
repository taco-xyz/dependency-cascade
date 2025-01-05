## User Action Flow:
1. Create a dependencies.toml file in the root of each of their modules / services / libraries.
2. Run the CLI command to generate the dependency graph (should be done in the CI/CD pipeline). This will
generate a dependencies.json file with a graph of the dependencies between the nodes.

## Use Cases

### Re-deploying a service when a library is updated
When a library is updated, the service that depends on it should be re-built, re-tested, and re-deployed.

To achieve this, one should create a `dependencies.toml` file in the root of the libraries with their name. Then,
create another `dependencies.toml` file in the root of the services that use those libraries. When the libraries get
updated, the CLI query will return a list of services that depended on the library and must go through the CI/CD pipeline.

### Detecting which end-2-end tests to run
End-2-end tests usually cover multiple services. To detect which of these to run, one should create a `dependencies.toml`
in the root of the tests that cover these specific services with the dependencies pointing to these services. When ANY 
of these services or their upstream dependencies are updated, the CLI query' returned list will include the end-2-end
tests node.

## Flow

1. `dependency-cascade prepare <root_dir>` -> JSON File
    - Find all dependencies.toml files in the repository and parse them into `Node` objects.
    - Throw warnings and errors if anything is anomalous (circular dependencies, dependdency doesn't exist, etc.)
    - Store all the node objects in a JSON file for future analysis. JSON is outputed to the console.
2. `dependency-cascade query <file_path> <file_path> ...`
    - Given a list of file paths, find all the nodes that are affected by the changes in these files, returning a list of
    node names that are affected.

# Using In-line
```
./dependency-cascade.exe query --graph-artifact "$(./dependency-cascade.exe prepare --dir test)" --files test/test_end2end/src/hey.txt
```