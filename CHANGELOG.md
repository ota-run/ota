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

- added workspace-owned repo task bindings under `repos.<name>.tasks.<workspace_task>.task`, so
  `ota workspace run <task>` no longer requires every repo to share the same local task name;
  workspaces can now map one shared workspace verb such as `prepare-dev` onto repo-local tasks
  like `generate-sdk`, while `ota workspace tasks` and workspace run receipts expose the resolved
  repo task truth instead of leaving mixed-name cross-repo execution buried in shell or tribal
  knowledge
- widened `prepare.kind: sequence` from structural prepare-only chaining to mixed structured setup
  ownership: ordered `prepare.steps` can now combine typed prepare lanes such as
  `dependency_hydration` and `tool_bootstrap` with deterministic native bootstrap mutations such
  as `ensure_env_file`, `ensure_file`, `ensure_directory`, `ensure_git_checkout`,
  `ensure_container_network`, and `reset_compose_service_volume`, so repos can model multi-step
  browser/setup truth like env materialization plus browser bootstrap without collapsing back to
  shell glue
- widened `prepare.kind: tool_bootstrap` with `prepare.tool: playwright_browsers` and
  `prepare.source.kind: node_package_manager` and `prepare.source.kind: poetry`, so repos can own
  documented `playwright install` browser bootstrap lanes structurally under npm, pnpm, yarn, bun,
  or Poetry instead of falling back to repo-local shell glue or leaving browser-runtime proof
  half-modeled
- tightened active repo-execution conflict ownership so Ota now derives shared writable-path
  ownership from the full selected task path, not just the requested leaf task; overlapping runs
  that converge on the same declared `effects.writes` path now fail fast with a `write_path`
  conflict instead of racing through dependency setup and corrupting shared state like
  `node_modules`
- widened structured `compose.kind: down` with `compose.timeout_seconds`, so graceful shutdown
  timeout truth like `docker compose down -t 2` no longer has to fall back to raw command bodies
  when a repo needs deterministic Compose teardown ownership
- widened structured Compose control tasks with `compose.kind: ps`, so staged inspection lanes like
  `docker compose ps main` can stay on the same first-class `compose` surface as `up`, `down`,
  `build`, `stop`, `restart`, `rm`, and `logs` instead of falling back to raw host command glue
- widened `prepare.kind: dependency_hydration` for `source.kind: docker_compose` with ordered
  `prepare.source.files` and `prepare.source.env_files`, while keeping `prepare.source.file` as a
  compatibility alias; Compose-backed image hydration can now own the same multi-file and
  interpolation env-file truth as the rest of Ota’s Compose surfaces instead of collapsing back to
  single-file-only pull glue
- widened `services.<name>.manager.kind: compose` with ordered `manager.files` and
  `manager.env_files`, while keeping `manager.file` and `manager.env_file` as compatibility
  aliases; compose-managed services can now own the same multi-file and multi-env-file overlay
  stacks that task/workflow adapter inputs already support, which closes the split-brain between
  overlay-aware compose tasks and single-file-only service ownership on repo contracts such as
  Open WebUI's Playwright sidecar runtime
- added first-class `action.kind: ensure_git_checkout`, so deterministic repo bootstrap can now
  materialize sibling or vendored Git checkouts under the same native setup/action surface as
  `ensure_file`, `ensure_env_file`, and `ensure_bundle`; Ota clones a declared Git source only
  when the repo-relative checkout path is missing, optionally checks out a declared ref, and
  intentionally leaves existing directories untouched instead of smuggling fetch/reset/update
  semantics into bootstrap; `ota validate` / `ota doctor` now also warn when checkout
  materialization omits `source.ref`, so moving-head pressure bootstrap does not silently read as
  deterministic proof truth
- added first-class `tasks.<name>.effects.adapter_state` for durable adapter-owned task state such
  as Compose volumes, and widened typed dependency hydration so `prepare.kind: dependency_hydration`
  can declare durable state through `effects.writes` or `effects.adapter_state` instead of being
  forced back to shell or fake repo writes on volume-backed Compose lanes
- added Unix host `OTA_HOST_GID` interpolation alongside `OTA_HOST_UID` so native and compose task
  env can truthfully model host user/group projections like `CURRENT_UID=<uid>:<gid>` without shell
  `id -g` glue
- widened `tasks.<name>.variants` so OS-scoped variants can declare `env`, `env_files`,
  `env_bindings`, `inputs`, `requirements`, and `adapter_inputs` without cloning the task body,
  which lets one structured task keep a single executable shape while specializing process,
  prerequisite, and Compose/Bake/Helm overlays per OS
- fixed persistent runtime ownership regeneration so recreated `.ota/state/ownership-id` files no
  longer mint a fresh repo identity and strand Ota-managed containers or dependency-isolation
  assets outside repo-scoped cleanup; repo ownership now regenerates deterministically from the
  working directory when state metadata is rebuilt
- fixed starter agent-boundary inference so `ota init` and `ota detect` no longer implicitly mark
  dependency-hydration `setup` lanes as `agent.safe_tasks`; setup can still remain the truthful
  agent entrypoint, but only genuinely safe verification tasks are inferred into the starter
  safe-task surface by default
- added first-class `launch.kind: compose` for long-running `docker|podman compose up` runtime
  starts, so repos can model persistent Compose-owned runtime launch truth under `launch` instead
  of forcing packaged stack startup through `launch.kind: command`; the same pass teaches opaque
  service-start governance to prefer `launch.kind: compose` when the shell lane is really
  `docker|podman compose up`
- fixed native structured Compose execution so adapter-owned compose working directories stay owned
  by the task adapter surface instead of being duplicated onto projected command `cwd`; native
  `compose` task bodies and `launch.kind: compose` no longer double-join adapter roots like
  `docker/docker` during spawn
- widened `ota proof runtime` with `--host-port`, so runtime proof can now consume the same
  projected host-listener override surface as `ota run` and `ota up` when concurrent pressure or
  host-port conflict isolation needs a remapped publication
- widened native structured Docker Compose host-port remap from generic container publication
  only to explicit native Compose publication ownership, so `ota run`, `ota up`, and
  `ota proof runtime` can now remap `docker compose up` listener publications through
  `tasks.<name>.runtime.listeners.<listener>.project.publication.compose.service` instead of
  failing early on native bind conflicts; validator now requires that publication owner truth to
  stay explicit and governed
- tightened native Compose bind-conflict guidance so `ota up` and run/proof failure surfaces now
  state explicitly when a listener is being published through native `docker|podman compose` and
  why `--host-port` is not available on that lane today, instead of implying the conflict should
  be solved through the generic projected-listener override path
- widened `tasks.<name>.compose` with first-class `compose.kind: up`, `compose.kind: down`,
  `compose.kind: build`, and `compose.remove_volumes: true` for `compose down -v`, so repos can
  model staged `docker|podman compose up [-d] <services...>`, project-scoped `compose down`,
  `compose down -v`, and `compose build [services...]` lanes without falling back to opaque
  host-shell `docker compose ...` glue; the same pass also widened workflow `prepare.task` to
  accept finite native `compose` bodies and tightened `ota up --dry-run --json` action text so
  `command` and `compose` tasks publish the actual structured execution preview instead of generic
  task placeholders
- widened `tasks.<name>.compose` again with first-class `compose.kind: restart`,
  `compose.kind: rm`, `compose.kind: logs`, `compose.force_recreate: true` for
  `compose up --force-recreate`, `compose.force: true` for `compose rm -f`, and
  `compose.follow: true` for `compose logs -f`, so repos can model Compose control lanes without
  falling back to opaque host-shell `docker|podman compose ...` glue
- widened `tasks.<name>.compose` again with first-class `compose.kind: stop`, so repos can own
  truthful stopped-container preconditions such as `docker|podman compose stop <services...>`
  structurally before `compose rm` cleanup instead of falling back to repo-local shell glue
- fixed runtime-proof workflow-instance selection and container Corepack run-path fulfillment, so
  `ota proof runtime --workflow <name>@<instance>` now preserves instance overlays in topology,
  cleanup, and proof artifacts, and container-mode setup/run lanes now honor selected container
  mode contexts while executing Corepack-owned package managers through the selected run path
- fixed container backend fulfillment for Corepack-owned package managers, so tasks that declare
  `toolchains.node.fulfillment.source: corepack` no longer fail preflight just because fresh
  container images do not already ship `pnpm` or `yarn`; ota now treats those package managers as
  source-managed task capabilities instead of image preconditions
- added first-class workflow-instance selection with `workflow@instance`, so one declared workflow
  can now own named instance overlays for task env, adapter inputs, and surfaced host ports without
  splitting that runtime family into pseudo-workflows; ota now also injects `OTA_HOST_HOME` for
  truthful host-home clone/cache derivation, and Penpot can model ws0/ws1/ws2 as one governed
  workflow family instead of one flattened ws0-only contract
- widened workflow instances with `topology.requires_instances`, so `ota up --workflow <name>@<instance>`
  can now honor declared sibling-instance bring-up order such as `ws1+` requiring `ws0` first, and
  validator now rejects broken or cyclic instance-topology declarations
- widened workflow env rendering with `env.profiles.<name>.render.files[]`, so selected workflows
  can now materialize structured `json` and `toml` artifacts from ordered merge sources with
  instance-aware placeholder substitution and optional `merge_into_existing`; Penpot can now model
  per-workspace MCP/client config generation as contract truth instead of a repo-local merge helper
- widened workflow-instance task overlays with `workflows.<name>.instances.<instance>.tasks.<task>.runtime`,
  so selected workflow instances can now specialize existing task runtime listeners and readiness
  fields without inventing a parallel workflow-local listener model; instance-scoped bind/project
  port truth now merges onto the base task runtime and stays visible to validation, dry-run, and
  runtime proof
- fixed runtime surface/listener split-brain by allowing one explicit task listener to share the
  same name as one attached published surface; surface-backed runtime proof and workflow exposure
  now align with the validator instead of forcing duplicate or renamed publication truth
- added first-class interactive workflow attach ownership with `workflows.<name>.attach.task` and
  `tasks.<name>.compose.kind: attach`, so detached session workflows can declare one canonical
  re-attach lane and `ota up --attach` now keeps the service runtime running, proves readiness, and
  then enters the declared interactive session instead of assuming the run task itself stays
  foreground
- fixed workflow-instance cleanup topology detection so shared Compose projects such as instance
  prerequisites or shared infra no longer count as dependent instance presence; runtime proof and
  workflow cleanup now only block on instance-specific dependent projects that are actually present
- hardened unreleased git installs in the official shell and PowerShell installers by defaulting
  `CARGO_NET_GIT_FETCH_WITH_CLI=true` on `--from-git` lanes, so contract-owned git revision and
  branch bootstrap paths no longer depend on Cargo's flakier libgit transport on hosted runners
- added first-class `tasks.<name>.compose.detach` for detached `docker|podman compose exec`
  lanes, so repos can model truthful in-service bootstrap/start commands without opaque shell
  `docker exec -d ...` glue
- added first-class `tasks.<name>.compose` execution bodies for structured `docker|podman compose exec/run` lanes, so repos can model finite service-side commands without falling back to opaque shell argv
- widened `prepare.kind: dependency_hydration` with optional `prepare.source.compose` invocation wrappers, so typed package hydration lanes such as npm, Bundler, uv, Poetry, Maven, Gradle, Cargo, and dotnet restore can execute through declared Compose service truth as structured compose wrapper execution instead of raw task-body shell glue
- fixed V10 receipt-diff correlation fallback so `ota receipt --baseline latest` now marks
  resolved blockers as likely related when contract drift clearly explains a resolved finding and
  no new material blockers were introduced; semantic snapshot diffs no longer collapse to
  `no_clear_correlation` just because the change fixed one blocker while other unrelated blockers
  still remain
- tightened V10 likely-related ordering for missing env drift so adjacent metadata siblings such
  as `secret` no longer ride along with causal `required` changes; receipt diff now publishes the
  sharpest env assumption instead of same-owner noise
- fixed `ota receipt --snapshot latest` and `--baseline latest` fallback for archived repo
  receipts that predate workflow-scoped identity widening, so Ota now resolves the newest
  matching contract lineage first by exact identity and then by same-contract lineage instead of
  failing archived snapshot and baseline lookup on valid older receipt archives
- added first-class `action.kind: reset_compose_service_volume`, so destructive Compose-managed
  data reset lanes can be modeled as structured Ota actions instead of shell `docker compose` /
  `docker volume rm` glue

- widened `effects.network_kind` with first-class `integration_test`, so live or staging-backed
  verification lanes no longer need to collapse into `broad`; doctor findings, policy effect
  governance, and contract docs now carry a dedicated network classification for real-service test
  paths
- fixed `integration_test` effect-kind aggregation so agent-safe advisories and selected-task
  effect closure reporting preserve real-service verification lanes instead of collapsing them into
  dependency hydration, and validate now nudges service-backed test tasks onto
  `effects.network_kind: integration_test`
- fixed native `ota proof runtime` post-run diagnosis to activate the same `mise` path surface as
  `ota doctor`, `ota up`, and `ota run`, so proof no longer re-checks toolchain-owned runtimes on
  an older parent PATH after the child `ota up --stream` path already resolved through
  `HOME/.local/bin` / `mise`-owned host toolchain truth
- injected `OTA_HOST_WORKSPACE` and `OTA_HOST_UID` into task execution so host-launched tasks can
  pass real repo-root and host-user identity through contract env truth instead of shell glue such
  as `pwd` or `id -u`
- fixed detached service-start readiness proof in `ota up` and `ota proof runtime`: when a
  service launcher such as `docker compose up -d` exits successfully before the runtime itself is
  ready, ota now keeps probing until the declared readiness budget expires instead of treating the
  launcher exit as the failure boundary
- deduped identical doctor findings and widened `.gitignore` hygiene recognition so parent `.ota/`
  ignore entries satisfy the repo-artifact protection checks
- widened task mode branches with first-class `execution.modes.<mode>.depends_on`, so one task can
  keep stable identity while truthfully switching prerequisite chains per execution plane; planner,
  runner, previews, `ota up` setup adjustment, task summaries, and boundary advisories now all
  resolve selected dependencies from the active mode branch instead of silently falling back to the
  task-level list
- added machine-readable dependency-plane provenance to `ota run --dry-run --json`, so
  `plan.dependency_steps[]` now reports each planned task step's parent task, selected backend,
  selected context, and backend-selection source including inherited parent backend resolution
- widened the same dependency-plane provenance into validate advisories and execution receipts, so
  `warning_details[].provenance` now explains selected dependency-boundary source lanes and
  task-backed `receipt.dependency_steps[]` preserves the backend-selection truth ota actually ran
- refined `ota proof runtime` artifact diagnostics so `proof/*/doctor.json` now upgrades generic
  run-exit blockers to concrete container-engine-unavailable execution findings when the captured
  `up.log` shows Docker or Podman backend reachability failures
- widened `ota receipt` with first-class workflow selection, so `ota receipt --workflow <name>`
  now reuses the selected workflow's env-profile/readiness lane and keeps `latest`, `promoted`,
  and archived receipt/snapshot selection scoped to that workflow instead of mixing receipt
  history across unrelated workflow baselines
- tightened V10 selected task-path external-state correlation so task
  `effects.external_state` drift is published as likely related instead of falling back to
  `no_clear_correlation`
- tightened V10 native prerequisite correlation so selected task-path
  `requirements.native` and `requirements.any_of[..].native` ownership outranks broad
  `native_prerequisites.*` declaration drift in semantic snapshot comparisons
- fixed selected-path standalone tool acquisition version trust across `ota doctor`, `ota up`,
  and `ota run`: task-local wildcard requirements such as `tools: { yq: "*" }` now preserve the
  exact owned version declared under `tools.<name>` when the selected path resolves through
  `tools.<name>.acquisition`, instead of collapsing that selected acquisition lane back to `*`
- fixed native post-fulfillment diagnosis for source-managed standalone tools: when Ota acquires a
  tool through `tools.<name>.acquisition.provider: release_asset`, follow-up `ota doctor` and
  `ota up` diagnosis now probe the repo-managed `.ota/state/source-managed/bin` path too instead of
  falsely saying the tool is unavailable on `PATH` after fulfillment already succeeded
- fixed repo-managed release-asset probe execution across provisioning and doctor: follow-up
  `vale --version` style probes now execute the managed binary correctly from the repo working
  directory instead of tripping over relative `.ota/state/source-managed/bin/...` paths during
  post-fulfillment verification
- Normalize detected Node engine unions like `22 || 24` into explicit semver branches and honor
  those unions during runtime/toolchain matching, so detector-owned Node version truth no longer
  misclassifies valid installed Node majors.

- added semantic contract snapshot identity to repo and workspace receipts, so
  `ota receipt --json` and `ota workspace receipt --json` now emit a normalized
  `receipt.contract_snapshot_hash` on every successful receipt and archive the matching
  normalized contract JSON under `.ota/contracts/sha256-....json` when `--archive` is used
- widened shipped `ota diff` onto normalized semantic contract truth, so diff no longer compares
  raw YAML structure; it now compares normalized assumption keys, accepts archived receipt JSON
  plus archived `.ota/contracts/...` snapshot JSON as inputs, and emits additive `category` /
  `risk` metadata per changed assumption
- added receipt-to-receipt semantic contract drift correlation, so `ota receipt --json --baseline`
  now compares the selected archived baseline snapshot against current normalized contract truth
  when available and emits additive `summary.comparison.contract_snapshot_changed`,
  `contract_changes[]`, and `likely_related_changes[]`; the initial correlation lane now covers
  env, tool, runtime, check, task, and required-service blockers instead of only the narrowest
  missing-env cases
- clarified semantic diff input identity, so `ota diff` text output now shows each side's resolved
  semantic input kind and any archive-backed snapshot path, while `ota diff --json` publishes
  additive `base_input` / `target_input` metadata instead of forcing operators to infer archived
  receipt snapshots from path patterns alone
- added first-class archived snapshot inspection under `ota receipt --snapshot`, so operators can
  read archived normalized contract truth directly from `latest`, `promoted`, archived receipt
  JSON, or archived `.ota/contracts/...` snapshot files without routing that inspection through
  `ota diff` or receipt-correlation output
- widened `ota receipt --snapshot` inspection evidence, so snapshot text and JSON now publish the
  canonical extracted assumption-set hash plus assumption count alongside the whole-snapshot hash
  and archived normalized contract JSON instead of forcing operators to derive semantic snapshot
  identity indirectly from the payload
- added additive `receipt.assumption_set_hash` identity for repo and workspace receipts, so
  archived and live receipt JSON now fingerprint the canonical extracted semantic assumption map
  separately from whole-snapshot identity and receipt diff can surface the same identity on both
  baseline and current sides
- refined receipt diff correlation with an explicit advisory verdict under
  `summary.comparison.correlation`, so automation can distinguish `likely_related`,
  `possibly_related`, and `no_clear_correlation` without inferring correlation posture from
  `likely_related_changes[]` alone; `possibly_related` is now reserved for coarse same-family
  overlap, while unrelated contract drift stays `no_clear_correlation`
- fixed archived receipt finding identity round-trip for V10 compare, so
  `ota receipt --json --baseline ...` no longer emits phantom `introduced[]` / `resolved[]`
  findings when the archived baseline and current receipt are semantically unchanged; archived
  flat `code` / `category` / `owner` finding fields now deserialize back into stable finding
  identity instead of collapsing to anonymous `OTA_DOCTOR_FINDING_UNKNOWN` matches
- tightened `likely_related_changes[]` ordering across adjacent plausible lanes, so receipt diff
  now ranks the published contract-change evidence globally by semantic match strength instead of
  emitting changes in finding-visit order when multiple adjacent assumptions all look plausible
- tightened missing tool/runtime correlation ordering across broad declaration vs selected-path
  requirement truth, so `likely_related_changes[]` now prefers
  `tasks.<name>.requirements.tools.<tool>` / `.requirements.runtimes.<runtime>` /
  `.requirements.toolchains.<toolchain>` over broader `tools.<name>` or `toolchains.<name>`
  catalog drift when the selected task path is the sharper operational owner of the failure
- moved more receipt diff entity recovery into declared doctor finding metadata, so env, tool,
  runtime, check, and service correlation matches now prefer published owner/entity truth over
  CLI-side summary parsing when ranking exact owner, requirement-reference, and name-reference
  contract changes
- tightened `likely_related_changes[]` ordering inside the same semantic owner lane, so receipt
  diff now prefers the nearer declared owner subtree before generic path-depth tie-breaks and
  publishes root `kind`-level evidence ahead of deeper nested detail when both changes are
  plausible for the same finding
- reduced residual receipt-diff fallback taxonomy for named-entity findings, so tool, runtime,
  check, and service correlation now stays on declared doctor owner/entity metadata instead of
  duplicating those lanes in CLI-only fallback matching; only the broad undeclared task-family
  fallback remains
- widened declared receipt-diff workflow correlation for named probes and surfaces, so workflow
  readiness failures now match top-level probe definitions, reusable top-level surface owners,
  workflow probe/surface references, and runtime surface definitions before falling back to broad
  workflow-family drift
- refined same-lane receipt-diff evidence ordering for workflow probes and surfaces, so adjacent
  endpoint drift such as probe path or surface port now publishes ahead of weaker success-rule,
  path, or metadata changes when multiple nearby assumptions are all plausible
- widened coarse receipt-diff overlap recovery to respect declared owner roots too, so broad
  correlation posture now still recognizes reusable top-level `surfaces.*` and
  `readiness.probes.*` drift as the same semantic family when exact workflow matching is not
  available
- widened runtime receipt-drift correlation to include execution-context runtime requirements, so
  runtime version-mismatch blockers can now correlate directly to
  `execution.contexts.<name>.requirements.runtimes.<runtime>` instead of falling back to
  `no_clear_correlation`
- added a dedicated local core spec for semantic snapshots and receipt correlation, so the V10
  operator path is documented inside `docs/spec/` instead of only through dispersed command pages
  and the public site reference
- documented the public semantic snapshot operator path in core README and command/spec docs, so
  humans and agents now have one canonical page for archived semantic truth, `ota diff`,
  `ota receipt --snapshot`, and receipt-to-receipt drift correlation
- refined receipt diff check-correlation ordering with current-contract check-name recovery, so
  named check failures can resolve `checks[<index>]` from the current contract even when the diff
  did not change `checks[...].name`, and the changed check body now outranks adjacent task-body
  drift instead of losing because array-index ownership was not recoverable from diff paths alone
- widened declared check-correlation ordering across adjacent reference lanes, so named check
  failures now treat explicit check-reference changes such as `tasks.<name>.requirements.checks[]`
  and workflow check-reference lanes as stronger evidence than a nearby generic task-body change
- widened declared service-correlation ordering across workflow reference lanes, so named service
  findings now treat `workflows.<name>.services.required[]` as stronger evidence than a nearby
  generic workflow change instead of flattening both into the same broad workflow lane
- widened declared env-correlation ordering across path-scoped requirement lanes, so named env
  findings now treat `tasks.<name>.requirements.env[]` as stronger evidence than a nearby generic
  task-body change instead of flattening both into the same broad task lane

## 1.6.21

- fixed structured Helm dependency hydration so repository-cache state no longer lives under the
  chart tree and poison `helm dependency build` with oversized cached index files; the runner now
  keeps Helm repo state out of chart inputs while preserving deterministic per-chart isolation
- widened structured task commands with `command.cwd` and `launch.kind: command` plus `launch.cwd`,
  so finite tasks and structured service launches rooted in subdirectories no longer need fake
  `cd ... && ...` shell bodies just to express working-directory truth; previews and task listings
  now surface the declared command cwd directly
- tightened replaceable-shell governance for package-manager install lanes, so obvious dependency
  hydration shells such as `pnpm install` and `yarn install --immutable` now point authors toward
  `prepare.kind: dependency_hydration` instead of the weaker generic `command` suggestion
- clarified bootstrap-source discoverability across the contract and install docs, so
  `agent.bootstrap.ota.source` now explicitly documents the exact shell and PowerShell mapping for
  `kind: version`, `kind: git_rev`, and `kind: branch`, and the setup/action/hosted-validation
  docs now point humans and agents at the same canonical install truth
- tightened `ota tasks` human output so task listings now show the effective runnable default mode,
  include first-class `command` bodies in command previews, hide empty placeholder rows, and keep
  `ota tasks` / `ota tasks --use` default plus alternate runnable mode guidance aligned with the
  effective mode instead of surfacing empty `Default Mode: -` task blocks, redundant default-only
  mode rows, or default-branch duplication inside `Mode Branches:`; multi-mode task output now
  also shows explicit alternate runnable invocations
- documented the new standalone `ota-run/action@v1` `source: contract` install mode in the
  GitHub Actions and hosted-validation specs, so first-party docs now cover both the canonical
  `setup + action install: never` split and the wrapper-owned contract-consumption path
- aligned the Ota repo's own `agent.bootstrap.ota.source` with deterministic unreleased proof
  truth by pinning active `1.6.21` work to an exact git revision instead of claiming unreleased
  `v1.6.21` release availability

- documented the unified GitHub install contract around `ota-run/setup@v1` contract mode, so
  repos can keep `agent.bootstrap.ota.source` as the single install truth without introducing a
  second GitHub-specific helper surface
- sharpened container-mode doctor probe guidance for repo-local executables backed by unresolved
  dependency hydration, so high-confidence Bundler and `node_modules` hydration failures now tell
  authors to hydrate the selected container dependency lane first instead of surfacing only a
  generic probe failure
- widened `agent.bootstrap.ota` with first-class `source.kind: version | git_rev | branch`, so
  repo contracts can now declare released proof pins, deterministic unreleased proof pins, and
  active pressure-testing branch truth structurally instead of hiding ota install ownership only
  inside raw shell strings; ota now renders shell and PowerShell installer commands from
  structured source truth, validates source-specific pin semantics, and warns when branch
  tracking is used as non-deterministic pressure bootstrap drift

- widened repo-level `tools.<name>.acquisition` with `provider: release_asset`, so standalone CLI
  tools can now own exact downloadable binary fulfillment directly in the repo contract instead of
  only through org policy; validator, doctor, selected-path preview JSON, and run-path
  provisioning now all honor the same surface
- widened `provider: release_asset` so platform assets may also declare archive extraction
  metadata; ota now supports downloading a release archive, extracting one declared executable,
  and installing that executable into the managed tool path for the selected task/workflow lane
- widened archive-backed `provider: release_asset` again so `archive.format` now supports `zip`
  alongside `tar_gz`, closing the first-class Windows-style release archive lane for standalone
  CLI ownership
- fixed execution-context requirement projection so container-selected and other non-native task
  paths preserve top-level detailed tool acquisition metadata instead of collapsing to bare
  version-only tool requirements during preview and doctor selection
- publish selected-path `provisioning` and `provisioning_request` in `ota run --dry-run --json`
  so direct tool acquisition source truth is machine-readable in task preview output
- allow `tools.<name>.acquisition.source_config` for package-manager-backed standalone CLI
  ownership and pass that provider truth through doctor provisioning requests
- widened `tools.<name>.acquisition` with package-manager-backed standalone CLI ownership across
  `apt`, `brew`, `winget`, `choco`, and `scoop`, including OS-specific `platforms.<os>.acquisition`
  overrides so repos can keep tool identity and host fulfillment truth in the `tools` layer
  instead of splitting standalone CLIs such as Helm across `tools` and `native_prerequisites`
- widened first-class adapter ownership with `adapter_inputs.overlays.helm.*`, so Helm task paths
  can now declare contract-owned `cwd`, `values_files`, `chart`, `release_name`, and `namespace`
  truth instead of hard-coding that selection in argv or shell
- Helm dependency hydration now owns clean-host repository bootstrap too: ota reads chart
  repositories from `Chart.yaml`, seeds them into isolated repo-owned Helm state under
  `.ota/state/helm/...`, and then runs `helm dependency build .` without depending on preexisting
  user-global `helm repo add` state
- Fix Helm tool version probing to use `helm version --short` before generic fallback probes, so
  Helm-backed contract requirements and dependency hydration lanes do not fail on healthy hosts
  due to the invalid generic `helm --version` probe path.

- fixed `ota skills install` so Codex and Claude installs now stage the full canonical Ota skill
  payload instead of a partial bundle; the installer now includes the referenced `agents/` and
  `references/` support files and validates that the staged skill tree is complete before replace
- widened first-class Node dependency hydration with explicit npm `force` ownership via
  `prepare.source.force: true` on `source.kind: node_package_manager` with `manager: npm` and
  `mode: install` or `mode: ci`, so repos can now model those exceptional override lanes
  structurally under `prepare.kind: dependency_hydration` instead of burying them in shell
- validate/doctor now warn when contracts declare the exceptional npm `--force` hydration path,
  and they also flag raw `npm install --force` / `npm ci --force` task bodies as replaceable
  structured dependency hydration
- tightened adapter-overlay ownership internals so backend adapter resolution now runs through the
  same adapter-field registry that already owns workflow/task advisory identity, making Compose and
  Bake overlay widening safer without splitting field truth across duplicate match arms
- widened the public adapter-overlay contract from family-specific `adapter_inputs.compose.*` /
  `.bake.*` teaching to the canonical `adapter_inputs.overlays.<family>.*` model, while keeping
  `compose` and `bake` as compatibility aliases for existing contracts
- fixed workflow overlay compatibility projection so legacy compose alias inputs such as
  `env.compose_files`, `env.compose_project_name`, and compatibility `adapter_inputs.compose.*`
  still merge correctly when the canonical workflow contract already uses
  `adapter_inputs.overlays.compose.*`
- tightened runtime-proof auth evidence recovery so likely-cause classification can recover service
  identity from resolved host hints and more endpoint-style auth logs
- refreshed canonical Rust, .NET, and Java examples onto current first-class setup surfaces, so
  setup now uses typed dependency hydration and finite verification/build tasks use `command`
- widened first-class Node dependency hydration with Yarn `inline_builds` support, so repos can
  now model `yarn install --inline-builds` structurally under
  `prepare.kind: dependency_hydration` instead of hiding that setup lane in shell
- validate/doctor now warn when task bodies still shell `yarn install --inline-builds` directly
  instead of keeping that dependency-hydration ownership on typed `prepare.kind: dependency_hydration`
- added first-class `prepare.kind: tool_bootstrap` for contract-owned tool installation, with the
  first shipped slice modeling `tool: uv` via `source.kind: pip` and explicit `source.exe`
- widened task network classification with `effects.network_kind: tool_bootstrap`, so ota can
  distinguish typed tool installation from repo dependency hydration and broader remote-call
  execution in doctor, validate, policy, and receipts
- validate/doctor now warn on replaceable shell-owned Python `pip install uv` task bodies when
  that tool bootstrap truth belongs on typed `prepare.kind: tool_bootstrap`
- agent-safe dependency-hydration governance is now narrower and more truthful: ota no longer
  emits the generic agent-safe dependency-hydration contract advisory when the task path is
  already modeled on the first-class `prepare.kind: dependency_hydration` surface; weaker or
  shell-modeled networked hydration still keeps the warning
- Corepack-backed native activation is now provider-owned instead of repo-burdened when `npm`
  is already available: `ota doctor` no longer blocks on a missing `corepack` shim that Ota can
  bootstrap itself, `ota up` now installs `corepack` before `enable/prepare` when needed, and
  direct native run-path Corepack activation follows the same bootstrap lane
- `ota doctor` now treats inferred task-command and launch-command executables as presence-owned
  prerequisites instead of strict versioned tools, so valid command surfaces no longer fail
  diagnosis just because their executable does not support `--version`; explicit `tools:` contract
  requirements remain strict and still fail on broken or unparseable version probes
- added a first-class typed `systemd` host-service manager lane on top of the existing
  `manager.kind: host` surface: services can now declare `manager.host.kind: systemd` plus a unit
  and optional scope so ota derives host lifecycle and supports structured
  `readiness.kind: systemd_active` without shell `systemctl` glue
- validate/doctor now warn when task bodies still shell `systemctl start`, `stop`, or
  `is-active` directly instead of keeping that host-service ownership on the typed `manager.host`
  plus `readiness.kind: systemd_active` surface
- added a dedicated self-hosted Linux native proof workflow for the typed systemd lane, using a
  temporary user-scoped systemd unit on an OrbStack-backed runner instead of pretending
  GitHub-hosted Ubuntu provides honest host-systemd runtime truth
- improved native run-path failure trust for command-owned Compose tasks: when a native task launches
  `podman` or `docker` and the selected engine backend is unavailable, `ota run` now reports
  first-class `Container engine unavailable` guidance with engine-specific repair steps instead of
  flattening the failure into generic non-zero-exit wording
- added first-class `podman compose` ownership across the shipped Compose-family surfaces:
  compose-managed services can now declare `services.<name>.manager.engine: podman`, docker-compose
  dependency hydration can declare `prepare.source.engine: podman`, and adapter governance/doctor
  now recognize replaceable Compose truth in both `docker compose` and `podman compose` task bodies
- runtime proof now short-circuits fast terminal `ota up --stream` failures from captured
  `up.log`, emits the stronger failure text into proof output and `doctor.json`, and avoids the
  previous long full-diagnosis stall when readiness proof was already terminally blocked
- workflow-owned adapter truth now flows through one internal shared overlay model before task
  projection, so Compose and Bake both bind through the same workflow overlay path and governance
  now points authors at that shared workflow adapter surface more explicitly
- generalized workflow-owned adapter overlays onto a workflow-level canonical surface:
  `workflows.<name>.adapter_inputs.*` now owns cross-adapter Compose/Bake overlay truth, while
  `workflows.<name>.env.adapter_inputs.*` stays accepted only as a compatibility lane and legacy
  `env.compose_files` / `compose_project_name` aliases now resolve toward the workflow-level
  surface
- sharpened runtime-proof root-cause recovery without widening the schema surface: DNS/service-name
  failures now recover lookup-style unresolved hosts more reliably, auth/credential failures can
  recover endpoint-style and `addr=` / `hostname=` host evidence, and readiness-target mismatch
  now catches path drift when the app advertises a live endpoint in proof artifacts
- tightened runtime-proof failure-class ordering onto one shared priority path, so
  install/toolchain startup breaks now classify consistently from proof logs even when the proof
  path only has log evidence and not a richer recovered likely-cause object
- `ota detect` write modes now publish first-class field-admission rationale alongside detect
  ownership metadata: written contracts and JSON output record
  `metadata.ota.detect.field_admission` as `direct` for high-confidence detector-owned fields and
  `promoted` for the narrow conservative detect-write starter fields admitted by policy
- tightened the conservative `ota detect --write` lane without broadening it into a full starter
  rewrite path: sparse Node repos can now still seed `toolchains.node` plus starter-owned npm
  dependency hydration from repo-root markers, and solution-backed .NET repos now treat
  `.sln`-derived `project.name` as high-confidence so the high-confidence write lane can proceed
- unified the detect-write policy surface across repo and workspace onboarding: repo
  `ota detect --write`, workspace auto-provision, and workspace rewrite now all build from the
  same conservative detect-write candidate instead of mixing stricter raw high-confidence writes
  with widened repo-local detect-write behavior
- detector-led `ota init` now widens strong starter signals onto current first-class contract
  surfaces instead of leaving obvious setup and finite task bodies in raw shell: supported Node,
  Ruby, Java, and .NET starter drafts now prefer `toolchains.*`, `prepare.kind:
  dependency_hydration` for `setup`, and `command` for simple modeled task execution, while `ota
  init --pack` now emits those same command-owned task bodies for shipped simple starter lanes
- runtime proof interrupt cleanup now resolves and signals the actual detached `ota up` process
  group before falling back to direct child termination, so `ota proof runtime` teardown is less
  likely to leave nested `ota up` or downstream workload processes running after user interruption
- validate/doctor now explicitly flag legacy toolchain compatibility ownership on
  `toolchains.<name>.provider` and flat `toolchains.<name>.fulfillment: run|none`, while keeping
  those shapes parseable as migration lanes; canonical public toolchain truth is now consistently
  pushed onto structured `fulfillment`
- `ota init` starter packs and `ota detect --merge` now stop emitting legacy shipped-toolchain
  `provider` fields for canonical toolchain shapes, so generated contracts align with the
  structured no-provider public contract model instead of writing migration-lane compatibility
  fields back into repo truth
- `ota init --pack` now seeds first-class dependency-hydration setup tasks for shipped Node
  package-manager lanes (`npm`, `pnpm`, `yarn`, `bun`), Ruby/Bundler, and Go modules instead of
  defaulting those starter packs back to opaque install shell when Ota already owns the setup
  path structurally
- validate/doctor now explicitly flag legacy service readiness shell ownership
  (`services.<name>.readiness.run`) and push canonical modeling onto structured
  `services.<name>.readiness.kind` or named `probe`, so service-readiness ownership stays
  compatible without remaining governance-silent
- validate/doctor now explicitly flag legacy top-level host service lifecycle ownership
  (`services.<name>.start` / `.stop`) and push canonical modeling onto `manager.kind: host`, so
  host-managed service ownership stays truthful for compatibility without remaining governance-silent
- aligned `ota clean` host-service cleanup with the rest of the lifecycle surface: cleanup now
  honors legacy host-owned `services.<name>.stop` commands in addition to structured
  `services.<name>.manager.stop`, and workflow-scoped cleanup now includes those legacy host-owned
  services instead of silently skipping them
- widened typed host service ownership so `services.<name>.manager.kind: host` can now own
  lifecycle through structured `manager.start` / `manager.stop` commands; `ota up` and
  `requires_services` execution paths now run that first-class host-manager lifecycle truth
  directly instead of forcing authors back to legacy top-level shell `start` / `stop` strings
- interrupt-driven cleanup now consumes typed host `manager.stop` for Ota-started required
  services, so detached/proof teardown can actually stop host-managed service lifecycles Ota
  started instead of treating structured shutdown as validation-only truth
- `ota clean` now consumes typed host `manager.stop` for Ota-owned host-managed services, scopes
  that cleanup to the selected workflow when requested, and reports host-service cleanup in both
  text and JSON instead of only owning container/volume repo state
- repo execution lock ownership now records Ota-owned host-managed services for the active run,
  `ota clean` now honors that lock before mutating lifecycle-owned repo state, and lock-busy
  diagnostics now show the active execution’s owned host services instead of only PID/process
  metadata
- proof/runtime execution receipts now emit first-class `host_service_cleanup` evidence for
  interrupt-driven host-managed service stop attempts, so machine-readable consumers can see the
  service, action, status, trigger, and any failure detail instead of inferring cleanup from
  summary prose alone
- runtime proof JSON now emits first-class `likely_cause_evidence` when ota can derive a
  higher-confidence readiness root cause from captured proof logs, so automation can distinguish
  loopback service config drift and detached-run failure signals without parsing advisory prose
- runtime proof JSON now also emits additive structured `cleanup_failure` detail when ota can
  classify proof-boundary cleanup failures, so automation can distinguish cleanup resource/reason
  truth without scraping top-level proof error prose
- centralized task adapter-input runtime env projection behind one shared registry-driven helper,
  so future adapter families widen one projection surface instead of repeating family-specific
  Compose/Bake binding logic across execution paths
- runtime proof now classifies address-in-use startup failures as first-class `bind_conflict`
  likely-cause evidence and failure class, and the published proof schema now documents
  `likely_cause_evidence` explicitly instead of leaving that machine-readable surface implicit
- runtime proof now also emits first-class `install_or_toolchain_failure` likely-cause evidence
  from captured `up.log`, so package-manager/compiler/toolchain startup breaks no longer stop at a
  class-only lane or prose-only diagnosis
- `proof-runtime.json` now explicitly publishes `workflow_env_artifacts` in the success schema, so
  the shipped machine contract matches the additive rendered-env evidence already present in proof
  JSON output
- runtime proof can now emit first-class `missing_env` likely-cause evidence and failure
  classification from captured `up.log`, including the missing variable name when Ota can recover
  it
- runtime proof can now emit first-class `readiness_target_mismatch` likely-cause evidence and
  failure classification when the declared proof endpoint differs from the runtime endpoint Ota
  observed in proof artifacts
- runtime proof can now emit first-class `dns_service_name_resolution_failure` likely-cause
  evidence and failure classification when proof logs show unresolved service-name or host
  resolution failures
- runtime proof can now emit first-class `auth_credential_failure` likely-cause evidence and
  failure classification when proof logs show rejected credentials or authorization
- policy-backed tool governance now resolves common executable aliases across provisioning and
  strict version policy lanes, so canonical tool names like `bundler` and `maven` can govern the
  executable-facing requirements `bundle` and `mvn` without forcing org policy authors onto
  shell-name quirks
- org policy validation now rejects unsupported provisioning sources instead of letting doctor/up
  plan unimplemented backends as if they were actionable, so provisioning truth stays aligned to
  the shipped mutating adapter set
- validate/doctor now flag non-canonical `effects.external_state` aliases like `docker_compose`,
  `postgresql`, and `k8s`, pushing repo contracts and policy packs onto the shipped canonical
  token vocabulary so cross-repo effect governance stays reusable
- fixed container-mode doctor precondition scoping across mixed container contexts, so runtime,
  tool, and toolchain probes only evaluate the selected workflow/task container images instead of
  leaking failures from unrelated container contexts
- widened first-class file checks with `checks[].scope: workspace`, so contracts can truthfully
  own sibling workspace inputs such as `../task-sdk/schema.json` without falling back to shell
  `test -f ...` glue; validator/doctor/docs/examples now keep repo-bound file checks strict by
  default and use explicit workspace scope only when that broader ownership is real
- replaced the whole-run repo execution mutex with an active-execution registry, so compatible
  runs can now coexist in one repo (`ota run dev` plus a finite task path) while `ota clean` and
  duplicate long-running service ownership still block on first-class execution conflicts instead
  of a coarse single-command lock; that conflict ownership now widens to shared host-managed
  services, shared Compose project ownership, shared persistent backend families, and shared
  deterministic env-file materialization outputs instead of staying task-name-only, and the
  conflict surface now emits typed reason identities instead of forcing operators and agents to
  infer the cause from owner detail text alone; failure receipts now also publish a first-class
  `execution_conflict` object derived from those reason identities instead of hiding the signal in
  generic blocked strings alone; `ota clean --json` now classifies active execution cleanup
  barriers as their own structured failure lane instead of collapsing them into generic repo-state
  errors
- made stream-phase loader timing explicit in the runner, so noisy task-output paths now use the
  governed delayed loader policy instead of ad hoc immediate spinner behavior that could dirty
  terminal output handoff before real task logs arrive
- widened `native_prerequisites` from diagnose-only package guidance into a first-class host
  preparation lane for `ota up` and `ota run`, so selected native prerequisite package-manager
  bundles (`apt`, `brew`, `winget`, `choco`, `scoop`) can now be fulfilled by Ota before
  rerunning preconditions instead of leaking that install burden into CI workflow glue
- tightened that native prerequisite fulfillment lane under org policy: when `.ota/org-policy.yaml`
  is active, selected host package installs now require explicit approval under
  `policies.native_packages.<manager>.approved`, `ota doctor` reports unapproved package bundles
  as policy violations, and policy-approved native package actions now flow through the same
  governed `ota up` / selected `ota run` provisioning path instead of bypassing policy
- removed generic `native_prerequisites.platforms.<os>.packages`; native prerequisite host package
  guidance is now manager-scoped only (`apt`, `brew`, `winget`, `choco`, `scoop`) so fulfillment,
  policy approval, and contract truth stay deterministic
- added first-class policy-backed `release-asset` provisioning for exact standalone tool binaries,
  so org policy can now approve platform-specific release URLs plus version probes, Ota can
  materialize those binaries into a source-managed workspace path for native/container execution,
  and repos no longer need local download helpers just to own exact tools like Mike Farah `yq`
- generalized adapter-overlay internals behind a shared adapter-input field registry, so
  workflow-overlay binding, duplicate-ownership governance, and runtime file-readiness checks now
  consume one canonical metadata surface instead of carrying separate Compose/Bake match ladders
- widened `adapter_inputs.compose` and `adapter_inputs.bake` with first-class `cwd`, so tasks and
  workflows can own adapter-root execution truth without shell `cd ... && docker compose/buildx`
  glue; Ota now reprojects repo-relative adapter files against that root at runtime and
  validate/doctor call out replaceable shell-owned adapter cwd patterns
- widened workflow-owned host preparation so `workflows.<name>.prepare` can now declare either a
  reusable `task` or an inline first-class `action`, letting workflows own deterministic bootstrap
  actions and bundles directly without synthetic helper tasks while keeping `ota up` and workflow
  summaries truthful
- widened `action.kind: ensure_bundle` so bundled deterministic native setup can now include
  `kind: ensure_container_network` steps, letting one setup owner compose shared Docker network
  bootstrap with other governed bootstrap mutations without falling back to shell orchestration
- added first-class container-network bootstrap through `action.kind: ensure_container_network`,
  so contracts can own external Docker network readiness without shell `docker network inspect ||
  docker network create` glue; validation, dry-run preview, contract schemas, capability
  publication, and replaceable-shell governance now understand the new surface
- fixed workflow prepare validation drift: `workflows.<name>.prepare.task` now accepts finite
  native `command` tasks, matching the published contract/reference surface instead of forcing
  authors back to shell `run`
- fixed workflow-owned adapter-overlay provenance during diagnosis: when Ota injects selected
  workflow Compose/Bake adapter inputs into runnable task paths for execution, doctor/validate no
  longer misclassify those synthetic bound values as task-authored duplicate ownership
- added three missing canonical example families across core and public examples: Bake
  adapter-input ownership through `adapter_inputs.bake.files`, deterministic env bootstrap through
  `action.kind: ensure_env_file`, and bundled host file preparation through
  `action.kind: ensure_bundle`, so users, agents, and crawlers now have compact reference shapes
  for those newer contract surfaces instead of only spec prose
- tightened adapter-input governance in two connected ways: workflow-owned Compose/Bake overlays
  now recognize `launch.kind: command` task paths as real adapter consumers, and validate/doctor
  now warn when `adapter_inputs.compose` or `adapter_inputs.bake` is declared as an empty marker
  instead of owning concrete adapter truth
- widened task adapter input ownership with first-class
  `tasks.<name>.adapter_inputs.compose.env_files` plus mode-branch overrides, so task-owned
  `docker compose` interpolation files no longer have to hide inside shell `--env-file` flags or
  misuse task `env_files`; run preview, `ota env`, execution, validate/doctor, task JSON output,
  and the published contract schema now all project the same declarative compose adapter input
  truth
- widened that same task-owned compose adapter surface beyond env-file interpolation: contracts can
  now declare `tasks.<name>.adapter_inputs.compose.files` and
  `tasks.<name>.adapter_inputs.compose.project_name` with mode-branch overrides, and Ota projects
  them through `COMPOSE_FILE` / `COMPOSE_PROJECT_NAME` so compose file selection and project
  naming no longer need to stay trapped in shell flags when the task owns them
- validate/doctor now treat shell-owned compose file and project-name flags as the same governance
  class as shell-owned `--env-file`, so hard-coded `docker compose -f` / `--file` / `-p` /
  `--project-name` truth is now pushed toward `tasks.<name>.adapter_inputs.compose.*`
- selected workflow env-profile materialization now routes rendered dotenv artifacts into
  compose-running tasks through `tasks.<name>.adapter_inputs.compose.env_files` instead of
  misrouting that workflow-owned compose interpolation input through process `env_files`; `ota env
  --task` now also validates task adapter-input files, not just task process env-files
- widened workflow-owned compose adapter truth beyond rendered dotenv injection: workflows can now
  declare canonical `workflows.<name>.env.adapter_inputs.compose.*` overlays, with
  `compose_files` / `compose_project_name` kept as backward-compatible aliases; Ota projects that
  workflow-owned adapter truth into compose-running task paths with branch-aware precedence,
  validate/doctor warn when task-local compose project naming drifts against workflow ownership,
  and the published contract schema now carries the same generalized workflow adapter surface
- proved that generalized adapter surface with a second adapter family: contracts can now declare
  `tasks.<name>.adapter_inputs.bake.files` plus
  `workflows.<name>.env.adapter_inputs.bake.files`, and Ota projects that truth into
  `docker buildx bake` task paths through `BUILDX_BAKE_FILE` instead of forcing Bake file
  selection to stay hidden in shell `-f` flags
- validate/doctor now also treat shell-owned `docker buildx bake -f` / `--file` truth as a
  governance smell, pushing Bake file selection toward `tasks.<name>.adapter_inputs.bake.files`
  instead of opaque shell flags
- widened the Compose adapter family again: contracts can now declare
  `tasks.<name>.adapter_inputs.compose.profiles` plus
  `workflows.<name>.env.adapter_inputs.compose.profiles`, Ota projects that truth through
  `COMPOSE_PROFILES`, and validate/doctor now treat shell `docker compose --profile ...` truth as
  the same adapter-ownership governance class as shell `--env-file`, `-f`, and `-p`
- widened Compose service-manager ownership to match those task/workflow surfaces:
  `services.<name>.manager.profiles` is now first-class for `manager.kind: compose`, Ota threads
  that declarative profile truth through derived compose service commands, and validation rejects
  empty profile entries or host managers pretending to own compose profile selection
- generalized the workflow adapter-overlay engine behind one shared adapter-family path for
  Compose and Bake, so support detection, selected-workflow validation, and overlay binding no
  longer depend on separate hand-wired branches in validator and runtime orchestration
- narrowed the public workflow Compose authoring story to one canonical surface:
  compatibility aliases (`compose_files`, `compose_project_name`) are still accepted, but shared
  alias interpretation and duplicate checks now live under the adapter-input engine and
  validation explicitly pushes authors toward `workflows.<name>.env.adapter_inputs.compose.*`
- completed the matching doctor/advisory parity for adapter-owned flags: Compose/Bake replaceable
  ownership advisories are now family-specific, no longer get suppressed by unrelated adapter
  inputs, and cover structured `command` bodies as well as shell `run`/`script` bodies
- tightened advisory identity governance behind that same surface: the duplicate workflow
  adapter-input advisory is now internally family-generic instead of Compose-hardcoded, while
  doctor identity fallback coverage now matches the current Bake and workflow adapter-ownership
  summary shapes
- widened duplicate workflow adapter-input governance beyond Compose `project_name`: doctor and
  validate now also surface field-specific duplicate ownership advisories for
  `compose.env_files`, `compose.files`, `compose.profiles`, and `bake.files`, with contract-pack
  JSON coverage locking those identities into the shipped doctor surface
- restored Python starter command provenance truth all the way through init JSON field paths, so
  generated structured `command` bodies now publish `tasks.<name>.command.exe` /
  `tasks.<name>.command.args.*` provenance instead of collapsing that ownership back to legacy
  `run` fields
- fail workflow-owned rendered env artifacts earlier on policy drift: run-path env rendering now
  rejects selected workflow outputs that violate the governing contract or can no longer be
  materialized cleanly, instead of deferring the mismatch into later runtime or service startup
  failures
- made workflow-owned env materialization more first-class across real execution paths: selected
  workflow profiles can now declare `env.compose_env_file_services` to bind one rendered dotenv
  artifact directly into named compose-managed services, direct `ota run` now materializes
  selected workflow dotenv artifacts before executing tasks in the workflow closure, and proof
  runtime likely-cause hints now point operators at workflow overlays, rendered artifacts, task
  `env_files`, and compose `manager.env_file` instead of implying shell glue is the primary fix
- widened first-party sync governance beyond `ota-run/skills`: the release gate and repo task
  surface now also govern `ota-run/ota-site` through one shared first-party consumer sync check,
  with explicit status records for both the skills package and the public docs site so canonical
  docs or contract-surface widening cannot ship quietly while one consumer still lags behind
- added a Rust-owned published canonical docs manifest at
  `docs/spec/published-docs/canonical-docs.json`, with release publication to
  `https://dist.ota.run/spec/published-docs/latest/canonical-docs.json`; this gives downstream
  consumers a machine-readable source-of-truth map for key docs surfaces such as contract,
  workspace, command, JSON output, topology, and doctor finding references instead of forcing
  them to scrape page chrome or hardcode upstream source paths
- clarified that `command.exe` is not allowlisted: the docs now say the executable may be any repo-truthful binary or path, and that the named examples are illustrative rather than exhaustive
- clarified the public command-body contract: `command.exe` is now documented explicitly as a generic executable name or path rather than an npm-shaped field, with representative families such as `npm`, `pnpm`, `yarn`, `bun`, `node`, `python3`, `go`, `bundle`, `docker`, absolute paths, and repo-local binaries
- widened finite setup modeling again: tasks can now declare `prepare.kind: sequence` to execute
  more than one ordered structural prepare step in a single truthful setup lane, which closes the
  remaining mixed-ecosystem fallback to ad hoc shell setup bodies such as Node hydration plus
  Python `uv` hydration in one repo-level `setup` task
- widened first-class dependency hydration across the remaining starter-owned setup families:
  contracts can now model Maven, Gradle, Cargo, and `dotnet restore` hydration structurally under
  `prepare.kind: dependency_hydration`, with Java wrapper awareness for `mvnw` / `gradlew`; `ota
  init --pack` now seeds those first-class setup bodies for Java, Rust, and .NET instead of
  falling back to raw shell install commands
- widened Maven hydration one step further under that same surface: `source.kind: maven` now
  supports explicit `mode: resolve|go_offline` plus optional `skip_tests`, so real setup lanes
  like `mvn -q -DskipTests dependency:go-offline` can stay first-class instead of falling back to
  opaque shell
- aligned published task inventory truth with the widened setup surface: `ota tasks` text now
  renders structured prepare lanes truthfully, the published `tasks.json` / `workspace-tasks.json`
  schemas now cover additive `prepare`, additive `aggregate`, additive `effects`, task `env` /
  `inputs`, `ensure_bundle`, and the concrete emitted task kinds (`sequence`,
  `dependency_hydration`, `aggregate`), and regression coverage now locks both the schema surface
  and the emitted JSON/task-listing shapes against future drift
- fixed recursive published-schema validation on the machine path itself: `ota json validate`
  no longer flattens every `$ref` before compilation, so recursive task schemas such as
  `tasks.json` / `workspace-tasks.json` validate through the library’s own internal-ref handling
  plus a preloaded published-schema document store, and regression coverage now proves that both
  repo and workspace task inventory payloads validate successfully through the shipped command
- widened finite task execution with first-class `command`: tasks and mode branches can now model
  stable argv-owned finite execution without shell glue, published task inventory schemas now emit
  structured `command` objects alongside existing `launch` summaries, and the Python starter plus
  first-party Python examples now use that surface for `uv run ...` test lanes instead of opaque
  `run` strings
- fixed Python starter machine-truth and Python ownership symmetry together: `ota init --pack python
  --json` now publishes provenance for the actual `tasks.setup.prepare` / `requirements` /
  `effects` fields instead of stale `tasks.setup.run`, and both the Python starter and uv-backed
  detector synthesis now declare `toolchains.python.package_managers.uv` when uv owns the lane
- strengthened the hosted install contract for CI and especially Windows GitHub Actions: the
  release installers now export the resolved ota bin directory to `GITHUB_PATH` automatically when
  that environment file is present, so workflow authors no longer need to guess post-install
  binary locations such as `%LOCALAPPDATA%\\ota\\bin`; the install docs and hosted-validation
  guidance now treat the plain installer invocation as the canonical GitHub Actions pattern

## 1.6.20

- added a release-gated skills-sync governance check for contract-surface widening: maintainer
  changes to core schema/validator/spec/example surfaces must now update
  `docs/policy/skills-sync-status.yaml` with either a synced `ota-run/skills` commit or an
  explicit waiver reason, and the release gate plus repo task surface now enforce that discipline
- moved published repo/workspace contract schema ownership into a Rust-backed generator module and
  added a sync/export path via `cargo run --bin sync_published_contract_schemas`; CI compatibility
  tests now fail when `docs/spec/json-schemas/contract.json` or
  `docs/spec/json-schemas/workspace-contract.json` drift from that generator, closing the remaining
  hand-edited published-schema split-brain
- tightened generated-schema governance again: the release gate and repo `compat` task now rerun
  the published-schema generator and fail on `git diff --exit-code` drift before the schema/output
  contract tests run, so publication proves regenerated artifacts rather than inferring sync only
  from downstream test coverage
- tightened the type-model boundary behind the published repo/workspace contract schemas: shipped
  examples and canonical contract docs examples now validate not only as authored YAML, but also
  after loading through the Rust contract types and projecting back to authoring JSON values, so
  the published schemas are exercised against the real Rust-owned authoring-model boundary rather
  than only raw source files
- published and enforced the full machine-readable workspace contract schema at
  `docs/spec/json-schemas/workspace-contract.json` /
  `https://dist.ota.run/spec/json-schemas/latest/workspace-contract.json`; shipped workspace
  examples and canonical workspace contract docs now validate against that published schema, and
  the compatibility surface now treats both repo and workspace contract schemas as release-gated
  public APIs
- published and enforced the full machine-readable repo contract schema at
  `docs/spec/json-schemas/contract.json` /
  `https://dist.ota.run/spec/json-schemas/latest/contract.json`; shipped example contracts and
  canonical repo contract docs now validate against that published schema so contract publication
  is a governed release surface instead of an optional artifact
- hardened finding identity across governance surfaces: advisory-backed doctor findings now carry
  explicit stable identity, `ota explain --json` preserves that code through `steps[].code`
  instead of re-deriving advisory identity from rendered summary text, and `ota annotations`
  now renders additive `Code:` segments when input finding JSON includes a stable code; the main
  structured doctor finding families for service readiness, declared checks, runtime/tool
  diagnosis, env value and env-source failures, backend/remote topology blockers, workflow
  probe/surface readiness findings, policy-backed findings, repo hygiene findings, native
  prerequisites, and contract drift now also emit explicit stable `code` / `category` / `owner`
  identity instead of depending on summary parsing; policy-backed version-rule and strict-version
  findings now also preserve policy metadata/provenance consistently in doctor JSON
- added a representative `ota doctor --json` contract pack across policy, workflow, service, env,
  provisioning, and remote finding lanes, and CI now fails if a shipped doctor finding is added
  without explicit identity metadata; doctor report-level contract coverage now also locks summary
  verdict/counts, primary-blocker mirroring, ready-without-blocker shape, and published schema
  truth for `mode: remote` plus `summary.primary_blocker.code`; the shipped doctor code catalog is
  now published in a generated reference doc synced from `src/doctor.rs`, and contract-drift
  findings now publish the documented `repo_signals` provenance instead of falling back to generic
  repo-contract provenance; monorepo aggregate `ota doctor --json` now also computes
  `summary.primary_blocker` from the same rewritten emitted findings surfaced in `members[]`; the
  published workspace doctor/check schemas now also expose top-level `summary.primary_blocker` and
  blocker `code` fields in line with the shipped JSON output, and workspace repo acquisition /
  repo-contract findings now also emit explicit stable workspace finding codes instead of falling
  back to anonymous summary-only blockers
- hardened remaining shipped command finding identity: contractless/onboarding doctor signals,
  inferred starter-agent signals, policy effect-governance decisions, and adapter-bootstrap
  failures now emit explicit stable `code` / `category` / `owner` identity instead of anonymous
  summary-only findings; the shipped finding reference catalog now also covers command-sourced
  production findings, and `src/cli/commands.rs` is guarded so new shipped command findings
  cannot reintroduce `identity: None`
- widened `ota env` with `--workflow` selection so the env read path can inspect the selected
  workflow’s env profile truth directly; text and JSON output now report the selected workflow,
  selected profile, and any workflow-owned rendered env artifacts such as rendered dotenv files
- widened execution-truth reporting for workflow-owned rendered env artifacts: `ota execution plan`,
  execution receipts, and `ota proof runtime --json` now surface the selected workflow env
  artifacts plus consuming task/service lanes, and validate/doctor now warn when tasks duplicate
  ownership of a workflow-rendered env file that Ota already auto-projects into the workflow task
  closure
- added first-class task `env_files` overlays for run-path process ownership, widened
  `action.kind: ensure_env_file` with deterministic `mode: replace`, `mode: remove`, and
  `from_env` projection from resolved Ota env truth, and sharpened `ota proof runtime --json`
  with optional `likely_cause` hints for high-confidence loopback service drift or detached-run
  failure excerpts discovered in captured proof logs; proof phase reporting now refines blocked
  runs from captured `up.log` so repo check / precondition failures do not collapse into
  misleading `service readiness` proof phases; invalid task `env_files` now fail `ota env --task`,
  run preview, and task execution before env resolution falls through to misleading missing-env
  diagnostics
- added first-class `env.profiles` plus `workflows.<name>.env.profile`, so selected workflow
  doctor/up/proof paths can prepend workflow-owned declared env sources and inject ordered
  workflow-scoped `env_files` / literal env overlays without repeating that ownership across task
  shells or task-local duplication
- widened `env.profiles` with first-class `render.dotenv` artifact materialization, so `ota up`
  can now render workflow-owned dotenv files from selected profile truth before service startup
  and setup, removing the remaining need for a separate `ensure_env_file` prepare task when the
  workflow only needed a compose/runtime interpolation artifact
- widened `env.profiles.<name>.render.dotenv` with optional `template` support, so workflow-owned
  dotenv artifacts can now be re-rendered from repo example truth plus selected profile overlays
  without hiding that materialization inside a separate setup task or stale shell rewrite glue
- widened first-class check governance with `checks[].kind: env`, a deterministic dotenv-backed
  assertion surface for repo-relative env files; contracts can now replace shell `grep` / `findstr`
  glue with governed `env.path` assertions over exact values, host values, or URL hosts using the
  initial `policy: not_loopback` surface plus first-class `state: present|missing` assertions, and
  `requirements.checks` / `when.checks` now accept those env checks directly; validate/doctor now
  also warn on obvious shell file-state and env-file checks that should be rewritten as
  first-class `kind: file` or `kind: env` checks
- widened `checks[].kind: env` host governance with `host.allowed` and `url_host.allowed`, so
  contracts can now assert one env key or URL/DSN host resolves to one of a small set of truthful
  service hostnames without falling back to shell `grep` glue
- widened `checks[].kind: env` with `not_equals`, so contracts can ban a small set of known-bad
  exact env values without shell assertions
- widened `workflows.<name>.prepare.task` from native `action`-only to native finite task bodies
  (`run`, `script`, `prepare`, or `action`) so workflow-scoped normalization and bootstrap steps
  no longer need to hide behind task dependencies; service-like prepare tasks still remain invalid
  through `launch`, `runtime`, and `requires_services` rejection
- added `services.<name>.manager.env_file` for `manager.kind: compose`, so Ota-owned compose
  service start/stop/ps/health command paths can carry one declarative `docker compose --env-file`
  input instead of forcing repo-local shell duplication
- validate/doctor now also warn on obvious shell `.env*` rewrite tasks so deterministic env-file
  mutation can move to `action.kind: ensure_env_file` with explicit replacement keys instead of
  platform-specific `sed` / `perl` glue
- validate/doctor now warn when task bodies hard-code `docker compose --env-file ...`, pushing
  compose interpolation ownership toward declarative task `env_files` or
  `services.<name>.manager.env_file`
- widened `action.kind: ensure_env_file` with `action.template_mode: replace`, so Ota can
  re-derive an env file from its template on every run before applying explicit key replacements;
  this closes the shell copy-plus-`sed` env normalization gap with one governed action surface
- widened managed-service topology modeling so service endpoint identity is no longer forced to
  equal execution-context identity: `services.<name>.endpoints.<name>.context`,
  `services.<name>.readiness.endpoint`, and `tasks.<name>.env_bindings.<VAR>.from_service.endpoint`
  now let one context expose multiple truthful service surfaces without falling back to opaque
  shell glue
- closed the assist authoring gap for named service endpoints: `ota assist declare-service` now
  accepts `--endpoint-context`, projects it into proposal JSON/YAML, and keeps generated readiness
  and replay commands aligned with the authored endpoint context
- widened Docker Compose service topology detection for explicit host-published ports: `ota detect`,
  `ota init`, and detect merge can now infer the canonical host execution slice
  (`execution.default_context`, `execution.contexts.host.backend`), plus matching
  `services.<name>.endpoints.host` and structured `readiness.from` / `readiness.kind: tcp`
  when one Compose service exposes exactly one deterministic TCP host-published port candidate;
  when one Compose service exposes multiple deterministic TCP host-published port candidates, Ota
  now emits named host endpoints such as `services.<name>.endpoints.host_3000.context: host`
  instead of dropping topology entirely, while intentionally withholding ambiguous readiness
  ownership
- closed the remaining readiness assist gap for multi-endpoint services: `ota assist
  declare-readiness` now accepts `--endpoint`, carries the selector through preview/apply
  JSON/text output, and writes `services.<name>.readiness.endpoint` plus the matching
  `readiness.from` context instead of forcing manual YAML edits when one service exposes multiple
  truthful projections
- fixed detect-merge execution governance for inferred Compose host topology: additive merge no
  longer rewrites an authored `execution.default_context`, and it now skips host-topology
  additions entirely when the existing contract still uses root shorthand execution
- upgraded Docker Compose service detection to the canonical managed-service surface: `ota detect`,
  `ota init`, detect merge, and detect-owned field tracking now emit `services.<name>.manager.*`
  and structured `readiness.kind: compose_health` for Compose-backed services instead of teaching
  legacy `provider/start/stop/healthcheck` as the inferred contract shape, with updated JSON
  output docs and regressions
- fixed Python toolchain host requirement projection for Poetry-owned repos: when
  `toolchains.python.package_managers.poetry` is declared without an explicit `package_managers.uv`
  requirement, Ota no longer injects `uv` as a mandatory host tool on check-only/run-path surfaces;
  `uv` remains available for explicit Python run fulfillment and for repos that actually declare
  `package_managers.uv`
- Fix native task fulfillment so inferred wildcard shell-tool requirements like `pnpm: "*"` no longer conflict
  with exact toolchain-owned package-manager versions; exact owned versions now win during requirement merging.
- fixed agent bootstrap determinism advisories so exact `OTA_GIT_REV=<commit>` source-install
  pins are now recognized as deterministic, while moving branch installs still warn correctly
- widened Python toolchain package-manager ownership so Poetry is first-class under
  `toolchains.python.package_managers.poetry`, while legacy standalone `tools.poetry`
  declarations remain temporarily accepted with migration warnings instead of a silent break
- expanded first-class Python setup so Poetry is now operational as well as declarative:
  `prepare.kind: dependency_hydration` now ships `source.kind: poetry`, and the existing
  Python `provider: uv` fulfillment lane can now install declared Poetry versions on the
  selected run path before tasks execute
- expanded the managed toolchain ownership surface into named execution contexts:
  `execution.contexts.<name>.requirements.toolchains` is now first-class, so repos can keep
  runtime and package-manager ownership under `toolchains` even when that truth is scoped to a
  host, container, or other named execution context instead of falling back to split
  `runtimes` / `tools` declarations
- expanded first-class task preparation for dependency hydration with a Yarn slice:
  `prepare.kind: dependency_hydration`, `medium: package_dependencies`, and
  `source.kind: node_package_manager` with `manager: yarn` now let ota execute
  `yarn install --immutable` structurally instead of hiding lockfile-backed Yarn hydration
  inside opaque shell `run` commands
- added first-class aggregate task bodies through `tasks.<name>.aggregate.tasks`: Ota now models
  named dependency-closure entrypoints such as `verify` structurally instead of teaching fake
  `run: "true"` wrappers, with schema/validation/runner/output support and updated docs/examples
- fixed mixed-mode task validation for top-level fallback bodies: when a task already declares its
  default native body at the task level, `execution.default_mode: native` no longer requires a
  redundant `execution.modes.native` branch just to add a non-default container override
- fixed workflow-scoped `ota up` service selection: when a selected workflow does not declare
  `workflows.<name>.services.required`, Ota no longer falls back to unrelated repo-global
  required services during the final service-readiness phase
- fixed Windows native POSIX-shell task execution to launch Bash without login-shell startup:
  repo-rooted commands such as Bundler dependency hydration now preserve Ota's selected working
  directory instead of letting `bash -l` reset execution into `$HOME`
- fixed native `ota doctor` / `ota up --dry-run` mismatch handling for `toolchains.ruby` with
  `fulfillment.source: ruby` and `mode: run`: when the selected path declares Bundler under the
  Ruby toolchain, Ota now checks the selected Ruby provider instead of blocking on an ambient
  `bundle` probe before selected-path fulfillment can occur, and the shipped Ruby fulfillment lane
  now installs declared Bundler versions via `ruby -S gem install bundler ...`
- clarified agent-facing task guidance: when a repo declares a matching safe or verification
  task, agents should prefer `ota run <task>` over raw package-manager or language-tool
  commands and fall back only when no truthful Ota task exists or when isolating an Ota defect
- expanded first-class task preparation for dependency hydration with a native Go slice:
  `prepare.kind: dependency_hydration`, `medium: package_dependencies`, and
  `source.kind: go_modules` now let ota execute `go mod download` structurally instead of
  hiding module hydration inside opaque shell `run` commands
- expanded first-class task preparation for dependency hydration with a Ruby/Bundler slice:
  `prepare.kind: dependency_hydration`, `medium: package_dependencies`, and
  `source.kind: bundler` now let ota execute repo-local gem hydration structurally instead of
  hiding `bundle config set path ... && bundle install` inside shell `run` commands
- hardened dependency-isolation volume cleanup after container-backed execution and proof:
  Ota now gives container engines a longer retry window before surfacing `volume is in use`
  cleanup failures, reducing false-negative proof tails when Docker lags briefly after
  container removal
- added first-class task preparation semantics through `tasks.<name>.prepare`: the shipped
  `prepare.kind: dependency_hydration` slices now model both docker image hydration and
  node package-manager dependency hydration structurally instead of forcing finite setup phases
  to live inside opaque shell `run` strings
- added executable task support for the first shipped `prepare` slice in schema, validation,
  summaries, and runner execution, including mode-branch support and validation that keeps
  docker-backed and node package-manager dependency hydration aligned with explicit
  `requirements` and `effects`
- updated the full contract example and the widened `immich` pressure contract to use
  first-class `prepare` for registry-backed docker image hydration instead of repo-local shell
  glue
- added a canonical orchestration model for repo-mediated execution: contracts can now declare
  top-level `orchestrators`, selected tasks can opt into `execution.orchestrator.ref` /
  `execution.orchestrator.mode`, and Ota ships `mise` as the first orchestrator for trust,
  install, and mediated task execution on the selected path
- fixed run-path preparation env parity for selected backends: orchestrator preparation,
  toolchain fulfillment, and Corepack activation now inherit the effective backend/task env
  before execution, so container-scoped repo-manager paths such as `mise trust` / `mise install`
  can use declared context env (for example writable `MISE_*` paths) instead of failing under
  backend-default locations
- fixed native `ota doctor` / `ota up --dry-run` mismatch handling for `toolchains.<name>.fulfillment.source: mise`
  with `mode: run`: when the selected path declares `mise` as the run-path authority, Ota now
  checks `mise` itself instead of blocking on an ambient host runtime version mismatch before
  selected-path fulfillment can occur
- upgraded `toolchains` to the canonical capability-first model: public contracts now use
  structured `fulfillment` (`source` + `mode`) instead of teaching legacy provider-coupled
  toolchain shapes, while runtime compatibility still accepts legacy `provider` and flat
  `fulfillment: run` input during migration
- updated Ota's own contract, examples, and spec docs to the new canonical model, including
  removal of managed-surface shell glue that duplicated declared toolchain truth
- made service-task readiness budgets authoritative during startup in `ota run`: when a service
  task declares a projected runtime endpoint and the configured readiness `start_period` /
  `interval` / `timeout` / `retries` budget is exhausted before that endpoint becomes reachable,
  Ota now fails startup instead of waiting indefinitely until the workload exits or the user
  interrupts it
- added structured readiness failure reporting for service startup in `ota run`: startup failures
  now carry the real probe budget and last probe error through terminal output and JSON-facing
  execution surfaces, and `--stream` readiness probes now emit live attempt progress instead of
  only a final generic readiness failure

## 1.6.19

- hardened repo execution lock guidance in `ota run`: rerun commands now preserve explicitly
  requested execution mode overrides including `--mode native`, and lock-contention errors now
  surface the active task/mode/lifecycle/pid/start-time metadata for the live Ota execution that
  currently holds the repo lock
- hardened streaming durable log capture for `ota run --log`: if the live stdout/stderr log path
  is deleted while a task is still running, Ota now recreates the named log file and continues
  writing there instead of silently writing only to the unlinked file handle
- fixed Windows native proof CI: `where` (Windows) and `which` (Unix) are now excluded from
  inferred tool requirements so that `run: where cl` in a task or check no longer triggers a
  spurious `where --version` probe that exits with code 1 on Windows
- hardened selected-path command inference for shell wrappers: `ota doctor` and `ota run --dry-run`
  no longer infer wrapper shells like `sh`, `bash`, or `zsh` from `run:` / `script:` bodies as
  required tools, avoiding bogus shell-version probes on wrapper forms such as `sh -c '...'`
- fixed Corepack-managed container tool probing in `ota doctor`: owned package-manager tools such
  as `yarn` are now probed through the normal container backend in the repo workdir, so contracts
  that rely on `packageManager` truth no longer drift to global image defaults during diagnosis
- hardened ephemeral container cleanup after container-backed probes and dry-runs: Ota now retries
  transient Docker `removal of container ... is already in progress` races instead of surfacing
  them as spurious probe failures when the container cleanup path briefly lags
- fixed a container workflow preflight trust gap in `ota doctor` and `ota up --dry-run`: selected
  container task/workflow paths now evaluate required env the same way `ota run --dry-run` does,
  instead of suppressing real env blockers behind the generic "host-only checks" note
- hardened `ota doctor` for compose-managed services: when a required compose service is not
  running, Ota now fails that service readiness path immediately from manager truth instead of
  spending the full declared TCP/HTTP probe retry budget before reporting the same not-ready state
- fixed workflow-scoped `ota doctor` service selection: selected workflows now diagnose only the
  services required by their explicit workflow service list plus selected task closure, instead of
  falling back to unrelated repo-global services whenever `workflows.<name>.services.required`
  was empty
- hardened workflow-scoped `ota doctor` gating: once selected workflow checks or probes produce a
  blocking error, Ota now skips later workflow surface readiness evaluation instead of spending the
  full surface retry budget on a path already proven invalid
- fixed Windows host tool probing for wrapper scripts such as `corepack.cmd`: `ota doctor` now
  executes `.cmd` / `.bat` tool probes through `cmd /C` instead of failing with a generic
  executable-not-found probe error when the tool is otherwise present on PATH
- added host-architecture scoping for execution contexts: `execution.contexts.<name>.only_arch`
  now lets contracts fail early in `ota doctor`, `ota run --dry-run`, and execution when a
  selected context is not supported on the current host architecture, instead of falling through
  to later container image manifest failures
- hardened container runtime-proof cleanup for dependency-isolation volumes: Ota now retries
  transient Docker `volume is in use` cleanup races with backoff instead of turning a proven ready
  container proof into a cleanup failure on the first short release lag
- improved `ota run` failure excerpts for noisy test runners: Ota now prioritizes real failing-test
  markers such as `FAIL`, `Failed Tests`, and `AssertionError` over surrounding passing test
  chatter, so captured output centers on the actual failure instead of adjacent success lines
- fixed a native runtime/tool trust bug across `ota doctor` and `ota run`: Ota no longer
  prepends `mise` shim directories into its own process PATH, and native version probes now run
  commands from the contract working directory instead of reconstructing shim paths, so active
  `mise exec` environments and repo-scoped version-manager context are preserved instead of being
  overwritten by Ota-owned PATH drift
- fixed a Python toolchain trust bug in `ota doctor`: `toolchains.python` with `provider: uv`
  now probes Python runtime candidates (`python3.12`, `python3`, `python`) instead of probing
  the `uv` executable as if it were the runtime, so native/container diagnosis no longer reports
  bogus `python@uv-version` mismatches or false missing-Python blockers on Python container images
- fixed a trust bug in failed-task output excerpts: relevance ranking now prefers real test
  failure summaries/assertions over incidental package-manager noise containing words like
  `failed`, so captured failure excerpts point at the actual failing test surface instead of
  unrelated install/build chatter
- fixed a trust bug in container run failure classification: managed isolated-path mutation now
  requires a real filesystem-mutation signal tied to the isolated path, so unrelated task failures
  (for example long-running Go test timeouts with `.go` paths in stack traces) keep their true
  primary diagnosis instead of being misclassified as isolated-path mutation errors
- added a governance advisory for long-running service task paths that still resolve to shell
  `run` or `script` with `runtime.kind: service`, so `ota validate` / `ota doctor` now push
  those paths toward `launch.kind: command` plus runtime-owned exposure/readiness truth
- added stable contract-advisory codes across validate/doctor governance output: `ota validate
  --json` now exposes additive structured `warning_details[]`, and advisory-backed
  `ota doctor --json` findings now classify under durable `OTA_CONTRACT_ADVISORY_*` codes
- added a contract advisory for container contexts that isolate `.yarn`/`.yarn/releases` while
  Yarn tasks run in that same context, so `ota doctor` warns before runtime about container
  isolation shadowing committed `.yarn/releases/yarn-*.cjs` artifacts
- fixed dotnet remediation trust drift across doctor and provider-owned toolchain guidance:
  dotnet install commands are now requirement-first, use `global.json` only when it satisfies the
  declared contract requirement, and fall back to channel-based install guidance for broad or
  range requirements (for example `9.0` or `>=9.0,<10.0`) instead of surfacing conflicting exact-
  version suggestions
- hardened `ota init --pack node` for monorepo/root-script truth: node starter contracts now seed
  `dev` and `test` tasks only when the root `package.json` actually declares `scripts.dev` and
  `scripts.test`, avoiding non-runnable default task emission on repos where those scripts live in
  subpackages only
- added first-class check-only `toolchains.dotnet` ownership for .NET repos and migrated
  `ota init --pack dotnet` to seed `provider: dotnet` under `toolchains` instead of split
  `runtimes.dotnet` / `tools.dotnet` / duplicate installed checks
- updated the shell installer and install docs to point optional skill setup at
  `npx skills add ota-run/skills --full-depth` instead of separate Codex and Claude Code
  `ota skills install` commands
- tightened the shell installer success receipt so optional skill setup now renders on one compact
  line (`Install Ota skill: npx skills add ota-run/skills --full-depth`) without changing the
  existing accent color treatment
- recorded the Hoppscotch pressure-test adoption outcome in the v9.1 pressure-test ledger
  (`hoppscotch/hoppscotch#6382` closed as not planned) while preserving the contract/matrix
  evidence as reusable Ota readiness signal
- promoted the remaining maturity backlog into explicit pressure-ledger entries with concrete
  acceptance evidence for: (1) toolchain fulfillment depth beyond check-only providers, (2)
  pre-execution effect-governance parity for `network` / `external_state`, and (3)
  planning-doc status hygiene
- clarified planning status semantics in the v9.1 plan (`planned` / `active` / `completed`) and
  marked the shipped v9.1 slice as `completed` so shipped work is not presented as still-planned
- tightened explicit mode-override trust in `ota run` / `ota execution plan`: when a task declares
  `execution.modes`, unsupported explicit overrides (for example `--mode container` with only a
  native branch) now fail with a task-scoped mode-branch error that reports requested mode and
  declared mode branches instead of generic execution-plan resolution text
- relaxed execution schema ergonomics for named-context contracts: `execution.default_context` /
  `execution.contexts` can now coexist with root `execution.lifecycle` and
  `execution.backends` defaults, while `execution.preferred` remains disallowed in named-context
  mode to keep backend selection unambiguous
- expanded toolchain package-manager version validation from token-only strings to shell-safe
  version constraints (for example `>=2.5.3,<2.6.0`) and aligned validation messaging to
  `version constraint` semantics
- expanded detector-led toolchain parity for Go and Ruby: `ota detect` now promotes detected
  `go` and `ruby` runtime lanes into canonical `toolchains.go` (`provider: go`) and
  `toolchains.ruby` (`provider: ruby`) ownership when repo signals confirm those ecosystems, and
  normalizes legacy `tools.bundler` detection into `toolchains.ruby.package_managers.bundler`
- expanded detector-led .NET ownership parity: detected `.NET` runtime/tool lanes now converge to
  canonical `toolchains.dotnet` (`provider: dotnet`) ownership on detect write/merge so contracts
  do not regress back to split `runtimes.dotnet` / `tools.dotnet` declarations
- expanded `toolchain_repo_signals(...)` support for Go (`go.mod`, `go.work`, `.tool-versions`)
  and Ruby (`Gemfile`, `Gemfile.lock`, `.ruby-version`, `Rakefile`, `.tool-versions`) so
  detector-led toolchain synthesis is gated by explicit ecosystem signals instead of broad runtime
  inference alone
- added first-class check-only `toolchains.go` ownership for Go repos, updated the Go starter pack
  to emit `provider: go` instead of split `runtimes.go` ownership, removed duplicate
  toolchain-owned `*-installed` starter checks from Node/Python/Go/Rust packs, and kept
  starter-pack agent boundary inference aligned with toolchain-owned stacks
- added first-class check-only `toolchains.ruby` ownership for Ruby repos and added
  `ota init --pack ruby` so starter contracts seed toolchain-owned Ruby setup/test paths instead
  of split runtime/tool declarations, including explicit Bundler version governance under
  `toolchains.ruby.package_managers.bundler`
- closed planning drift for task launch sources: `tasks.<name>.launch` is now documented as a
  shipped baseline (command + container launch kinds), and the v9.1 pressure-test ledger now marks
  first-class task launch sources as closed
- tightened uv-backed Python ownership so `toolchains.python` owns the `uv` tool surface and
  starter packs no longer duplicate it under top-level `tools.uv`
- reject env requirements that combine `secret: true` with a contract `default` during validation,
  so `ota validate`, `ota doctor`, and dry-run previews fail before task execution reaches env
  resolution
- fixed `ota run <task> --dry-run` env reporting so mode-specific task env overrides are resolved
- fixed selected-path toolchain trust across `ota doctor` and `ota run --dry-run`: Ota now
  infers required executables from ordinary task `run:` / `script:` command bodies (not only
  `launch` blocks), projects toolchain-owned commands such as `uv`, `go`, `dotnet`, and
  `bundler` into the selected requirement surface, and blocks early when the chosen native or
  container context does not actually provide that owned tool instead of incorrectly reporting the
  path as runnable
- extended `toolchains.python.provider: uv` so `package_managers.uv` declares the required uv
  version explicitly; `ota doctor`, `ota run --dry-run`, and real `ota run` now block early when
  the selected path resolves a uv binary outside that required version range instead of failing
  later during `uv sync` / `uv run`
- fixed alias-aware selected-path owned-tool projection so Bundler version governance survives
  task commands that invoke `bundle`; `ota doctor`, `ota run --dry-run`, and real `ota run` now
  block early on mismatched Bundler versions instead of treating `bundle` as an unrelated wildcard
  tool
- stopped treating repo-local executables such as `bin/bundle` as host tool probes during selected
  path inference; `ota doctor` and dry-run previews now let repo wrappers run inside the selected
  working tree instead of incorrectly failing early on missing global `bin/...` commands
- added task `env_bindings.<NAME>.from_service` so contracts can derive task environment values
  from declared service endpoints, including container callers that need host-view services, and
  added `password_env` for secret-safe service URL credentials while keeping literal `password`
  limited to documented local/dev use
- clarified container image acquisition failures during container runtime/tool probes so Ota reports
  one actionable container-image blocker instead of downstream runtime/tool probe noise
- fixed `ota run <task> --dry-run` precondition scoping so task previews evaluate only the
  selected task dependency path, avoiding unrelated top-level precondition/native-prerequisite
  checks while still blocking when the selected task actually requires them
- fixed `ota run <task> --dry-run --mode container` next-step rewriting so explicit host-readiness
  guidance (`ota doctor --mode native`) is preserved instead of being rewritten to container-mode
  `ota up --dry-run` commands
- added targeted run-failure guidance for container non-root package-install errors (for example
  `apt-get` permission failures on `/var/lib/apt/lists`) so Ota points operators to baking OS
  dependencies into the container image instead of installing them inside task commands
- added repo-scoped execution locking for `ota run` task execution so concurrent local runs fail
  fast with explicit lock guidance instead of contending silently on shared package/tool caches
- added first-class effect governance policy under `policies.effects` (mode + `tasks` +
  `safe_tasks`, each using `allow|warn|deny` for network lanes and external-state targets),
  enforced deny decisions pre-execution in both `ota run` and `ota up` preflights, and surfaced
  resolved governance decisions in run receipts under the existing `policy` evidence lines
- hardened `ota up` effect-governance enforcement to fail closed before provisioning/setup side
  effects whenever selected-path policy decisions resolve to deny, including workflows that declare
  a setup task
- allowed policy-governed `fulfillment: run` for `toolchains.go`, `toolchains.ruby`, and
  `toolchains.dotnet` (removing check-only validation blocks for those providers) while keeping
  provisioning authority in org policy/back-end requirement fulfillment
- allowed policy-governed `fulfillment: run` for `toolchains.node` (`provider: corepack`) and
  `toolchains.java` (`provider: sdkman`) by removing check-only validation blocks and treating
  those fulfillment lanes as selected-path run intent governed by org policy/provisioning rules
- added `--effect-override <effect>=<allow|warn|deny>` to `ota run` and `ota up` for explicit
  per-invocation effect-governance overrides on selected task/workflow paths

## 1.6.18

- added `--ready-timeout <DURATION>` to `ota proof runtime` so runtime-proof readiness waits are
  explicitly bounded in CI and local automation (for example `90s`, `5m`, `1h`), aligned timeout
  failures to the explicit `TIMEOUT` runtime-proof status, and normalized timeout-only JSON
  classification to `failure_class: readiness_timeout`
- hardened `ota proof runtime` interruption semantics for CI and automation cancellation paths:
  runtime proof now captures termination signals and emits deterministic interruption output
  (`INTERRUPTED` status in text, `phase: interrupted` and `failure_class: interrupted` in JSON)
  instead of an opaque cancellation result
- added first-class file-aware container isolation mounts: file-like
  `attachments.isolated_paths` entries (for example `.pnp.cjs`) now mount through deterministic
  `.ota/state/isolated-file-mounts/*` bind files instead of invalid volume targets, while
  directory-like isolation paths continue using managed dependency-isolation volumes
- kept Corepack-backed native task execution from running `corepack enable` when the selected task
  already invokes `corepack ...` directly, and scoped Corepack activation to each direct task instead
  of the full dependency closure
- changed container Corepack command wrapping to always bootstrap shims for Corepack-owned tasks,
  including commands that already start with `corepack ...`, so nested package-manager script
  calls (for example scripts that invoke `pnpm` recursively) resolve reliably in container lanes
- scoped container Corepack command wrapping to each direct task's own toolchain requirements,
  preventing aggregate tasks from inheriting Corepack activation from already-run dependency tasks
- set a writable default `HOME=/tmp` for non-root container runs launched with the host UID/GID,
  preventing package managers such as Corepack from trying to write under `/.cache`
- prepared managed dependency-isolation directory volumes for the selected host UID/GID before
  container task startup, so package managers can write isolated paths such as `node_modules`
  without falling back to root-owned workspace artifacts
- made `ota run` block on selected precondition failures before starting the task process, matching
  `ota run --dry-run` for container-image missing-tool blockers and runtime/tool version mismatch
  blockers, and kept existing contract/env validation errors on their more specific diagnostic
  paths
- extended the same real-run precondition gate to version mismatch blockers, so `ota run` stops
  before dependency tasks when the selected path requires a different runtime or tool version
- unified `ota run` precondition version-mismatch output across runtime/tool blockers with the
  same structured task-first layout (`task ... is blocked`, plus `Where`, `Field`, bulletized
  `Why`, and ordered `Next`), including backend-aware rerun guidance for container and remote
  lanes
- fixed `ota run` version-mismatch `Next` rendering to split combined `... and rerun ...`
  guidance into separate ordered actions (`run <install>`, `run ota doctor`, `run ota run ...`)
  for clearer task-first remediation flow
- fixed container-image probe wording so run/doctor errors consistently say "inside the configured
  container image" when a required runtime or tool is missing or cannot be probed in the selected
  image
- tightened `ota policy review` text output for pure info-only success: approved version,
  provisioning, and adapter-bootstrap policy surfaces now render as a compact `Surfaces /
  Approved / Next` summary instead of the heavier diagnostic finding layout, while warnings and
  errors keep the existing detailed review path
- matured detector-led contract writes from real pressure-test repos: `package.json#engines.node`
  plus versioned `pnpm`/`yarn` package-manager ownership now writes, merges, rewrites, and tracks
  drift through the canonical `toolchains.node` Corepack shape instead of legacy split
  `runtimes.node` + standalone package manager tools, Docker Compose service
  `start`/`stop`/`healthcheck` commands are written with their service declarations, and
  watch/dev/serve verifier scripts are no longer inferred as agent-safe tasks
- improved Node mismatch remediation so `ota doctor` prefers the provider actually found on the
  probed executable path (`mise`, `asdf`, `volta`, `nodenv`, or `pyenv`) before falling back to
  repo file hints such as `.nvmrc`; this keeps `Next:` guidance aligned with the tool the host is
  really using
- hardened `ota init` starter-pack ownership to match shipped toolchain contracts and avoid
  generator-led drift: Node pack now seeds `toolchains.node` (Corepack-owned Node, default pnpm
  package-manager ownership, and Corepack-prefixed pnpm/yarn task commands) instead of split
  `runtimes.node` + top-level package-manager tools, Rust pack now seeds `toolchains.rust`
  (`provider: rustup`) instead of split `runtimes.rust` + `tools.cargo`, and Python pack now
  seeds `toolchains.python` (`provider: uv`) with uv-native setup/test commands instead of the
  legacy requirements.txt starter shape
- added a non-blocking contract advisory for legacy manual Node split ownership
  (`runtimes.node` + standalone `tools.pnpm`/`tools.yarn` without `toolchains.node`), including
  validate/doctor guidance to migrate onto `toolchains.node` Corepack ownership
- extended agent-safe `effects` advisories (`effects.network`, `effects.network_kind`,
  `effects.external_state`) across the full reachable task closure, so a safe task now reports
  dependency-path network/external-state blast radius instead of only direct task-node effects
- added `ota tasks` safety and backend-lane filters: `--safe`, `--unsafe` (mutually exclusive),
  and `--via native|container`; safety filtering uses the effective safe set
  (`safe_for_agent: true` plus `agent.safe_tasks`)
- refined `ota tasks --use` / `ota tasks` text output to keep one canonical run command per task
  and add a compact `Modes` block only for true multi-mode tasks; mode variants now render near the
  end of each task block (after notes) so the default run lane stays primary
- added explicit command-reference and quickstart guidance for `ota tasks` filtering lanes
  (`--safe`, `--unsafe`, `--via native|container`) including valid combined `--use` flows
- added a v9.1 pressure-test gap ledger documenting closed platform gaps, remaining maturity work,
  and acceptance evidence expectations for new gap intake
- added first-class task execution conditions with `tasks.<name>.when.checks`, so `ota run`
  now evaluates declared precondition/file/changed_files checks before dependency/service startup
  and skips that task deterministically when the condition lane does not pass
- surfaced task execution conditions in `ota tasks` output as `When Checks` and updated the
  published `tasks.json` schema with `when_checks[]` for machine-readable parity
- added contract-capability/minimum-version detection for `tasks.when.checks`, so older binaries
  now render an explicit unsupported-feature upgrade hint instead of a generic parse failure
- added first-class multi-step bootstrap orchestration with `action.kind: ensure_bundle`, so one
  task can execute ordered deterministic setup actions (`copy_if_missing`, `ensure_env_file`,
  `ensure_file`, `ensure_directory`) without shell glue; validation, run-path idempotence, and
  capability/minimum-version detection now include `tasks.action.ensure_bundle`
- fixed Corepack-backed container task execution so ephemeral and persistent container runs
  activate `corepack enable` inside the real task shell instead of a throwaway preflight path;
  this keeps bare repo-internal `pnpm`/`yarn` commands working after `corepack pnpm ...` /
  `corepack yarn ...` entrypoints
- switched container Corepack shim activation to a user-writable install directory
  (`corepack enable --install-directory "$HOME/.local/bin"` plus PATH export) before task
  execution, avoiding `/usr/local/bin` permission failures in non-root container runs
- defaulted Docker/Podman task containers on Unix hosts to run as the host UID:GID (`--user`) for
  Ota-managed container execution, reducing root-owned workspace artifact drift between container
  and native lanes in mixed-mode pressure-test matrices
- added one-shot container dependency-isolation recovery for permission-denied install failures:
  when a container task fails with an isolated `node_modules`/`.pnpm-store` EACCES signature, Ota
  now resets the selected context's dependency-isolation volumes and retries the task once
- hardened `ota proof runtime` detached Unix service teardown by running detached proof runs in a
  dedicated process group and signaling that full group on shutdown, reducing lingering native
  listeners that can cause late bind conflicts across sequential proof lanes

## 1.6.17

- scoped selected task/workflow requirement resolution so non-native paths (container/remote) no
  longer inherit host-global `tools` fallback when no scoped tool requirements are declared; global
  tool fallback remains for native selected paths, preventing host-only tools from leaking into
  unrelated container/remote readiness surfaces
- made `tasks.<name>.requirements.tools` and `requirements.any_of[].tools` self-contained tool
  gates: task-path tool names no longer require duplicate top-level `tools.<name>` declarations
  just to validate, while toolchain-owned names still require explicit
  `tasks.<name>.requirements.toolchains` scoping to keep ownership deterministic
- refined task network side-effect semantics with optional
  `tasks.<name>.effects.network_kind: dependency_hydration|broad`: lockfile-backed package-manager
  hydration can now be declared as a narrower network lane than generic API/remote-call execution,
  validator now requires `effects.network: true` when `network_kind` is declared, doctor/agent
  advisories now render that distinction explicitly, and task/workspace JSON schemas now include
  `network_kind`
- tightened `ota run --dry-run` context semantics so preview text now shows both
  `Task Context` and `Execution Context` explicitly (with `Contract -> Resolved Context`), and
  JSON now includes additive `requested_context` and `selected_context` fields for machine-stable
  context interpretation
- extended agent-safe write-boundary validation across transitive task chains so a safe task now
  fails validation when any reachable dependency/follow-on task writes a protected path or writes
  outside `agent.writable_paths`
- expanded first-class task disjunctive requirement branches with
  `tasks.<name>.requirements.any_of` to support context/backend-scoped alternatives across
  `runtimes`, `tools`, `toolchains`, `native`, `env`, and `checks`, and wired selected-path
  resolution into doctor/up/run requirement surfaces so mixed paths (for example local-host vs
  docker-host) do not force both lanes at once
- added first-class `action.kind: ensure_env_file` for deterministic env bootstrap without shell
  glue: Ota can now create/seed env files and append only missing keys (literal or generated random
  values) while preserving existing user-edited entries; version capability reporting and minimum
  version gating now include `tasks.action.ensure_env_file`
- added first-class `action.kind: ensure_file` for deterministic single-file bootstrap without
  shell glue: Ota can now create one repo-relative file from exactly one source (`template`,
  literal `value`, or generated `random`) while leaving existing files untouched on repeat runs;
  version capability reporting and minimum-version gating now include
  `tasks.action.ensure_file`
- added first-class `action.kind: ensure_directory` for deterministic directory bootstrap without
  shell glue: Ota can now create a repo-relative directory when missing, no-op when it already
  exists as a directory, and fail clearly when the path exists as a non-directory; version
  capability reporting and minimum-version gating now include `tasks.action.ensure_directory`
- added first-class `checks[].kind: changed_files` for git-diff-backed conditional checks using
  explicit path matchers and optional `base_ref` / `head_ref` range control; selected-path
  diagnosis now treats these checks as precondition-style gates where requested, and version
  capability reporting and minimum-version gating now include `checks.changed_files`
- added first-class `services.<name>.readiness.kind: compose_health` for compose-managed service
  health-state readiness without host-port probing: Ota now supports direct compose container
  health gating (`healthy`), validates compose-only readiness shape (`manager.kind: compose` with
  no endpoint-probe fields), surfaces the capability in minimum-version gating, and extends
  `ota assist declare-readiness --service ... --style compose-health` plus
  `ota assist declare-service --style compose-health` for service-side proposal generation
- added `agent.exceptions.sensitive_writes` contract advisories that flag non-sensitive or
  posture-redundant exceptions so intentional boundary exceptions stay narrow and meaningful
- added first-class `tasks.<name>.runtime.readiness.signal_probes` so one service runtime can gate
  readiness on multiple named listener probes (for example API + worker liveness) instead of only
  one aggregate endpoint check; version capability reporting includes
  `tasks.runtime.readiness.signal_probes`
- expanded `tasks.<name>.runtime.readiness.signal_probes` for native service runtimes so named
  same-task listener probes may use `target.address_view: internal` with fixed listener bind
  endpoints; this lets worker/internal listeners participate in runtime readiness without forcing
  host endpoint projection
- hardened smoke-workflow run-preview JSON assertions to validate verdict shape without
  hardcoding a fixed verdict enum, reducing CI brittleness as verdict taxonomy evolves
- made smoke-workflow preview checks schema-driven by validating `ota run --dry-run --json` and
  `ota up --dry-run --json` payloads against the published contract schemas
  (`docs/spec/json-schemas/run-preview.json`, `docs/spec/json-schemas/up.json`) across repo and
  example lanes, keeping only minimal semantic assertions on top
- kept smoke schema validation fully local/offline by resolving published schema `$ref` paths
  from the repository schema tree instead of fetching remote schema IDs during CI
- expanded published `tasks.json` task item shape with optional `context` and `notes` fields so
  `run-preview.json` validation for `requested_task` remains schema-accurate on real contracts
- added first-class `ota json validate` support in the Rust CLI so CI can run command
  execution, payload capture, published-schema validation, and optional assertion checks without
  Python-side validator scripts
- extended `ota json validate` with artifact-first input mode (`--input <file|->`) so CI can
  validate existing JSON payloads without rerunning producer commands; input mode keeps assertion
  parity (including exit-map checks via synthetic exit code `0`) and makes `--write-payload`
  optional
- removed deprecated `RefResolver`-based schema validation paths from smoke CI by switching to
  the new core `ota json validate` command surface
- added workflow guard checks that fail CI if deprecated `RefResolver` usage reappears in
  `.github/workflows`
- added `ota validate` warning coverage for unpinned `agent.bootstrap.ota.sh` /
  `agent.bootstrap.ota.powershell` commands so agent bootstrap install paths keep explicit ota
  version pins instead of drifting with latest installer releases
- expanded workflow service summaries to include transitive task `requires_services` in addition to
  `workflows.<name>.services.required`, so `ota tasks --workflow ...` reports the full service
  footprint used by prepare/setup/run task closures

## 1.6.16

- removed `visual_studio_build_tools: true` from docs and shipped workflow examples so public
  guidance only teaches the structured `platforms.windows.visual_studio.components` form while
  keeping legacy shorthand compatibility in the parser
- added structured `native_prerequisites.<name>.platforms.windows.visual_studio.components`
  support plus platform-scoped `native_prerequisites.<name>.platforms.<os>.requires`
  runtime/tool/toolchain/env/check requirements, so contracts can model Windows native build
  bundles through an Ota-owned `vswhere` probe, attach dependencies such as Python, and preserve
  receipt provenance without embedding long raw PowerShell checks in `checks.run`
- made `ota --version` build identity explicit for source builds by including commit and dirty
  markers, and added `ota --version --json` with stable provenance fields (`semver`,
  `source_build`, `commit`, `dirty`), `schema_version`, and additive
  `contract_capabilities[]` entries so machines can distinguish build identity from contract
  capability support
- formalized `ota --version --json` as a compatibility-locked JSON surface with a published
  `version.json` schema and conformance coverage
- enforce `metadata.ota.minimum_version` at contract load time across command surfaces (not only
  `validate`), so `doctor`, `up`, and related commands fail early with a clear minimum-version
  message when the running binary is too old
- centralized preview-status semantics so `doctor`, `ota run --dry-run`, and `ota up --dry-run`
  share one verdict model for `READY` / `READY WITH WARNINGS` / `BLOCKED` and
  `RUNNABLE` / `RUNNABLE WITH WARNINGS` / `BLOCKED`, and fixed preview JSON helpers to keep
  `preview_status` present on repo and member `ota up --dry-run` payloads
- minimum-version compatibility diagnostics now call out detected unsupported contract
  capabilities when the contract uses a known feature newer than the running binary
- minimum-version compatibility errors are now feature-first and operator-grade: they report the
  contract minimum, current binary identity, detected unsupported contract feature when known, and
  the next install/rebuild step with `ota --version --json` as the confirmation lane
- documented the product rule that `schema_version` moves only for non-additive contract-generation
  changes while additive compatibility growth extends `contract_capabilities[]`
- added an `ota-readiness` CI guard lane (`json-schema-guard`) that runs
  `json_schema_contracts` and `json_output_conformance` before readiness execution, preventing
  schema/output contract drift from reaching the main readiness lane
- aligned `ota up --dry-run` and `ota run --dry-run` preview messaging with explicit
  `preview_status` (`RUNNABLE`, `RUNNABLE WITH WARNINGS`, `BLOCKED`) while keeping canonical
  shared readiness verdicts in `summary.verdict`
- `ota doctor` now warns when `.devcontainer/devcontainer.json` advertises a Node image that
  drifts from the repo contract's declared `runtimes.node` requirement, so repo-owned
  devcontainer shells do not silently lag repo readiness truth
- `ota init` starter agent boundaries now lock `ota.yaml` into `agent.protected_paths` as an
  explicit default and carry the matching `agent.exceptions.sensitive_writes: [ota.yaml]` rule
  whenever a starter intentionally grants contract-authoring authority with writable `ota.yaml`
- `ota proof runtime` now treats warning-only risk findings such as
  `effects.external_state` / `effects.network` selected-path advisories as visible proof context
  instead of proof-failing blockers; only error-level proof findings now collapse a successful
  readiness proof
- `ota doctor` now also warns when a repo-owned devcontainer bootstrap command uses a different
  Node package manager than the repo contract declares, and when `agent.writable_paths` includes
  sensitive lockfile, env/config, runtime-topology, CI, or repo-contract paths beyond the
  declared `agent.posture`; narrow intentional exceptions can be acknowledged through
  `agent.exceptions.sensitive_writes` while the legacy
  `agent.acknowledged_sensitive_writable_paths` alias still loads for compatibility
- agent boundary validation now rejects overlapping `agent.writable_paths` and
  `agent.protected_paths` entries when they duplicate the same normalized path, while still
  allowing protected carve-outs under broader writable roots
- fixed `ota proof runtime` wait-budget derivation for selected service surfaces so proof now
  honors declared readiness timing policy (`start_period`, `interval`, `timeout`, `retries`)
  instead of collapsing heavy startup paths down to a small timeout-only window; fresh CI Docker
  builds such as Hoppscotch self-host proof now respect the contract's actual startup budget
- added `execution.contexts.<name>.only_on` so contracts can declare supported host OSes per
  execution context; `ota doctor`, `ota up`, and task execution now fail early and explicitly
  when a selected context is not supported on the current host platform
- fixed contract identity counts in preview and related output so scoped execution-context and
  task requirements are counted honestly instead of reporting misleading zero-runtime/zero-tool
  summaries when the selected workflow is actually blocked on scoped requirements
- improved native/container/remote Python runtime probing to accept versioned aliases such as
  `python3.12` and `python3.13` when they satisfy `runtimes.python`, preventing false
  `Missing runtime: python` or mismatch findings on repos that already expose a compatible Python
  interpreter through standard aliases
- shipped `toolchains.python` with `provider: uv` as a first-class managed Python runtime owner:
  validator, doctor, dry-run, and run-path fulfillment now understand uv-backed Python toolchains,
  `fulfillment: run` can invoke `uv python install <version>` for installable version references,
  and detector/init now prefer the shipped Python toolchain contract over fallback Python
  opportunity guidance when `uv.lock` and Python version signals are present
- fixed selected workflow/task toolchain scoping so preview/doctor/up no longer fall back to every
  declared toolchain when the selected closure does not require one, preventing false runtime/tool
  blockers on unrelated workflows such as host-Docker setup paths
- updated detector-led `ota init` starter agent bootstrap commands to pin the installer target to
  the running Ota version via `OTA_VERSION=v<current>`, so generated contracts avoid floating
  `latest` install targets for both shell and PowerShell bootstrap paths
- added Slack release announcements to the release gate workflow by publishing the same generated release summary text to `SLACK_RELEASE_WEBHOOK_URL` when configured, while keeping Discord publishing unchanged.
- run branch-protection-required maintainer checks on branch pushes as well as `main`, and remove docs-only trigger filtering from `docs-quality`, so protected branch-first merges work without requiring a public PR flow
- taught `ota doctor` to short-circuit later service/check readiness probing when selected-path
  preconditions already contain blocking errors, so broken setups stay bounded and diagnosis
  output keeps the real blocker in front instead of spending time on unreachable surfaces
- added `tasks.<name>.effects.writes` as first-class task side-effect metadata, and now validate
  agent-safe task writes against declared `agent.protected_paths` plus `agent.writable_paths`
  when that writable boundary is present
- expanded task side-effect metadata with `tasks.<name>.effects.network` and
  `tasks.<name>.effects.external_state`, surfaced those effects through `ota tasks --json`,
  `ota workspace tasks --json`, text task inventory, and generated `AGENTS.md`, added
  validation for machine-readable external-state tokens, and now surface agent-safe risk warnings
  plus selected-path doctor signals when those tasks depend on network access or mutate external
  systems

## 1.6.15


- fixed `ota run` captured-failure rerun guidance to preserve the effective execution mode, so
  container failures now suggest `--mode container --stream` instead of defaulting to native-mode
  rerun hints
- fixed `ota run --mode container` dependency execution selection so dependencies that declare
  container mode branches run on the selected container backend instead of silently falling back to
  native when a task also has a native default mode
- activated Corepack shims on the run path for Corepack-owned toolchains before task execution,
  so repo tasks that call package-manager entrypoints (for example `pnpm`) remain runnable without
  requiring separate manual shell bootstrap
- tightened the first-party Ota skill contract-authoring guidance with production-readiness gates
  for scope honesty, deterministic setup, agent safety, workflow fidelity, CI proof posture, and
  toolchain/runtime/tool ownership boundaries
- scoped runtime-proof cleanup to the selected workflow/task closure instead of all declared
  execution contexts, so `ota proof runtime --workflow <host-workflow>` no longer fails cleanup on
  unrelated container contexts that are not part of the selected proof path
- fixed runtime-proof JSON/result consistency for non-blocking informational findings: proof now
  ignores `info`-severity primary blockers when computing `error`/`next` and success, preventing
  false failed proof output when verdict is `ready`
- improved container probe remediation when image manifests do not match the current engine
  platform request (`no matching manifest ...`), including explicit guidance to align Docker mode
  and image platform tag instead of surfacing only generic probe failure guidance
- fixed `ota up` detached service-run readiness semantics: successful run-process exit no longer
  drops workflow surface-readiness failures, so `up` now stays aligned with `doctor` instead of
  reporting false `READY` when the declared workflow surface never becomes reachable
- fixed native service-task startup classification when a command exits `0` before its declared
  runtime endpoint is reachable: `ota run` now treats that path as a failed start instead of
  reporting false success, which closes common `EADDRINUSE` startup-misclassification cases
- improved detached `ota up` run-failure diagnostics by surfacing a sanitized tail hint from the
  detached run log (for example explicit `address already in use (EADDRINUSE)`), so operator
  output points to startup bind conflicts without requiring manual artifact triage first
- fixed native task execution to preserve the same resolved `PATH` used by toolchain probes
  instead of invoking a login shell that could reorder Node/Corepack/pnpm on macOS and other Unix
  hosts
- fixed detached native `ota up` service proof so an already-occupied fixed listener port fails as
  a bind conflict instead of being mistaken for proof that the newly launched service became ready
- made automatic `ota up` service proof selection honor the selected execution mode's runtime shape,
  so tasks that declare service runtimes only under `execution.modes.<mode>` are still handled as
  services for that mode
- corrected container-scope info text so plural host-bound surfaces like `checks` render with the
  right grammar in `doctor`/`up` output
- added `failure_class` to `ota proof runtime --json` status output so CI and automation can
  distinguish cleanup, readiness, and run/install-or-toolchain failure classes without brittle
  log-parsing

## 1.6.14

- inferred `launch.kind: command` executables as scoped tool requirements in workflow/task
  requirement surfaces, so `doctor`/`up` now diagnose and activate launch-command dependencies
  (for example `npx`) without requiring duplicate manual `requirements.tools` entries, and scoped
  workflow selection no longer falls back to unrelated global tools when launch-command tools are
  the only task-level tool requirements
- fixed `ota up`/`ota proof runtime` service-readiness proof behavior when the detached proof run
  process exits successfully before readiness is observed: Ota now keeps probing for a short,
  bounded grace window and only fails if readiness still does not arrive, preventing false
  `Run task exited before readiness` outcomes on startup paths that finalize quickly
- fixed detached `ota up` proof-run dependency behavior so `--skip-deps` is only applied when the
  selected run task actually declares `depends_on`, preserving expected startup dependencies for
  dependency-free run tasks
- widened Windows container-engine unavailable classification for cleanup/proof teardown to include
  Docker named-pipe API failure signatures (for example `pipe/docker_engine`), so proof cleanup
  errors are classified consistently as engine-unavailable instead of generic cleanup failures

## 1.6.13

- aligned the repo readiness contract with the container-first execution path: the root
  toolchain now targets Rust 1.95.0, `setup` explicitly provisions `rustfmt`, and the hosted
  readiness workflow now validates the container execution mode instead of the host-only path
- added a first-class repo run preview surface: `ota run <task> --dry-run` now renders `RUN
  PREVIEW` with the shared readiness vocabulary and selected execution/requirement plan, while
  `ota run <task> --dry-run --json` emits the matching machine-readable preview payload; repo-level
  `--json` is now documented and enforced as preview-only for `ota run`
- published and locked the `ota run <task> --dry-run --json` contract: `run-preview.json` now
  defines the shipped single-target, blocked-preview, aggregate-member, and pre-preview error
  envelopes, and end-to-end conformance coverage now checks real command output against that schema
- added a dedicated GitHub Actions smoke workflow for the public Ota surface: core repo smoke now
  runs on Linux, macOS, and Windows against the installed ota binary, while canonical example and
  workspace smoke stays Linux-only for now to maximize signal without turning examples into a noisy
  cross-OS matrix
- added explicit `ota up` service-run behavior controls with `--attach`, `--detach`, and
  `--ready-timeout`; default `ota up` now runs service-runtime readiness proof in cleanup-owned
  mode (prepare + verify + teardown + return), while `--detach` keeps the proved workload running
  intentionally
- strengthened unsupported managed-toolchain opportunity diagnostics and JSON guidance surfaces:
  `doctor`, `detect --dry-run --json`, and `init --json` now share one declared unsupported
  ecosystem list and include stable agent-facing `toolchain_opportunity` metadata (including
  `agent_note`) for unsupported ecosystems such as `python`
- fixed `ota up --detach` runtime lifecycle consistency across backends: native keep-running
  paths now launch detached proof-run workers in their own process group so the proved workload
  remains alive after `ota up` returns, and detach success receipts now report the selected
  workflow run execution context instead of falling back to diagnosis-only context metadata
- fixed `ota up --native` workflow surface diagnosis drift for service-runtime proof mode:
  workflow surface checks now honor the effective execution override backend instead of always
  resolving from default task-context backend, and proof-teardown mode now suppresses stale
  post-teardown workflow-surface findings so successful native proof runs return `READY` instead
  of false post-up `NOT READY` surface timeouts
- fixed service proof-readiness exit handling so successful detached `ota run <task> --stream`
  exits no longer auto-mark `ota up` ready when non-surface readiness still fails; Ota now only
  suppresses stale workflow-surface findings in that exit path and preserves real probe/check
  blockers as `NOT READY`
- improved `ota clean` container-engine failure handling across both text and JSON output:
  cleanup errors now surface `Container engine unavailable` with structured engine/resource
  semantics, concrete `Next:` guidance, and a compact `Details:` line instead of raw cleanup
  failure text with an empty follow-up section
- added schema-backed JSON conformance coverage for `ota clean --json` across repo, workspace,
  stale, and structured engine-unavailable failure paths, so the published `clean.json` contract
  is now validated against real command output instead of helper-only shaping
- fixed the public `ota clean --json` command boundary so the CLI now accepts repo-scoped clean
  JSON directly and preserves structured clean failure JSON on stderr instead of rewrapping it
  into generic `Operation failed` prose
- fixed the remaining generic `ota clean --json` failure paths so contract target resolution,
  invalid-contract, wrong-target, and repo-load failures now stay inside the published
  `clean.json` envelope instead of leaking `ValidateFailure` or bare raw-error payloads
- expanded end-to-end JSON contract hardening beyond `clean`: representative success/failure stream
  placement is now locked at the CLI boundary, `validate`, `env`, `doctor`, and
  `workspace tasks` now have schema-backed command conformance coverage, `check`, `receipt`,
  `up`, `workspace check`, `workspace doctor`, and `workspace up` now have end-to-end schema
  coverage too, `validate.json` now admits the shipped `warn_count` summary field, `doctor.json`
  now admits the shipped execution `default_context`, `contexts`, and env `source` fields,
  `workspace-check.json` now admits shipped repo execution summaries, `up.json` now admits the
  shipped preview execution `context` field, and `workspace-up.json` now admits the shipped
  receipt `status` plus top-level repo readiness counts
- wired the JSON contract suite into the canonical compatibility gate and GitHub release gate so
  `json_output_conformance` now runs automatically alongside `json_schema_contracts`

- improved runtime-proof failure classification when startup exits before readiness: proof output now
  prioritizes `Run task exited before readiness` over generic workflow-surface readiness blockers
  when both are present, so deterministic startup-exit root causes surface first
- fixed runtime-proof cleanup/backend scoping and severity handling: `ota proof runtime` now runs
  owned cleanup only when the selected workflow/task closure actually uses a container backend,
  and info-only doctor blockers no longer force `ok: false` proof wrappers
- fixed runtime-proof exit/error reconciliation: if the selected proof path reaches a final ready
  doctor verdict with no blocking finding, an `ota up --stream` process exit observed during
  readiness waiting no longer forces a proof failure wrapper by itself
- bounded container-backed cleanup command execution during `ota clean`/proof cleanup paths so a
  stalled engine call cannot hang the CLI indefinitely; timeout exits now return explicit timeout
  stderr context instead
- widened bounded cleanup-command timeout coverage to Ota-owned `ota clean` engine operations so
  Windows PowerShell proof wrappers cannot stall indefinitely in cleanup/finally paths when the
  engine command path hangs
- added fail-fast timeout caching for Ota-owned internal `ota clean` engine calls: once an engine
  command times out within a clean invocation, subsequent internal clean calls to that engine
  fail immediately with bounded timeout context instead of repeating long waits
- added Windows crash-code decoding guidance for common negative exit codes in both run and doctor
  failure paths, including `0xC0000005` (access violation), `0xC0000409` (fast-fail/stack buffer
  overrun), and `0xC000013A` (interrupt), so remediation output is actionable without manual code
  translation
- added first-class non-gating workflow readiness signals with
  `workflows.<name>.readiness.signal.{checks,probes,surfaces}`: Ota now executes those surfaces as
  informational diagnostics that do not block repo readiness verdicts, while preserving strict
  reference and attachment validation; overlap between gating and signal readiness lanes is now
  rejected explicitly so one item cannot be both blocking and non-gating
- tightened toolchain-owned package-manager modeling and workflow dry-run scoping: task
  `requirements.tools.<name>` now validates against both top-level `tools` and selected
  toolchain-owned tools/package managers (for example Corepack-owned `pnpm`), task-level
  requirements are no longer rejected as duplicate ownership when the tool is owned by a selected
  toolchain, and selected-workflow `ota up` activation/provisioning/preview lanes now render or
  act on Corepack package-manager requirements only when the selected workflow/task closure
  actually requires those tools
- tightened validator determinism for toolchain-owned task tools: when a task references a
  toolchain-owned tool in `requirements.tools` without explicitly scoping
  `requirements.toolchains`, validation now fails with an explicit remediation message instead of
  implicitly inferring ownership from all declared toolchains
- fixed `ota proof runtime` readiness waiting for packaged/container startup paths by replacing
  the fixed 180-second wait budget with a deterministic strategy-aware budget (with floor/ceiling),
  so cold image pulls and first-run packaged launches have enough headroom before proof times out
- tightened workflow-surface readiness observation latency in `ota doctor` by reducing the bounded
  retry windows and capping per-attempt probe timeout to 5 seconds, so repeated
  `ota doctor --workflow ...` polling no longer blocks for long windows while startup is still in progress
- fixed host-loopback readiness probing across `ota doctor`, `ota up`, and runtime probes so
  loopback surfaces declared as `127.0.0.1` now also resolve canonical local aliases (`::1`,
  `localhost`) before failing; this removes false not-ready outcomes on macOS/Windows when the
  runtime binds IPv6 loopback
- tightened loopback alias probe behavior for explicit readiness timeouts: primary endpoints keep
  the declared timeout, while fallback alias connect attempts use a short capped timeout, so
  Windows/MINGW loopback probing does not burn most of the retry window on slow fallback sockets
- fixed long-running workflow-surface doctor calls when startup checks fail repeatedly: failed
  readiness retries are now capped to a bounded observation window (matching timeout retry
  windowing), so `ota doctor --workflow ...` no longer blocks for many minutes on slow or
  still-booting packaged startup paths
- widened the bounded workflow-surface observation window used by doctor/proof retries so slow
  first-run packaged startup paths (notably on Windows and Docker cold starts) get more startup
  headroom without reverting to unbounded readiness hangs
- raised default failed-probe retry capacity for workflow-surface readiness to match the bounded
  observation window, so slower startup surfaces can use the full bounded budget instead of
  stopping early at the legacy 120-check default
- tightened workflow-surface probe timing in `ota doctor` by capping per-attempt probe timeout to
  a bounded effective ceiling before retry budgeting, so default-surface observation stays
  responsive even when declared surface timeouts are large
- fixed workflow-surface readiness retry budgeting in `ota doctor` to avoid false early
  not-ready outcomes on real startup paths: timeout retries now use a longer default window,
  and selected surfaces now honor `readiness.start_period`, `readiness.interval`, and explicit
  `readiness.retries` when evaluating workflow-surface readiness
- fixed workflow-surface doctor probe hangs for high readiness timeouts by capping timeout-driven
  retry budgets to a bounded observation window, so a single `ota doctor --workflow ...` call no
  longer appears stuck for many minutes on slow or non-responsive startup paths
- fixed native backend version probing to resolve and execute the concrete runtime/tool binary
  directly (with Windows `.cmd`/`.bat` wrapper handling) instead of relying on shell-shaped
  probe commands, so backend-fulfillment checks now match doctor-style command resolution and no
  longer drift under shell/path differences
- improved workflow-surface readiness diagnostics in `ota doctor`: readiness probes now retry
  briefly before failing, and surface blocker details now include backend, endpoint, timeout, and
  probe-attempt context so false early failures are reduced and real failures are easier to
  diagnose
- fixed Windows native `launch.kind: command` task startup parity by resolving launch executables
  through the same command-path resolver used by doctor/probes before spawn, so workflows that
  run commands like `npx` no longer fail with Windows-specific program-not-found spawn errors
- fixed workflow-scoped doctor/check selection for selected workflow task-requirement surfaces:
  when a workflow/task path is explicitly scoped but does not declare readiness checks, Ota no
  longer falls back to unrelated global checks (for example monorepo `node_modules` file checks)
  that contradict the selected workflow path
- improved `ota proof runtime` failure prioritization so deterministic doctor primary blockers are
  now surfaced before generic up-process timeout/exit messages in proof text and JSON wrappers;
  runtime process failures still surface when no primary blocker exists
- fixed `ota proof runtime` readiness waiting to observe selected workflow surfaces/probes with
  lightweight host polling and capture the canonical doctor report once a real runtime state
  change is observed, instead of rerunning full diagnosis on every wait iteration; this keeps
  runtime proof responsive on slow Docker and Windows startup paths
- redesigned the shared `AGENT` summary block used by `ota tasks`, `ota doctor`, and `ota check`
  into grouped `Overview`, `Execution`, and `Boundary` sections, with counted wrapped lists and
  writable-path root/exception collapsing so large agent boundaries stay readable in terminal
  output; `ota check --json` now also keeps additive `agent` details alongside its verdict/finding
  payload
- deepened `toolchains.node`: Corepack-backed Node toolchains can now own declared package-manager
  activation through `package_managers`, project those tools into doctor/up/policy activation
  lanes, and reject duplicate `tools.<package-manager>` ownership when the toolchain already owns
  that package manager
- tightened the shipped toolchain provider boundary so `toolchains.rust` must use
  `provider: rustup` and `toolchains.node` must use `provider: corepack`, with validator errors
  driven by the shipped provider contract registry instead of a Rust-first fallback
- `ota run` receipts now keep actual toolchain fulfillment explicit: selected `receipt.toolchains[]`
  entries can record additive `fulfilled` and `commands[]` evidence when ota ran provider
  fulfillment commands on that execution path, instead of forcing users or automation to infer
  that from stdout/stderr
- hardened the PowerShell bootstrap installer: checksum mismatches and missing official checksum
  manifests now fail closed instead of falling back to git installs, and Windows user PATH
  persistence now requires explicit `-SetupPath`
- fixed 22 Windows CI test failures in container-mode tests: replaced the broken
  `set "__OTA_ARGS=%*"` / delayed-expansion approach in the Windows fake-docker probe
  normalizer with stable positional-arg checks (`%2==--rm && %3==--name` for version
  probes, `%3==-i` for provisioning probes), and added `./bin/node.cmd` → `./bin/node`
  normalization for the `explain_narrow_premium.txt` snapshot to handle the Windows
  `.cmd` extension on fake commands
- moved the first canonical `ota` skill source of truth to the dedicated `ota-run/skills`
  distribution repository for `skills.sh` and public installation, while keeping `ota skills install`
  as the CLI-managed install surface for Codex and Claude Code
- added README and installation-doc references to `ota-run/skills`, including the
  `npx skills add ota-run/skills` install path for users who install skills through `skills.sh`
- added `ota skills install --agent codex|claude` as the canonical first-party skill lifecycle
  surface; the CLI fetches the Ota skill from the distribution repository, stages and validates the
  complete skill tree before replacing an existing install, and the shell installer receipt now
  points users to the CLI command instead of teaching installer-managed skill setup
- fixed the release gate path/render regressions from the recent path-normalization refactor:
  backticked URLs and commands are no longer rewritten as filesystem paths, and Windows native
  task execution now routes POSIX-style `script:` bodies through Git Bash even when the runner only
  exposes the Git-for-Windows install path, while receipt/archive paths and repo follow-up commands
  keep their expected Windows-vs-contract path formatting
- fixed the macOS release-gate install step to invoke the active toolchain's real Cargo binary via
  `rustup which cargo`, avoiding the broken cached proxy/shim path on GitHub-hosted macOS runners
- added the first shipped `toolchains` contract slice: top-level `toolchains`,
  task-scoped `requirements.toolchains`, Rustup-backed diagnosis, and Rustup-backed run-path
  fulfillment; duplicate ownership across `toolchains`, `runtimes`, and `tools` is now a hard
  validation error by default, so one capability must have exactly one owner; Ota's own contract
  now declares the Rust toolchain natively instead of hiding `rustfmt` provisioning in shell
  setup, and the contract/site docs publish the ownership boundary as a first-class reference
  surface; validation also rejects unsupported toolchain declarations so the shipped surface stays
  explicitly bounded to `toolchains.rust` with `provider: rustup`
- polished the `doctor`, `up`, and `run` command surfaces around toolchain ownership: duplicate
  ownership now renders as a structured invalid-contract error instead of a generic validation blob,
  `ota up --dry-run` explains selected toolchains with honest `fulfillment: none` vs
  `fulfillment: run` semantics, and run/up fulfillment failures now name the selected toolchain,
  provider, checked requirement slice, and rerun path without falling back to standalone-tool
  wording for toolchain-owned capabilities
- expanded the shipped toolchain surface with `toolchains.java` and `provider: sdkman` as a
  first-class, check-only Java ecosystem owner: Java detection now writes `toolchains.java`
  instead of `runtimes.java` for strong repo signals, duplicate ownership now covers `java` and
  `javac`, and unsupported-toolchain opportunity guidance stays focused on ecosystems Ota still
  does not ship yet
- fixed mixed-backend `ota up` preflight so selected workflow prerequisites stay on their own
  execution boundary instead of being flattened into one doctor mode; native selected-path
  toolchains now diagnose on the host even when setup runs in a container, and the dry-run preview
  matches that backend-aware preflight
- consolidated the shipped Rust toolchain ownership model behind one internal provider definition
  so validation, diagnosis, dry-run preview, and run-path fulfillment all derive ownership,
  provider labels, primary executables, and fulfillment commands from the same Rustup-backed
  registry slice instead of repeating Rust-specific assumptions in each command layer
- tightened the provider-boundary contract for `toolchains`: the validator now teaches the shared
  provider-agnostic fields (`provider`, `version`, `fulfillment`, `required`, `only_on`, and
  `platforms.<os>.version`) vs Rustup-specific compatibility fields (`profile`, `components`,
  `targets`), and the built-in `examples/basic-rust` contract now uses `toolchains.rust` instead
  of teaching duplicate `runtimes.rust` / `tools.cargo` ownership
- formalized the shipped Rustup provider contract inside Ota so validator field legality, duplicate
  ownership, doctor-managed-surface checks, dry-run requirement rendering, and run-path
  fulfillment all read from one provider contract instead of repeating parallel Rustup-specific
  assumptions across command layers
- moved Rustup-specific field-shape validation (`profile`, `components`, `targets`, including
  platform overrides) behind that same provider contract so the validator no longer carries a
  parallel copy of provider-specific field rules
- added the second shipped toolchain contract slice: `toolchains.node` with `provider: corepack`
  now gives Node one managed runtime/executable owner without claiming package-manager ownership;
  the shipped Corepack contract stays intentionally narrow by using only the shared
  provider-agnostic fields, staying check-only (`fulfillment: none`), and leaving `pnpm` / `yarn`
  activation explicit under `tools.<package-manager>.acquisition.provider: corepack`
- sharpened toolchain preview wording so dry-run and fulfillment-facing output now names the owned
  runtime capability alongside the provider contract, which keeps `toolchains.node` honest as
  “Node via Corepack” instead of reading like Corepack itself is the runtime being checked
- exposed selected toolchain decisions as first-class machine-readable evidence: `ota doctor --json`
  now emits top-level `toolchains[]`, `ota run <task> --dry-run --json` emits top-level
  preview-path `toolchains[]`, and receipt-bearing surfaces such as `ota up --json` and
  `ota receipt --json` now emit additive `receipt.toolchains[]` entries with provider, backend,
  target OS, version, fulfillment mode, owned runtime, and owned tools/components/targets for the
  selected path
- bridged selected toolchain-owned runtime lanes into org-policy version/provisioning reasoning, so
  `ota doctor`, `ota up`, and execution-policy previews can show approved runtime versions and
  approved install sources for `toolchains.rust` / `toolchains.node` without re-declaring
  duplicate runtime ownership

## 1.6.12

- added first-class workflow `prepare.task` for host file-prep before setup: workflows can now
  declare one native `action` task that `ota up` runs before pre-setup services or setup, so
  container-backed repos can keep `setup` on the selected backend while deterministic host file
  actions such as `copy_if_missing` stay explicit; workflow summaries, JSON schemas, docs, and
  public site guidance were updated to surface the new phase
- made task-scoped `requirements.env` execution-complete: `ota doctor`, `ota env --task`,
  `ota run`, workflow-driven `ota up`, and execution/receipt env reporting now all treat
  `tasks.<name>.requirements.env` as selected-path required env truth without forcing the same
  `env.vars.<name>` entry to become repo-global `required: true`; docs, JSON reference, and
  public site env guidance were updated to match the shipped behavior
- added workflow `notes` support (contract, CLI text, and JSON output): contracts can now declare
  `workflows.<name>.notes`, surfaced in `ota workflows` and workflow-scoped `ota tasks --workflow`
  output to provide operator guidance and setup context without overloading `description`
- refreshed `ota workflows` text output to use the same flat scan-friendly layout as `ota tasks`,
  including workflow-native `Use` / `Proof` command hints, per-entry `Default` status, and inline
  workflow notes where declared
- expanded docs for workflow notes and workflow output contracts: `docs/spec/contract-reference.md`,
  `docs/spec/command-reference.md`, and `docs/spec/json-output-reference.md` now mention workflow notes
  in the correct operator/API surfaces
- added `ota validate` semantic guardrails for Node/Corepack modeling: contracts now fail
  validation when `tools.node` uses `acquisition.provider: corepack` or when any Corepack
  acquisition declares `package: node`; diagnostics now direct authors to declare Node under
  `runtimes.node` and reserve Corepack acquisition for package managers such as `pnpm`/`yarn`
- hardened Windows `mise-bootstrap` follow-through for native provisioning: after `winget
  install jdx.mise`, Ota now probes additional real install locations (including WinGet package
  directories/links), validates `mise --version` from those paths, and activates the resolved
  `mise.exe` directory on the current process `PATH` so same-run host provisioning can continue
  instead of failing with `mise executable not found after bootstrap`
- hardened native `ota up` pre-provisioning sequencing for policy-backed `mise` flows: when
  adapter bootstrap installs `mise` into standard user-local locations (for example
  `~/.local/bin`), Ota now activates that path in-process before retrying provisioning, so
  selected workflow setup paths no longer short-circuit to immediate `Missing tool` /
  `Version mismatch` precondition blocks in the same run
- fixed unmanaged native `ota up` backend fulfillment blocking: when a selected setup/run path
  is missing required runtimes/tools and no org policy pack is active, Ota now surfaces a
  canonical blocker finding (for example `Tool probe failed: <tool>`) and returns a normal
  blocked provisioning result with standard `UP SUMMARY` output (`Cause: missing runtime/tool`)
  instead of aborting with a raw backend-fulfillment policy-pack error
- corrected unmanaged native `ota up` runtime fallback classification: when no org policy pack
  is active and a required runtime is missing, fallback findings now use canonical
  `Runtime probe failed: <runtime>` wording (instead of version-mismatch wording) and still
  classify `UP SUMMARY` cause as `missing runtime/tool`
- fixed Windows native runtime/tool version probing shell semantics: Ota now emits a
  Windows-native probe command shape (`where ...`) for native Windows backends instead of
  POSIX `command -v ...`, so host runtime checks (for example `node`) no longer fail
  immediately under Windows native `ota up` / backend fulfillment paths due to shell mismatch
- fixed `mise` tool activation follow-through after native policy provisioning: after
  `mise install <tool@version>`, Ota now resolves the installed binary with `mise which`,
  runs `mise use -g <tool@version>` when the tool is not yet active, and prepends the resolved
  tool directory to the current process `PATH` so same-run `ota up` precondition checks can
  observe the provisioned version instead of remaining blocked
- hardened command startup activation for mise-managed tools: `ota doctor`, `ota up`, and
  `ota run` now activate detected mise bin/shims directories on process startup so subsequent
  command invocations can resolve policy-provisioned tools without requiring manual shell
  activation between steps
- fixed Windows cross-command native tool visibility for policy-provisioned mise runtimes:
  command startup activation now adds detected Windows mise shim directories (for example
  `%LOCALAPPDATA%\\mise\\shims`) alongside `mise.exe` so follow-up commands like `ota doctor`
  can resolve provisioned tools (such as `bun`) after a successful `ota up`
- fixed mixed-mode dependency orchestration for `ota up` / `ota run`: when a requested task
  runs in one backend but a dependency declares its own default mode (for example native
  `copy_if_missing` setup actions before a container workflow), Ota now resolves that dependency
  against its declared/default execution mode instead of force-applying the requested task mode
- fixed container backend trust on Windows and other mixed-host setups: `ota doctor` now treats
  a declared/preferred container path as blocked when the selected engine CLI exists but `docker
  info` / equivalent cannot reach a usable backend, `ota up` now preflights that backend before
  provisioning so Docker connectivity failures are surfaced as backend availability problems
  instead of misleading `mise` / tool-install diagnosis, and multi-engine contracts now prefer a
  healthy engine when one candidate is down but another is usable
- bounded service readiness retries when omitted to prevent `ota doctor` hangs:
  `services.<name>.readiness` checks now default to a finite probe budget (120 attempts) instead
  of waiting indefinitely when `retries` is not explicitly set
- hardened container task execution for mounted-repo git operations: Ota now injects a
  container-local `safe.directory=/workspace` git config surface for container command runs
  (unless the task already provides explicit `GIT_CONFIG_*` overrides), preventing
  `detected dubious ownership` failures when repo tasks invoke `git` inside the mounted
  workspace path

## 1.6.11

- fixed `ota proof runtime` lifecycle handling so the spawned `ota up --stream` process is always
  stopped on proof-exit paths and no longer leaks work when doctor artifacts are not captured
- improved `ota proof runtime` diagnostics: readiness waits now short-circuit if the proof-up process
  exits early, and that termination reason is surfaced in proof text/JSON output as the command-level
  error with actionable next steps
- fixed Windows `ota doctor --container` false-missing tool diagnostics by keeping container
  tool probing honest when a declared command resolves to shim-like entrypoints, so the same
  image no longer reports `Missing Node` in one failure mode and a probe failure in another
- updated Windows command resolution so command probing checks extensioned shims (for example
  `.CMD`) even when `PATHEXT` is sparse, reducing platform-specific false negatives in
  `ota doctor` and `ota run` prerequisite checks

## 1.6.10

- hardened Windows native prerequisite activation so `visual_studio_dev_shell` now applies to the
  real native task bodies selected by `ota run` and `ota up`, `ota up --dry-run` only advertises
  that activation on native workflow paths, conflicting task-level native activations are rejected,
  and the public docs/site now describe the same execution behavior the runner uses
- fixed container-mode workspace mounts on Windows so Docker no longer receives verbatim
  `\\?\\...` repo paths from canonicalized worktrees, and clarified container readiness output so
  `ota doctor` / `ota up` now say explicitly that container validation covers the selected
  execution image and container path while leaving host-only checks to native diagnosis
- added `ota proof runtime` as the native runtime-proof surface: ota can now validate one
  selected runtime path, capture the canonical `execution topology`, `doctor`, and `up` artifacts
  under `.ota/proof/`, and tear the runtime back down without repo-local glue scripts
- extended Ota-owned repo artifact hygiene to cover `.ota/proof/` alongside `.ota/state/` and
  `.ota/receipts/`, so doctor warnings, doctor fixes, and starter gitignore writes stay aligned
- split and published the dedicated `ota execution topology --json` schema, expanded
  `ota assist wire-setup` so it can author `action.kind: copy_if_missing` setup tasks directly,
  taught starter init/detect to attach detected env-template copy actions to setup, and added
  first-class Windows native prerequisite activation guidance for Visual Studio Developer Shell
  workflows
- added first-class file checks and native setup actions: contracts can now use `kind: file`
  checks for repo-relative file/directory state and `action.kind: copy_if_missing` for
  cross-platform template materialization instead of POSIX `test` / `cp` snippets; `ota doctor`,
  `ota run`, `ota tasks --json`, workspace task inventory, schemas, and docs now expose the new
  action/check surface
- added first-class tool acquisition metadata under `tools.<name>.acquisition`, with
  Corepack-managed and explicit shell-command activation as shipped providers: selected
  workflow/task requirement surfaces can now declare one honest acquisition lane per tool, `ota
  doctor` explains missing acquisition providers through the selected prerequisite path instead of
  repo-global guesswork, and `ota up` can activate only the selected tools before setup without
  pulling unrelated quickstart or Docker prerequisites into the same lane
- tightened the first-run command/help/docs path so root help now privileges
  `doctor -> detect/init -> validate -> up -> run/proof`, and the public docs/site describe the
  same narrower adoption lane instead of leading with the broader advanced command surface
- added a dedicated Windows native proof workflow that exercises
  `visual_studio_dev_shell` through `ota doctor`, `ota up`, and `ota proof runtime` on a clean
  GitHub-hosted Windows runner and uploads the proof artifacts for review
- added task-scoped prerequisite surfaces under `tasks.<name>.requirements`: workflows can now
  scope runtime, tool, env, and precondition-check diagnosis to the selected setup/run dependency
  closure instead of treating every front door in a multi-path repo as repo-global truth; `ota doctor`
  and `ota up` now honor that selected closure directly, `ota check` additively includes explicit
  task-scoped prerequisite checks when declared, and the flagship plus `n8n` case-study contracts
  now demonstrate contributor, quickstart, and packaged-runtime prerequisite scoping explicitly
- fixed scoped prerequisite diagnosis so runtime probes no longer short-circuit selected tool
  findings: native/container `ota doctor` now always diagnoses both runtime and tool surfaces for
  the selected workflow/task closure, and remote prerequisite diagnosis now honors the same scoped
  task requirement surface instead of falling back to unrelated repo-global truth
- tightened the workflow prerequisite boundary so an explicitly selected workflow without
  `setup.task` no longer inherits legacy `tasks.setup`, and selected task paths with scoped
  requirements no longer run unrelated top-level precondition checks unless those checks are
  referenced from `requirements.checks`
- clarified the reusable surfaces docs so object-form attachment overrides now say explicitly that
  `runtime.surfaces.<name>` still references the declared top-level reusable surface, while
  `bind` means the runtime-local listener and `project.host` means the host-facing projected
  endpoint ota reports, checks, and exposes
- hardened the Windows PowerShell installer wrapper so downloaded `bootstrap.ps1` is staged in a
  private temp directory, cleaned up after execution, and used for normal release installs even
  when a stale `bootstrap.ps1` happens to exist beside a downloaded `install.ps1`; repo-local
  `-FromSource` installs still use the checked-out bootstrap, and bootstrap failures now propagate
  the correct installer exit code
- updated the published detect/init JSON schema contract so inferred annotations now admit the
  additive metadata Ota emits today: `type`, `signal`, and task-scoped `agent_safe` /
  `agent_signal`; schema regressions now cover the richer shared inference shape directly so
  machine consumers validating `ota detect --json` or `ota init --json` do not reject valid
  annotation output
- tightened inferred annotation metadata into explicit machine-facing enums: detect/init now emit
  stable enum-backed `type`, `signal`, `agent_safe`, and `agent_signal` values instead of free-form
  strings, and the command/json reference pages now call out the exact shipped value sets
- fixed PowerShell repo detection so `ota detect` / detector-led `ota init` now infer `runtimes.pwsh` for `pwsh`-based script repos instead of emitting the legacy `runtimes.powershell` key that caused `ota doctor` to probe Windows PowerShell incorrectly
- made starter-agent inference explicit in `ota detect --dry-run` and detector-led `ota init --dry-run`: both previews now render an `Agent boundary` outcome (`Inferred`, `Partially inferred`, or `Omitted`) so repos without a safe inferred task see why the starter omits `agent` instead of having to reverse-engineer that omission from the YAML preview
- added first-class task launch sources: tasks can now declare structured `launch` in addition to
  shell `run` and `script`, with `kind: command` for inspectable packaged-command entrypoints and
  a narrow `kind: container` slice for packaged service runtimes that still preserve
  `runtime.surfaces` as the canonical publication truth; `ota tasks`, `ota workflows`,
  `ota execution topology`, workspace task inventory output, receipts, and JSON surfaces now carry
  launch details additively instead of forcing common runtime front doors into opaque shell strings
- hardened container launch execution for production use: named launch containers are replaced
  only when Ota ownership labels prove they belong to the current repo/task, attached container
  launches now observe readiness while the packaged service is still running, service launch
  lifecycle semantics are documented as persistent/Ota-managed for this slice, and the published
  execution/workspace JSON schemas now admit workflow/task launch summaries emitted by the CLI
- extended reusable runtime surfaces additively: surfaces now support optional UX metadata
  (`label`, `purpose`, `visibility`), `kind: https` now maps cleanly onto the existing HTTPS
  listener/readiness model, and `ota execution topology --json` now exposes additive
  `surface_attachments` intent alongside normalized listener truth
- consolidated the modern workflow/surface authoring story across examples and docs: the
  `examples/full-contract/ota.yaml` contract now demonstrates listener shorthand for one host-only
  service, reusable top-level `surfaces`, attachment overrides for container publication, and
  workflow `readiness.surfaces` / `{ surface: ... }` exposes in one canonical example, while the
  execution-topology docs now explain the declared-surface plus normalized-listener split directly
  and the JSON output reference now documents `ota workflows --json`
- added `ota workflows` as a read-only workflow inventory command: repo contracts can now list
  declared workflows directly, inspect the default workflow and each workflow's setup/run tasks,
  readiness surfaces, probes, checks, and resolved exposes without falling back to the full task
  inventory surface
- added first-class top-level `surfaces` as reusable runtime endpoint definitions: repo contracts can now declare one `surfaces.<name>` block for shared HTTP/TCP endpoint truth, attach those surfaces to service-task runtimes through `tasks.<name>.runtime.surfaces`, and use either list-form default attachments or object-form publication overrides for bind/project shaping and primary selection without creating a second listener system; workflow readiness and workflow exposes can reference surfaces directly, `ota execution topology` shows both declared surfaces and normalized attached listener shape, surface attachment is validated strictly, and derived runtime readiness now follows a single attached surface or the primary attached surface when one runtime publishes multiple surfaces
- added listener shorthand as authoring sugar for common local listeners: `listeners.<name>.http:
  <port>` and `listeners.<name>.tcp: <port>` now normalize into the existing verbose listener
  model with conservative `127.0.0.1` bind/host defaults, topology JSON still reports the normal
  expanded listener shape, and mixed shorthand/verbose forms are rejected clearly at parse time
- added first-class reusable readiness probes under `readiness.probes`: checks can now reference
  `probe` instead of duplicating shell commands, workflows can now declare `readiness.probes`, and
  repo readiness no longer has to restate HTTP readiness as inline helper commands just to keep
  `doctor`, `check`, and workflow-scoped diagnosis aligned; task runtime readiness and
  `services.<name>.readiness` can now also reuse those same named HTTP probes instead of
  duplicating transport fields inline
- fixed named runtime probe endpoint selection so `tasks.<name>.runtime.readiness.probe` may now
  keep `readiness.listener` as an explicit non-default listener selector, and ota validates that
  selected listener as the real HTTP service surface instead of rejecting the field or silently
  collapsing back to the primary listener
- added topology-derived readiness probes on top-level `readiness.probes`: probes can now resolve
  from declared task listeners or service endpoints instead of copying host/port URLs, while
  `ota execution topology` now also surfaces the task-probe reachability plane explicitly as
  `target.resolution_plane: command_host` so machine consumers can distinguish the shipped
  command-plane host-view slice from broader task-target semantics
  literal `url` probes remain supported for external endpoints and quick-start adoption
- extended task-target readiness probes with first-class observer-task resolution: top-level
  probes may now declare `target.observer.kind: task` plus `target.observer.task` so `host`,
  `topology`, and `internal` task views resolve exactly from that named task's effective backend
  plane instead of pretending the invoking host process sees every topology the same way
- tightened observer-backed probe reuse and timeout behavior: contract-level reusable probe
  resolution now preserves observer-backed task probe contracts without forcing host-view endpoint
  resolution, rejects unknown observer tasks/listeners/service endpoints even on the contract-only
  path, and observer-backed backend probe commands now return deterministic timeout status instead
  of collapsing Python fallback timeouts into generic failures; the generated Python probe branches
  now preserve that timeout classification instead of short-circuiting it through unconditional
  shell success/failure glue
- tightened reusable probe validation so `readiness.probes.<name>.target.observer` is now rejected
  for `target.kind: service` instead of being silently accepted and ignored
- tightened topology-derived task-probe validation so `ota validate` now rejects task targets that
  name one host-view listener without a real `project.host`, a fixed projected host port, or
  `protocol: http` when the probe itself is `kind: http`, instead of deferring those failures to
  runtime resolution
- aligned reusable HTTP probes with the canonical readiness request model: `readiness.probes`
  now supports `method`, `headers`, `success.status`, and `body.contains` in addition to the
  older single-status shorthand, so literal and topology-derived probes can own the full HTTP
  readiness contract instead of collapsing to path-plus-status only
- extended `ota execution topology` with first-class `readiness_probes` output so the declared
  machine-facing graph now exposes reusable probe definitions directly, including literal-vs-target
  source details and the declared HTTP/TCP request contract, instead of forcing consumers to infer
  probe truth indirectly from runtime/workflow references
- clarified probe authoring guidance so docs now say explicitly that Ota supports all three HTTP
  success styles: omit both fields for default `200`, use `expect_status` as the one-status
  shorthand, or use `success.status` when the fuller status-list model is clearer
- added a dedicated workflows concept page so the docs now explain what repo workflows are, when to add them, why they exist beyond tasks, and how they relate to `ota up`, `ota doctor`, and `agent.default_task`
- clarified workflow summary text so repo command output now labels the surfaced workflow neutrally as `Name` instead of incorrectly calling an explicitly selected `--workflow <name>` path the repo `Default`
- fixed workflow-scoped readiness semantics so `ota up --workflow <name>` and workspace `repos.<name>.workflow` now keep the final service and post-up diagnosis scoped to the selected workflow instead of falling back to repo-wide blockers, and workflow run selection no longer substitutes `agent.default_task` / `agent.entrypoint` when a workflow omits `run.task`
- taught workspace orchestration about per-repo workflow selection: `ota.workspace.yaml` can now declare `repos.<name>.workflow`, workspace validation now rejects unknown repo workflow names against the referenced repo contract, workspace `check` / `doctor` / `up` / `status` now target that selected workflow instead of silently assuming the repo default path, workspace `list` now reports readiness against the pinned workflow when present, and workspace JSON surfaces now expose the selected workflow name per repo
- extended execution planning to the same canonical workflow model: `ota execution plan` now supports `--workflow <name>` and resolves through the selected workflow's setup or run task instead of guessing from repo-wide execution defaults, while `ota workspace execution plan` now honors `repos.<name>.workflow` and exposes additive per-repo `workflow` / `task` in text and JSON output
- added first-class repo `workflows` with `workflows.default` as the canonical operational path: `ota doctor`, `ota check`, `ota tasks`, and generated `AGENTS.md` now surface the default workflow, `ota up` now targets workflow setup/run/services instead of hard-coding repo-wide `setup`, and workflow-declared service/runtime readiness is now the long-term source of truth with legacy `tasks.setup` and repo-level required services preserved only as compatibility fallbacks
- added canonical workspace producer ownership on `services.<name>.producer`: required services can now point at a producer task in another repo declared under `ota.workspace.yaml`, `ota doctor` / `ota up` / `ota run` now surface and honor that ownership through the producer repo contract, and `ota assist declare-service` can now author the producer-owned service shape directly; the shipped cross-repo service slice stays intentionally explicit by supporting `producer.address_view: host` only and requiring one fixed `project.host` endpoint on the producer listener
- added first-class workspace repo producer refs under `tasks.<name>.targets.<target>.service.repo`: consumer tasks can now resolve another repo declared in `ota.workspace.yaml` through its host-projected service endpoint, and host-view `activation.mode` can now reuse or start that producer through the owning repo contract before the consumer runs; the shipped cross-repo slice stays explicit by supporting `address_view: host` only and requiring one fixed `project.host` endpoint on the producer listener
- taught Ota to diagnose task mutation of managed isolated attachment paths end-to-end: `ota validate` and `ota doctor` now warn when an obvious task body cleanup like `rm -rf .next` targets a declared `execution.contexts.*.attachments.isolated_paths` path, and `ota run` now upgrades matching `resource busy` task failures into a product-level `Task mutated managed isolated path` blocker instead of leaking only the raw runtime error
- hardened native service bind env projection so tasks can keep container-friendly `bind.address: 0.0.0.0` while native runs prefer the declared local `project.host.address` for app-facing aliases like `HOST` and `SERVER_ADDRESS`
- improved installer ASCII fallback branding so PowerShell and shell install scripts now render a real `ota` wordmark instead of collapsing to a bare `ota` line when Unicode output is unavailable

## 1.6.9

- hardened repo status trust across `ota doctor`, `ota check`, and `ota up --dry-run`: single-repo `check` text now uses the shared verdict-driven readiness header, `up --dry-run --json` now carries the shared `summary` verdict block, and warning-only previews now surface the first actionable readiness finding instead of looking silently `READY`
- hardened parser and workspace cache behavior so poisoned cache mutexes now clear the tainted cache and fall back to fresh parsing instead of panicking the CLI on the next contract or workspace load
- removed the shipped `ota studio` CLI surface so the supported product stays aligned with the current doctor/init/detect/up/run adoption path instead of carrying an unadvertised local Studio export mode
- fixed Windows release installs again so Git Bash/MSYS/MINGW and PowerShell now both use the published Windows `.zip` release path instead of a nonexistent `.tar.gz`, verify `ota.exe` correctly in shell-installer post-install checks, and make explicit release-mode installs/self-updates fail honestly instead of silently falling back to Cargo git builds when the prebuilt asset download fails
- tightened `ota detect --write` to fail fast when project name/contract confidence is insufficient, so weak detections no longer produce an auto-written starter contract; this also applies detector-inferred agent boundaries (`agent.writable_paths`, `agent.protected_paths`, and provenance) before writing and keeps blocked JSON/text next steps explicit for the targeted repo path
- fixed `ota detect --write` for high-confidence candidates whose lower-confidence setup task is excluded, so derived agent guidance is now based on the exact contract being written and no longer blocks valid Maven-style detections with stale `agent` task references
- fixed the Windows bootstrap/self-update replacement path again so locked `ota.exe` updates no longer leak raw PowerShell `Copy-Item` file-in-use failures; the bootstrap script now routes wrapped locked-file errors through the deferred replacement scheduler consistently and reports the update as pending until verification
- hardened the Git Bash/MSYS/MINGW shell installer path so Windows installs use ASCII-safe operator output, locked `ota.exe` replacements are staged as pending instead of leaking raw `mv`/file-in-use failures, and release install receipts verify the binary that was just installed before falling back to older PATH entries
- fixed passive update notifications on Windows so first/stale checks wait long enough for the release lookup to complete, recent lookup failures are throttled instead of slowing every command, PowerShell fallback covers `pwsh(.exe)` and `powershell(.exe)`, and interactive `ota --version` can surface cached/new-release notices without showing failure noise
- redesigned `ota agents --review` around the real boundary states: reviewed boundaries now report `Boundary sync` as `in sync` or `update needed`, inferred boundaries report `blocked until review`, fully synced reviews end with `Boundary is already synced.` plus an inline `Next: run \`ota doctor\` ...`, and the older `AUTHORED` / `explicit` wording is gone

## 1.6.8

- added a first-class contract confirmation workflow for inferred agent boundaries: `ota agents --review` now inspects the current `agent` boundary and provenance directly from `ota.yaml`, `ota agents --confirm --dry-run` previews the exact reviewed-boundary contract mutation, and `ota agents --confirm` writes `agent.inferred_boundary.reviewed: true` before any downstream `AGENTS.md` sync
- extended the execution-selector family onto `ota doctor` and `ota receipt`: both commands now accept backend shorthands (`--native`, `--container`, `--remote`), real lifecycle override via `--lifecycle`, and lifecycle shorthands (`--persistent`, `--ephemeral`), while preserving the selected lifecycle in receipt identity, doctor execution context reporting, and rerun guidance instead of silently collapsing container diagnosis back to ephemeral
- evolved the starter `agent` contract surface beyond raw writable/protected path inference: starter init/detect now emit `agent.inferred_boundary.reviewed: false` plus provenance for inferred boundary entries, `ota doctor` warns when that inferred boundary has not been confirmed yet, and doctor’s agent summary now shows whether the current boundary is reviewed or still inferred
- standardized implicit no-contract repo command failures around one blocked onboarding surface: commands like `ota agents`, `ota tasks`, `ota run`, `ota up`, `ota env`, `ota explain`, `ota receipt`, `ota policy review`, `ota extensions`, the assist commands, and related repo surfaces now report `Contract missing`, reuse the compare-first onboarding lane, and include `Repo Signals` instead of falling back to low-level contract-resolution errors
- added `ota run --skip-deps` as an explicit local execution override: it skips only the requested task's declared `depends_on` chain, leaves required service acquisition and hooks intact, rejects tasks with no declared dependencies, and marks the override explicitly in run summaries, receipts, and follow-up guidance so it never masquerades as the canonical declared task flow
- corrected the repo-owned `bump:version` next-step guidance so it now points at the canonical `ota run ci` verification task instead of bypassing the contract with a raw `cargo test`
- made several command `Next:` lanes more helpful and consistent: init and detect write paths now explain why `validate`, `tasks --use`, `doctor`, and `up --dry-run` are the right follow-up sequence, detect preview/review lanes now describe the decision behind each next command, starter-pack catalog entries explain why the preview command is next, and shell-completion recovery guidance now explains when to use the explicit setup/remove/check commands
- reshaped `ota doctor` execution environment output so execution facts stay compact, environment resolution gets its own section, required-missing counts are explicit, and env entries are grouped as policy-backed, process-backed, source-backed, defaulted, or missing instead of rendering as a flat repeated `Env:` list
- normalized another public CLI output-coherence slice: workspace detect/init scaffold mutations now keep stable command headers with result status in-body, `ota workspace tasks` no longer fakes a readiness verdict, preview `Contract` sections now use the newer unpunctuated grammar, `ota assist` previews now use `Next:` instead of legacy `Apply:` tails, and `ota receipt` now groups archive metadata inside a proper `Archive` section
- improved no-contract `ota doctor` signal formatting in rich mode so the detected values now stand out visually without changing the labels, plain output, or JSON behavior
- redesigned `ota agents` when the repo contract lacks `agent`: preview mode now reports `Agent contract missing` as a blocked boundary-sync diagnosis with inferred repo signals and inferred starter agent boundaries, and `ota agents --write` now refuses until the contract declares a real agent boundary
- improved no-contract `ota doctor` output so it now reports `Contract missing`, shows trustworthy repo signals under `Repo Signals`, and uses the compare-first onboarding lane with `ota detect --dry-run`, `ota detect --contract`, and `ota init --dry-run`
- expanded no-contract `ota doctor` signal coverage for Node/package-manager repos so it now surfaces repo type, detected package manager, likely runnable tasks, and host tool availability from existing detector signals
- expanded no-contract `ota doctor` signal coverage across the broader detected repo families too: Python, Go, Java, .NET, PHP, Ruby, Elixir, Scala, and Swift repos now surface repo-type, dependency/build-tool, and host-tool hints from the same detector truth instead of falling back to an empty signal section
- finished the next contractless `ota doctor` coverage tier too: C/C++, Clojure, Haskell, Lua, OCaml, and F# repos now surface the same detector-backed repo-type, build/dependency-tool, and host-tool hints instead of falling through to the generic no-signal path
- finished the long-tail contractless `ota doctor` tier too: Dart/Flutter, Julia, R, Nim, Erlang, Zig, D, Fortran, Crystal, Elm, Perl, Haxe, Gleam, V, Ada, Solidity/Foundry, Tcl, Racket, shell, PowerShell, and Deno repos now surface the same detector-backed repo-type, build/dependency-tool, and host-tool hints instead of falling through to the generic no-signal path
- refined that broader no-contract doctor coverage so Kotlin-first Gradle repos now surface as Kotlin instead of being mislabeled as Java
- starter contract previews and writes now keep derived `agent` guidance more consistently: detect preview, detect exact starter text, detect write, init preview, and init write all preserve the starter `agent` block, writable-path inference now covers common app directories such as `app`, `components`, `lib`, and `public`, and default agent verification now prefers verifier-style safe tasks such as `typecheck` when `test` is absent
- starter `agent.writable_paths` inference is now broader and more durable: ota still includes common source/app directories explicitly, but it also performs a bounded source-root scan so custom code roots can surface in starter agent guidance without falling back to `.`
- tightened that starter writable-path inference again so operational directories such as `config`, `database`, `migrations`, `manifests`, `deploy`, and `infra` no longer enter the default starter allowlist just because they exist
- tightened that starter writable-path scan further so custom roots are now stack-aware instead of purely structural, which keeps repo-local source trees in `agent.writable_paths` while leaving unrelated non-source directories out by default
- tightened starter writable-path precision further for detected repos: ota now prefers detector-backed nested project roots such as `src/Ota.App` over broad container paths such as `src` when the repo shape provides that stronger ownership signal
- made detected starter boundaries more explicit too: detect/init now seed `agent.protected_paths` with detector-backed control files such as manifests, lockfiles, and nested project descriptors so starter agent boundaries say both what may be edited and what must stay out of bounds
- made that stronger boundary visible in the generated starter notes too: detect/init now explicitly tell authors to review `agent.writable_paths` and `agent.protected_paths` before trusting automation
- hardened workspace drift semantics for automation: `ota workspace diff --json` and `ota workspace status --json` now expose additive per-repo `drift_kind` so local dirtiness, commit divergence, missing repo, missing contract, target ambiguity, and unresolved comparison are machine-readable directly
- refined workspace drift semantics further: `ota workspace diff --json` and `ota workspace status --json` now also expose additive per-repo `target_source` so automation can tell whether the comparison target came from declared `source.ref` or from the repo's upstream branch
- clarified workspace drift text too: `ota workspace diff` and `ota workspace status` now make each `Target:` line explicit about declared-source-ref versus upstream-branch comparison provenance
- refined workspace drift roll-ups too: workspace diff/status summaries now break the previously collapsed `Missing` and `Unresolved` buckets into explicit missing-contract and target-unavailable subcounts when those cases are present
- clarified workspace source governance further: when drift is being compared against upstream-branch fallback instead of declared `source.ref`, repo-level follow-up now says that explicitly and suggests declaring `source.ref` when the workspace should own the target
- pinned the workspace refresh machine surface explicitly with a dedicated `workspace-refresh.json` schema so preview/apply refresh output is no longer documented only by shared prose
- hardened workspace source-target trust for `ota workspace refresh`: refresh now resolves targets in the explicit order `--ref` → declared `source.ref` → repo upstream branch, and refuses before preview or apply when none exists instead of falling through to a vague `git pull` failure
- refined workspace refresh failure routing further: wrong remote target (`source.ref` / `--ref`) now stays distinct from source-access failures and generic local git-state failures so the follow-up lane stays specific
- hardened the workspace lifecycle lane so `ota workspace diff` and `ota workspace status` now carry additive top-level and per-repo `next` / `next_steps` follow-up guidance, and successful `ota workspace refresh` previews now point back into the apply-and-recheck loop more explicitly
- hardened execution failure routing for `ota run`: backend-configuration failures now point through `ota execution plan` before contract edits or retries, and declared env-source failures now point through `ota env --task <name>` before file repair and rerun
- hardened execution failure routing for `ota up`: execution-plane precondition failures, backend startup failures, and provisioning failures now point through `ota execution plan` before execution-setting edits or retries
- kept repo-level `ota up` execution receipts aligned across text and JSON by appending shared receipt follow-up guidance after the final `UP SUMMARY` block and carrying the same execution-plan-first lane onto repo-target `receipt.next`
- refined the execution receipt JSON contract with additive `receipt.next_steps`, so receipt-bearing `up`, `workspace up`, `workspace run`, and `receipt` outputs expose ordered follow-up steps without forcing agents to split the human `next` string
- polished the compact human execution summaries so `RUN SUMMARY` and `UP SUMMARY` lead with `Status`, making success, failure, blocked, and interrupted outcomes easier to scan before the longer execution details
- hardened the workspace readiness and execution surfaces too: `ota workspace doctor` / `ota workspace check` now expose per-repo additive `primary_blocker`, `ota workspace explain --json` now exposes one top-level ordered workspace `actions` lane before the repo drill-in, and `ota workspace up` / `ota workspace run` now carry repo-owned additive `next` / `next_steps` alongside the shared workspace receipt follow-up lane
- hardened workspace onboarding too: first workspace creation is now compare-first between `ota workspace detect --dry-run` and `ota workspace init --dry-run`, workspace doctor/validate/list/status/receipt missing-contract guidance now points through that preview lane, and successful workspace writes now hand directly into `ota workspace validate`, `ota workspace up --dry-run`, and `ota workspace up`
- restored `ota detect --contract` as the minimal exact starter preview and removed the brittle explain JSON command-lane surface so `ota explain --json` / `ota workspace explain --json` expose only structured `actions` and `steps` instead of scraping machine commands back out of prose
- tightened the detect merge success lane so remaining diff now stays on detect-owned review (`ota detect --merge --dry-run` / `ota detect --rewrite --dry-run`) instead of incorrectly handing users to `ota explain`, and clarified the review/write/preparation wording in README and public onboarding examples
- aligned the remaining onboarding-facing docs and help surfaces with the stronger first-contract lane: repo README, command reference, and root help now teach `ota doctor`, `ota detect --dry-run`, `ota detect --contract`, `ota init --dry-run`, then the explicit write/preparation path instead of skipping the exact starter comparison step
- completed the detect mutation onboarding lane: successful `ota detect --write` now hands users directly to `ota validate` and `ota up --dry-run`, successful merge writes now route to `ota validate` plus detect-owned review when drift still remains or `ota up --dry-run` when the contract is execution-ready, and successful rewrites now point straight to `ota validate` and `ota up --dry-run`
- tightened the first-contract apply lane too: successful detector-led `ota init` now points to `ota up --dry-run` after validation so the onboarding path flows from review into preparation instead of bouncing back into generic diagnosis
- tightened the first-contract onboarding lane again so no-contract `ota detect --dry-run` now points operators to compare `ota detect --contract` with `ota init --dry-run` before any write, and detector-led `ota init --dry-run` now renders that same compare-first review path explicitly instead of jumping straight to `ota init`
- `ota explain` now orders grouped remediation actions deliberately instead of inheriting raw finding order, so preview-first and contract-authoring fixes surface ahead of later runtime follow-ups when several blockers exist at once
- aligned `ota explain --json` and `ota workspace explain --json` with the ordered remediation story shown in text by adding grouped `actions` alongside detailed finding-level `steps`, so machine consumers get the same stable first-action plan without losing per-finding detail
- expanded the safe `doctor --fix` repo-hygiene surface so the same `.gitignore` fix path now protects both `.ota/state/` and `.ota/receipts/` as Ota-owned local artifacts, with matching init/detect write behavior and updated doctor messaging
- hardened the doctor-first onboarding lane: `ota doctor` now renders the repo state as `READY`, `READY WITH WARNINGS`, or `BLOCKED`, warning-only reports still single out one highest-priority primary finding, ready repos no longer get told to rerun `ota up`, contractless guidance is preview-first (`ota detect --dry-run` / `ota init --dry-run`), and deterministic next steps now point into `ota assist` where Ota can safely author the missing contract surface
- tightened doctor's service guidance further: unverifiable required services now route into `ota assist declare-readiness` when only the probe is missing, or `ota assist declare-service` when the managed service declaration still lacks a start path and wider service shape
- tightened doctor's setup guidance too: missing-file precondition failures now point to `ota up` / `ota run setup` when `tasks.setup` already exists, or to `ota assist wire-setup` when the contract still lacks a setup path Ota can own
- kept the no-task doctor lane preview-first as well: taskless contracts now point to `ota detect --dry-run` before any detect write, while still offering `ota assist add-task` when the right fix is one explicit runnable task

- added the first shipped `ota assist` operation with `ota assist declare-readiness`: it previews or applies deterministic readiness declarations for existing task runtime services and top-level managed services, supports monorepo `--member` targeting, emits a stable proposal/apply JSON shape, and validates writes through the same contract rules as the rest of Ota
- added `docs/spec/assist-operations.md` to formalize the long-term `ota assist` direction as a deterministic contract-operation surface with a stable command catalog, stable preview/apply proposal model, explicit AI boundary, and canonical first implementation order
- added `docs/spec/assist-workflow.md` and tightened the command and JSON references so the shipped `ota assist declare-readiness` slice now has a complete operator guide, concrete task/service/member examples, explicit refusal rules, and replacement visibility guidance alongside the long-term assist spec
- added the second shipped `ota assist` slice with `ota assist declare-service`: it previews or applies deterministic managed-service declarations, creates or refines one `services.<name>` block at a time, supports explicit manager and endpoint inputs plus optional structured readiness, honors monorepo `--member` writes, and now has matching command, workflow, and JSON reference coverage
- added the fourth shipped `ota assist` slice with `ota assist bind-task`: it previews or applies deterministic `tasks.<consumer>.targets.<name>` mutations, binds one consumer task to one producer runtime listener through the current target contract, supports monorepo `--member` and `--producer-member` edges, refuses ambiguous listener selection instead of guessing, and now has matching command, workflow, JSON schema, and public-site coverage
- added the fifth shipped `ota assist` slice with `ota assist declare-env`: it previews or applies deterministic env contract mutations for one root `env.vars` requirement, one curated `env.sources[]` entry, or one explicit task-local `tasks.<name>.env` value, preserves current env precedence rules, supports monorepo `--member` writes, and now has matching command, workflow, JSON schema, and public-site coverage
- added the sixth shipped `ota assist` slice with `ota assist add-task`: it previews or applies deterministic new-task declarations, creates one `tasks.<name>` entry at a time, supports explicit `command`, `service`, `setup`, `check`, and `sandbox` starter kinds, requires explicit service listener inputs instead of guessing runtime shape, supports monorepo `--member` writes, and now has matching command, workflow, JSON schema, and public-site coverage
- added the seventh shipped `ota assist` slice with `ota assist normalize`: it previews or applies one deterministic normalization intent that moves an existing setup-like task into the canonical `tasks.setup` slot, forces the canonical setup task back to `internal: true`, refuses inherited member-overlay sources it cannot safely delete, and now has matching command, workflow, JSON schema, and public-site coverage
- added the third shipped `ota assist` slice with `ota assist wire-setup`: it previews or applies deterministic `tasks.setup` mutations, can create or refine setup bodies with explicit `--run` or `--script`, owns the phased `setup.requires_services` boundary for `ota up`, supports monorepo `--member` writes, and now has matching command, workflow, JSON schema, and public-site coverage
- expanded the maintainer version bump scripts so one bump now updates `Cargo.toml`, rolls `CHANGELOG.md` from `Unreleased` into the requested version heading, and repins the readiness workflow's `ota-version` consistently
- tightened the adoption path around Ota's own dogfood and first-run UX: the readiness workflow now pins `1.6.7`, root help now emphasizes `doctor -> detect/init -> explain -> up -> run`, `doctor --fix` explicitly presents its current repo-hygiene-only scope, and the repo's own contract now avoids warning-producing install drift and execution-only ephemeral lifecycle advice during self-hosted readiness checks

## 1.6.7

- expanded structured readiness across both task runtime services and top-level managed services: repos can now declare HTTP/TCP readiness with probe `method`, request `headers`, accepted `success.status` codes, a required `body.contains` substring, and timing controls (`interval`, `timeout`, `retries`, `start_period`); validation, doctor/runtime probing, topology/output summaries, and canonical docs/examples now follow the same richer readiness contract while legacy top-level `readiness.run` remains supported, and omitted `retries` now means ota keeps waiting by default on both surfaces instead of doing a shallow one-shot service probe
- fixed `ota up` service ordering so `setup.requires_services` now defines the pre-setup service phase: ota starts and verifies only those declared setup prerequisites before running `setup`, then starts the remaining required services after setup before final readiness diagnosis
- added `ota studio --open` on top of the existing read-only snapshot flow, so Studio can now export `.ota/state/studio/index.html` and open it in the default browser in one step without introducing a live server or a second Studio mode
- added explicit `--setup-path` support to the POSIX installer so `install.sh` can persist the chosen bin directory into the detected shell startup file only when requested, while keeping default installs non-mutating; docs now show the opt-in PATH setup path and the readiness workflow pin stays on the latest published release until `1.6.6` is actually available
- tightened `ota detect --write` blocked-write output so the existing error shell now surfaces the actual low-confidence blocker first, keeps the dry-run and retry guidance in `Next:`, and summarizes eligible inferred fields without dumping the full provenance ledger into the default failure path
- added `ota execution topology` as the first stable Studio-facing execution inspect surface: it stays read-only, validates the selected repo or member contract first, and reports declared execution intent, shared backends, services, runtime listeners, and task target bindings in both human and JSON form for topology viewers and guided contract tooling
- added a first `ota studio` prototype that exports a self-contained read-only HTML snapshot under `.ota/state/studio/index.html`, built from the existing `doctor`, `detect --dry-run`, and `execution topology` JSON surfaces so users can inspect readiness, draft contract data, and declared topology visually without introducing a parallel Studio model
- refined the Studio snapshot to feel more intentional for adoption review: detect now summarizes inferred fields and draft changes instead of only dumping raw contract JSON, the contractless state calls users toward `ota doctor` / `ota detect --dry-run .`, and the page states the current repo-first boundary explicitly instead of implying hidden workspace support
- expanded the Studio snapshot itself to cover the full declared inspect surface more honestly: services now render alongside tasks and shared backends, and task cards now drill into runtime, readiness, listener, and target detail using the same `ota execution topology --json` data rather than a Studio-only interpretation
- turned the Studio detect panel into a real contract review surface: the snapshot now shows current contract text, inferred draft text, semantic detect comparison changes/removals, contractless first-run guidance, explanation cards for blockers/topology behavior, and reviewed apply guidance for `ota init`, `ota detect --merge`, and `ota detect --rewrite` without adding a write engine to Studio itself
- refined the Studio contract-review presentation so current contract and inferred draft now render side by side, semantic detect differences are grouped as changes versus potential removals, and reviewed apply paths are rendered as clearer exact commands instead of generic pills
- grouped the Studio `Why` panel by operator intent (`Onboarding`, `Blocker`, `Activation`, `Shared backend`) so explanation now reads like a guided review surface instead of one flat stack of mixed reasons
- made the grouped Studio `Why` sections collapsible and ordered by operator relevance, and added inline `Why` callouts inside task listener and target drill-ins so topology explanation stays close to the object being inspected instead of living only in a separate panel
- expanded Studio contract review again so the snapshot now includes exact reviewed merge/rewrite contract outputs, copyable reviewed apply commands, and flatter task scan strips (`serves`, `targets`, `backend`) before deeper topology drill-ins
- added `ota studio --serve` as the first interactive Studio mode: it still serves the same local snapshot, but now enables the first safe reviewed write actions through Ota core (`ota init` for starter contracts and `ota detect --merge` for additive updates) while keeping rewrite terminal-first
- taught served Studio to refresh its review state in place from a fresh localhost snapshot after reviewed writes, instead of forcing a full browser reload after each successful apply
- added artifact-backed recent activity to Studio: the snapshot now surfaces recent repo receipt/log evidence from archived Ota artifacts, and served Studio polls the same localhost snapshot so outside `ota run` activity can appear visually without adding a daemon or a second execution model
- made Studio recent activity easier to scan by rendering compact execution-timeline strips and a receipt-truth readiness row from the same archived step data, so recent runs read like an execution trail instead of a metadata dump
- added lightweight Recent Activity focus controls in Studio so users can jump between all archived runs, failures, ready runs, or current-contract activity and see the latest failure/ready markers without changing the underlying receipt-backed history model
- added expandable receipt detail inside Studio activity cards so one archived run can now show its full step trail and archived findings without leaving the page or falling back to raw JSON
- added copyable durable log path actions inside Studio receipt details so recent activity cards now bridge directly to archived stdout/stderr artifacts instead of only listing them as passive metadata
- made Studio recent activity provenance explicit by surfacing current-contract versus older-contract receipt labels on each card, so mixed archive history is understandable without opening receipt details
- added temporal archived-age hints to Studio recent activity so cards now distinguish the most recent current-contract receipt from older current-contract or older archived runs at a glance
- added a compact recent-failure rollup above Studio activity cards so the panel now answers what broke most recently, where it failed, and which recovery command Ota last suggested before users scan the full archive list
- added the matching ready-side rollup above Studio activity cards so the focus block now summarizes both the most recent failure and the most recent ready run before users filter or expand the archive list
- promoted contract review into the top-level Studio summary cards so the page now surfaces `starter review`, `draft differs`, or `draft aligned` at a glance before users scroll into the full contract-review section
- promoted recent execution state into the top-level Studio summary cards too, so the hero now surfaces `no activity yet`, `recent failure`, or `recent ready` from the newest archived receipt before users scroll into Recent Activity
- added an `Action needed` hero card in Studio that prefers `doctor.summary.primary_blocker.next` and otherwise falls back to the latest failed receipt’s recovery hint, so the page now surfaces the single most important next step at first glance
- fixed persistent container service interruption cleanup so interrupted workloads are removed inside the shared backend instead of leaving stale in-container listener state behind, and kept the user-facing cleanup note concise

## 1.6.6

- fixed launch-facing onboarding drift: contractless quickstart/help flows no longer tell users to run `ota explain` before `ota.yaml` exists, `ota detect` rerun/contextualization now targets repo roots instead of appending `ota.yaml`, and the readiness workflow/README execution example now match the current `1.6.6` / `rust:1.94-bookworm` release surface
- expanded curated detect/init env-source inference to include the standard Spring YAML files `src/main/resources/application.yml` and `src/main/resources/application.yaml`, and fixed detector high-confidence contract replay so inferred `yaml` and `toml` source kinds preserve their actual runtime kind instead of falling back incorrectly during contract reconstruction
- clarified the shipped topology surface in repo docs: `tasks.<name>.targets.<target>` plus optional `override_input` is the current service-target-default authoring model, local-topology docs now describe the shipped workload-local shared-backend model instead of treating it as future-only, and backend-provider remote activation guidance now reflects shared-remote `host` / `topology` / `internal` support when `activation.provider_managed_cleanup: true`
- tightened launch-facing adoption docs around example discovery: added an `Examples by Goal` guide that maps repo shapes to the right canonical contract or workspace example, linked it from the README and quickstart, and surfaced the shared-local and shared-remote topology examples directly instead of leaving users to infer which raw example file proves which feature

## 1.6.5

- expanded prebuilt release publishing to the full mainstream target matrix: Linux `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`, macOS `x86_64-apple-darwin` and `aarch64-apple-darwin`, and Windows `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`
- made release checksum manifests and installers agree on asset names, while keeping installers backward-compatible with older `dist/<asset>` checksum entries during the transition
- improved shell and PowerShell installer fallback messaging so unsupported or unpublished prebuilt targets are reported explicitly before cargo fallback is attempted
- added narrow monorepo cross-member service targets through `tasks.<name>.targets.<target>.service.member`: repo commands can now resolve another declared `workspace.members` producer through its fixed `address_view: host` endpoint, and also through `address_view: topology` / `address_view: internal` when consumer and producer share one declared backend binding on the active plane; the shipped cross-member slice stays explicit and honest by keeping `address_view: host` manual-only, allowing `ensure_started` / `restart_ready` / `ensure_running` / `ensure_ready` only for shared-backend `topology` / `internal` member targets, and recording member/task/listener evidence separately in receipts
- broadened declared runtime env-source loading beyond dotenv: `env.sources[].kind` now supports curated `properties` and `json` loaders with strict parsing, deterministic key normalization, explicit failure on collisions/nulls/arrays/object leaves, and unchanged runtime precedence through `ota run` / env resolution
- extended curated declared runtime env-source loading to `yaml` and `toml`, reusing the same structured-source flattening, normalization, collision, and fail-fast visibility model already used for `json` while keeping detect/init inference scoped to the earlier explicit standard file set
- added operator visibility for declared env sources across `ota env`, `ota doctor`, and additive JSON output: source entries now report `kind`/`path`/`label`/status, resolved env values keep declared-source provenance where applicable, and doctor surfaces source-scoped missing/parse/structure/collision failures with deterministic next steps
- taught `ota detect` and detector-led `ota init` to infer curated declared env sources from standard files only: `.env.local`, `.env`, `src/main/resources/application.properties`, `appsettings.json`, and `appsettings.Development.json` now participate in detect/init provenance, confidence, and merge/apply flows without weakening the runtime rule that source loading stays declaration-driven
- added context-wide execution env defaults under `execution.contexts.<name>.env`, with execution-time precedence now resolved as context env, then `tasks.<name>.env`, then selected mode env, while keeping ota-derived cache env as fallback only
- ota now injects `OTA_WORKSPACE` for task execution and derives fallback cache wiring automatically for known attachment pairs (`.m2` -> `MAVEN_OPTS`, `.npm` -> `NPM_CONFIG_CACHE`, `.pnpm-store` -> `PNPM_STORE_DIR`, `.gradle` -> `GRADLE_USER_HOME`, `.pip-cache` -> `PIP_CACHE_DIR`, `.pypoetry-cache` -> `POETRY_CACHE_DIR`) when the task resolves through a compatible execution context
- added cross-boundary `depends_on` advisories to `ota validate` and `ota doctor`, calling out context/backend/lifecycle drift and explaining that only durable external side effects survive across that dependency edge
- made execution summaries and attachment guidance more explicit by surfacing effective in-container attachment paths such as `/workspace/<path>`, and added warnings when a declared attachment is likely unused because an explicit tool env points somewhere else
- made `ota env --task <name>` inspect the effective execution env for the selected task instead of only the raw task-local env, including context env, ota-injected `OTA_WORKSPACE`, and derived cache env when they are part of the resolved task execution
- broadened shared-remote `activation.mode: ensure_ready` for built-in remote providers (`ssh`, `tsh`, `kubectl`, `daytona`): shared-remote `address_view: host` now auto-starts against fixed `project.host` endpoints, and shared-remote `runtime.readiness.kind: http` can now probe the remote plane for `address_view: topology` / `address_view: internal`
- added `tasks.<name>.targets.<target>.activation.mode: ensure_started` as the shallow producer auto-start mode: ota now starts a supported producer and returns as soon as startup is handed off, keeps `ensure_running` for listener reachability and `ensure_ready` for deeper declared `runtime.readiness`, records distinct `started_started` / `reused_started` activation evidence, and continues to reject unsupported producer ownership shapes instead of guessing orchestration
- added `tasks.<name>.targets.<target>.activation.mode: restart_ready` as the explicit bounce-and-verify mode: ota now stops a currently reachable supported producer through its owned cleanup path, starts it again, waits for readiness, records distinct `restarted_ready` activation evidence, and keeps unsupported producer ownership shapes failing clearly instead of guessing orchestration
- added `tasks.<name>.targets.<target>.activation.mode: ensure_running` as the narrower producer auto-start mode: ota now reuses or starts a producer until the declared target listener itself becomes reachable, keeps `ensure_ready` reserved for deeper declared `runtime.readiness` contracts, records distinct `started_running` / `reused_running` activation evidence, and validates non-manual activation shapes even when the producer omits `runtime.readiness`
- completed backend-provider remote activation parity on the shared-remote slice: when caller and producer share one declared remote backend binding, `address_view: host`, `address_view: topology`, and `address_view: internal` now support `ensure_started`, `restart_ready`, `ensure_running`, and `ensure_ready` for backend-provider producer services when the matching `backend_provider` extension declares `activation.provider_managed_cleanup: true`; ota now sends provider command contexts (`run`, `activation`, `activation_probe`, `activation_cleanup`), waits honestly on the selected plane, and preserves provider-owned cleanup/restart semantics
- made remote execution receipts and summaries more truthful by carrying remote `provider` and optional `cwd` alongside the existing remote `target`, so `ota run`, archived receipts, and receipt history no longer flatten remote backend identity into one generic target string
- added first-class `tasks.<name>.targets.<target>.url` for explicit declared URL targets, keeping target precedence and `OTA_TARGET_<TARGET>` export intact without requiring fake service identity, while restricting activation to service targets only
- broadened service target identity slightly: `service.listener` may now be omitted when the producer task exposes exactly one declared listener name, while multi-listener producers still require an explicit listener selector

## 1.6.4

- breaking contract change: `execution.local_backends` has been renamed to `execution.shared_backends`; legacy `local_backends` is no longer accepted, and the shipped shared-backend families are now local `container`, local `native`, and remote `remote`
- extended policy-governed run-path fulfillment to direct container execution contexts: `execution.contexts.<name>.fulfillment: run` now provisions declared runtimes/tools inside the actual resolved execution container, including deferred fulfillment for ephemeral container tasks before the task body runs, while rejecting unsupported native/remote context declarations and leaving direct ephemeral service-task fulfillment unclaimed for now
- added first target-activation support under `tasks.<name>.targets.<target>.activation.mode` with `manual` and `ensure_ready`, including explicit override skip semantics, validation for self/cyclic activation graphs, activation evidence in run receipts/summaries, and a first honest auto-start slice for persistent container producer services when the target binding itself already resolved truthfully (for example `host`, shared-backend `topology`, or shared-backend `internal`)
- added first-class service-task `runtime.readiness` support for `http` and `tcp` probes on projected host endpoints, and taught `activation.mode: ensure_ready` to wait for declared producer runtime readiness instead of treating an open listener socket as sufficient
- broadened `activation.mode: ensure_ready` to include unix native producer services via an activation-owned native startup path, while keeping interrupt cleanup semantics that stop activation-started producer services and leave reused producers running
- broadened shared-backend target resolution so `address_view: topology` and `address_view: internal` now resolve truthfully across shared `container`, shared `native`, and shared `remote` backend boundaries; remote service runtimes are now allowed when they declare fixed contract endpoints
- broadened `activation.mode: ensure_ready` again to include built-in remote producer services (`ssh`, `tsh`, `kubectl`, `daytona`) for shared-remote `address_view: topology` / `address_view: internal` targets with TCP readiness, while leaving backend-provider remote activation as a later slice
- fixed persistent container service workloads to run under a managed detached in-container wrapper with pid/status/log tracking, so long-running dev servers like Next.js `next dev` stay alive across route compilation and request handling instead of spuriously exiting after readiness when launched through the old attached `docker exec` service path

## 1.6.3

- fixed `address_view: host` target binding resolution for container callers so loopback-only producer host projections (`127.0.0.1` / `localhost`) are translated into caller-reachable host aliases (`host.docker.internal` / `host.containers.internal`) based on the caller container backend, instead of leaking container-local loopback addresses that break cross-backend reachability
- added policy-governed shared-backend environment resolution for `execution.shared_backends.<name>.environment` (`profile` / `image_alias` / `image`), including policy-backed profile and alias approval, allowed/denied source and registry enforcement, deterministic effective image selection on the run path, and declared-vs-effective environment evidence surfaced in run summaries and `receipt.steps[*].shared_local_backend.environment`
- added backend-scoped run-path fulfillment for shared local backends: ota now computes deterministic runtime/tool requirement unions for the resolved backend unit, honors `execution.shared_backends.<name>.fulfillment` (`none`/`run`), attempts approved provisioning on the actual run path when enabled, and reports distinct missing-requirements vs fulfillment-failed outcomes with structured receipt evidence
- made run receipts and summaries fully backend-resolution truthful for shared backends: backend/context/lifecycle/image/memory now derive from resolved execution backend bindings, step-level backend fulfillment evidence is preserved, and dependency/hook steps retain machine-readable `target_resolutions` provenance
- tightened host-view target binding resolution to fail on conflicting root-vs-mode or mode-vs-mode host projections, while still allowing mixed-backend consumers when the producer host projection is unambiguous
- added first-class task target bindings under `tasks.<name>.targets.<target>` with typed service identity (`service.task`, `service.listener`, `service.address_view`) and optional `override_input` operator channels
- added strict target-binding validation for unknown service tasks/listeners, non-service targets, missing `override_input` declarations, and ambiguous duplicate override-input mappings across targets
- added run-time target resolution precedence: explicit override input > resolved target binding > compatibility literal input default, plus explicit run-time failures when requested address views cannot be resolved truthfully in current topology support
- added declared-versus-effective target evidence in run receipts and JSON under `receipt.steps[*].target_resolutions`, and surfaced resolved target bindings in run summary output
- resolved task target bindings for dependency/hook tasks as well as requested tasks, while preserving existing required-input enforcement behavior for non-requested relations
- preserved per-step target-resolution evidence in receipts for dependency/hook steps so machine-readable provenance remains truthful beyond the requested task step
- exported resolved target bindings without `override_input` as `OTA_TARGET_<TARGET>` so first-class targets remain operational without legacy input shims
- allowed `address_view: host` to resolve service listeners independently of caller backend when the producer listener declaration is unambiguous
- added first-class shared local backend declarations under `execution.shared_backends.<name>` and task opt-in bindings via `tasks.<name>.runtime.backend_binding` so multiple long-running tasks can intentionally share one ota-managed local backend boundary
- added strict shared-backend validation for unknown bindings, backend-family mismatches, context/lifecycle conflicts, and multi-context bound groups without explicit `execution.shared_backends.<name>.context`
- wired container backend resolution to honor shared local backend identity for lifecycle/context/publication shape reconciliation, deterministic persistent create/reuse/recreate semantics, and `ota clean` discovery/removal of shared-backend persistent containers
- added the first native shared-backend family under `execution.shared_backends.<name>.backend: native`, currently scoped to `scope: local` + `lifecycle: persistent`, with host-target run-path fulfillment and shared-backend receipt evidence while keeping container image/environment semantics container-only
- expanded `address_view: topology` truthfulness for container callers: topology resolution now succeeds only when caller and producer share the same declared local backend binding, and still fails clearly for unresolved/non-shared/internal cases without host bridge guessing
- added declared-versus-effective shared-backend receipt evidence per executed step under `receipt.steps[*].shared_local_backend`, and surfaced requested-task shared-backend identity/reuse state in run summary output


## 1.6.2

- made `ota init`, `ota init --pack ...`, and repo-contract detect write flows (`ota detect --write`, `--merge`, `--rewrite`) automatically create or extend repo `.gitignore` files with `.ota/state/` so local Ota runtime state stays out of source control by default
- added a first safe `ota doctor --fix` path with `--dry-run` preview mode, currently scoped to deterministic repo-hygiene repair for `.ota/state/` gitignore protection (`# Ota local runtime state` + `.ota/state/`) with planned/applied status surfaced in text and JSON output
- tightened `ota doctor --fix` truthfulness and scope: contractless repos no longer surface `.ota/state/` gitignore repair before Ota is onboarded, dry-run/apply summaries now report only actually plannable file changes, and no-contract `--fix` output now points operators back to `ota detect --dry-run` / `ota init --bootstrap` instead of low-value hygiene mutations
- added additive named-context inheritance for execution topology: `execution.contexts.<name>.extends` now resolves parent context shape with deterministic merge semantics (scalars override, maps merge, lists replace) while preserving both existing single-context shorthand (`execution.preferred`/`execution.lifecycle`/`execution.backends`) and existing named-context contracts
- moved `execution.contexts.<name>.extends` semantic failures (`unknown parent`, `cycle`, unresolved backend) into validator findings, and now reject backend-family switches across inheritance (`container`/`remote`/`native`) with explicit errors instead of inherited cross-backend drift
- made runtime context selection consume resolved named-context inheritance across execution planning and diagnosis surfaces (`ota execution plan`, `ota up`, `ota run`, and doctor mode context checks) while preserving existing shorthand behavior for single-context contracts
- fixed task dependency and hook execution so child tasks resolve their own declared backend/context instead of inheriting the parent task's resolved backend; host-scoped dependencies like `compose:up` now stay on `host` even when invoked from a container-context parent task
- fixed reused persistent container service runs so Ota preflights fixed in-container listener binds before exec and raises an explicit field-scoped listener conflict when a stale workload process is still holding the port
- fixed interrupted persistent container service runs so Ota now cleans up the interrupted in-container service workload instead of deleting the shared persistent backend, and classifies the stop from the exec workload outcome instead of the backend container sleep loop
- finalized execution-inheritance docs/output guidance so repo docs and `ota-site` now present three additive authoring patterns clearly (single-context shorthand, named contexts, named contexts with `extends`), clarify that backend-family switches across `extends` are rejected, and keep `extends` positioned as optional multi-context dedup instead of a shorthand replacement
- tightened execution contract validation so repos must choose one default execution declaration mode: shorthand-only (`execution.preferred` / `execution.lifecycle` / `execution.backends`) or named-context mode (`execution.default_context` / `execution.contexts`), with mixed overlap now rejected explicitly instead of allowing duplicated default execution truth
- tightened streamed service-stop classification so inspected container exit evidence wins over late interrupt flags, preserving real crash/exit causes instead of rewriting them as `Interrupted`
- improved streamed service interrupt UX before readiness: user `Ctrl+C` during startup is now reported as an interruption-before-readiness case (not a generic task failure), while real pre-readiness non-zero startup failures remain normal failures
- finalized terminal-cause consistency across run rendering and dependency propagation: exit-code-only interrupts (`130`/`143`) now still classify as user interruption for direct and dependency-driven runs, while late raw interrupt signals no longer overwrite real non-zero failures or clean inspected service exits in banners, summaries, and receipt status notes
- made persistent container reconciliation treat Compose attachment namespaces as part of execution shape, so changing `execution.contexts.<name>.attachments.compose` now recreates stale persistent backends instead of silently reusing containers bound to the old Compose network family
- made streamed `ota up` provisioning progress environment-aware: the loader now shows the selected native/container/remote target and container lifecycle/image where relevant instead of a generic preparation spinner
- fixed Windows release installs and `ota self-update` / `ota upgrade` overwrite behavior in `scripts/bootstrap.ps1` so existing `ota.exe` is replaced via copy-and-cleanup semantics instead of failing with PowerShell `Move-Item` "file already exists" errors

## 1.6.1

- refreshed the README entry surface with a tighter brand hero, release/status badges, a reduced primary nav, and direct links to the live get-started, docs, reference, examples, governance, releases, Discord, and X surfaces
- added execution-selection shortcut flags for quicker override ergonomics: `ota run`, `ota up`, `ota execution plan`, and `ota workspace execution plan` now support `--native` (`--mode native`), `--container` (`--mode container`), and `--persistent` (`--lifecycle persistent`) alongside `--ephemeral`, with updated help/completion boundary handling
- made streaming run interruption semantics explicit: user `Ctrl+C` now classifies as `interrupted` across run summaries/receipts (including service workloads), interrupted service-termination classification now wins over generic post-readiness stop wording, receipt step/status metadata aligns with intentional interruption instead of generic failure text, and late post-exit interrupts no longer overwrite concrete non-zero task/container failure causes
- added `ota run --log` durable run artifacts under `.ota/state/logs/<run-id>/` (`stdout.log` and `stderr.log`) for native/container runs, including ephemeral container runs that clean up immediately after failure or interruption; run receipts/summaries now surface log paths, streamed runs now tee output into the same artifacts, and log-capture write failures are surfaced directly in run output notes instead of failing silently
- fixed `--stream --log` durability gaps: stream-mode capture now honors capture toggles in runner streaming helpers, ephemeral container streaming now captures output when log persistence is enabled, and run summaries now render log-capture write failures as explicit warnings
- fixed `ota detect --merge --apply tasks.<name>.internal` trust boundaries: projected/default `internal` fields are no longer auto-eligible, and merge/apply now remains strictly high-confidence inference-backed (including explicit `tasks.<name>.internal` inferences when emitted by detector provenance)
- made ephemeral container `ota run` interruption-aware: Ctrl-C now still attempts to remove the repo-owned container created for that run, and the final run summary reports incomplete cleanup instead of silently leaving interrupted residue behind
- reclaims repo-owned orphaned ephemeral containers on later runs before starting new ephemeral container execution, uses bounded conflict-recovery retries, and can reclaim legacy running ephemerals without `dev.ota.owner_pid` when they are the stale published-port holder blocking a new run
- hardened `ota clean` cleanup integrity: drift rediscovery now keys off repo ownership labels plus `.ota/state/managed-engines`, falls back to best-effort local engine probing only when no repo engine evidence exists, and keeps ownership-ambiguous managed state visible without unsafe deletion
- added `tasks.<name>.internal: true` as an orchestration visibility boundary: internal tasks still run normally in dependency/hook graphs and via direct `ota run`, while default `ota tasks`/`ota tasks --use` omit them unless `--all` is requested (with `internal: true` surfaced in JSON for included internal entries)
- made generated `setup` tasks internal by default across starter-writing flows (`ota init`, `ota init --bootstrap`, `ota init --pack ...`, and `ota detect --write`), with dry-run/write parity so generated previews and written contracts agree on `setup.internal: true`
- aligned task discovery surfaces with `internal: true`: `ota run` shell completion and `ota workspace tasks` now hide internal task nodes by default so operator-facing listings stay consistent
- fixed ephemeral container `ota run <task> --host-port <port>` execution truth so the override now drives the actual engine `-p` publication args (not just projected metadata), with aligned runtime env/receipt/summary port reporting
- added container-context memory resources plus `ota run --memory <size>`: container contexts can now declare `resources.memory.minimum/default`, runs now pass the resolved memory request to Docker/Podman for ephemeral and persistent containers, persistent reconciliation treats memory drift as shape drift, and receipts/summaries surface the resolved container memory for truthful operator visibility
- hardened ephemeral container service failure reporting: ota now inspects container termination state before teardown, classifies post-readiness service stops as first-class run failures, and records structured `service_termination` metadata (including explicit `oom_killed` cause when reported by the engine) in receipts/JSON
- hardened persistent container execution into a reconciled model: ota now reuses named containers when the resolved execution shape is equivalent, recreates them when image/publication/dependency-isolation shape drifts, and records that reuse/recreate truth in run summaries and receipts
- persistent service-task failure semantics now match ephemeral truthfulness: post-readiness exits are classified as structured service-stop outcomes (including interrupts) while keeping persistent reuse/recreate reconciliation notes aligned in run summaries and receipts
- fixed persistent reconciliation and cleanup tracking for legacy unlabeled containers: ota now detects repo-scoped legacy persistent containers during reconciliation to clear conflicting old host publications, and `ota clean` now retains `.ota/state/managed-engines` when repo-scoped ambiguous managed state still exists on an engine
- fixed `ota run <task> --host-port <port>` with task dependencies so dependency containers no longer inherit the requested task’s published listener ports; this unblocks flows like `dev -> setup` where `setup` should run unpublished while `dev` uses the overridden host port
- added `ota run --host-port <port>` as a one-run projected host/public port override for container workload listeners with `project.host.port.mode: fixed`, keeping internal bind ports unchanged while aligning runtime env (`OTA_PUBLIC_URL`/`OTA_PUBLIC_PORT`), summaries, and receipts to the overridden public URL
- added task-level mode-aware execution branches under `tasks.<name>.execution` so one task can declare mode-specific `context`, `lifecycle`, `env`, `run`/`script`, and `runtime`, with `execution.default_mode` support, clear run-time errors for missing mode branches, and updated `ota tasks` JSON/text branch rendering
- replaced the generic task-exit banner for container host-port bind conflicts with a specific `Host publication failed` error across both captured and interactive runs, pointing at the owning listener field and carrying the ingress-specific run-summary note
- made `ota explain` and `ota workspace explain` show `BLOCKED` when the plan contains actionable remediation steps, instead of a misleading `READY` banner
- made invalid task listener bindings render as field-specific contract errors with direct `Next:` guidance across `ota validate`, `ota doctor`, `ota explain`, and `ota receipt` instead of falling back to generic load or repair banners
- made shell-based task execution forward `Ctrl+C`/termination to the task process group so long-running container and native dev commands stop cleanly instead of leaving orphaned listeners behind
- turned malformed fixed host projections into a structured `ota run` contract error with an explicit `Field:` path instead of a validation panic
- fixed auto-projected container endpoints so `ota run` resolves the published host port after the workload starts instead of failing before the engine has reported the mapping
- fixed ephemeral container workload endpoints so successful `ota run` service tasks keep their prepared public URL/runtime metadata even if the container engine cannot report the published port after shutdown
- relabeled the early stream-mode workload URL as `🦦Endpoint (planned)` so pre-start reservation output stays visible without looking like an already-live service
- fixed `ota run --stream` teardown for ephemeral container tasks so `Ctrl+C` removes the Ota-managed container and releases the published host port instead of leaving the app running behind the prompt
- fixed workload-endpoint projection boundaries so `ota up` no longer advertises task endpoints before the workload actually ran, task-scoped host publications no longer leak into unrelated container tasks, and persistent container cleanup/reuse now follows the same seeded identity as execution
- added task-scoped workload endpoint projection through `tasks.<name>.runtime.kind: service`, including named listeners with bind plus host projection settings, validation for impossible native/container projection plans, resolved runtime endpoints in `ota run` receipts/JSON, and workload endpoint reporting for tasks `ota up` actually executes during preparation
- polished multi-listener ingress output and validation: projected listeners now require one explicit `project.host.primary` when more than one endpoint is published, run/up/receipt summaries render a clear primary endpoint plus secondary count, and runtime JSON now carries stable `primary_listener`, `primary_endpoint`, and `exposed_endpoints` fields
- fixed JSON schema/runtime-ingress alignment by adding `receipt.runtime` + `receipt.workloads` schema coverage, making fixed-bind diagnostics accurately cover non-fixed modes, and rejecting projected listener name collisions that would overwrite `OTA_PUBLIC_URL_<LISTENER>`
- added container dependency isolation for `execution.contexts.<name>.attachments.isolated_paths` using engine-managed named volumes, with deterministic mount naming and `ota clean` cleanup for both persistent and ephemeral container contexts
- made container dependency-isolation volumes discoverable with Ota ownership labels plus a stable repo ownership token under `.ota/state/ownership-id` so `ota clean` can remove drifted isolation state even after the repo path, image, engine, or isolated-path declaration changes, and can safely distinguish volumes owned by different repos that share a `project.name`
- hardened `ota clean` dependency-isolation rediscovery to avoid fragile `volume ls --filter label=...` behavior by listing candidate volumes broadly and validating ownership labels through per-volume inspect metadata before removal
- updated the flagship adoption example and spec docs so containerized app ingress is modeled as task-owned workload topology instead of overloading `services`
- expanded the contract reference so the workload listener section now explains `fixed`, `discover`, and `auto` mode semantics plus the current native/container support rules
- fixed Windows-only test support utility compilation in `provisioning` command tests by centralizing shim executable setup, keeping executable behavior correct on Unix and avoiding brittle permission mutation on non-Unix platforms
- stabilized release-gate behavior by ensuring the same fix ships with existing test fixtures and contract validations so release automation can complete without platform-specific failures

## 1.6.0

- refreshed the shipped example contracts and public execution templates to the current execution-context and typed-service-manager model, including the adoption starter, container/remote templates, and the in-repo advanced examples
- expanded the topology guide with a service-scoped requirements comparison so users can distinguish contract-wide runtimes/tools from service-owned lifecycle and readiness concerns
- added a docs decision guide that compares top-level requirements, execution-context requirements, service managers, task prerequisites, and execution modes, and linked it from the docs home page so users can choose the right contract shape faster
- fixed `requires_services` enforcement so task service prerequisites fail on any readiness/healthcheck finding, including warning-severity service checks
- fixed `requires_services` runner behavior so readiness is re-checked for every task or hook that declares a service, while service start commands still dedupe within a run
- added `requires_services` to the published `ota tasks --json` and `ota workspace tasks --json` schemas, with matching docs/reference examples so valid output no longer fails schema validation
- added task-level `requires_services` so tasks can declare canonical services that must be ready before the task body runs, and surfaced that requirement in task text/JSON output plus the execution-topology docs and site examples
- surfaced resolved execution context names directly in `ota run` failure cards and `ota up`
  phase/blocker cards, so the primary human-facing error path now matches the execution-topology
  truth already present in receipts and summaries
- made legacy `execution.preferred/...` contracts honor the branch’s single-context compatibility
  model by surfacing the implicit workload context `app` in `ota run` / `ota up` receipts and
  post-setup diagnosis, instead of dropping context names whenever the repo had not been upgraded
  to explicit `execution.default_context` / `execution.contexts`
- restore visible run progress indicators after the repo moved to container-first execution:
  interactive `ota run <task>` once again relies on the run command's own streaming loaders, and
  runs now show a short preflight loader while resolving execution backends before task spawn

- made grouped policy findings in `ota doctor`, `ota up`, and the shared JSON summaries read like operator guidance instead of policy declarations, using active labels such as `Review active policy surfaces`, normalized item text like `Approved provisioning sources are configured`, and next steps that point into `ota policy review` when users need the active policy boundary.
- made single version-policy findings in `ota doctor` and `ota policy review` use the same operator-facing wording and next-step path into `ota policy review`, instead of leaving the card as a raw declared-policy summary with generic guidance.
- redesigned `ota policy review` output so policy findings no longer point back to `ota policy review` itself; the command now leads with a `Policy` context block, uses action-shaped summaries like `Approved provisioning and bootstrap surfaces are configured`, and points `Next:` at changing the repo contract, using approved sources, or updating `.ota/org-policy.yaml`.
- made shared policy-surface findings point into explicit `ota policy review <repo>` follow-ups in receipt JSON, so external-repo adoption flows no longer fall back to generic “use this policy surface” wording for approved provisioning and bootstrap guidance.
- redesigned blocked `ota up` provisioning output so it now surfaces a single primary `Reason:` and `Next:` path, demotes policy/host notes into `Additional context`, uses `BLOCKED` consistently when setup cannot clear prerequisites, suppresses leaked `setup` command framing on that path, and omits synthetic ephemeral container targets from the human `UP SUMMARY`.
- added an `Execution Topology` design spec draft that proposes execution contexts, typed service managers, context-scoped endpoints, context-scoped readiness, and context-scoped requirements for mixed host/container service repos.
- added the first execution-topology foundation slice: contracts can now declare `execution.default_context`, `execution.contexts`, and `tasks.<name>.context`, and `ota run` / `ota up` setup now resolve their execution backend from the bound task context instead of only the repo-wide default.
- added the next execution-topology service slice: services can now declare typed managers for both `manager.kind: compose` and `manager.kind: host`, `ota up` / `ota doctor` derive Compose start and healthcheck commands from the Compose manager while host managers keep readiness on the host without fake derived lifecycle commands, and `ota services` now surfaces manager-backed service control in text and JSON output.
- added the next execution-topology topology slice: services can now declare context-scoped `endpoints` plus `readiness.from`, `ota doctor` evaluates contextual readiness from the declared execution context, container task contexts now attach to declared Compose networks, and `ota services` exposes the projected service topology in text and JSON output.
- made execution-context requirements real in readiness flows: `ota doctor`, `ota up`, and backend-scoped policy guidance now resolve runtime/tool requirements from the relevant execution contexts instead of only the legacy repo-wide `runtimes` / `tools` maps, and container-mode diagnosis now also includes host control-plane requirements when typed Compose managers are in play.
- receipts, previews, and execution summaries now expose named execution contexts directly: `ota run`, `ota up`, and receipt-diff surfaces report which context executed the work, and `ota doctor` / other declared-execution summaries now show the default context plus the declared context topology instead of only flat backend metadata.
- made contextual service readiness diagnosis honest when the declared readiness context is not executable: `ota doctor` now emits an explicit topology blocker with the projected endpoint and backend-resolution failure instead of collapsing that case into a generic “service readiness failed” result.
- expanded `ota doctor --mode remote`: remote mode now probes executable remote contexts directly for runtime/tool versions, detects the remote target OS for policy-backed provisioning selection, diagnoses remote provisioning/installability failures through the same canonical provisioning path, emits approved version/provisioning policy surfaces per remote context, and blocks explicitly when a named remote context or remote OS probe is not executable.
- made remote-topology diagnosis explicit in native doctor mode: when a repo depends on remote execution contexts, `ota doctor` now emits a partial-evaluation note instead of silently implying that local runtime/tool checks represent the remote environment too.
- stopped `ota up` from silently remapping remote setup contexts onto native diagnosis: repos whose `setup` task resolves to a remote context now fail fast with an explicit blocker in normal and `--dry-run` flows instead of pretending host preconditions and post-setup diagnosis are authoritative.

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
- made Windows `ota uninstall` report a clear scheduled-success state instead of a scary pending/unverified message, and moved the deferred remover into a temp helper script that keeps retrying after the current process exits, removes empty install directories when possible, and cleans itself up.
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
