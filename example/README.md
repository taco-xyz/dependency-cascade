# Example Explanation
This example shows a simple project with three nodes: `example_app`, `example_lib`, and `example_test`.

- `example_app` depends on `example_lib`
- `example_test` depends on `example_app`

Whenever the contents of `example_app` are changed, `example_test` will show up in `dependency-cascade query`.
If the contents of `example_lib` are changed, both `example_app` and `example_test` will show up in `dependency-cascade query` because `example_test` depends on `example_app`.

This is a simple example, but in the real world, you might have a more complex project with many dependencies and many tests.

# Pipeline Examples

## Github Actions

## Gitlab Pipeline
Help wanted!

## Jenkins Pipeline
Help wanted!

## CircleCI
Help wanted!

