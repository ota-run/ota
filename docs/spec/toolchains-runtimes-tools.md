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

# Toolchains, Runtimes, Tools, and Orchestrators

Use this page when you need to decide where prerequisite truth belongs.

The canonical ownership model is:

- `toolchains` for required ecosystem capability truth
- `orchestrators` for repo-level trust, prepare, and mediated task execution
- `runtimes` for simple unmanaged runtime checks
- `tools` for standalone commands on PATH
- `native_prerequisites` for host-native build bundles and shell activation

The goal is one owner per capability. If the same truth is declared twice, the contract is weaker,
not safer.

## Canonical toolchain model

`toolchains` are capability-first.

They answer:

- what runtime or ecosystem the repo actually needs
- what package-manager versions that toolchain owns
- whether ota may fulfill that toolchain on the selected run path

Canonical direction:

```yaml
toolchains:
  node:
    version: "24.15.0"
    package_managers:
      pnpm: "10.33.4"
    fulfillment:
      source: mise
      mode: run

  go:
    version: "1.26.0"
```

Use this when ota must understand more than "does this executable exist?"

Examples:

- Rust plus `rustfmt` and target triples
- Node plus declared Corepack package-manager ownership
- Java plus `javac`
- Python plus declared `uv` or `poetry`
- Ruby plus declared Bundler

Do not use `toolchains` when the repo only needs a plain runtime version check and no managed
ecosystem ownership matters.

## Canonical orchestrator model

`orchestrators` capture repo-level mediation such as `mise`, `devbox`, or `devenv`.

They answer:

- what repo-level manager exists
- what config files define it
- whether trust is part of readiness
- whether install/prepare is part of readiness
- whether selected tasks should run through that manager

Canonical shape:

```yaml
orchestrators:
  mise:
    kind: mise
    required: true
    config_files:
      - mise.toml
    activation:
      trust: true
    prepare:
      install: true
```

Use `orchestrators` when the repo truth is not just "install node" or "install python", but
"trust this manager, install through it, and run selected tasks through it."

Additional shipped kinds:

```yaml
orchestrators:
  devbox:
    kind: devbox
    required: true
    config_files:
      - devbox.json
    prepare:
      install: true

  devenv:
    kind: devenv
    required: true
    config_files:
      - devenv.nix
```

Do not force that truth into `run: mise ...` shell strings if ota can model it directly.

## Task mediation through an orchestrator

Tasks can declare orchestrator-mediated execution directly:

```yaml
tasks:
  server:verify:
    context: host
    run: //server:ci-unit
    execution:
      orchestrator:
        ref: mise
        mode: task

  setup:
    context: host
    run: pnpm install
    execution:
      orchestrator:
        ref: mise
        mode: exec
```

Use `mode: task` when the task body is the orchestrator task name.
Use `mode: exec` when the task body is a normal command that should run inside the orchestrated
environment.
Command-backed `prepare.kind: dependency_hydration` and `prepare.kind: tool_bootstrap` also
support `mode: exec` in the current shipped slice.
Mixed/native `prepare.kind: sequence` does not.

Shipped mediation semantics:

- `mise`
  - `mode: task` -> `mise run <task>`
  - `mode: exec` -> `mise exec -- <command>`
  - supports `activation.trust` and `prepare.install`
- `devbox`
  - `mode: task` -> `devbox run <task>`
  - `mode: exec` -> `devbox run -- <command>`
  - supports `prepare.install`
  - does not support `activation.trust` in this slice
- `devenv`
  - `mode: task` -> `devenv tasks run <task>`
  - `mode: exec` -> `devenv shell <command>`
  - does not support `activation.trust` or `prepare.install` in this slice

## Ownership rule

Pick the highest useful owner and do not repeat the same capability below it.

- `toolchains` own managed ecosystem capability truth
- `orchestrators` own repo-level trust, prepare, and mediated task execution
- `runtimes` own simple unmanaged runtime version checks
- `tools` own standalone commands on PATH
- `native_prerequisites` own OS-native bundles

If a declared toolchain owns the capability, do not also declare it under `runtimes` or `tools`.

Examples of invalid duplication:

- `toolchains.node` plus `runtimes.node`
- `toolchains.node` plus `tools.node`
- `toolchains.python` plus `runtimes.python`
- `toolchains.python.package_managers.poetry` plus `tools.poetry`
- `toolchains.rust.components: [rustfmt]` plus `tools.rustfmt`
- `toolchains.ruby.package_managers.bundler` plus `tools.bundler`

Treat duplicate ownership as cleanup work, not extra safety.

## When to use each layer

### `toolchains`

Use when ota should understand ecosystem truth.

Examples:

- Rust via rustup
- Node plus declared package-manager ownership
- Java plus `javac`
- Python plus `uv` or `poetry`
- Ruby plus Bundler

Do not use when the repo only needs a plain runtime check.

### `orchestrators`

Use when one repo-level manager mediates trust, install, or execution.

Examples:

- `mise trust`
- `mise install`
- `mise run //server:ci-unit`
- `mise exec -- pnpm lint`

Do not use as a substitute for `toolchains`. `orchestrators` do not own language capability truth.

### `runtimes`

Use when the repo only needs a simple unmanaged runtime version check.

Examples:

- `node >=24`
- `python >=3.12`
- `pwsh 7.6.0` on Windows only

Do not use when a declared toolchain already owns that runtime.

### `tools`

Use for standalone commands on PATH that are not owned by a declared toolchain.

Examples:

- `docker`
- `gh`
- `jq`
- `maven` when Java stays under `toolchains.java`

Do not use for `node`, `python`, `cargo`, `uv`, `bundler`, or similar toolchain-owned surfaces
when the corresponding toolchain is declared.

### `native_prerequisites`

Use for host-native bundles or shell activation that are not just one runtime or one CLI.

Examples:

- Xcode Command Line Tools
- Visual Studio Build Tools
- Linux compiler packages

## Fulfillment model

`toolchains.<name>.fulfillment` is now structured:

```yaml
fulfillment:
  source: corepack
  mode: run
```

Meaning:

- `mode: none` means diagnose only
- `mode: run` means ota may fulfill the selected toolchain on the selected execution path
- `source` says which fulfillment source ota should use when fulfillment is allowed

Use:

- `mode: none` when the repo requires a toolchain version to exist, but ota should only check and
  report it. This is the right default for repos that rely on host-owned installation, CI runner
  setup, or prebuilt base images.
- `mode: run` when the selected `ota up` or `ota run` path should be allowed to activate or
  provision the toolchain on that path. Use this only when the repo truth is that ota can own that
  fulfillment lane through the declared source.

Supported modes today are only:

- `none`
- `run`

Rules:

- `source` is optional when the toolchain uses its canonical shipped fulfillment path
- `source: mise` is the current non-canonical supported source for repos whose selected path is
  mediated by `mise`
- legacy flat `fulfillment: run` is still accepted for compatibility, but it is not the canonical
  public model and `ota validate` / `ota doctor` now warn and push authors onto structured
  `fulfillment`
- legacy `provider` is still accepted for compatibility, but it is not the canonical public model
  and `ota validate` / `ota doctor` now warn and push authors onto toolchain-owned structured
  `fulfillment`

Canonical shipped fulfillment sources today are:

- `rustup`
- `corepack`
- `sdkman`
- `uv`
- `go`
- `ruby`
- `dotnet`

## Compatibility policy

Public docs, examples, and new contracts should use:

- structured `fulfillment`
- no legacy toolchain `provider`
- explicit `orchestrators` when repo-level mediation exists

Compatibility lanes remain parseable, but validate/doctor now treat them as migration-only and
emit advisories until the contract moves onto the canonical structured shape.

Runtime compatibility still accepts the old provider-based shape temporarily.

That compatibility is a migration lane, not the permanent public truth.

## Before and after

Before:

```yaml
tools:
  mise: "*"

tasks:
  setup:
    run: mise trust && mise install
  server:verify:
    run: mise run //server:ci-unit
```

After:

```yaml
toolchains:
  node:
    version: "24.15.0"
    package_managers:
      pnpm: "10.33.4"
    fulfillment:
      source: mise
      mode: run

orchestrators:
  mise:
    kind: mise
    required: true
    config_files:
      - mise.toml
    activation:
      trust: true
    prepare:
      install: true

tasks:
  server:verify:
    run: //server:ci-unit
    execution:
      orchestrator:
        ref: mise
        mode: task
```

The second shape is better because the repo truth is machine-readable instead of hidden in shell
strings.

## Current shipped slice

Today ota ships:

- top-level `toolchains`
- top-level `orchestrators`
- execution-context-scoped `execution.contexts.<name>.requirements.toolchains`
- task-scoped `requirements.toolchains`
- task-scoped `execution.orchestrator`
- structured `toolchains.<name>.fulfillment`
- canonical fulfillment for Rust, Node, Java, Python, Go, Ruby, and .NET
- non-canonical `fulfillment.source: mise`
- `orchestrators.mise`

This slice is intentionally narrow, but it is the canonical direction.
