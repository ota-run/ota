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

# Quickstart

Goal:

- move from zero context to runnable repo quickly and safely

1. Confirm CLI is available:

```bash
ota --version
```

1. In a repository with `ota.yaml`:

```bash
ota validate
ota tasks
ota doctor
ota up
```

1. Run a task:

```bash
ota run test
```

1. If you do not have a contract yet:

```bash
ota init
ota detect --dry-run /path/to/repo
```

1. Write only after review:

```bash
ota detect /path/to/repo
ota detect --merge --dry-run /path/to/repo
ota detect --merge /path/to/repo
```

For full command semantics, see the Reference section.

## Typical adoption paths

### Existing mature repo

When:

- team already has scripts/manifests but no Ota contract

Use:

```bash
ota detect --dry-run .
ota detect .
ota doctor
ota up
```

### New contributor onboarding

When:

- engineer cloned repo and needs a deterministic first run

Use:

```bash
ota doctor
ota up
ota run test
```

### CI safety gate

When:

- enforce contract correctness and checks in PR pipelines

Use:

```bash
ota validate --json
ota check --json
```
