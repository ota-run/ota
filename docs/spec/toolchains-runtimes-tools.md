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

# Toolchains, Runtimes, and Tools

Use this page when you need to decide whether a prerequisite belongs under a managed toolchain,
a simple runtime, a standalone tool, or an OS-native prerequisite.

Current shipped scope:

- top-level `toolchains`
- task-scoped `requirements.toolchains`
- Rustup-backed diagnosis and run-path fulfillment for Rust toolchains
- duplicate-ownership warnings when the same prerequisite is declared under both `toolchains` and
  `runtimes` or `tools`
- one supported contract shape today: `toolchains.rust` with `provider: rustup`

## Ownership rule

Pick the highest useful owner and do not repeat the same capability below it.

- `toolchains` own managed ecosystem environments
- `runtimes` own simple unmanaged runtime version checks
- `tools` own standalone commands on PATH
- `native_prerequisites` own host-native build bundles and shell activations

If a declared toolchain owns the capability, require the toolchain. Do not also require the same
runtime or tool unless it is deliberately standalone outside that toolchain.

## When to use each layer

Use `toolchains` when Ota must understand more than "does this executable exist?"

Examples:

- Rust via Rustup, including components such as `rustfmt`

Current parser boundary:

- the only shipped toolchain contract today is `toolchains.rust` with `provider: rustup`
- future-fit examples for Node, Python, .NET, or Java are ownership-model examples only; they are
  not valid contract entries until ota ships those provider adapters and schema fields

Use `runtimes` when the requirement is simply "this runtime must exist at this version."

Examples:

- `node >=24`
- `python >=3.12`

Use `tools` for standalone commands that are not owned by a declared toolchain.

Examples:

- `docker`
- `jq`
- `gh`

Use `native_prerequisites` for host-native build bundles and shell activation.

Examples:

- Visual Studio Build Tools
- Xcode Command Line Tools
- Linux compiler packages

## Before and after

Before:

```yaml
tools:
  cargo: "*"

tasks:
  setup:
    run: cargo fetch && (rustup component add rustfmt 2>/dev/null || true)
```

After:

```yaml
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
    components:
      - rustfmt
    fulfillment: run

tasks:
  setup:
    requirements:
      toolchains:
        - rust
    run: cargo fetch
```

## Task selection rules

Use the owner that actually applies to the selected task path.

```yaml
tasks:
  setup:
    requirements:
      toolchains:
        - rust
```

```yaml
tasks:
  start:
    requirements:
      runtimes:
        node: ">=24"
```

```yaml
tasks:
  docker:proof:
    requirements:
      tools:
        docker: ">=27"
```

```yaml
tasks:
  install:
    requirements:
      native:
        - node-native-build-tools
```

## Duplication rules

Ota now warns when the same requirement is split across multiple ownership layers.

Current Rustup-first warnings include:

- `toolchains.rust` plus `runtimes.rust`
- `toolchains.rust` plus `tools.cargo`
- `toolchains.rust.components: [rustfmt]` plus `tools.rustfmt`

Treat those warnings as contract cleanup signals, not as extra truth to maintain.

## Fulfillment rules

Toolchain fulfillment is strict:

- `ota doctor` never mutates
- run-path fulfillment only happens when the toolchain declares `fulfillment: run`
- the current shipped provider-backed fulfillment path is `provider: rustup`

For Rustup-backed fulfillment:

- `toolchains.<name>.version` must be an installable Rustup toolchain reference when
  `fulfillment: run` is enabled
- examples: `stable`, `beta`, `nightly`, `1.94.0`
- comparator-style ranges such as `>=1.94` are valid for plain diagnosis elsewhere in Ota, but not
  for Rustup fulfillment because Rustup needs one concrete toolchain reference to install

## Current shipped limits

This slice is intentionally narrow:

- Rustup is the only shipped `toolchains` provider today
- toolchains are selected at the task path, not through execution-context requirements
- duplicate ownership currently warns; it does not hard-fail
- the current shipped Rustup slice owns diagnosis and run-path fulfillment for the declared
  toolchain and its components/targets
- contracts that declare any toolchain other than `toolchains.rust` with `provider: rustup` fail
  validation today

That is enough to remove shell-based Rust component workarounds cleanly without introducing a new
parallel provisioning system.
