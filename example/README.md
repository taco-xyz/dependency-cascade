# Example Explanation
This example shows a simple project with three nodes: `example_app`, `example_lib`, and `example_test`.

- `example_app` depends on `example_lib`
- `example_test` depends on `example_app`

Whenever the contents of `example_app` are changed, `example_test` will show up in `dependency-cascade query`.
If the contents of `example_lib` are changed, both `example_app` and `example_test` will show up in `dependency-cascade query` because `example_test` depends on `example_app`.

This is a simple example, but in the real world, you might have a more complex project with many dependencies and many tests.

# Pipeline Examples

## Github Actions

```yaml
name: Dependency Cascade Example

on:
  push:
    branches: [ main ]

jobs:
  check-dependencies:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Pull dependency-cascade image
        run: docker pull ghcr.io/tubarao312/dependency-cascade:main
        
      - name: Get changed files # For this step we format each file as a -f argument for dependency-cascade query so that we have an easier time parsing the output
        id: changed-files
        run: |
          if [ "${{ github.event_name }}" == "pull_request" ]; then
            files=$(git diff --name-only origin/${{ github.base_ref }} ${{ github.sha }})
            args=""
            for file in $files; do
              args="$args -f $file"
            done
            echo "files=$args" >> $GITHUB_OUTPUT
          else
            files=$(git diff --name-only ${{ github.event.before }} ${{ github.sha }})
            args=""
            for file in $files; do
              args="$args -f $file"
            done
            echo "files=$args" >> $GITHUB_OUTPUT
          fi

      - name: Prepare dependency graph
        id: prepare
        run: |
          docker run --rm -v $(pwd):/workspace -w /workspace ghcr.io/tubarao312/dependency-cascade:main prepare -d . > dependency-graph.json
          
      - name: Print dependency graph
        run: |
          cat dependency-graph.json

      - name: Print changed files
        run: |
          echo ${{ steps.changed-files.outputs.files }}

      - name: Query affected nodes 
        run: |
          docker run --rm -v $(pwd):/workspace -w /workspace ghcr.io/tubarao312/dependency-cascade:main query -g dependency-graph.json ${{ steps.changed-files.outputs.files }}

```

## Gitlab Pipeline
Help wanted!

## Jenkins Pipeline
Help wanted!

## CircleCI
Help wanted!

