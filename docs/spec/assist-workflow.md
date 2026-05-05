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

# ota Assist Workflow

This document is the operator guide for the currently shipped `ota assist` surface.

For the long-term product contract, see [assist-operations.md](assist-operations.md).
For exact CLI syntax, see [command-reference.md](command-reference.md).
For the machine-readable payload, see [json-output-reference.md](json-output-reference.md).

## What assist is

`ota assist` is a reviewed contract mutation surface.

It is not:

- chat
- freeform YAML generation
- a second source of repo truth

It works like this:

1. inspect the existing repo and contract truth
2. propose one bounded change
3. show assumptions and the exact mutation
4. write only when `--write` is explicit
5. revalidate after apply

Preview is the default.

## Current shipped operation

The currently shipped assist operations are:

- `ota assist declare-readiness`
- `ota assist declare-service`
- `ota assist bind-task`
- `ota assist wire-setup`

It can target:

- `tasks.<name>.runtime.readiness` with `--task <name>`
- `services.<name>.readiness` with `--service <name>`
- a monorepo member overlay with `--member <name>`

It can currently propose:

- `spring-http`
- `http`
- `tcp`

`ota assist declare-service` can:

- create or refine one `services.<name>` block
- set `manager.kind`, `manager.name`, `manager.file`, and `manager.service`
- set one endpoint projection with `endpoint`, `address`, and `port`
- set `required`
- optionally attach structured readiness with `--style`

`ota assist wire-setup` can:

- create or refine `tasks.setup`
- set the setup body through `--run` or `--script`
- set or clear `setup.requires_services` as the pre-setup service phase for `ota up`
- refine `tasks.setup.internal` without widening into general task authoring

## Canonical examples

Task runtime preview:

```bash
ota assist declare-readiness --task dev
```

Task runtime apply:

```bash
ota assist declare-readiness --task dev --write
```

Explicit Spring-style task readiness:

```bash
ota assist declare-readiness --task dev --style spring-http
```

Managed HTTP service:

```bash
ota assist declare-readiness --service api --style http
```

Managed TCP service:

```bash
ota assist declare-readiness --service postgres --style tcp
```

Managed service declaration:

```bash
ota assist declare-service --name postgres --manager compose --compose-file docker-compose.yml --port 5432 --style tcp
```

Managed service apply:

```bash
ota assist declare-service --name api --manager compose --compose-file docker-compose.yml --port 3000 --style http --write
```

Task binding preview:

```bash
ota assist bind-task --task smoke --target api --to dev:http
```

Task binding apply:

```bash
ota assist bind-task --task smoke --target api --to dev:http --write
```

Task binding with inferred single listener:

```bash
ota assist bind-task --task smoke --target api --to dev --json
```

Monorepo member write:

```bash
ota assist declare-readiness --member api --task dev --style spring-http --write
```

Machine-readable preview:

```bash
ota assist declare-readiness --task dev --json
```

Setup wiring preview:

```bash
ota assist wire-setup --run "test -f .env.local || cp .env.example .env.local" --service postgres
```

Setup wiring apply:

```bash
ota assist wire-setup --member api --run "npm install" --service postgres --write
```

## Task versus managed service behavior

Task runtime targeting is allowed only when the task already declares a runtime service surface.

Managed service targeting is stricter:

- a managed service endpoint only proves projected address and port
- it does not prove protocol truth
- if the service does not already carry structured readiness, pass `--style` explicitly

This is intentional. Assist must not quietly propose an HTTP readiness path for a plain TCP service.

## Task binding behavior

`bind-task` is the current shipped assist slice for `tasks.<consumer>.targets.<name>`.

It currently:

- binds one consumer task to one producer task runtime listener
- proposes a reviewed `service` target block, not a literal guessed URL
- supports `--producer-member` for cross-member producer tasks already declared under the root monorepo contract
- preserves an existing `override_input` unless a new one is explicitly supplied
- validates the proposed edge through the normal contract rules before preview or write succeeds

It intentionally does not yet:

- bind directly to top-level managed service endpoints
- guess a listener when multiple producer listeners are equally valid
- hide `address_view` or `activation.mode` as non-reviewable internal defaults

## Setup wiring behavior

`wire-setup` is intentionally narrow:

- it only owns `tasks.setup`
- it can create that task when you give an explicit `--run` or `--script`
- it can set `setup.requires_services` to define the pre-setup service phase for `ota up`
- it preserves unrelated existing setup fields instead of rewriting the whole task

This is the current product model:

- services named in `setup.requires_services` start before `setup`
- other required managed services start after `setup`
- `wire-setup` should help express that truth, not invent a second orchestration layer

## Monorepo member behavior

When `--member` is present:

- ota resolves the merged member contract through the root monorepo contract
- assist validates the proposal against that merged truth
- assist writes only to the selected member overlay file

This keeps writes narrow without pretending the member overlay is a standalone repo contract.

## Refusal cases

Assist should refuse instead of guessing when:

- multiple candidate listeners exist and no safe selection is possible
- the selected task or service does not exist
- the selected task has no runtime service surface
- the selected service has no protocol signal and no explicit `--style`
- the selected listener protocol conflicts with the requested style
- a new managed service does not declare an explicit manager kind
- a service endpoint is ambiguous and `--endpoint` was not given
- a producer task binding is ambiguous because multiple listeners are available and no safe existing selection can be reused
- a producer task does not declare any service listener runtime surface
- a new `tasks.setup` declaration is requested without an explicit `--run` or `--script` body
- `wire-setup` names a managed service that is not already declared under `services`

Refusal is part of the trust model, not a UX bug.

## Replacement visibility

When assist would replace an existing readiness block:

- text preview must show the current readiness block first
- then the proposed readiness block
- JSON includes both `before` and `after` through `changes`

This includes replacement of older legacy top-level readiness shapes such as `from` plus `run`.

## After apply

The canonical follow-up is:

- `ota validate`
- `ota doctor`

`ota assist` should always point back into the same validator and diagnosis surfaces the rest of ota already trusts.
