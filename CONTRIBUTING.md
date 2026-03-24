<!--
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Contributing

Ota is being built as open infrastructure for repo readiness. Contributions should preserve trust, determinism, and a small dependable surface.

## Development workflow

Build and test locally:

```bash
cargo fmt
cargo test
```

Run the V4 compatibility gate when changing command behavior, JSON contracts, or output semantics:

```bash
./scripts/test-compat.sh
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

V1 is complete and frozen. New work should reinforce the shipped V1 trust model while following the
active V2 plan.

- deterministic contract validation
- deterministic task execution
- trustworthy diagnosis
- conservative detection
- clear OSS-facing documentation

Detailed sequencing lives in:

- [docs/planning/v1/phases.md](docs/planning/v1/phases.md)
- [docs/planning/v2-plan.md](docs/planning/v2-plan.md)
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
