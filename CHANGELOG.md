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

## 1.0.1

- fixed the Windows test scaffolding so the release gate compiles cleanly on windows-latest.

## 1.0.0

- shipped the core adoption path as a stable surface: `doctor`, `explain`, `init`, `detect`, `up`, `run`, and workspace bootstrap.
- shipped advanced examples and adoption guides so the product story is grounded in real repo patterns, not just command lists.
- fixed Windows installation so release binaries install cleanly on PowerShell and Git Bash/MSYS/MinGW/Cygwin without forcing a cargo fallback.

## 0.5.6

- published Windows release assets so the installer no longer falls back to cargo on Git Bash or PowerShell when the release binary is available.
- taught the shell installer to install `ota.exe` on Windows targets so the release archive and installed binary stay aligned.

## 0.5.5

- fixed Windows installer detection so Git Bash, MSYS, MinGW, Cygwin, and PowerShell install paths resolve the release binary instead of falling back to cargo.
- documented the Windows installer paths in the README and installation guide so users do not need Rust to install ota on Windows.

## 0.5.4

- added tighter task-note spacing so `ota tasks` and `ota tasks --use` read as proper blocks instead of compressed blobs.

## 0.5.3

- tightened top-level help and onboarding copy so doctor/explain lead the CLI surface.
- made plain-mode output ASCII-only on the shared help, doctor, explain, and workspace surfaces.
- removed duplicate failure presentation and narrowed explicit path resolution to the targeted repo boundary.
- aligned summary titles and workspace receipt spacing so the main summary blocks read consistently.

## 0.5.2

- added `ota workspace receipt` as a read-only workspace receipt artifact for CI and archiving.

## 0.5.1

- added `ota workspace diff` as a read-only workspace drift view before refresh.
- added `ota workspace status` as a compact combined readiness-and-drift view.
- tightened workspace summary styling so the workspace family reads consistently.

## 0.5.0

- added `ota workspace refresh --dry-run` so workspace refresh can be previewed without mutating repo state.
- kept workspace refresh explicit with `--force`, `--prune`, and `--ref` for stricter sync control.
- tightened workspace acquisition and discovery boundaries so bootstrap stays scoped to the current workspace and repo root.
- improved repo and workspace UX around primary findings, failed phases, and backend-versus-task failure details.
- fixed release-gate and CI flakiness, including docs publishing, install-script publishing, and test-shim behavior.

## 0.4.2

- fixed GitHub Actions failures by updating stale test expectations and shared test shims.
- narrowed markdownlint scope so planning docs no longer block release validation.
- tightened workspace and doctor UX so primary findings, blockers, and failure summaries read clearly.
- added explicit output for backend-versus-task failures so setup and task issues surface the real root cause.
- locked JSON schema publication coverage so release artifacts stay aligned with the public spec path.
- updated release notes discipline so user-visible changes are tracked before each version bump.

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
