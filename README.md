# Dependency Cascade

![Dependency Cascade](./image.webp)

**Dependency Cascade** automates and visualizes cross-module dependencies for your monorepo. It helps you:
- Identify and respond to changes in upstream libraries or modules **within** your organization's monorepo.
- Determine which end-to-end test suites need to run based on the modules they cover.

### What it is
- Built in Rust
- Lightweight and easy to use
- No need to learn a new language or build-tool
- Easy to integrate over time with new `dependencies.toml` files
- Works well for distributed teams since they don't need to care about managing upstream dependencies, they can simply specify what
their modules depend on and the tool will indicate when their services need to be re-tested or re-deployed.

### What it isn't
- A build tool (like Bazel)
- Slow (like Bazel)
- It's not like Bazel, seriously.

# Typical Use Cases

### 1. Re-deploying a service when an internal library is updated
1. In each library, create a `dependencies.toml` specifying its name (e.g., `"library-foo"`).
2. In each service, create a `dependencies.toml` listing the libraries it depends on (e.g., `"library-foo"`).
3. Whenever you update `library-foo`, run `dependency-cascade query --files <changed-files>` to see which services depend on it. This lets you:
   - Re-build;
   - Re-test;
   - Re-deploy
   those impacted services automatically.

### 2. Detecting which End-to-End Tests to Run
1. In each end-to-end test suite root, create a `dependencies.toml` listing the services under test.
2. If **any** of those services (or their dependencies) change, a `dependency-cascade query --files <changed-files>` will reveal which test suites must run.

# Installation
> **Assumption**: You have the prebuilt binary or have built from source. Adjust the steps below to match your environment. Go to the [releases page]() to download the pre-built binary.

1. Install `dependency-cascade` in your PATH.
2. Confirm it runs:

```bash
   dependency-cascade --help
```

3. You can run it in-line using the following command:

```bash
dependency-cascade query --graph-artifact "$(dependency-cascade prepare --dir test)" --files test/test_end2end/src/hey.txt test/test_lib/src/hey.txt
```

# How it works
1. You start by creating multiple `dependencies.toml` files in your monorepo. These files are used to specify which other modules each module depends on. A module can be a library, service, a test suite, or whatever you want!
2. You run `dependency-cascade prepare --dir <root-dir>` to generate a JSON file that represents the dependency graph for your entire mono-repo. This command outputs a JSON artifact that you should store for the rest of your build process.
3. Based on the `git diff` of the files that have changed, you can run `dependency-cascade query --graph-artifact <graph-artifact> --files <changed-files>` to see which modules are impacted by the changes.
4. Based on the output of the query, you can decide what to do next. For example, you can re-build, re-test, or re-deploy the impacted modules.
5. **BONUS:** You can encode extra information about your modules in the `metadata` field of the `dependencies.toml` file. This information is returned along with the query results and you can then use that to decide what to do (just a test suite to run? A full service to re-deploy? What's the order in which I should run tests?)
