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

# Contract (`ota.yaml`)

`ota.yaml` is the one file Ota uses to explain a repo to humans, CI, and agents.

Use it when you want:

- deterministic setup instead of tribal knowledge
- one place for runtime, tool, service, and task expectations
- the same contract for local development and automation

## Primary sections

- `version`: contract schema version. Today this is `1`.
- `project`: stable repo identity and high-level classification.
- `runtimes`: required language/runtime versions.
- `tools`: external CLI and tool dependencies.
- `env`: required environment variables, defaults, and allowed values.
- `services`: supporting services such as databases or queues.
- `checks`: explicit preconditions and health checks.
- `tasks`: named commands that humans and agents can run.
- `execution`: where tasks run, such as native, container, or remote.
- `agent`: safe-task and writable-path hints for agents.
- `extensions`: staged extension-contract data that Ota parses but does not execute yet.
- `workspace`: monorepo root/member mapping.

## Quick read

Think about the file in this order:

1. `version` and `project` identify the repo.
2. `runtimes`, `tools`, `env`, and `services` describe what the repo needs.
3. `checks` and `tasks` describe what the repo can verify and run.
4. `execution`, `agent`, and `extensions` describe how Ota should run, expose those actions, and stage future extension behavior.
5. `workspace` is only for monorepo root/member orchestration.

## Example

```yaml
version: 1
project:
  name: example-repo
  type: application
runtimes:
  node: "22"
tools:
  pnpm: "10"
env:
  OTA_ENV:
    required: true
    default: local
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -U app -d app
checks:
  - name: node-installed
    kind: precondition
    severity: error
    run: node --version
tasks:
  setup:
    run: pnpm install
    safe_for_agent: true
execution:
  preferred: native
extensions:
  demo:
    kind: checker
    command: ota-ext-demo
    api_version: 1
agent:
  default_task: setup
workspace:
  type: monorepo
  members:
    - apps/web
    - services/api
```

## What each section means

### `project`

Use `project` for the repo’s stable identity. Keep churn-heavy metadata out of it unless the contract
explicitly needs it.

### `runtimes`

Use `runtimes` for the language/runtime versions the repo needs before it is runnable.

### `tools`

Use `tools` for command-line dependencies that must be present on PATH.

### `env`

Use `env` for required environment values, defaults, and allowed values.

### `services`

Use `services` for supporting infrastructure the repo expects to start, stop, or health-check.

### `checks`

Use `checks` for explicit preconditions and health checks that should be run and reported.

### `tasks`

Use `tasks` for deterministic repo commands such as `setup`, `test`, `lint`, and `dev`.

Example flow:

```yaml
tasks:
  setup:
    run: pnpm install
  build:
    depends_on:
      - setup
    run: pnpm build
  package:
    depends_on:
      - build
    run: tar -czf dist/release.tar.gz dist/
  upload:
    depends_on:
      - package
    run: ./scripts/upload-artifact.sh dist/release.tar.gz
```

### `execution`

Use `execution` to describe where Ota should run those tasks when native execution is not enough.

### `extensions`

Use `extensions` for adapter contract data. Each entry is a typed adapter descriptor with `kind`,
`command`, and `api_version`, plus optional `description` and `config`. Supported kinds today are
`checker` and `publisher`. `checker` is runnable with `ota extensions --run <name>` when
`api_version: 1` is declared. `publisher` is runnable with `ota extensions --publish <name>` when
`api_version: 1` is declared. The validator requires `kind` to be one of the supported kinds,
`command` to be non-empty, and `api_version` to be greater than zero.

Real-world uses include:

- uploading a release artifact bundle to an internal endpoint
- publishing scan or compliance reports through one standard adapter
- exposing a custom checker, codegen helper, or sync tool in a stable contract slot

Example:

```yaml
extensions:
  release-upload:
    kind: publisher
    command: ota-ext-upload
    api_version: 1
    description: Upload the release bundle to the artifact endpoint
    config:
      endpoint: https://artifacts.example.com/upload
      artifact: dist/release.zip
```

Use `ota extensions` to inspect the contract data. Use `ota extensions --run <name>` for
`checker` descriptors and `ota extensions --publish <name>` for `publisher` descriptors.

### `agent`

Use `agent` to tell Ota which tasks are safe for agents and which paths are writable.

### `workspace`

Use `workspace` only for monorepo root/member orchestration across multiple repos.

## Good starting point

Start minimal, then expand:

1. define `project`
2. add required `runtimes`
3. add real `tools`, `env`, and `services`
4. add `checks`
5. add `tasks`
6. add `execution`, `agent`, `extensions`, and `workspace` only when they are actually needed

## Canonical reference

- [Spec contract reference](docs/spec/contract-reference.md)
- [GitHub source](https://github.com/ota-run/ota/blob/main/docs/spec/contract-reference.md)
