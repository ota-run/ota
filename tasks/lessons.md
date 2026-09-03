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

# Lessons

## 2026-06-08

- Pattern: While validating sibling repos, it is easy to fall back to direct `npm`, `pnpm`, or
  language-tool commands even when the repo already exposes a truthful Ota task for the same
  check.
- Correction: Prefer `ota run <task>` whenever the contract already declares the matching safe or
  verification task, and use direct tool commands only when no truthful Ota task exists or when
  isolating an Ota defect.
- Rule: For repo validation, Ota is the default execution surface. Do not bypass it with raw
  package-manager or language-tool commands when the contract already models the check.

## 2026-05-30

- Pattern: A pressure-test finding can expose a real Ota gap, but fixing it immediately without
  first telling the user blurs the line between diagnosis and implementation.
- Correction: Call out the Ota-owned gap, state the proposed platform fix, and wait for explicit
  approval before changing Ota when the user asks to discuss gaps first.
- Rule: For pressure-test work, diagnose first and implement second; do not silently turn an Ota
  gap finding into a code change unless the user has already asked to fix it.

## 2026-05-20

- Pattern: Port conflicts can distract from the deeper execution-path bug when probes and tasks
  use different shell semantics.
- Correction: Verify that Ota's selected backend, task mode, and resolved `PATH` are preserved from
  diagnosis through execution before treating cleanup as the root cause.
- Rule: When readiness or bind failures differ between `doctor`, `run`, and `up`, check mode
  routing and shell/toolchain resolution first; do not reduce the issue to manual process cleanup.
- Pattern: Detached `up` proof can race against a pre-existing listener and accidentally treat an
  unrelated service on the declared port as proof for the service being launched.
- Correction: Preflight native fixed listener binds before detached service proof so occupied ports
  fail deterministically instead of relying on endpoint reachability alone.
- Rule: `up` proof must prove the selected execution path, not just that something is reachable at
  the declared URL.

## 2026-03-21

- Pattern: Source files need valid repository license notes, but non-language comment formats or decorative Unicode headers can break compilation.
- Correction: Use a short ASCII language-valid license header in Rust source files and verify the compiler still accepts the file afterward.
- Rule: When adding or restoring license notes in code, use the target language's native comment syntax only and avoid decorative Unicode banners.

## 2026-03-24

- Pattern: Docs pages that only list commands are low adoption value and create user confusion.
- Correction: Rewrite command/docs pages to include when-to-use, why, and concrete use-cases with practical command examples.
- Rule: For OSS-facing docs updates, require adoption-first guidance (when/why/use-case), not enumeration-only content.
- Pattern: A schema-valid contract can still be operationally incomplete (for example no runnable tasks), which can produce misleading `doctor` readiness.
- Correction: Add explicit readiness checks for operational surface (at minimum one task for repo execution workflows).
- Rule: Treat schema validity and operational readiness as separate checks; `doctor` must fail on missing core execution entrypoints.

## 2026-03-25

- Pattern: A user asking for status or clarification may not be asking for implementation.
- Correction: Answer the status directly first, and only change code when the user explicitly asks to do so.
- Rule: Do not turn a confirmation question into a code change without a clear request to implement.

## 2026-04-07

- Pattern: Doctor output can be technically correct but still too repetitive for humans when every finding repeats the same `Next:` action.
- Correction: Group repeated doctor findings by shared remediation, keep the primary blocker explicit, and keep JSON unchanged.
- Rule: In human doctor output, group by action/family, not severity alone; collapse identical or equivalent `Next:` lines into one shared remediation block.
- Pattern: Grouping by literal `Next:` prose is too weak because equivalent operator actions still fan out once tool names or versions change.
- Correction: Derive grouping from normalized remediation/action classes, then render one shared remediation block with the existing Ota visual language.
- Rule: For doctor-style grouped output, normalize by operator action first and only fall back to exact remediation text when no stable action class exists.
- Pattern: CLI-level fallback guidance can reintroduce blank-line noise even when lower-level renderers are already structured correctly.
- Correction: Trim the boundary between structured error bodies and injected footer guidance before inserting fallback `Next:` lines.
- Rule: When adding fallback guidance above structured command output, preserve section spacing intentionally and never rely on existing trailing blank lines.
- Pattern: ANSI-styled separator lines can survive later blank-line collapsing and still show up as visible gaps in rendered output.
- Correction: Remove unwanted separators at the formatter source instead of relying on downstream blank-line cleanup.
- Rule: If a rendered spacing bug survives post-processing, tighten the original formatter string first; blank-line collapse is not a reliable fix for ANSI-decorated output.
- Pattern: CLI error output can already contain a `Next:` block before command-level fallback guidance runs, which means an early return can preserve visible blank gaps.
- Correction: Normalize spacing before existing `Next:` / `Try:` lines even on the early-return path; do not assume only newly injected guidance needs tightening.
- Rule: Guidance-spacing cleanup must run for both existing and injected action lines, especially before `RUN SUMMARY` / `UP SUMMARY`.
- Pattern: A persistent container backend that still exists by name may no longer be running, and trying to `exec` into it leaks runtime-specific errors to users.
- Correction: Detect a stopped persistent container before reuse and recreate it rather than treating name existence as sufficient readiness.
- Rule: For persistent execution backends, verify liveness, not just existence, before reusing a target.
- Pattern: Detect drift rendered as raw dotted field paths makes task changes look like schema noise instead of actionable repo updates.
- Correction: Present task removals by task name with concrete removal actions, and reserve raw field paths for non-task fallback cases.
- Rule: In human drift output, group by the operator’s unit of change when one exists; prefer `Task <name>` over raw `tasks.<name>.*` paths.
- Pattern: Child action markers that share the same hue family as parent bullets flatten hierarchy and make grouped output harder to scan.
- Correction: Give nested action bullets a clearly distinct accent color from their parent group/task markers while preserving the overall palette.
- Rule: In rich CLI output, use color to reinforce structure; child markers should not visually blend into parent bullets.
- Pattern: A roll-up line that sits visually like a sibling section can compete with the content it is supposed to frame.
- Correction: Render drift impact as a lightweight block directly under the section title, not as a full section headline or inline sentence fighting with the real warning groups.
- Rule: Summary/impact metadata should frame the next block, not visually rival it.
- Pattern: Existing-contract detect previews become less useful when the inferred contract dump appears before the actual comparison and drift review.
- Correction: Lead existing-contract detect text output with comparison/drift, then show the inferred contract and annotations as supporting detail.
- Rule: In review-oriented CLI flows, show the delta before the payload.
- Pattern: Successful orchestration commands can feel noisy and low-quality if captured backend logs are dumped by default.
- Correction: Suppress successful phase output on the happy path and reserve detailed command/service logs for failures or explicit inspection surfaces.
- Rule: Default success output should confirm the outcome and next step, not replay backend chatter.
- Pattern: Default readiness output can still feel internal and heavy even after the most important findings are reordered to the top.
- Correction: Render execution and agent detail as compact summary blocks with strong visual hierarchy instead of YAML-shaped detail dumps.
- Rule: In default human output, prefer compact operator summaries over schema-shaped blocks unless the extra structure is needed for actionability.
- Pattern: Raw task logs that are acceptable in an interactive terminal become noisy and low-trust when the same command runs in a captured or non-interactive context.
- Correction: Keep live streaming for interactive repo task runs, but buffer non-interactive output into a bounded excerpt with an explicit `--stream` escape hatch.
- Rule: When a command can be both interactive and captured, optimize the default presentation for the active context instead of forcing one output mode everywhere.
- Pattern: Premium sibling commands drift apart when some still end at a bare status while others close with obvious next steps.
- Correction: Give successful low-noise commands like `validate` explicit next actions, and reuse the same concise section naming across adjacent commands like `explain`.
- Rule: Core first-contact commands should not end in dead air; successful output should still point to the next useful Ota action.
- Pattern: Reusing a shared findings renderer across repo and workspace commands can silently reintroduce verbose `Why:` lines and spacing bugs if the concise/section rules are not carried over too.
- Correction: When promoting shared CLI renderers, verify concise-mode behavior, primary-blocker spacing, and single-finding formatting on every consumer, not just the original command.
- Rule: Shared output helpers must preserve the UX contract of every command that adopts them, especially concise-mode omission rules and section separators.
- Pattern: Adapter bootstrap failures can get lost if the backend shell error is allowed to fall through to a later setup phase.
- Correction: Stop at the bootstrap boundary, insert a first-class bootstrap failure finding, and keep the raw backend stderr underneath it.
- Rule: When bootstrap is the selected recovery path, it owns the failure report until it either succeeds or fails explicitly.
- Pattern: A missing adapter command can look like an ordinary provisioning command failure if the backend does not translate the shell symptom into a semantic missing-command error.
- Correction: Detect the missing adapter command at the backend boundary and return `MissingCommand` so the approved bootstrap path can run and report the real bootstrap failure if it still fails.
- Rule: When a backend depends on an adapter command, translate `command not found` into a semantic missing-command error before falling back to higher-level provisioning handling.
- Pattern: Bootstrap findings become misleading when they hardcode guessed prerequisite failures instead of reflecting the actual bootstrap stderr.
- Correction: Derive bootstrap `Why:` and prerequisite `Next:` text from the real bootstrap stderr when possible, and fall back to generic wording only when the stderr provides no concrete signal.
- Rule: Premium failure output must be evidence-led; do not state a bootstrap root cause unless the backend output actually supports it.
- Pattern: Adapter bootstrap lookup can silently fail if policy is queried with the raw missing executable (`sdk`) instead of the provisioning source (`sdkman`).
- Correction: Derive bootstrap candidates from provisioning request sources and only fall back to the raw missing command when the request has no source information.
- Rule: Bootstrap policy resolution belongs to adapter/source semantics, not shell command names.
- Pattern: Fixing one backend's installability failure in isolation creates uneven trust if other adapters can fail the same way but still fall back to generic stderr.
- Correction: When a provisioning failure is classified into a user-facing root cause, define the classification model once and apply it across all adapters that can surface the same failure class.
- Rule: Backend-specific trust fixes must be designed as shared taxonomy, not one-off backend patches, unless the product explicitly scopes the behavior to a single adapter family.
- Pattern: Global cleanup can silently lie if backend discovery treats a failed `docker ps` / `podman ps` query as “no stale containers found”.
- Correction: Surface stale-clean backend query failures as command errors with the real engine output instead of collapsing them into an empty result.
- Rule: Cleanup discovery must fail closed; when ota cannot inspect ownership safely, it must report the query failure, not success.
- Pattern: Fake container engines in tests often call real system tools like `dirname`, `cat`, `grep`, and `rm`; truncating `PATH` to only the fake engine bin dir makes those helper calls fail and produces false negatives.
- Correction: When a fake engine script depends on external shell tools, prepend the fake bin dir to the existing `PATH` instead of replacing it.
- Rule: Test harnesses for fake container engines should preserve a usable system `PATH` unless the test explicitly stubs every helper command the script invokes.
- Pattern: Global stale cleanup can fail unnecessarily if one available engine is down even though another available engine can still report and remove stale ota containers.
- Correction: Treat `ota clean --stale` as best-effort across accessible engines; only fail when no engine can be queried or stale removal itself fails.
- Rule: Global stale cleanup should prefer partial success over total failure when at least one engine can still be queried and cleaned safely.
- Pattern: Coarse per-engine progress lines can become noisy when the command already has an interactive spinner and the user did not ask for stream output.
- Correction: Keep the spinner for interactive stale cleanup, but do not stream engine-scan lines by default.
- Rule: Default interactive progress should be lightweight; reserve explicit streaming text for commands that already expose `--stream` or similar.
- Pattern: Backend-specific diagnosis work can drift into a one-adapter implementation if the command layer is patched before the shared provisioning taxonomy is finished.
- Correction: Put the generic diagnosed-failure model in the provisioning layer first, then let command and doctor surfaces consume it; backend-specific subtype detail should hang off that shared path instead of replacing it.
- Rule: When a premium failure surface is meant to apply across adapters, the canonical boundary must be generic before any adapter-specific wording is added.
- Pattern: Public docs that enumerate the currently covered backends drift quickly once parity work continues adapter by adapter.
- Correction: Once the implementation truly covers the shipped adapter family, describe the capability at the family level and only call out backend-specific nuance where it materially differs.
- Rule: Prefer durable capability wording like "all shipped mutating provisioning adapters" over long backend lists once parity is real.

## 2026-04-18

- Pattern: Verifying hook reruns at the parent task only is not enough when the real side effect lives in a dependency like `setup`.
- Correction: Treat hook reruns as fresh execution subtrees and verify the exact dependency path that produces the user-visible artifact, not just the hook task itself.
- Rule: For task outcome hooks, validate the whole rerun subtree the user depends on; a rerun is incomplete if its stale dependency side effects still stay cached.
- Pattern: Repo-run parser and validator changes can look correct on a root contract while still regressing monorepo member runs or turning existing contracts into silent breaking changes.
- Correction: When `ota run` syntax depends on the effective contract shape, test root and `--member` resolution paths, and pair any new validation rejection with migration guidance in the error, docs, and changelog.
- Rule: CLI and validation changes are not complete until member-scoped runs and contract-migration surfaces are covered alongside the main happy path.

## 2026-05-08

- Pattern: User-visible Ota feature work can drift if `ota` code/spec changes land without the matching `ota-site` public docs update.
- Correction: Carry `ota-site` alongside every user-visible contract, command, or behavior change in `ota`, updating the relevant public docs/changelog in the same workstream.
- Rule: When a feature changes the shipped Ota surface, update `ota-site` in the same task by default; do not wait to be asked.

## 2026-05-15

- Pattern: A product-direction discussion about CLI shape can be mistaken for approval to implement the command immediately.
- Correction: Treat strategic agreement as direction only; wait for an explicit implementation request before changing CLI code.
- Rule: Do not implement a new command from design discussion alone. First confirm the user has asked for code changes, then keep edits to the requested surface.

## 2026-07-28

- Pattern: A provider-enforcement plan can accidentally place provider-relative support states inside canonical policy or let runner defaults behave like hidden policy.
- Correction: Keep canonical requirements, explicit restriction overlays, effective policy, provider capability, and witnessed application as separately identified objects.
- Rule: Capability describes what a provider can enforce; only explicit policy authority may narrow execution, and observed application never becomes declaration.
- Pattern: A start-time application attestation can overclaim enforcement across the complete execution.
- Correction: Bind execution to one boundary lease and require immutable-lifetime evidence or terminal reinspection after every completion and interruption path.
- Rule: Completed enforcement requires authenticated authorship and terminal boundary evidence, not only a preflight success flag or matching hash.
- Pattern: A closure-wide sandbox policy can erase legitimate phase differences, and an optional provider flag can become an enforcement bypass.
- Correction: Preserve an ordered per-step policy graph under the closure identity and resolve an enforcing target automatically or refuse whenever authoritative runtime controls apply.
- Rule: Never flatten differing phase boundaries, infer policy from provider defaults, or let omission of an execution flag weaken declared enforcement.
- Pattern: A policy-aware lower-level runner can appear complete while the production command's Doctor gate refuses earlier and drops the typed admission evidence.
- Correction: Capture one command-scoped observation set before admission, pass its policy projection through Doctor and execution, and emit refusal evidence from the production boundary rather than reconstructing it through a generic receipt command.
- Rule: Test policy refusals through the real top-level command path; generic readiness receipts must never stand in for admission-produced execution or refusal receipts.
- Pattern: A policy decision can be known before native provisioning but still be enforced after provisioning, and an unavailable observation can accidentally count as complete coverage.
- Correction: Order replay admission as observation, hard-pin enforcement, policy enforcement, then ordinary prerequisites, and treat unavailable or unknown observation states as unconditional denial.
- Rule: No governed replay lane may mutate the host before admission or turn missing preflight evidence into an allow decision.
- Pattern: Replay-input admission can be complete on ordinary run/up paths while proof wrappers or aggregate Doctor output bypass or discard the same policy truth.
- Correction: Treat every command that can execute tasks, start services, create proof artifacts, or aggregate member diagnosis as a consumer of the canonical command-scoped replay preflight.
- Rule: Proof admission must precede every parent and child side effect, and aggregate machine output must retain each member's canonical policy record.
- Pattern: Agent readiness can report an invisible warning when verdict derivation treats an intentionally empty writable set as a missing boundary despite declared protected paths.
- Correction: Derive readiness from the same filesystem-boundary semantics used by enforcement: either writable or protected paths establishes a boundary, while neither remains risky.
- Rule: Every risky Doctor verdict must correspond to actionable visible evidence, and summary derivation must not drift from the runner's boundary model.
- Pattern: Converting an active policy-load error to `None` makes broken authority indistinguishable from absent authority, while dependency-only admission lets outcome hooks execute outside governance.
- Correction: Retain policy-load failure as typed fail-closed preflight evidence and derive replay admission from the recursive execution closure, including every outcome-hook edge.
- Rule: Policy authority must never disappear on load failure, and admission must cover every task the runner can execute before the first process starts.
- Pattern: Reusing a replay-policy evaluation while Doctor, provisioning, effect findings, or receipts reload the active policy can combine two authorities inside one command; projecting a dependency-only closure can also omit executable hooks already governed by admission.
- Correction: Retain the loaded policy pack in the command preflight, pass that exact snapshot through every downstream policy consumer, and derive CI projection closure from the same recursive execution-closure helper as admission.
- Rule: One command consumes one policy snapshot, and every authoritative projected closure must enumerate every task the governed runner can execute.
- Pattern: Unifying replay-policy consumers is still incomplete when agent admission or claim assurance reloads the same org authority independently, especially across a detached proof child.
- Correction: Load the complete org-policy pack before any admission decision, pass it through every governance domain, and give internal child execution a private snapshot or explicit absence marker.
- Rule: Command-scoped authority applies across policy domains and process boundaries; no later consumer may rediscover policy from ambient state.
- Pattern: A mutating diagnosis command can preserve one policy snapshot during its fix decision but accidentally reload authority when refreshing post-fix observations.
- Correction: After deterministic fixes, refresh repository observations and re-evaluate them against the original loaded policy and load error; never reconstruct the complete preflight.
- Rule: One-command policy authority remains immutable across command-owned mutations, including post-fix diagnosis.
- Pattern: Sharing one policy snapshot does not guarantee safe ordering if a mutating Doctor fix runs after that snapshot has already produced a refusal.
- Correction: Admit the full selected replay-input closure before any repo-hygiene write or tool activation, and return a zero-applied typed fix summary on refusal.
- Rule: Every mutating execution surface, including repair commands, must enforce command-scoped policy and hard-pin admission before its first side effect.
- Pattern: A pressure workflow can be proven on a fork branch and then accidentally carry that branch-specific push filter or manual branch-source override into an upstream integration PR.
- Correction: Before opening an upstream-facing PR, audit every active contract, workflow, and public instruction for pressure branches; replace push filters with the repository's real default branch and remove unreleased source selectors when the contract pins a released Ota version.
- Rule: Fork branches are temporary evidence sources, never durable workflow or bootstrap authority; shipped governance must run after merge on the upstream default branch and consume contract-owned release truth.
- Pattern: Pinning Ota's setup Action while leaving other changed workflow Actions on moving major tags or branches preserves a supply-chain drift path.
- Correction: Resolve every Action reference introduced or modified by an upstream governance PR to its current full commit SHA and retain the human-readable release line as a comment.
- Rule: Upstream-facing Ota governance changes must not add mutable GitHub Action references; full commit identities are the execution authority.

## 2026-07-31

- Pattern: An agent can validate an agent-safe Ota task through the ordinary human lane and thereby
  skip the exact admission posture the contract advertises for agents.
- Correction: Run every agent-safe `ota run` and `ota up` validation with `--agent`; use the human
  lane only when the task is explicitly review-required and the user has approved that execution.
- Rule: Codex must preserve its actor mode when invoking Ota. Agent-safe validation means
  `--agent`, not merely a task that would also run successfully for a human.

## 2026-08-11

- Pattern: A later interactive task can transfer terminal ownership backward into typed hydration
  or bootstrap, silently removing Ota's canonical loader from runner-owned preparation phases.
- Correction: Resolve interaction per executed phase: typed preparation remains noninteractive and
  loader-owned, while only the actual interactive command inherits the terminal.
- Rule: Every runner-owned silent phase must retain Ota's loader and active task label; terminal
  passthrough is an explicit exception for the command currently owning the user's terminal.
- Pattern: Treating a task name as long-running service ownership blocks disjoint native/container
  listeners while missing collisions between differently named tasks that bind the same endpoint.
- Correction: Derive active ownership from the complete executable closure, effective runtime
  namespace, projected listener, and normalized write-path ancestry; preserve unknown legacy or
  unresolved ownership as fail-closed.
- Rule: Execution conflicts follow actual shared resources, not task identity. Explicit host-port
  remaps and isolated storage namespaces must permit coexistence only when Ota can prove separation.
- Pattern: Treating `--host-port` as container-only makes a resource-shaped execution option vary
  unnecessarily by backend and prevents truthful concurrent native/container development.
- Correction: Define the flag around the selected host-facing listener. Remap only publication for
  containers and Compose, but update both bind and projection for direct native execution where no
  publication boundary exists.
- Rule: Execution-option semantics should be uniform across capable backends, while backend-specific
  mechanics remain explicit and fail closed when Ota cannot project the requested resource.
- Pattern: Improving one conflict diagnosis can accidentally redesign the established run summary
  instead of adding the minimum evidence needed to explain the refusal.
- Correction: Preserve the canonical summary order and mode-specific fields; add only reason and
  resource rows at the existing ownership boundary, while keeping detailed remediation above it.
- Rule: Error-specific UX may enrich the shared summary, but it must not replace or reorder the
  stable summary contract.

## 2026-08-25

- Pattern: Canonical semantic identities can still split when validation accepts Unicode aliases or
  silently normalizes authored path aliases before hashing.
- Correction: Give every identity-bearing string a versioned byte profile and require authored
  paths to equal their canonical reconstruction before identity derivation.
- Rule: Identity validation must reject alternate byte representations at admission; canonical
  hashing cannot repair an ambiguous input domain after the fact.
- Pattern: A representative review matrix can pass while missing identity-bearing fields, Unicode
  byte aliases, path aliases erased by platform helpers, generated metadata overrides, or an exact
  consumer-version refusal.
- Correction: For identity-sensitive surfaces, enumerate every semantic input and domain, mutate
  each independently, test alternate byte representations, inspect generated/public metadata, and
  run the exact pinned consumer without lowering its declared minimum version.
- Rule: Do not call an identity boundary reviewed from representative controls alone; require an
  exhaustive input/domain matrix plus exact-version cross-repository validation or name the latter
  as an explicit open gate.
- Pattern: A detailed roadmap can make optional provider and platform carriers look mandatory even
  when no operator has requested or can pressure the exact boundary.
- Correction: Use signal-rich public repositories for V12 through V12.2 technical discovery, but
  require documented operator or design-partner demand plus immutable native pressure before
  activating provider-attested, macOS, or Windows carriers.
- Rule: Technical pressure may select general OSS semantics; expensive optional carriers remain
  inactive and are formally deferred when exact provider/platform adoption demand is absent.
- Pattern: A pre-side-effect refusal added only to `ota run` can leave `ota up` and proof-owned
  `up` execution free to mutate setup state before reaching the same execution-disabled action.
- Correction: Put one read-only closure verifier before every real repo execution entrypoint,
  preserve dry-run as preview, and return the refusal through each command's canonical text and
  JSON result envelope.
- Rule: Trust-boundary ordering is an entrypoint matrix, not a single-call-site property; test
  direct run, up orchestration, inherited proof paths, side-effect absence, and output-schema parity.
- Pattern: A large inline pressure-workflow shell block can repeatedly fail on fixture and assertion
  defects while exposing only one undifferentiated step, even when the product boundary is sound.
- Correction: Keep the exact pressure carrier in a locally runnable checked-in script, derive
  adversarial fixtures from pristine inputs, and persist the last completed and failed named stage.
- Rule: Hosted pressure must use the same stage-reporting carrier exercised locally; workflow YAML
  should orchestrate the carrier and retain evidence rather than contain the trust test itself.
- Pattern: A producer regression can pass while its published schema accepts action-specific
  contradictions or its returned acknowledgement identities remain unasserted.
- Correction: For identity-bearing output, enumerate each discriminated branch in both producer and
  schema tests, mirror runtime normalization rules including Unicode boundary whitespace, add
  isolated contradictory fixtures, and assert every returned identity against its independently
  derived subject.
- Rule: Trust-sensitive output is complete only when producer behavior, local schema constraints,
  and returned identity bindings are independently locked.

## 2026-08-27

- Pattern: After closing and pressure-proving a feature slice, the next-step recommendation can jump
  directly to the next acceptance feature while forgetting an already agreed behavior-neutral
  extraction at the boundary between batches.
- Correction: Before naming the next implementation slice, reconcile the active plan, current
  handoff, and recorded architecture sequencing decisions, including any extraction deliberately
  deferred until the preceding pressure gate completed.
- Rule: Every "what is next" recommendation must include pending between-batch architecture work;
  complete narrow hotspot extraction before adding another feature to the same orchestration path.

## 2026-08-28

- Pattern: Focused command regressions can pass while a checked-in hosted pressure carrier still
  asserts an older artifact lifecycle, causing every hosted platform to fail after a correct
  pre-side-effect boundary moves earlier.
- Correction: After changing trust-boundary ordering, run the exact checked-in pressure script
  locally and update its assertions to the stable machine refusal plus required artifact presence
  or absence, rather than legacy human text or files the stronger boundary intentionally removed.
- Rule: A hosted pressure workflow is part of the acceptance surface; its local carrier must pass
  against the exact release binary before the implementing revision is pushed.

## 2026-08-30

- Pattern: Treating retained pressure evidence as generic editorial cards buries provenance,
  exercised scope, limits, and the primary artifact link in one undifferentiated content block.
- Correction: Reuse the established schema-card composition with an evidence-specific variant:
  provenance in the eyebrow, one proven fact as the headline body, an explicitly labeled evidence
  boundary, and a direct hosted-evidence action. Use masonry for uneven limit text and generate
  every card from the Core-owned registry.
- Rule: Public pressure cards must optimize for evidence navigation and bounded interpretation, not
  promotional summarization; visual variants may change semantics and labels, never duplicate the
  canonical evidence source.

## 2026-09-01

- Pattern: A first-party Site task was run through direct package commands and an isolated worktree,
  bypassing the repository's declared Ota task surface and leaving the user unable to inspect the
  intended active-worktree result.
- Correction: Work in the active first-party repository unless isolation is explicitly required.
  Before validation, generation, preview, or launch, run `ota tasks --use` and execute the
  declared task through `ota run` or `ota up`. Treat direct package commands as read-only
  diagnostics only, never as authoritative validation or launch.
- Rule: For Core, Site, Skills, and Examples, the local `ota.yaml` is the operational authority.
  State the declared Ota task used in every completion summary; do not substitute `npm`, `cargo`,
  or another tool command when a contract-owned task exists.

## 2026-09-02

- Pattern: An unrelated Core Release Gate was monitored and narrated inside an Eris design-partner
  pressure closeout, obscuring which evidence and blockers belonged to the external repository.
- Correction: Keep each pressure repository bound only to its own source revision, workflow run,
  artifacts, contract, findings, and explicit limits. Track Core feature or release gates as a
  separate workstream even when the pressure work helped motivate them.
- Rule: Never make an external pressure-repository status depend on an unrelated Ota Release Gate;
  connect them only when the pressure contract pins that exact Core revision as an explicit gate.

## 2026-09-03

- Pattern: The canonical Skill required material task notes and directed agents to
  `ota tasks --safe --use`, but that exact text renderer hid task notes while ordinary task output
  and JSON retained them. Deferring the renderer mismatch would leave the canonical agent workflow
  unable to consume the guidance it requires.
- Correction: Repair the canonical discovery surface immediately when contract truth already exists
  and the omission prevents agents from seeing a declared boundary; preserve JSON compatibility and
  lock rich/plain text parity with command-level regressions.
- Rule: Do not defer a bounded visibility defect on the canonical agent path as optional UX work.
  If the Skill requires agents to consult a field before execution, the named discovery command must
  expose that field in the same release slice.
- Pattern: A design-partner PR description included Ota's internal product-gap and roadmap ledger,
  even though the upstream maintainer only needed the proposed integration, exact evidence,
  unchanged repository findings, and proof limits.
- Correction: Keep partner PR descriptions scoped to what the maintainer must review and decide.
  Record Ota platform gaps, widening opportunities, and future ownership in Ota-owned plans,
  handoffs, pressure records, and engineering notes instead.
- Rule: Upstream PRs are review artifacts, not Ota planning ledgers. Include only information that
  explains the diff, its evidence, its repository impact, or its explicit limitations.
- Pattern: A reference-fixture reliability repair was applied in Core while the identical fixture
  in the standalone Examples repository remained stale.
- Correction: Treat duplicated reference fixtures as one propagation surface. After changing one,
  compare and update every owned copy, then assess Site, Skills, Learn, FAQ, and Glossary for
  connected changes and record unaffected decisions explicitly.
- Rule: Never declare an Ota or reference-example change complete until Core and standalone
  Examples copies are reconciled and all first-party consumers have an explicit impact decision.
