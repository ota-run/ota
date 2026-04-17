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

`policies.env` remains the org-level approved-value map. See [`policy-packs.md`](policy-packs.md)
for how ota finds the policy pack itself.

`PATH` is still the special env key that supports `prepend` and `append`. Ordinary env vars such as
`JAVA_HOME` should stay as single explicit values.

## Fields

### `env.vars.<NAME>`

- `required`: whether the value must resolve
- `secret`: whether ota should redact the value in output and receipts
- `default`: fallback value when no higher-precedence source resolves it
- `allowed`: fixed allowed values
- `prepend`: `PATH`-only entries to place before the resolved base value
- `append`: `PATH`-only entries to place after the resolved base value

### `env.sources[]`

- `kind`: source type. Today ota ships `dotenv`
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

## Resolution Order

When a repo declares an env name in `env.vars`, ota resolves it in this order:

1. `tasks.<name>.env` for the task that declares it
2. `policies.env`
3. the shell process environment
4. declared `env.sources`, in order
5. the contract `default`

If none of those provide a value and the env is required, validation or execution fails.

If a declared dotenv source is present but invalid, ota fails instead of silently skipping it.
If a declared source has `must_exist: true` and is missing, ota reports that as a readiness failure
even if another layer provides the env value.

## What To Put Where

- use `env.vars` in `ota.yaml` to say which values the repo needs
- use `env.sources` when the repo intentionally relies on dotenv files
- use `required: true` when a value must exist
- use `default` when the repo has a safe fallback
- use `allowed` when the value must stay within a fixed set
- use `tasks.<name>.env` when one task needs a fixed override
- use `policies.env` when the organization wants an approved shared value
- use `prepend` and `append` only on `PATH`

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
    JAVA_HOME: /opt/jdk-22
    AWS_PROFILE: ota-prod
```

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
- `run` and `up` consume env values during execution
- receipts show which value won
- `explain` should turn env failures into a fix plan

## What Policy Is Not

- not a replacement for `env.sources`
- not a general application config system
- not silent env mutation
- not hosted control-plane workflow logic
- not waiver or approval orchestration
- not fleet reporting or retention policy
