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

## Current Baseline

The shipped contract already supports:

- required values
- defaults
- allowed values
- task env overrides
- policy-provided env values
- validation in `doctor`
- default application in `run`
- `PATH` `prepend` and `append`

`policies.env` today is a flat `NAME: VALUE` map. Provenance labels such as `source` are output
vocabulary, not YAML fields. See [`policy-packs.md`](policy-packs.md) for how ota finds the policy
pack itself.

`PATH` is special because it is an ordered executable search path. Ordinary env vars such as
`JAVA_HOME` should stay as single explicit values.

## How Ota Picks a Value

When a repo declares an env name in `env`, ota resolves it in this order:

1. `tasks.<name>.env` for the task that declares it
2. `policies.env`
3. the shell process environment
4. the contract default

If none of those provide a value and the env is required, validation or execution fails.

## What to Put Where

- use `env` in `ota.yaml` to say which values the repo needs
- use `required: true` when a value must exist
- use `default` when the repo has a safe fallback
- use `allowed` when the value must stay within a fixed set
- use `tasks.<name>.env` when one task needs a different value
- use `policies.env` when the organization wants an approved shared value
- use `prepend` and `append` only on `PATH`

## Examples

```yaml
env:
  DATABASE_URL:
    required: true
  JAVA_HOME:
    default: /opt/jdk-21
  PATH:
    prepend:
      - ./node_modules/.bin
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
  PATH:
    prepend:
      - ./node_modules/.bin
```

then the final `PATH` is:

```text
./node_modules/.bin:/usr/local/bin:/usr/bin:/bin
```

## Where It Shows Up

- `doctor` diagnoses missing or invalid env
- `run` and `up` consume env values during execution
- receipts should show which value won
- `explain` should turn env failures into a fix plan

## What Policy Is Not

- not a replacement for `.env`
- not a general application config system
- not silent env mutation
- not hosted control-plane workflow logic
- not waiver or approval orchestration
- not fleet reporting or retention policy
