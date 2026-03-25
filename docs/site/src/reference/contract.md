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

`ota.yaml` is the canonical repo readiness contract.

When to use:

- any repository that wants deterministic setup and execution behavior

Why:

- replaces scattered setup truth with one machine-readable contract

Typical use-cases:

- onboarding new contributors without tribal setup steps
- giving CI and agents the same execution contract as humans
- documenting runtime/tool/service expectations in one file

Primary sections:

- `version`
- `project`
- `runtimes`
- `tools`
- `env`
- `services`
- `checks`
- `tasks`
- `execution`
- `agent`
- `workspace` (for monorepo root/member model)

Minimal example:

```yaml
version: 1
project:
  name: example-repo
runtimes:
  node: "22"
tasks:
  test:
    run: npm test
```

Trust and behavior rules:

- `ota validate` enforces structural and semantic correctness.
- `ota doctor` reports readiness findings without hidden mutation.
- `ota detect` writes conservatively from high-confidence fields only.
- `ota detect --merge` applies additive high-confidence fields only.

Runtime version syntax examples:

- `8` is an example of an exact required version
- `>=8` is an example of accepting any version at or above `8`
- `^8` is an example of a compatible version range, usually the same major line
- Ota compares numeric version parts and accepts common prefixes such as `go1.24.2` or `v1.24.2`
- use `>=` when a newer runtime is acceptable
- use `^` when compatibility is acceptable but exact equivalence is not required

## Authoring guidance

Start minimal, then expand:

1. define `project`, `tasks`, and required runtimes
1. add tools/env/services only when they are real blockers
1. run `ota validate` and `ota doctor` after every change

Prefer explicitness:

- name tasks by intent (`setup`, `test`, `lint`, `dev`)
- keep checks actionable and bounded
- avoid hidden side effects in task commands
- for SDK/library repos without runnable entrypoints, set `project.type: sdk` (or `library`) so
  `ota doctor` treats missing `tasks` as a warning instead of a blocking error
- some tool keys map to different executables; for example, `tools.maven` is checked via `mvn`

Canonical contract reference in repository:

- `docs/spec/contract-reference.md`
- <https://github.com/ota-run/ota/blob/main/docs/spec/contract-reference.md>
