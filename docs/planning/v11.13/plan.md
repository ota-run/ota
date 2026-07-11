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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.
-->

# V11.13: Generated Artifact Lineage

Status: active implementation slice.

## Problem

Ota can execute a code-generation command and declare broad filesystem writes, but it cannot
declare the resulting generated source as a named contract artifact. Consumers therefore cannot
state that they require the exact output of a generator, and receipts cannot distinguish a missing
generated artifact from an ordinary command failure.

This appears in generated SDK and client workflows: a schema or engine changes, a generator writes
typed client source, and verification consumes that source. Ordering alone is insufficient because
the operational boundary is the generated artifact, not merely the generator command.

## Canonical Model

Artifact truth is repo-scoped and global, not duplicated under task effects:

```yaml
artifacts:
  typescript-sdk:
    kind: generated_source
    producer: sdk:generate
    paths:
      - sdk/typescript/src/api/client.gen.ts
    inputs:
      - core/schema

tasks:
  sdk:generate:
    command:
      exe: dagger
      args: [generate, typescript]
    effects:
      writes:
        - sdk/typescript/src/api/client.gen.ts

  sdk:test:
    depends_on: [sdk:generate]
    requires_artifacts: [typescript-sdk]
```

- `artifacts.<name>` owns one semantic generated artifact identity.
- `producer` names the one task that materializes it.
- `paths` are the repo-relative produced outputs.
- `inputs` name repo-relative source paths that explain the artifact's derivation boundary.
- `tasks.<name>.requires_artifacts` means the task consumes the artifact and must explicitly depend
  on its producer.

## First Cut

1. Parse and serialize the global artifact map plus task consumer references.
2. Validate names, paths, producer existence, output-path overlap, consumer references, and the
   required explicit producer dependency edge.
3. Make selected consumers fail clearly when their declared artifact paths are absent.
4. Surface producer/consumer lineage in task JSON and receipts.
5. Pressure Dagger's narrow SDK generation path, then a real independent sibling-repo case.

## Non-Goals

- Do not infer freshness from mtimes or Git state.
- Do not copy, transport, or synchronize artifacts between repositories yet.
- Do not invent a Dagger adapter; the generator remains an ordinary typed `command` until a real
  adapter-specific ownership gap appears.
- Do not claim that a present artifact is fresh. Freshness requires later captured derivation input
  identity and receipt-backed comparison.

## Acceptance

- A valid producer/consumer graph is accepted and exposed consistently in human and JSON task
  discovery.
- Dangling, ambiguous, overlapping, and dependency-disconnected artifact declarations fail
  validation with direct ownership guidance.
- A selected consumer cannot execute when a required generated path is absent.
- The first receipt identifies the consumed artifact and its producer without reconstructing lineage
  from task text.
