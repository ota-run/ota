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
- Corepack-backed diagnosis for Node toolchains
- uv-backed diagnosis and run-path fulfillment for Python toolchains
- Go-backed diagnosis for Go toolchains
- Ruby-backed diagnosis for Ruby toolchains
- hard validation errors when the same prerequisite is declared under both `toolchains` and
  `runtimes` or `tools`
- six supported contract shapes today:
  - `toolchains.rust` with `provider: rustup`
  - `toolchains.node` with `provider: corepack`
  - `toolchains.java` with `provider: sdkman`
  - `toolchains.python` with `provider: uv`
  - `toolchains.go` with `provider: go`
  - `toolchains.ruby` with `provider: ruby`

## Ownership rule

Pick the highest useful owner and do not repeat the same capability below it.

- `toolchains` own managed ecosystem environments
- `runtimes` own simple unmanaged runtime version checks
- `tools` own standalone commands on PATH
- `native_prerequisites` own host-native build bundles and shell activations
- current shipped ownership is provider-defined, not free-form: today Ota derives Rust capability
  ownership from `toolchains.rust` with `provider: rustup` and Node runtime/executable plus
  declared Corepack package-manager ownership from `toolchains.node` with `provider: corepack`

If a declared toolchain owns the capability, require the toolchain. Do not also require the same
runtime or tool unless it is deliberately standalone outside that toolchain.

## When to use each layer

Use `toolchains` when Ota must understand more than "does this executable exist?"

Examples:

- Rust via Rustup, including components such as `rustfmt`
- Node via Corepack, when the repo wants one owner for the Node runtime, `node` executable, and
  declared Corepack package-manager activation
- Python via uv, when the repo wants one owner for the Python runtime while tools such as Poetry
  remain standalone under `tools`
- Go via the built-in Go provider, when the repo wants one owner for the Go runtime boundary
- Ruby via the built-in Ruby provider, when the repo wants one owner for the Ruby runtime boundary

Current parser boundary:

- the shipped toolchain contracts today are `toolchains.rust` with `provider: rustup`,
  `toolchains.node` with `provider: corepack`, `toolchains.java` with `provider: sdkman`,
  `toolchains.python` with `provider: uv`, `toolchains.go` with `provider: go`, and
  `toolchains.ruby` with `provider: ruby`
- those shipped contracts are fixed name/provider pairs: `toolchains.rust` must use `provider: rustup`,
  `toolchains.node` must use `provider: corepack`, `toolchains.java` must use `provider: sdkman`,
  `toolchains.python` must use `provider: uv`, `toolchains.go` must use `provider: go`, and
  `toolchains.ruby` must use `provider: ruby`
- the shared provider-agnostic toolchain fields are currently `provider`, `version`,
  `fulfillment`, `required`, `only_on`, and `platforms.<os>.version`
- validation and command behavior read from a provider contract, not from free-form capability
  text; today those contracts are the shipped Rustup/Corepack/sdkman/uv/Go/Ruby slices behind
  `toolchains.rust`, `toolchains.node`, `toolchains.java`, `toolchains.python`,
  `toolchains.go`, and `toolchains.ruby`
- that provider contract also owns Rustup field-shape validation, so empty `profile`,
  `components`, or `targets` entries fail as provider-contract violations rather than generic
  schema drift
- Corepack-backed Node toolchains currently support one provider-specific field:
  `package_managers` (plus `platforms.<os>.package_managers`) and stay check-only
- Ruby-backed toolchains support `package_managers` too, but only `bundler` is valid there; use it
  to make Bundler version governance explicit under `toolchains.ruby`
- `profile`, `components`, and `targets` are Rustup-specific compatibility fields, not a generic
  ecosystem-wide toolchain schema
- future-fit examples for .NET remain ownership-model examples only; they are not valid contract
  entries until ota ships those provider adapters and schema fields

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

Ota now rejects the same requirement when it is split across multiple ownership layers.

Current Rustup-first invalid combinations include:

- `toolchains.rust` plus `runtimes.rust`
- `toolchains.rust` plus `tools.cargo`
- `toolchains.rust.components: [rustfmt]` plus `tools.rustfmt`
- `toolchains.python` plus `runtimes.python`

Treat those validation errors as contract cleanup signals, not as extra truth to maintain.

## Fulfillment rules

Toolchain fulfillment is strict:

- `ota doctor` never mutates
- run-path fulfillment only happens when the toolchain declares `fulfillment: run`
- the current shipped provider-backed fulfillment paths are `provider: rustup` and `provider: uv`

For Rustup-backed fulfillment:

- `toolchains.<name>.version` must be an installable Rustup toolchain reference when
  `fulfillment: run` is enabled
- examples: `stable`, `beta`, `nightly`, `1.94.0`
- comparator-style ranges such as `>=1.94` are valid for plain diagnosis elsewhere in Ota, but not
  for Rustup fulfillment because Rustup needs one concrete toolchain reference to install

For uv-backed Python fulfillment:

- `toolchains.python.version` must be one installable uv Python reference when
  `fulfillment: run` is enabled
- examples: `3.12`, `3.12.10`, `3.13`
- comparator-style ranges such as `>=3.12,<3.14` are valid for plain diagnosis elsewhere in Ota,
  but not for uv fulfillment because `uv python install` needs one concrete Python reference to
  install

## Current shipped limits

This slice is intentionally narrow:

- shipped toolchain contracts today are:
  - `toolchains.rust` with `provider: rustup`
  - `toolchains.node` with `provider: corepack`
  - `toolchains.java` with `provider: sdkman`
  - `toolchains.python` with `provider: uv`
  - `toolchains.go` with `provider: go`
  - `toolchains.ruby` with `provider: ruby`
- toolchains are selected at the task path, not through execution-context requirements
- duplicate ownership is invalid and fails validation
- Rustup currently owns diagnosis plus run-path fulfillment for the declared toolchain and its
  components/targets
- uv currently owns diagnosis plus run-path fulfillment for the declared Python runtime version
- org-policy version/provisioning reasoning now sees the selected toolchain-owned runtime lane too,
  so approved runtime versions and approved install sources can govern `toolchains.rust`,
  `toolchains.node`, `toolchains.java`, `toolchains.python`, `toolchains.go`, or
  `toolchains.ruby` without re-declaring duplicate runtime ownership
- Corepack-backed Node toolchains are currently diagnosis-only; `tools.node` is invalid duplicate
  ownership, and package managers declared under `toolchains.node.package_managers` must not be
  redeclared under `tools`
- Ruby-backed toolchains are currently check-only; `tools.bundler` is invalid duplicate ownership,
  and Bundler should be modeled under `toolchains.ruby.package_managers.bundler`
- contracts that declare any other toolchain/provider combination fail validation today

That is enough to remove shell-based Rust component workarounds cleanly without introducing a new
parallel provisioning system.
