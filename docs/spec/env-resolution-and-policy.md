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

# Environment Variables

Status: spec candidate.

This document explains how ota chooses and prioritizes environment variables when a task runs.

The named examples in this page, like `DATABASE_URL`, `JAVA_HOME`, `AWS_PROFILE`, and `PATH`, are
examples only. The rules apply to any env variable the contract declares.

## Contract Shape

The shipped contract now treats environment requirements and environment sources as separate
concepts:

- `env.vars` declares which values the repo needs
- `env.sources` declares where ota may read values from

`policies.env.values` is the org-level approved-value map. See [`policy-packs.md`](policy-packs.md)
for how ota finds the policy pack itself.
Repo contracts do not declare `policies.env`; approved shared values live in `.ota/org-policy.yaml`.

`PATH` is still the special env key that supports `prepend` and `append`. Ordinary env vars such as
`JAVA_HOME` should stay as single explicit values.

## Why Root `env` Exists

Root `env` is not only a validation surface. It is the repo-wide environment contract ota uses for
real execution.

- it tells ota which env names belong to repo readiness
- it lets ota resolve those names from task env, policy env, the shell, declared sources, and
  defaults in one deterministic order
- it lets ota enforce `required`, `allowed`, `secret`, and `PATH` composition before the task starts
- it gives `doctor`, `ota env`, receipts, and JSON output one honest source of truth

Without root `env`, ota still inherits whatever the shell already has, but ota does not know that
those values are part of repo truth. That means no repo-level provenance, no allowed-value checks,
no declared source support, no policy-supplied shared values, and no secret-aware output.

## What ota Injects

When ota starts a process through `ota run` or `ota up`, it resolves declared root env values and
passes the winning values into the spawned process environment.

- this is how Node sees values in `process.env`
- the same applies to Python, Go, Java, Rust, shell scripts, and other stacks launched by ota
- `PATH` is the one special env name that ota can compose with `prepend` and `append`

This is process-scoped execution injection, not machine-wide mutation.

- ota does not rewrite your source code
- ota does not create a `.env` file for you
- ota does not permanently change your shell session
- values are applied to the process ota starts, not to unrelated processes outside ota

If a repo standardizes on `ota run` or `ota up`, ota can functionally replace in-app dotenv loading
for many projects. If developers still start the app directly outside ota, then shell exports or the
app's own dotenv loader may still matter.

## Fields

### `env.vars.<NAME>`

- `required`: whether the value must resolve
- `secret`: whether ota should redact the value in output and receipts
- `default`: fallback value when no higher-precedence source resolves it
- `allowed`: fixed allowed values
- `prepend`: `PATH`-only entries to place before the resolved base value
- `append`: `PATH`-only entries to place after the resolved base value

What these mean in practice:

- `required`: the env var must resolve or ota fails readiness or execution
- `secret`: ota should treat the value as sensitive and redact it in output and receipts
- `default`: use this fallback only when no higher-precedence layer wins
- `allowed`: the winning value must be one of the declared values

`env.vars` is a requirement map, not a raw assigned-value map.

Valid:

```yaml
env:
  vars:
    DISCORD_CHANNEL:
      default: general
```

Invalid intent:

```yaml
env:
  vars:
    DISCORD_CHANNEL: general
```

Use `default` when the contract itself provides a fallback. Use `tasks.<name>.env`,
`policies.env.values`, the shell, or `env.sources` when another layer should supply the value.

Example:

```yaml
env:
  vars:
    DISCORD_TOKEN:
      required: true
      secret: true
    RELEASE_CHANNEL:
      default: stable
      allowed:
        - stable
        - canary
```

This means:

- `DISCORD_TOKEN` must resolve and should not be shown in plain text
- `RELEASE_CHANNEL` falls back to `stable`
- `RELEASE_CHANNEL=beta` is rejected even if it came from policy, the shell, or a declared source file

### `env.sources[]`

- `kind`: curated source type. Today ota ships `dotenv`, `properties`, `json`, `yaml`, and `toml`
- `path`: source path relative to the contract directory
- `must_exist`: whether the source artifact itself is part of readiness

`must_exist` is about the file, not about a particular env var.

Example:

```yaml
env:
  vars:
    DISCORD_TOKEN:
      required: true
      secret: true
    CRON_TIMEZONE:
      default: Africa/Lagos
  sources:
    - kind: dotenv
      path: .env.local
    - kind: dotenv
      path: .env
      must_exist: true
```

This means:

- ota may read values from `.env.local`, then `.env`
- `.env.local` is optional
- `.env` itself must exist
- `DISCORD_TOKEN` still has to resolve from policy env, the shell, a declared source, or a default

Detect/init onboarding inference:

- `ota detect` and detector-led `ota init` may infer curated `env.sources` entries from known standard files only
- today that curated list is `.env.local`, `.env`, `src/main/resources/application.properties`, `src/main/resources/application.yml`, `src/main/resources/application.yaml`, `appsettings.json`, and `appsettings.Development.json`
- `toml` is supported at runtime when declared, but this curated detect/init inference list does not yet auto-suggest standard TOML paths
- inferred entries carry the same provenance/confidence model as other detect/init fields
- runtime commands do not auto-discover those files; `ota run`, `ota up`, `ota env`, and `ota doctor` only load sources that are already declared in `env.sources`
- merge/apply stays explicit: inferred sources are reviewed and written through the existing detect/init flows instead of being silently loaded at execution time

## Resolution Order

When a repo declares an env name in `env.vars`, repo-level commands resolve it in this order:

1. `tasks.<name>.env` for the task that declares it
2. `policies.env.values`
3. the shell process environment
4. declared `env.sources`, in order
5. the contract `default`

Workspace commands add one higher-priority shared layer:

1. `tasks.<name>.env` for the task that declares it
2. `ota.workspace.yaml` `policies.env.values`
3. org policy `policies.env.values`
4. the shell process environment
5. declared `env.sources`, in order
6. the contract `default`

If none of those provide a value and the env is required, validation or execution fails.

Execution env layers are separate from root `env.vars` resolution.

- root `env.vars` decides which repo-owned values are real readiness and execution inputs
- `execution.contexts.<name>.env` adds context-wide execution defaults
- `tasks.<name>.env` overrides those defaults for one task
- `tasks.<name>.execution.modes.<mode>.env` overrides the selected execution mode branch
- ota also injects `OTA_WORKSPACE` automatically and derives fallback cache env for known
  attachment/tool pairs such as `.m2`, `.npm`, `.pnpm-store`, `.gradle`, `.pip-cache`, and
  `.pypoetry-cache`

Execution env precedence for those injected layers is:

1. selected task mode env
2. task env
3. context env
4. ota-derived fallback execution env

If a declared source is present but invalid, ota fails instead of silently skipping it.
If a declared source has `must_exist: true` and is missing, ota reports that as a readiness failure
even if another layer provides the env value.

Declared source loading is strict and deterministic:

- `dotenv` uses dotenv parsing
- `properties` uses a flat Java-properties style source
- `json` requires an object root, `yaml` requires an object root, and `toml` requires a table root
- structured sources flatten nested maps with `.`, then normalize keys into env names
- arrays, object leaves, and `null` values are rejected explicitly
- unsupported structured scalar classes such as TOML datetimes are rejected explicitly
- if two keys inside one source normalize to the same env key, ota fails that source instead of guessing

Remote execution caveat:

- ota redacts `secret: true` values in output and receipts
- ota rejects secret env values for remote shell execution instead of inlining them into remote
  command strings

## What To Put Where

- use `env.vars` in `ota.yaml` to say which values the repo needs
- use `env.sources` when the repo intentionally relies on declared source files such as dotenv,
  properties, json, yaml, or toml
- use `required: true` when a value must exist
- use `default` when the repo has a safe fallback
- use `allowed` when the value must stay within a fixed set
- use `tasks.<name>.env` when one task needs a fixed override
- use `policies.env.values` when the organization wants an approved shared value
- use `prepend` and `append` only on `PATH`

Use root `env` when the repo needs env values to stay visible and enforceable at execution time,
not just when you want validation.

## Examples

```yaml
env:
  vars:
    DATABASE_URL:
      required: true
    JAVA_HOME:
      default: /opt/jdk-21
    PATH:
      prepend:
        - ./node_modules/.bin
  sources:
    - kind: dotenv
      path: .env.local
    - kind: dotenv
      path: .env
tasks:
  test:
    env:
      CI: "true"
    run: pnpm test
```

```yaml
policies:
  env:
    values:
      JAVA_HOME: /opt/jdk-22
      AWS_PROFILE: ota-prod
```

```yaml
env:
  vars:
    DOCS_SITE_BASE_URL:
      required: true
    RELEASE_CHANNEL:
      default: stable
      allowed:
        - stable
        - canary
  sources:
    - kind: dotenv
      path: .env.local

policies:
  env:
    values:
      DOCS_SITE_BASE_URL: https://docs.internal.example
      RELEASE_CHANNEL: stable
```

This means:

- `DOCS_SITE_BASE_URL` is a repo requirement
- the org policy pack can satisfy it without relying on the shell or declared source files
- `RELEASE_CHANNEL` still has to pass `allowed`, regardless of whether it came from policy, the shell, or a declared source

```yaml
policies:
  env:
    values:
      OTA_TEST_SHARED: workspace-policy
```

In `ota.workspace.yaml`, that shape supplies workspace-wide shared values that outrank the org policy
pack during `ota workspace ...` commands.

```yaml
# ota.workspace.yaml
policies:
  env:
    values:
      RELEASE_CHANNEL: canary
      DOCS_SITE_BASE_URL: https://workspace.internal.example
```

```yaml
# repo ota.yaml
env:
  vars:
    DOCS_SITE_BASE_URL:
      required: true
    RELEASE_CHANNEL:
      default: stable
      allowed:
        - stable
        - canary
  sources:
    - kind: dotenv
      path: .env.local
```

This means:

- `ota workspace ...` prefers the workspace policy values first
- the org policy pack is still consulted after the workspace policy layer
- the repo contract still defines which env names are real readiness requirements
- the shell, declared env sources, and contract defaults remain lower-precedence fallbacks

If the process `PATH` is:

```text
/usr/local/bin:/usr/bin:/bin
```

and the contract says:

```yaml
env:
  vars:
    PATH:
      prepend:
        - ./node_modules/.bin
```

then the final `PATH` is:

```text
./node_modules/.bin:/usr/local/bin:/usr/bin:/bin
```

## Where It Shows Up

- `doctor` diagnoses missing or invalid env values and reports missing or broken declared sources
- `ota env` shows both declared source status and the winning source for each env var
- `run` and `up` inject the resolved env into the spawned task process during execution
- receipts show which value won
- `explain` should turn env failures into a fix plan

Example `ota env` JSON shape:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "contract_count": 3,
    "source_count": 2,
    "source_issue_count": 0,
    "task_count": 1,
    "resolved_count": 3,
    "missing_count": 0,
    "invalid_count": 0
  },
  "sources": [
    {
      "kind": "properties",
      "path": "app.properties",
      "label": "properties:app.properties",
      "must_exist": false,
      "status": "loaded"
    },
    {
      "kind": "json",
      "path": "env/runtime.json",
      "label": "json:env/runtime.json",
      "must_exist": true,
      "status": "loaded"
    }
  ],
  "env": [
    {
      "name": "DISCORD_TOKEN",
      "kind": "contract",
      "required": true,
      "value": "***",
      "source": "properties:app.properties",
      "source_kind": "properties",
      "source_path": "app.properties",
      "source_status": "loaded",
      "source_label": "properties:app.properties",
      "status": "resolved"
    },
    {
      "name": "DOCS_SITE_BASE_URL",
      "kind": "contract",
      "required": true,
      "value": "https://docs.internal.example",
      "source": "org policy",
      "status": "resolved"
    },
    {
      "name": "CI",
      "kind": "task",
      "required": false,
      "value": "true",
      "source": "task",
      "status": "task"
    }
  ]
}
```

Canonical declared-source status values are:

- `loaded`
- `missing_optional`
- `missing_required`
- `parse_failed`
- `invalid_structure`
- `collision`

## What Policy Is Not

- not a replacement for `env.sources`
- not a general application config system
- not silent env mutation
- not hosted control-plane workflow logic
- not waiver or approval orchestration
- not fleet reporting or retention policy
