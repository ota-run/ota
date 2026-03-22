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
cargo run -- init
cargo run -- detect --dry-run /path/to/repo
```

Write only if the `high` confidence projection is sufficient:

```bash
cargo run -- init --write
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
- `settings.gradle(.kts)`
- `build.gradle(.kts)`
- `gradle/wrapper/gradle-wrapper.properties`
- `pom.xml`

## Example contract

Examples:

- [../../examples/basic-node/ota.yaml](../../examples/basic-node/ota.yaml)
- [../../examples/basic-python/ota.yaml](../../examples/basic-python/ota.yaml)
- [../../examples/basic-go/ota.yaml](../../examples/basic-go/ota.yaml)
- [../../examples/mixed-node-python/ota.yaml](../../examples/mixed-node-python/ota.yaml)
- [../../examples/fullstack-node-go/ota.yaml](../../examples/fullstack-node-go/ota.yaml)
- [../../examples/full-contract/ota.yaml](../../examples/full-contract/ota.yaml)
- [../spec/command-reference.md](../spec/command-reference.md)
- [../spec/contract-reference.md](../spec/contract-reference.md)
- [philosophy.md](philosophy.md)
