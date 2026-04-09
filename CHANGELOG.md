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

- added `ota up --dry-run` with text and JSON preview output so operators can review the selected backend, lifecycle, target, planned provisioning/setup work, current skips, and first blocker before ota mutates repo or execution state.
- added `ota clean --stale` with `--dry-run` and `--json` so exited ota-managed containers can be previewed or removed across repos without requiring an `ota.yaml`, while keeping plain `ota clean` scoped to the current repo contract.
- started labeling new persistent ota containers for ownership-safe stale cleanup and kept a legacy `ota-*` name fallback so older containers remain discoverable.
- added a first-class `Adapter bootstrap failed: sdkman` finding to container-mode `ota up` so the bootstrap boundary is explicit while preserving the real backend stderr and the container-specific rerun hint.
- made `ota up` bootstrap failure findings derive their `Why:` and prerequisite `Next:` text from the real bootstrap stderr instead of hardcoded root-cause guesses, and enabled the shared command spinner for repo-level `ota up` so slow provisioning paths show progress.

## 1.2.1

- added `ota doctor --mode container` so readiness can be diagnosed against the declared container execution boundary instead of only the host context.
- made doctor JSON explicit about the selected diagnosis mode and kept container-mode next steps pointed at `ota doctor --mode container` so machine consumers and text users see the same execution boundary.
- added `ota policy review` as a read-only policy-authority lens so policy-vs-contract conflicts and approved sources can be reviewed without mutating either side.

## 1.2.0

- updated the premium `run` failure snapshot so the current task-output excerpt and compact `Next:` footer are released together.
- made runtime/tool remediation exact and manager-aware when ota has strong signals, including
  repo-local hints such as `.nvmrc`, `.python-version`, `.sdkmanrc`, `.tool-versions`, and
  policy-backed provisioning sources.
- broadened exact remediation coverage further with stronger manager signals such as `volta`,
  `nodenv`, `pyenv`, `rbenv`, `goenv`, `rustup`, and explicit runtime `provider` hints.
- broadened exact remediation again for `.sdkmanrc`-backed maven installs and `global.json`-
  backed `.NET` SDK installs, and tightened rerun paths so nearby external contracts compact to
  shorter relative targets instead of noisy absolute paths.
- removed duplicate adapter-bootstrap info findings by loading the policy pack once for doctor
  diagnostics and reusing that source across provisioning and bootstrap surfaces.
- tightened workspace text parity so `workspace check` now promotes the primary blocker and
  workspace execution sections use the same compact, highlighted summary style as repo doctor.
- upgraded the root help and onboarding docs into a clearer doctor-first chooser flow with repo
  and workspace entry paths.
- locked the premium text UX with checked-in golden snapshots for root help, repo `doctor`,
  `detect`, `up`, `run`, and workspace `validate`, `doctor`, `explain`, `up`, and `run`.
- expanded the UX review loop to snapshot-lock `doctor --plain` and narrow-width `explain`
  rendering so plain-mode and small-terminal regressions are caught before release.
- fixed grouped tooling remediation hints so repeated version-mismatch blocks only surface real
  exact commands and never degrade into stray version tokens.
- made `ota agents` text output more adoption-friendly by pointing preview users at
  `ota agents --write` and pointing write users back to `ota doctor`.
- made repo-targeted text guidance truthful when commands operate on external contracts, so
  `validate`, `agents`, `doctor`, `explain`, and `up` now rewrite follow-up commands with an
  explicit target instead of implying the current working directory.
- tightened the public adoption path in README and quickstart with a clearer existing-repo flow,
  an explicit `ota agents` path, and a chooser for shipped examples by goal.
- promoted `ota agents` and the doctor-first repo/workspace start paths more explicitly in root
  help, README, quickstart, and command reference so the derived guidance path is part of the
  obvious first-session value story.
- added a repo-local `ux-review` task plus a dedicated UX review loop doc so maintainers can keep
  the premium text surfaces and snapshot-backed CLI presentation intentionally reviewed.
- added an adoption readiness gate doc so enterprise-facing scope stays behind an explicit product
  usefulness bar instead of drifting ahead of first-session value.
- tightened `ota run` failure output so existing `Next:` guidance never leaves a blank spacer line
  before the action line, even when the error already carried guidance before `RUN SUMMARY`.
- hardened persistent container reuse so `ota run --mode container` recreates a stale stopped
  backend instead of trying to `exec` into it and surfacing `cannot exec in a stopped container`.

## 1.1.3

- grouped repeated remediation findings across doctor-style text outputs so `doctor`, `check`,
  `up`, and workspace variants collapse obvious repeated actions with shared Ota styling.
- normalized `finding_groups.action_key` around stable semantic action classes instead of rendered
  `Next:` or summary prose so grouped JSON summaries stay stable when copy changes.
- tightened `ota run` failure formatting so injected `Next:` guidance now sits immediately under
  `Why:` and no longer leaves extra blank gaps before `RUN SUMMARY`.
- regrouped detect contract-drift removals by task in text output so task-related drift reads as
  actionable task changes instead of a raw dotted-path diff dump.
- retuned the rich-mode `»` child-action accent to a brighter cyan so grouped drift/action items
  stand apart more clearly from their parent bullets.
- split detect task drift into command vs agent-safety sections, added impact summaries, concise
  task-count views, stronger task ordering, and wrapped command removals with clearer verb/code
  styling.
- restyled detect drift impact as a stacked `Impact:` block so the roll-up reads as framing
  metadata instead of a competing section headline.
- highlighted detect task names in grouped drift blocks with a dedicated yellow code accent so the
  task being discussed reads as the primary unit of change.
- made existing-contract detect previews lead with comparison and drift before the inferred
  contract/annotation dump, and clarified zero-addition previews as "no additive changes" when
  only stale drift remains.
- kept successful `ota up` text output focused by suppressing noisy captured task/service logs on
  the happy path while still showing them for failures.
- reordered default `ota doctor` text so verdict and next-step guidance appear before execution and
  agent detail, and added ready-path next actions for repos that have no findings.
- compacted default `ota doctor` execution and agent sections into higher-signal summary blocks,
  with calmer cyan child markers and highlighted code values instead of YAML-style detail dumps.
- made repo `ota run` text output context-aware: interactive terminals still stream live logs,
  while non-interactive runs now buffer into bounded excerpts with a new `--stream` escape hatch
  for raw live output.
- brought workspace `validate`, `doctor`, `check`, and `explain` text output closer to the repo
  UX bar with clearer next steps, shared grouped findings, `Plan`/`Overview` section naming, and
  a properly separated primary-blocker section.
- upgraded empty-state text for `ota services`, `ota extensions`, and `ota policy` so those
  commands explain the absence and point to the next useful Ota action instead of ending cold.
- made wrapped task and extension detail lines terminal-width aware, and taught captured output
  excerpts to prefer the most relevant failure window instead of always dumping the tail.
- upgraded default `ota validate` success output with clear next-step guidance into `ota doctor`
  and `ota tasks --use`.
- renamed repo `ota explain` text sections to `Plan` and `Overview` so the remediation surface
  matches the higher-signal style now used across `doctor`, `check`, and `tasks`.
- added `ota detect --contract` so ota can preview the exact starter contract init would write
  without annotations or comparison noise.
- moved detect preview `Next:` guidance to the end of the preview so the contract readout comes
  first.

## 1.1.2

- added an explicit `install-from-source` repo task for source-based reinstalls.
- tightened glossary guidance so documentation links point at the most specific useful section instead of page roots.

## 1.1.1

- documented `ota policy` and `ota uninstall` in the command reference and aligned the policy command output with the standard ota header style.

## 1.1.0

- added platform-specific provisioning overrides so a single policy entry can choose brew, apt, or choco by OS.
- made the platform-specific provisioning example explicit for macOS so the default fallback is not implied.
- grouped custom source configuration by adapter family so the policy examples read more clearly.

## 1.0.4

- aligned `ota run` failure output so `Why` and `Next` appear before the trailing `RUN SUMMARY` block.
- kept the user-facing execution override flag on `--mode`, with `--backend` remaining as a compatibility alias.

## 1.0.3

- tightened adapter bootstrap fallback notes so `ota up` says which missing adapter, approved source, and backend failure blocked the bootstrap attempt.
- surfaced adapter bootstrap diagnostics in `ota doctor --json` and `ota workspace doctor --json` so the approved bootstrap path is visible per repo.
- kept container-local provisioning and real fixture coverage aligned so the container target and adapter bootstrap paths stay testable.
- aligned the public docs and starter-contract wording around AI-agent-safe task hints, shared policy discovery, and the current adapter support matrix.

## 1.0.2

- clarified policy pack discovery so shared org rules live in `.ota/org-policy.yaml` and workspace trees can inherit them deterministically.
- documented policy-backed provisioning, adapter coverage, and the public provisioning-policy example so the current behavior is easier to adopt.
- improved starter contract generation with task notes, safer agent defaults, and a protected `ota.yaml` by default.
- fixed `ota diff` exit semantics so changed contracts return a nonzero code and JSON `ok` matches the semantic result.

## 1.0.1

- fixed the Windows release-gate stack so the real repository fixture suite completes on windows-latest.

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
