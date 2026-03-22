# Ota Quickstart

## Use an existing contract

From a repo with `ota.yaml`:

```bash
cargo run -- validate
cargo run -- tasks
cargo run -- doctor
cargo run -- up
```

Run a task explicitly:

```bash
cargo run -- run test
```

Tasks can use either a single-command `run` or an inline multiline `script`.

## Detect a starting contract

Review first:

```bash
cargo run -- detect --dry-run /path/to/repo
```

Write only if the `high` confidence projection is sufficient:

```bash
cargo run -- detect /path/to/repo
```

Current detect sources:

- `package.json`
- `.nvmrc`
- `.node-version`
- `.tool-versions`
- `pyproject.toml`
- `.python-version`
- `go.mod`

## Example contract

Examples:

- [examples/basic-node/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/basic-node/ota.yaml)
- [examples/basic-python/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/basic-python/ota.yaml)
- [examples/basic-go/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/basic-go/ota.yaml)
- [examples/mixed-node-python/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/mixed-node-python/ota.yaml)
- [examples/fullstack-node-go/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/fullstack-node-go/ota.yaml)
- [examples/full-contract/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/full-contract/ota.yaml)
- [docs/command-reference.md](/Users/bobai/Workspace/Ota.run/ota/docs/command-reference.md)
- [docs/contract-reference.md](/Users/bobai/Workspace/Ota.run/ota/docs/contract-reference.md)
- [docs/philosophy.md](/Users/bobai/Workspace/Ota.run/ota/docs/philosophy.md)
