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

# Compatibility Surface Inventory

Purpose: define the V4 compatibility baseline that must remain stable unless an explicit versioned change is made.

## CLI surfaces

Repo commands:

- `ota version`
- `ota validate`
- `ota tasks`
- `ota run`
- `ota doctor`
- `ota check`
- `ota init`
- `ota detect`
- `ota up`
- `ota clean`

Workspace commands:

- `ota workspace validate`
- `ota workspace tasks`
- `ota workspace list`
- `ota workspace doctor`
- `ota workspace check`
- `ota workspace run`
- `ota workspace up`
- `ota workspace refresh`

## Compatibility-locked dimensions

For each command above, V4 must preserve:

- exit behavior and mapping
- JSON top-level shape and key semantics
- deterministic ordering for list outputs
- human output status semantics (`READY`, `NOT READY`, `VALID`) and failure clarity
- for `ota --version --json`, build identity fields and the contract capability catalog semantics

Published contract schemas:

- `docs/spec/json-schemas/contract.json` /
  `https://dist.ota.run/spec/json-schemas/latest/contract.json`
- `docs/spec/json-schemas/workspace-contract.json` /
  `https://dist.ota.run/spec/json-schemas/latest/workspace-contract.json`

These are compatibility-locked machine-readable public APIs for `ota.yaml` and
`ota.workspace.yaml` authoring. Changes to their semantics or required fields must be treated like
other contract-surface changes: additive when possible, explicitly documented when not.
The checked-in JSON files are generated artifacts owned by the Rust publisher in
`src/published_contract_schemas.rs`; regenerate them with
`cargo run --bin sync_published_contract_schemas` instead of hand-editing the published files.
The release gate and local compatibility task both rerun that generator and fail if
`git diff --exit-code` sees schema drift afterward.
Shipped repo/workspace examples and canonical docs examples are also checked as raw YAML values.
Ota now also validates those repo/workspace examples after loading them through the Rust contract
types and projecting them back to authoring JSON values, so the published schemas stay aligned
with both authored contract truth and the actual Rust-owned authoring-model boundary.

Published canonical docs manifest:

- `docs/spec/published-docs/canonical-docs.json` /
  `https://dist.ota.run/spec/published-docs/latest/canonical-docs.json`

This is the compatibility-locked machine-readable publication surface for the canonical docs
boundary itself. It tells downstream consumers which upstream `ota` source files own key docs
surfaces such as contract, workspace, command, topology, and machine-output references. The
checked-in JSON file is a generated artifact owned by the Rust publisher in
`src/published_docs_manifest.rs`; regenerate it with
`cargo run --bin sync_published_doc_manifests` instead of hand-editing the published file.
The release gate and local compatibility task rerun that generator and fail if `git diff
--exit-code` sees manifest drift afterward.

## Existing authoritative docs

- `docs/spec/exit-codes.md`
- `docs/spec/json-output-reference.md`
- `docs/spec/command-reference.md`
- `docs/spec/contract-reference.md`
- `docs/spec/published-docs.md`
- `docs/spec/workspace-reference.md`

## Baseline tests that must remain green

- parser/validator semantic tests
- command compatibility lock tests in `src/cli.rs`:

- `repo_commands_json_success_contract_is_stable`
- `repo_commands_json_validation_failure_contract_is_stable`
- `repo_commands_text_status_contract_is_stable`
- `doctor_not_ready_text_status_contract_is_stable`
- `repo_commands_exit_code_contract_is_stable`
- `workspace_commands_json_success_contract_is_stable`
- `workspace_commands_json_validation_failure_contract_is_stable`
- `workspace_doctor_text_status_contract_is_stable`
- `workspace_up_text_status_contract_is_stable`
- `workspace_commands_exit_code_contract_is_stable`
- `monorepo_member_json_contract_is_stable`
- `monorepo_member_text_status_contract_is_stable`
- `monorepo_member_exit_code_contract_is_stable`
- monorepo and workspace command behavior tests in `src/cli.rs`
- detector confidence/provenance tests in `src/detector.rs`

## Fast compatibility gate

Run this before merging behavior changes in V4:

```bash
./scripts/test-compat.sh
```

The repository contract also exposes this as a task:

```bash
ota run compat
```

Equivalent expanded command set:

```bash
cargo test contract_is_stable
cargo run --bin sync_published_contract_schemas
cargo run --bin sync_published_doc_manifests
git diff --exit-code -- docs/spec/json-schemas/contract.json docs/spec/json-schemas/workspace-contract.json docs/spec/published-docs/canonical-docs.json
cargo test --test json_schema_contracts
cargo test --test json_output_conformance
cargo test --test examples_validate
cargo test --test detect_fixtures
```

## V4 change rule

If a change modifies any compatibility-locked dimension:

- update the relevant normative doc in the same change
- add/adjust regression tests that lock the new behavior
- call out the change explicitly in planning notes

## Version provenance policy

- `schema_version` is the coarse contract-generation marker. Change it only when ota changes the
  machine-readable contract generation or compatibility interpretation in a non-additive way.
- `contract_capabilities[]` is the additive feature catalog for cross-version contract support.
  Extend it when ota learns a new compatibility-relevant contract feature but the surrounding
  contract generation stays compatible.
- capability entries should exist for features that materially affect whether one ota binary can
  parse, validate, or honestly interpret a contract written for another binary.
- minimum-version compatibility errors should stay feature-first when ota can identify the newer
  contract surface, and should always report the contract minimum, current binary identity, and a
  concrete install/rebuild next step.
