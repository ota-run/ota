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

# Reference

`docs/spec` is the canonical source of truth.
`docs/site/src/reference` is the public-facing reference layer derived from it.
This section turns the spec into operator-facing documentation with examples and use cases.

The migration rule is:

- keep the spec accurate first
- derive the public page from the spec
- add examples, use cases, and operational guidance so the page stands on its own
- avoid asking readers to jump back to the spec for basic understanding

That means the reference pages should be detailed enough for real use, while still staying aligned
with the canonical contract and shipped behavior.

This section summarizes the stable surfaces of Ota:

- commands
- repo contract (`ota.yaml`)
- workspace contract (`ota.workspace.yaml`)
- machine-readable JSON output
- hosted validation workflow
- output and brand style conventions

Canonical specification files remain under `docs/spec/`.

Use this section when you need precision:

- exact command behavior for automation
- schema/contract clarity for repository standards
- stable machine interfaces for CI and agents
- hosted validation workflow for PR gating and CI

## Key references

- [Commands](commands.md)
- [Remote runner metadata and editor surface](remote-runner-and-editor-surface.md)
- [JSON output](json-output.md)
- [Hosted validation](hosted-validation.md)
- [Semantic diff and explain](semantic-diff-and-explain.md)
- [Execution receipt](execution-receipt.md)
- [Env resolution and policy](env-resolution-and-policy.md)
- [Commercial policy](../../policy/commercial-policy.md)
- [Brand policy](../../policy/brand-policy.md)
- [Support and enterprise](../../policy/support-and-enterprise.md)
