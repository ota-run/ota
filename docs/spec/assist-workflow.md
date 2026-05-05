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

The first shipped assist slice is:

- `ota assist declare-readiness`

It can target:

- `tasks.<name>.runtime.readiness` with `--task <name>`
- `services.<name>.readiness` with `--service <name>`
- a monorepo member overlay with `--member <name>`

It can currently propose:

- `spring-http`
- `http`
- `tcp`

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

Monorepo member write:

```bash
ota assist declare-readiness --member api --task dev --style spring-http --write
```

Machine-readable preview:

```bash
ota assist declare-readiness --task dev --json
```

## Task versus managed service behavior

Task runtime targeting is allowed only when the task already declares a runtime service surface.

Managed service targeting is stricter:

- a managed service endpoint only proves projected address and port
- it does not prove protocol truth
- if the service does not already carry structured readiness, pass `--style` explicitly

This is intentional. Assist must not quietly propose an HTTP readiness path for a plain TCP service.

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
