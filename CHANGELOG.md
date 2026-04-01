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

# Changelog

## Unreleased

- Future changes will be tracked here before the next version bump.

## 0.4.1

- expanded the changelog to cover all shipped releases instead of only the latest one.
- added a repository rule to keep `CHANGELOG.md` updated for every user-visible change and release bump.
- updated JSON schema `$id` values to the stable `latest` URL form used by the published R2 path.
- kept release-gate publishing aligned with tag-based release flow and JSON schema uploads.
- tightened README ordering and quickstart examples so installation and first-use paths are first-class.

## 0.4.0

- added `ota agents` to generate or sync `AGENTS.md` from the contract agent block.
- preserved user-authored `AGENTS.md` content while appending generated guidance.
- improved agent guidance output with explicit `ota run ...` forms and clearer managed-block labeling.
- added richer provenance and readiness details for `doctor`, `diff`, `explain`, and workspace equivalents.
- tightened path-boundary handling so repo and workspace commands stay honest about their targets.
- hardened workspace policy env resolution and receipts for clearer provenance.

## 0.3.0

- added provenance to `diff` and `explain` outputs.
- propagated workspace policy env into workspace run and recorded the provenance in receipts.
- added policy env provenance to execution receipts and workspace explanation output.
- introduced `diff` and `explain` schemas plus contract tests for the new machine output.
- documented deferred enterprise gaps and normalized output-path expectations.

## 0.2.0

- added support for running tasks without an explicit path argument.
- improved README examples to use the `ota` CLI directly instead of `cargo run`.
- added release checksum generation and included checksums in published assets.
- added agent contract sections and copyright headers to example contracts.
- improved update notice styling, install script output, and environment provenance docs.
- added `basic-dotnet` example coverage and refreshed example descriptions in the README.

## 0.1.5

- restored the workspace check path in the release gate and kept the gate native.
- moved workspace doctor diagnostics into a dedicated module.
- added a newline after clearing stderr lines when stopping spinner output.
- extracted contract-drift and execution helpers to dedicated modules.

## 0.1.4

- aligned JSON contracts and fixture expectations across doctor, explain, tasks, workspace run, and real fixtures.
- refined execution receipts, summary formatting, and lifecycle note rendering.
- added execution mode and container details banners to task run output.
- added drift-aware doctor metadata and improved spinner cleanup.
- introduced the gold dot verdict styling and banner formatting improvements.

## 0.1.3

- stabilized tests and CI by improving isolation, single-threaded test execution, and stderr assertions.
- restored update environment variables after command execution to prevent state leakage.
- refined update notice styling and added `ota run bump-version` documentation examples.
- added release checksum generation and included checksums in published assets.
- clarified agent contract source, run argument order, and local source install guidance.
- added example agent contract sections and improved workspace acquisition failure context.

## 0.1.2

- aligned the release gate with agent summary output.
- tightened update logic, release workflow output, and real fixture expectations.
- improved installer output streaming and banner styling.
- clarified env resolution policy, provenance, and workspace inheritance in docs.
- updated README examples to use the `ota` CLI directly.
- added support for stable and latest update channels.

## 0.1.1

- stabilized optional tool version handling in `doctor`.
- improved doctor fixture coverage and error handling around optional tool versions.
