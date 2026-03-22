# Contributing

Ota is being built as open infrastructure for repo readiness. Contributions should preserve trust, determinism, and a small dependable surface.

## Development workflow

Build and test locally:

```bash
cargo fmt
cargo test
```

Run the CLI during development:

```bash
cargo run -- --help
cargo run -- validate
cargo run -- doctor
```

Validate public examples:

```bash
cargo run -- validate examples/full-contract/ota.yaml
```

## Project priorities

Contributions should reinforce the current V1 path:

- deterministic contract validation
- deterministic task execution
- trustworthy diagnosis
- conservative detection
- clear OSS-facing documentation

Detailed sequencing lives in:

- [docs/planning/v1-phases.md](docs/planning/v1-phases.md)
- [ROADMAP.md](ROADMAP.md)

## Contribution rules

- keep changes phase-aligned
- prefer extending existing command paths over adding parallel flows
- avoid speculative abstractions
- keep docs honest to the shipped implementation
- treat `detect` as trust-sensitive

## Detection changes

Detection work should follow the write gate in:

- [docs/design/detect-write-gate.md](docs/design/detect-write-gate.md)

That means:

- improve `--dry-run` first
- attach provenance and confidence to inferred fields
- use fixtures for real repo shapes
- keep write mode conservative

## Examples and fixtures

If you add new contract surface or detect behavior, update the smallest connected public materials:

- examples under [examples](/Users/bobai/Workspace/Ota.run/ota/examples)
- detect fixtures under [tests/fixtures/detect](/Users/bobai/Workspace/Ota.run/ota/tests/fixtures/detect)
- public docs under [docs](/Users/bobai/Workspace/Ota.run/ota/docs)
