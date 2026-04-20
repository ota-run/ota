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

- restore visible run progress indicators after the repo moved to container-first execution:
  interactive `ota run <task>` once again relies on the run command's own streaming loaders, and
  runs now show a short preflight loader while resolving execution backends before task spawn

- made grouped policy findings in `ota doctor`, `ota up`, and the shared JSON summaries read like operator guidance instead of policy declarations, using active labels such as `Review active policy surfaces`, normalized item text like `Approved provisioning sources are configured`, and next steps that point into `ota policy review` when users need the active policy boundary.
- made single version-policy findings in `ota doctor` and `ota policy review` use the same operator-facing wording and next-step path into `ota policy review`, instead of leaving the card as a raw declared-policy summary with generic guidance.
- redesigned `ota policy review` output so policy findings no longer point back to `ota policy review` itself; the command now leads with a `Policy` context block, uses action-shaped summaries like `Approved provisioning and bootstrap surfaces are configured`, and points `Next:` at changing the repo contract, using approved sources, or updating `.ota/org-policy.yaml`.
- made shared policy-surface findings point into explicit `ota policy review <repo>` follow-ups in receipt JSON, so external-repo adoption flows no longer fall back to generic “use this policy surface” wording for approved provisioning and bootstrap guidance.

## 1.5.2

- made `ota receipt` keep explicit repo-target follow-up commands inside JSON receipt and finding remediation, so non-current-directory adoption flows no longer drop the repo path when they tell you to rerun `ota doctor`.
- made generated `AGENTS.md` task guidance more readable by rendering `safe_tasks` and `verify_after_changes` as short nested lists instead of one long wrapped inline sentence, and trimmed trailing whitespace in generated notes.
- added a worked existing-repo adoption example that shows the concrete `doctor -> up -> run ci -> receipt`, CI archive, and promoted-baseline flow, and linked it from the main onboarding docs.
- removed the Ota copyright/license banner from `ota agents` generated `AGENTS.md` output, keeping only short provenance (`Generated from ... by \`ota agents\`.`) plus the actual repo-local guidance.
- added a focused one-team rollout guide that turns the scattered `doctor -> up -> run -> receipt`, CI archive, promoted baseline, and local policy steps into one explicit adoption playbook, and linked it from the main docs entry points.
- made `ota policy` more actionable when a policy pack is already loaded by adding direct next-step guidance into `ota policy review` and `ota doctor`, so the effective-policy view now points cleanly into boundary review and readiness.
- tightened the receipt/baseline adoption story in the public docs by adding one explicit promoted-baseline workflow for local and CI use, including `ota receipt --json --archive --promote-baseline`, `ota receipt --json --baseline promoted`, and `ota annotations --mode receipt-diff --format markdown`.
- tightened the public first-success docs around the proven repo loop by leading existing-repo quickstarts with `doctor -> up -> run -> receipt`, moving `ota agents` to an explicit follow-up, and adding a repeatable one-repo local/CI rollout story.
- tightened the extension-facing JSON contract lock so `ota doctor --json`, `ota tasks --json`, `ota env --json`, and `ota receipt --json` now have regression coverage for key nested semantics such as verdicts, primary blockers, agent defaults, missing env entries, and repo receipt scope/details instead of only top-level envelope fields.

## 1.5.1

- fixed a Windows PowerShell installer interpolation bug in `scripts/bootstrap.ps1` so replacement failures now report the destination path cleanly instead of aborting on `${drive}:` parser handling
- taught detector-led `ota init` to carry existing repo-root dotenv sources such as `.env.local` and `.env` into `env.sources`, while keeping explicit `ota init --pack ...` starters on their conventional no-inference boundary.
- kept task inputs like `mode` and `jobs` valid in contracts while tightening `ota run` / `ota workspace run` parsing so pre-task ota flags still work, post-task overlapping inputs can be disambiguated cleanly, and generated env-only starters stay lean and accurately counted.
- validator now rejects repo-local `policies.version_policy`, `policies.provisioning`, and `policies.adapter_bootstrap` in `ota.yaml`, so provisioning authority can no longer be declared inertly in the repo contract instead of the real `.ota/org-policy.yaml` policy pack.
- taught `ota validate` and `ota doctor` to reject `.ota/org-policy.yaml` as the wrong target with a dedicated, structured message instead of falling through the generic repo-contract failure wrapper.
- taught `ota validate` and `ota doctor` to render repo-contract validation failures as structured `Invalid contract` output instead of the generic `Operation failed / INVALID ota.yaml / ota init` wrapper.

## 1.5.0

- improved `ota run` so a missing non-path token like `ota run version:bump patch` can be reinterpreted as a single declared task input instead of a fake repo path, while still preserving explicit path-like tokens such as `./repo` or `foo/ota.yaml`.
- extended that single-input shorthand to monorepo member runs, so `ota run version:bump --member api patch` now resolves `patch` as the declared task input instead of a missing repo path.
- prevalidated requested-task inputs before dependency execution in `ota run`, so invalid top-level task input flags or values now fail before any `depends_on` work can mutate repo state.
- clarified run receipt step details for hook reruns so follow-up executions now explain when a task reran via `after_success` / `after_failure` / `after_always` and when a dependency reran as part of that fresh hook subtree.
- clarified `ota run --help` and `ota workspace run --help` so the operator rule is explicit: put Ota command flags before task inputs, with concrete examples for input-bearing task syntax.
- fixed task outcome hooks so `after_success`, `after_failure`, and `after_always` can rerun a task together with its dependency subtree even when that work already ran earlier in the same top-level invocation through `depends_on`, which fixes flows like `version:bump` followed by a post-bump `build` that must rerun `setup`.
- dogfooded task outcome hooks in the `ota` repo and the public examples repo so `after_success`, `after_failure`, and `after_always` now appear in real shipped contracts instead of docs-only examples
- added first-class task outcome hooks with `after_success`, `after_failure`, and `after_always`, made the runner treat hook failures as part of the parent task result, and updated workspace task inventory plus contract docs so the new execution edges are visible in both runtime and machine-readable surfaces
- CI wrapper scripts `scripts/emit-ota-findings.sh` and `scripts/emit-ota-findings.ps1` now
  delegate directly to `ota annotations`, including markdown summaries and `receipt-diff`, so
  wrapper paths reuse the canonical CLI renderers instead of maintaining duplicate formatting;
  they also resolve the current checkout binary before falling back to an ambient install

- polished `ota annotations` for CI and PR consumption by suppressing duplicate primary-blocker finding lines and labeling additive `Provenance:` plus `Next:` segments when those fields exist in the input JSON.
- added `ota annotations --format markdown` as the canonical compact summary renderer for doctor and workspace-doctor JSON, so step summaries and PR comments can reuse ota’s own status, blocker, provenance, and next-step wording instead of rebuilding it downstream.
- extended `ota annotations` with `--mode receipt-diff --format markdown`, so baseline compare output now has the same canonical compact renderer for PR comments and step summaries instead of forcing wrappers to rebuild compare/gate wording from raw receipt diff JSON.
- extended `ota receipt --baseline --fail-on-new-blockers` so the compare gate now carries the first blocking summary, next step, and provenance in both JSON and text output, making CI summaries and PR comments easier to render without scraping the full introduced-finding list.
- added a compact additive receipt diff comparison summary so JSON and text output can surface baseline/current identity labels plus readiness drift in one small block instead of forcing wrappers to reconstruct that view from the full baseline/current sections.
- refined explicit `ota init --pack ...` advisories so the text output now compares both sides of the mismatch more clearly with suggested signals, selected-pack incidental signals, and an explicit score gap while keeping the pack choice authoritative.
- extended `pack_advisory` in `ota init --json` with additive comparison fields such as `score_gap` and `selected_signal_details`, making the advisory easier to explain in automation without scraping human text.
- clarified the env-resolution docs so root contract env is explained as a repo-wide execution contract, not just a validation surface, including the injected-process boundary for `ota run` / `ota up` and when ota can functionally replace in-app dotenv loading.
- made the env docs more explicit about what `required`, `allowed`, `secret`, and `default` mean in practice, including a concrete `DISCORD_TOKEN` plus `RELEASE_CHANNEL` example.
- added stronger env docs for authoring and operations, including valid-versus-invalid `env.vars` examples, the remote-secret execution caveat, a workspace env precedence example, and a realistic `ota env` text/JSON example.

## 1.4.19

- removed the redundant `Suggestions` title from zsh completion menus while keeping commands and tasks ahead of global `--flags`
- redesigned env resolution end to end around `env.vars`, `env.sources`, and typed policy values at `policies.env.values`, making dotenv loading explicit, org policy values explicit, and the precedence surface honest across repo, workspace, and execution output.
- added declared dotenv source resolution to `ota doctor`, `ota env`, `ota run`, and execution summaries, including ordered source precedence, `must_exist` readiness checks, and winning-source provenance such as `dotenv:.env`.
- updated the contract/env docs, JSON env schema reference, and shipped examples so the public contract, command output, and repo fixtures all use the new env-source model consistently.
- added `php-composer` as a workflow-shaped starter pack for explicit Composer-managed PHP repos, including pack-catalog discovery, Composer-backed advisory matching, and a review-first `does_not_infer` boundary instead of a vague language-level PHP pack.
- expanded the explicit starter-pack catalog with `dotnet`, seeding a conventional `dotnet restore` / `dotnet build` / `dotnet test` first draft plus dotnet-aware advisory matching from `global.json`, solution, and project signals.
- extended `ota init --packs` so each catalog entry now exposes explicit `does_not_infer` boundaries in both text and JSON, making the starter-pack scope visible without inventing fake pack knobs.
- enriched `ota init --pack ... --json` advisories with explicit selected-versus-suggested signal scores plus structured weighted signal details, and mirrored the same strength summary in text output.
- clarified human `ota init --pack ...` advisories so text output now explains why the mismatch exists, shows weighted signal markers directly, and keeps the explicit review step obvious without weakening pack authority.
- removed the remaining native fallback branches from explicit `ota up --mode container` provisioning resolution, so container mode now fails in preconditions instead of ever escaping into host provisioning or host `setup`.
- added explicit `ota init --pack` knobs for the first conventional starter variants: `--package-manager npm|pnpm|yarn|bun` on the Node pack and `--test-runner pytest|unittest` on the Python pack, including catalog metadata, JSON `pack_options` for explicit overrides only, and variant-specific provenance.
- tightened background update-notice delivery so successful interactive commands keep the short non-blocking wait budget instead of riding the full release-check timeout on slow or offline networks.
- made explicit `ota init --pack ...` advisories compare distinct repo-signal strength instead of suppressing warnings as soon as the selected pack has any incidental match.
- replaced runtime/tool OS scoping via `platforms.<os>.required` with a cleaner `only_on` contract surface, while keeping root `required` as the blocking-vs-warning control and `platforms` for per-OS value overrides only.
- upgraded the advanced full-contract example and its `.ota/org-policy.yaml` to dogfood `only_on`, Java runtime distributions, explicit `version_policy`, policy-backed provisioning, and adapter bootstrap, and added example validation coverage for shipped org policy examples and policy-doc YAML.

## 1.4.18

- added platform-aware runtime and tool resolution so required versions, providers, and required flags can now vary by OS through `platforms` entries, with resolver/diagnostics using the effective per-OS values.
- added org policy version controls for runtimes/tools at both global and per-platform levels, plus per-platform policy violations surfaced during policy evaluation.
- strengthened contract validation for new platform-specific details by rejecting unsupported platform keys and empty platform-level runtime/tool metadata before execution/provisioning.

## 1.4.17

- fixed `ota up` so effective container mode now stays container-authoritative during provisioning; missing `docker` / `podman` stops in preconditions instead of silently falling back to host provisioning or host `setup`.
- added a post-release GitHub Actions job that auto-generates Discord `#releases` messages from published GitHub release metadata and posts them via `DISCORD_RELEASE_WEBHOOK_URL`, removing the need to maintain per-tag Discord note files.
- made `ota self-update` always force published release installation instead of switching to a local source build when run inside the ota repo, changed update notices to use cached release results so they stay fast without dropping slow network checks, taught stale no-update caches to refresh synchronously so the first command after a new release can still surface the notice, hardened the Windows deferred self-update helper to keep retrying replacement after process exit, and switched Unix release installs to staged renames instead of direct live-binary overwrites.
- kept the background update-notice foreground wait budget small and consistent across platforms so successful commands stay responsive, and shortened the transient update-check failure cooldown to one hour everywhere instead of leaving Unix-like systems silent for a full day.
- made Windows `ota uninstall` report a pending, unverified removal state instead of implying the running binary was already deleted, and extended the detached delete helper to keep retrying after the current process exits.
- expanded the explicit starter-pack catalog from `node|python|java-maven|java-gradle` to `node|python|go|rust|java-maven|java-gradle`, adding conventional Go and Rust starter contracts through `ota init --pack ...`.
- upgraded `ota init --packs --json` so each catalog entry now carries the exact pack-selection `command` plus a safe dry-run `next` command, making the starter-pack catalog a more complete product surface for automation and docs generation.
- clarified the `ota init` command reference by separating detector-led and pack-led starter paths in the detailed docs without promoting `--pack` to a separate command surface.
- added advisory-only pack mismatch detection for `ota init --pack ...`, so explicit starter mode can warn when strong repo signals disagree while still keeping the selected pack authoritative.

## 1.4.16

- added `ota init --packs` as a read-only starter-pack catalog so users and automation can discover the built-in conventional starter packs, inspect what each one seeds, and jump straight to the matching `ota init --pack ... --dry-run .` preview path.
- expanded explicit starter packs from `node|python` to `node|python|java-maven|java-gradle`, including stable `catalog` JSON output for `ota init --packs --json` and pack-specific Maven/Gradle starter tasks, checks, tools, and provenance.
- made the new Java starter packs prefer repo-local `mvnw` and `gradlew` wrappers when those files already exist, falling back to explicit global Maven or Gradle prerequisites only when the repo does not ship a wrapper.
- upgraded `ota init --packs` text rendering so each starter pack now reads like a first-class command detail block with structured `Description`, `Notes`, seeded runtimes/tools/checks/tasks, and a `Next:` preview command instead of the earlier dense summary line.
- simplified the `ota init --packs` detail rows by removing the extra arrow markers, keeping the pack command lead line while rendering the structured metadata as cleaner indented labeled fields.
- aligned explicit pack preview/write output so `Policy:` now renders as the same keyed detail row as `Mode:` and `Pack:` instead of falling back to an unstyled prose line.

## 1.4.15

- added `ota completion --remove` as the managed uninstall path for shell completion setup, and updated completion guidance so setup and removal commands are shown together.
- updated completion setup status handling so rerunning `ota completion --setup` reports `Status: updated` when managed support files are refreshed, while `ota completion --remove` remains idempotent with `Status: not configured` when nothing is installed.
- changed generated zsh completion output to preserve raw candidate ordering with explicit `nosort` handling and separate `Suggestions`/`Options` groups, keeping commands and tasks ahead of global options in completion menus.
- upgraded zsh completion display labels to `token -- description` and simplified completion rendering so candidate values stay plain while menu text remains clearer.

## 1.4.14

- added `ota init --pack node|python` as an explicit conventional starter path so repos can seed a reviewable `ota.yaml` pack without depending on detector confidence, including pack-aware text output, stable JSON `pack` metadata, per-field provenance, starter checks, and deterministic starter tasks.
- pack-generated starter tasks now include short `description` fields, and task-name shell completion can surface those descriptions as candidate help so users see the authoring pattern immediately instead of only learning it from docs.
- extended short task `description` seeding beyond explicit packs so canonical detector-led starter tasks and workspace-bootstrap repo starters can teach the same task-authoring pattern, and `ota workspace tasks` now surfaces declared descriptions in both text and JSON output.
- added `ota completion --setup` as an idempotent shell-completion installer that auto-detects the current shell when possible, writes the managed completion hook into the right profile or completion file, adds `ota completion check` for hook verification plus current binary-path visibility, adds `ota completion <shell> --script` for raw registration-script inspection, and keeps the existing `ota completion <shell>` manual hook guidance for explicit setup.
- hardened zsh completion setup so it now writes a managed `_ota` completion file alongside the profile hook, which makes `ota <TAB>` work reliably even in shells that bind `<TAB>` through wrappers such as `fzf-completion`.
- made zsh completion setup stable across `XDG_CONFIG_HOME` changes by pinning the managed support file to `~/.config/ota/zsh/_ota`, and made `ota completion zsh` print a complete manual setup with both the `_ota` file and the `.zshrc` loader.
- aligned `ota completion --setup` and successful `ota completion check` output with the shared rich CLI styling so completion setup summaries use the same colored key and status treatment as the rest of the interactive surface.
- added `ota execution plan` as a read-only execution inspector so users and automation can see the resolved backend, lifecycle, image, engine selection, target strategy, compact contract identity, and effective overrides without running `ota up` or `ota run`.
- added `ota workspace execution plan` as the workspace-level execution inspector so users and automation can see per-repo resolved backend, lifecycle, image/provider/target selection, compact repo contract identity, and honest unrunnable execution failures without running `ota workspace up` or `ota workspace run`.
- tightened execution and dry-run detail rows so `Execution`, `Contract`, `Plan`, and `Dry run only` sections now render `→ Label: value` without the extra padded spacing around the arrow.
- added shell-completion guidance with contract-aware dynamic suggestions so `ota <TAB>` completes commands, `ota run <TAB>` completes only tasks that have one satisfiable shared invocation across the selected repo/member target set, `ota run <task> <TAB>` completes shared task input flags plus constrained values that remain valid across that target set, `ota env --task <TAB>` completes task names, `ota extensions --run/--publish <TAB>` completes declared extension names, `ota receipt --baseline <TAB>` completes `latest`, `promoted`, and archived receipt files from the active repo, `--member <TAB>` completes monorepo member names, and workspace task/repo filters complete from the active workspace contract while keeping `ota workspace run <TAB>` and `ota workspace run <task> <TAB>` limited to tasks and shared inputs that stay satisfiable across the available repos.

## 1.4.13

- made receipt archive parsing backward compatible with older archived receipts that do not carry `contract_identity.execution.supported`, so baseline history and receipt compare flows keep working across schema drift.

## 1.4.12

- stabilized the release-gate test harness under parallel execution by serializing CLI test invocations and locking cwd-sensitive receipt fixtures so macOS CI no longer races the compact contract identity checks.

## 1.4.11

- added compact `contract_identity` output on repo execution surfaces so `ota receipt`, `ota up --json`, `ota up --dry-run` text/JSON, and monorepo preview members can show declared project, selected metadata, execution intent, and compact contract counts without inlining the full contract.
- extended compact `contract_identity` output into workspace execution receipts so `ota workspace up`, `ota workspace run`, and `ota workspace receipt` expose the workspace contract identity and workspace repo/policy counts in both JSON and receipt text.
- expanded `ota receipt --baseline ...` diff output with symmetric current/baseline contract identity details so compare consumers can see both the stable repo-local identity key and the compact declared contract summary on each side.
- made update notices more reliable on Windows by preferring the faster curl-based release check there when available, hardening the PowerShell fallback for GitHub TLS, and giving the background notice path a less brittle wait budget.

## 1.4.10

- added discoverable `-V, --version` help output so users can find the version flag directly from `ota --help`.
- made the Windows bootstrap installer fail cleanly when `ota.exe` is locked by a running process, with direct guidance to close ota processes and rerun the installer.
- added `ota receipt --fail-on-new-blockers` for baseline compare mode so receipt diffs can exit non-zero only when they introduce new blocker findings, while exposing additive diff gate metadata in JSON and text output.
- added provider-neutral receipt baseline promotion with `ota receipt --archive --promote-baseline` and `ota receipt --baseline promoted`, plus additive baseline provenance fields so diff output can explain exactly which repo-owned baseline was selected.
- added execution target reporting to execution receipts and container probe diagnostics so receipt targets reflect the actual execution target and failed container probes are classified more precisely.
- enhanced execution summaries for ephemeral containers so they keep real target names and use stable ephemeral container naming during task execution and diagnosis.

## 1.4.7

- expanded diagnosis provenance so `ota doctor --json` findings and `ota explain` steps can trace repo-contract, org-policy, and repo-signal sources without inventing a parallel diagnosis schema.
- clarified policy-backed provisioning text so execution summaries show mapped package aliases like `node (package: nodejs)` instead of only the contract key.
- made native and container tool/runtime diagnosis probe-aware so `ota doctor` now distinguishes missing commands from failed or unparseable version probes, and surfaces the resolved executable path plus probe command in both text and JSON evidence.
- added `Image:` to execution summaries and receipt JSON when container execution is selected, while keeping `Target:` reserved for real named targets such as persistent containers and remote backends.
- made update-check failures honest but non-spammy across platforms by showing a lightweight failure notice only after successful commands and rate-limiting repeated failed checks locally, while keeping the existing newer-version notice unchanged.

## 1.4.6

- added `ota receipt --baseline <latest|FILE>` as the first receipt compare surface, classifying findings as introduced, resolved, or unchanged against an archived or explicit repo receipt JSON baseline without mutating repo state.
- added `ota receipt --history` as a read-only repo archive index over `.ota/receipts`, with stable text and JSON output for archived receipt inspection without rerunning diagnosis, skipped malformed archive reporting, and explicit repo-directory or `ota.yaml` path semantics.
- fixed `ota init --json --dry-run` overwrite failures so machine-readable `error` strings stay plain and `next` remains a separate JSON field with no embedded ANSI styling or `Next:` block.
- documented the official `ota-run/action@v1` GitHub Actions workflow, linked it from the command and hosted-validation references, and corrected GitHub-hosted install snippets to add the ota install directory to `GITHUB_PATH` for direct CLI steps.
- expanded the GitHub Action reference to document the full shipped input and output contract, plus the canonical `ota-run/action` repo link for examples and releases.
- added a provider-neutral CI workflow reference that explains the canonical `validate` + `doctor` + plain annotations + archived receipt split for non-GitHub runners such as GitLab CI, Jenkins, and CircleCI.
- fixed execution-summary status rendering so `NOT READY` post-execution failures render as `failed`, kept only pre-execution blockers as `blocked`, lowered the summary status labels, and pinned the internal GitHub readiness workflow to ota `1.4.4`.

## 1.4.4

- added a canonical `Status:` line to the shared execution-summary block so `ota run`, `ota up`, and workspace execution summaries now show an explicit execution outcome without changing receipt JSON shape.

## 1.4.3

- fixed the policy-backed provisioning docs and backend formatting after the release-gate CI cleanup, including the package-rule heading and related provisioning output formatting.

## 1.4.2

- fixed `ota diff` to exit `0` when comparison succeeds, even when differences exist, matching the command reference semantics.
- aligned policy-backed provisioning fixtures with explicit package identifiers for OS package managers so release-gate coverage matches the shipped policy rules.

## 1.4.1

- added first-class detect ownership tracking with `owner_kind` on existing-contract comparison JSON, persisted ota-managed field ownership under `metadata.ota.detect.field_ownership`, and tightened drift so normal detect/doctor warnings only treat ota-managed fields as detector-owned while rewrite preview still shows full replacement impact.
- fixed detect write-mode JSON so successful `--write`, `--merge`, and `--rewrite` responses now return the exact written contract, including persisted ownership metadata, and detect merge now fails clearly when `metadata.ota.detect` cannot be recorded because an existing metadata path is not mapping-shaped.
- extended `ota init --json` with per-field starter provenance so machine consumers can distinguish detector-inferred contract fields from template-derived starter defaults, including `source` and `confidence` on detector-backed entries.
- extended `ota workspace init --json` and `ota workspace detect --json` with per-field scaffold provenance so machine consumers can distinguish workspace-derived repo entries, preserved workspace-declared merge fields, and template-derived workspace defaults.
- added `ota policy init` with `--dry-run`, `--json`, and explicit starter presets (`required-sections`, `provisioning`, and `agent`) so teams can scaffold a conservative minimal `.ota/org-policy.yaml` starter or choose a stronger reviewed starting point without overwriting an existing policy pack.
- made bare `ota policy init --dry-run` more discoverable by listing the preset preview commands alongside the default minimal policy-pack preview, while keeping preset-specific dry runs faithful to the exact file they would write.
- added semver-aware policy-backed provisioning approval so `approved_versions` can authorize major shorthand and semver ranges, while doctor JSON and policy-aware receipt text now surface `requested_version`, `normalized_requirement`, `policy_match`, and `resolved_version` without inventing concrete install versions from range-only policy.
- added policy-backed provisioning `package` mappings so org policy can pin backend install identifiers (required for `apt`, `dnf`, `pacman`, `winget`, `choco`, and `scoop`) while keeping contract keys stable, and surfaced that mapping in provisioning JSON and previews.
- fixed the `ota policy init --preset provisioning` example to avoid duplicate YAML keys and show a valid OS package mapping.
- added `--archive` on receipt commands to persist JSON receipts under `.ota/receipts` with bounded retention, and exposed `archive_path` in receipt output.

## 1.4.0

- hardened premium text wrapping so `Why:` lines, detail lines, and detect drift bullets stop inserting premature newlines and now rely on real terminal width instead of stale formatter caps.
- enriched existing-contract `ota detect` comparisons with stable `provenance_key` labels and direct detector `source`/`confidence` evidence in both JSON and premium text output.
- added `ota receipt` as a read-only repo receipt artifact with `--json` and `--mode`, reusing the existing execution-receipt model for CI and audit workflows without mutating repo state.
- fixed `ota receipt --json` failure routing so contract load or validation failures now emit the shared `ValidateFailure` payload on stdout, matching the published schema and JSON reference.
## 1.3.1

- refreshed the premium doctor plain-text snapshot so the advisory ephemeral-lifecycle warning stays stable in the release gate.

## 1.3.0

- fixed `ota init --json --dry-run` so the machine-readable preview now matches the reviewed starter contract, including derived starter defaults such as a safe minimal `agent` block when ota can infer one.
- tightened starter agent generation to fail closed on writable paths: when ota cannot infer safe writable paths, it now omits the generated `agent` block instead of granting repo-wide writes with `writable_paths: [.]`.
- tightened detect drift removals so task commands and `safe_for_agent` entries are no longer suggested for removal on detector silence alone.
- added explicit detect-comparison `ownership` and `provenance` fields in JSON so editors and CI can distinguish repo-signal add candidates from stale repo-contract drift without re-deriving that boundary from prose.
- added stable machine-readable `provenance_key` fields to finding and explain JSON output while keeping the human-readable `provenance` labels unchanged.
- fixed the hosted-validation workflow doc so the shipped `ota.yaml` Postgres example now matches the actual service contract schema.
- added broader docs example validation so the canonical repo and workspace reference pages are now exercised by the shipped contract validators in test.
- documented the maintainer-led governance and contribution-policy boundary across the README and policy docs so the public repo story matches the actual operating model.

## 1.2.4

- added `ota up --stream` for repo-level text runs so provisioning, required service `start` commands, and the `setup` task can expose raw live child output on demand while default `ota up` stays compact and keeps failed child output inside the final report.
- added backend-aware provisioning diagnosis so `ota up` now surfaces higher-level installability failures across the shipped adapters while preserving raw backend output, and `ota doctor --mode container` now uses safe non-mutating installability probes across the shipped mutating provisioning adapters, with richer `apt` classification for pinned-version unavailable, package unavailable, and apt-index/source failures.
- taught `ota up` to reuse the read-only installability probe when a provisioning command fails with only generic backend stderr, so runtime-manager and package-manager failures keep the richer diagnosis without losing the original backend output.

## 1.2.3

- improved `ota clean --stale` so it keeps querying available container engines, surfaces engine query failures, and removes exited ota-managed containers without silently treating daemon or permission errors as an empty stale set.
- tightened stale-cleanup output and container-engine simulation coverage so older `ota-*` containers stay discoverable and the cleanup path stays explicit about what it matched or removed.

## 1.2.2

- added `ota up --dry-run` with text and JSON preview output so operators can review the selected backend, lifecycle, target, planned provisioning/setup work, current skips, and first blocker before ota mutates repo or execution state.
- fixed container execution and `ota up --dry-run --mode container` probing to use a non-login shell inside images, preserving image-defined `PATH` entries such as Rust toolchains, and made preview output/json show the selected container image with preview-specific rerun guidance.
- made `ota up --dry-run` service planning truthful by only listing `start service ...` when a service actually declares `start`, and by separating readiness checks from service starts in the preview plan.
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
