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

# Hosted Validation

Use hosted validation when Ota needs to gate a pull request or CI run without mutating the repo.

## What to run

- `ota validate --json`
- `ota doctor --json`
- `ota workspace validate --json`
- `ota workspace doctor --json`
- `ota workspace list --json` for inventory and readiness summary

## What to fail on

- `ok: false`
- any `error` or `errors`
- any `severity: error`
- non-zero exit when validation is expected to pass

## What not to do

- do not run `ota init`
- do not run `ota detect --write`
- do not run `ota workspace init --bootstrap`
- do not infer behavior from human text output

## Example

```bash
#!/usr/bin/env bash
set -euo pipefail

ota validate --json | tee .ota-validate.json
ota doctor --json | tee .ota-doctor.json
ota workspace validate --json | tee .ota-workspace-validate.json
ota workspace doctor --json | tee .ota-workspace-doctor.json
```

Hosted validation is read-only. It surfaces blockers early and leaves mutation to local,
explicit commands.
