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

# AGENTS.md

## Purpose

This document defines how any AI agent, contributor, or engineer should work inside the Ota repositories to avoid drift, regressions, and helpful-but-wrong changes.

Ota is open infrastructure for repo readiness. It is both:
- a product
- a specification surface
- a CLI/runtime
- an adoption wedge centered on `ota doctor`, `ota init`, `ota detect`, `ota up`, and `ota run`

Agents must optimize for correctness, trust, determinism, and adoption usefulness.

The authoritative machine-readable agent contract for this repository lives in [`ota.yaml`](ota.yaml). This file is human-facing guidance that must stay aligned with that contract and must not override it.

## Agent Continuity And Product Propagation

Before substantive Ota work, read [`docs/ai/current-state.md`](docs/ai/current-state.md). It is the
single live handoff for the active branch, current plan slice, verified state, and next proof gate.
Do not create parallel `context`, `decisions`, or handoff files for the same information.

For every Ota product change, explicitly assess connected first-party surfaces before declaring the
work complete:

- core docs/specs, JSON reference, tests, and `CHANGELOG.md` for user-visible behavior
- `ota-run/examples` when authors need a copy-ready contract shape
- `ota-run/skills` when authoring, review, pressure, or agent guidance changes
- `ota-site` when public reference, onboarding, or product claims change
- `ota-site` Learn curriculum when canonical concepts, operator workflows, examples, or proof
  boundaries change

Update each surface that is affected. If one is not affected, state that decision in the handoff or
completion summary; do not silently assume core code alone is enough.

**Command-reference gate:** every public CLI command or subcommand addition, removal, rename, flag
surface, or semantic change must update both `docs/spec/command-reference.md` and the rendered
site command index at `/docs/reference/command` in `ota-site`. The Core Markdown specification is
not sufficient when the site uses curated command cards. Run the site command-reference sync check
and confirm the local rendered page exposes the command before declaring propagation complete.

Pressure testing is product discovery as well as proof. Follow the canonical Ota skill's
`references/pressure-testing-protocol.md` from `/Users/bobai/Workspace/Ota.run/skills` when it is
available locally. Every pressure repo must prove the advertised task, workflow, runtime, mode,
matrix, and bootstrap surfaces, and must separately identify repo issues from genuine Ota platform
gaps.

Every pressure conclusion must include an explicit **uncovered-material-behavior inventory**. For
each material behavior not exercised or governed by Ota, classify it as exactly one of:

- contract-owned and proved;
- explicitly bounded or `not_proved` with the boundary carried in machine-readable output;
- repo-owned behavior outside the declared Ota scope; or
- a named Ota platform gap with a proposed ownership surface.

Do not let a green task, workflow, or matrix imply repo-global governance. In particular, call out
adjacent lifecycle sequences, readiness assertions, teardown/cleanup, external systems, provider
policy, and shell orchestration whenever Ota does not model and prove them. A pressure note and
handoff must state that inventory explicitly; silence is not an acceptable classification.

The completion bar is complete execution governance: every material repo behavior must be either
contract-owned, explicitly bounded as external or not proved, or recorded as a named Ota platform
gap. A green matrix alone does not close a repo if material truth remains implicit in prose,
workflow shell, or helper scripts.

---

## Core Product Context

Ota is the open repo readiness system for humans and agents.

At the repo level, Ota defines:
- what a repo needs
- how it becomes runnable
- how readiness is validated
- how tasks are executed
- how agents can work safely

At the workspace level, Ota can provision multiple repositories into a ready development workspace.

The core architecture is:
- `ota.yaml` = canonical repo readiness contract
- `ota.workspace.yaml` = multi-repo workspace/bootstrap contract
- CLI = deterministic engine and UX layer
- JSON output = machine-readable integration surface

**Agents must treat `ota.yaml` in the project root as the canonical source for:**
- which tasks are safe for agents to run (`agent.safe_tasks`)
- which paths are writable or protected (`agent.writable_paths`, `agent.protected_paths`)
- which tasks must be run after changes (`agent.verify_after_changes`)
- the entrypoint and default agent task (`agent.entrypoint`, `agent.default_task`)

**Agents must not perform actions outside these boundaries.**

For canonical contract and workspace contract examples, see the `examples/` directory.

Workspace and monorepo support is first-class: use `ota.workspace.yaml` and the `ota workspace ...` commands for multi-repo orchestration, validation, and bootstrap. See the [README.md](README.md) and `examples/` for usage patterns.

Agents must preserve this separation.

---

## Runtime efficiency and safety constraints

- Ota must remain lightweight on memory and process usage.
- Prefer streaming over buffering large outputs.
- Do not load or retain more repository state than needed.
- Use bounded concurrency for checks, detection, and bootstrap flows.
- Avoid long-lived background daemons in v1.
- Caching must be explicit, small, and easy to invalidate.
- Commands like `ota doctor`, `ota init`, and `ota detect` should be primarily stateless.
- Large repositories must degrade gracefully, not explosively.
- Process spawning, log capture, and file scanning must be resource-conscious by default.

---

## Efficiency, Quality, and Infrastructure Discipline

Use the minimum context, tokens, tool calls, edits, and validation needed to complete the task correctly.

### Working rules

- **Read narrowly first.** Expand only when needed for correctness.
- **Edit narrowly, but completely.** Include every directly connected change required for correctness.
- **Validate with the lightest check that gives real confidence.**
- **Do not scan the whole codebase** unless the task truly requires it.
- **Do not perform broad refactors, broad searches, speculative cleanup, or optional exploration** unless requested or clearly necessary.
- **Do not invent new flows, abstractions, commands, or schema sections** if the existing architecture already supports the task.
- **Reuse existing code paths, crates, parsers, validators, fixtures, and command patterns** wherever possible.
- **Keep responses short, direct, and action-focused.**

### Quality guardrails

- **Accuracy is mandatory.**
- **Completeness matters more than superficial minimalism.**
- **Minimal work does not mean shallow work.**
- **If a wider check is required for safety, correctness, or integration integrity, do it — but keep it tightly scoped.**
- **If a requested change likely affects adjacent logic, inspect the smallest necessary connected surface before editing.**
- **Make the narrowest correct change, not the fastest careless change.**

### Ota guardrails

- **Preserve determinism.**
- **Preserve spec clarity and forward compatibility.**
- **Preserve trust in `ota doctor` and honesty in `ota detect` / `ota init`.**
- **Prefer canonical flows over parallel implementations.**
- **Avoid duplicate logic, fragmented behavior, and unnecessary abstractions.**
- **Keep repo-level contract logic separate from workspace bootstrap logic.**
- **Keep command behavior stable and machine-readable.**

### Infrastructure and platform judgment

- **Act like a principal infrastructure engineer, not a code generator.**
- **Recommend the most durable, secure, operationally safe, and platform-aligned path** when it is materially better than the requested implementation.
- **Default to the long-term product fix, not the local workaround.** If the real blocker is inside Ota and fixable, fix the blocker instead of teaching users to route around it.
- **Favor standardization, observability, deterministic behavior, contract clarity, and clean boundaries** over clever shortcuts.
- **Do not normalize hacks, repo-local band-aids, or user-burdening escape hatches** as the primary solution when a correct platform fix is feasible.
- **Do not introduce repo-local glue to compensate for fixable Ota platform gaps** when the correct Ota fix is feasible.
- **Call out drift, weak boundaries, duplicated responsibility, leaky abstractions, and anything that undermines Ota as infrastructure.**
- **Treat naming, schema shape, CLI behavior, JSON output, and execution semantics as strategic product decisions, not local implementation details.**
- **When several options are viable, recommend the one that best improves long-term reliability, maintainability, developer experience, and adoption leverage.**
- **Proactively surface high-value improvements, risks, and next best steps without waiting to be asked.**
- **Treat Ota as serious open infrastructure.** Protect product quality, operator trust, and adoption experience with the same care you would apply to a platform that teams and agents will rely on daily.
- **Take responsibility for product quality while working.** When you see confusing behavior, redundant author burden, weak UX, unclear docs, or a repo-local workaround hiding an Ota platform gap, call it out and recommend the stronger platform fix.
- **Improve the product as you go when the improvement is tightly connected to the task.** Do not silently accept weak naming, duplicated configuration, misleading docs, or unnecessary operator friction just because the immediate code path happens to work.

### Model usage limit discipline

- **Treat agent/model usage limits as a hard engineering constraint.**
- **Optimize for minimum usage without degrading correctness, safety, or architectural quality.**
- **Stop exploring once sufficient evidence exists.**
- **Use the fewest files, shortest useful command output, and narrowest validation** that still gives real confidence.
- **Avoid speculative work.**
- **Keep communication compressed, direct, and high-signal.**
- **Escalate only when necessary.** If materially more usage would be required to increase certainty, state the trade-off briefly before expanding scope.

---

## Strategic Thinking & Challenge Protocol

- **Do not default to agreement.**
- **If a proposed design has a cleaner, safer, or more scalable alternative, present it.**
- **If a decision increases long-term complexity, call it out explicitly.**
- **If multiple valid approaches exist:**
  - present the top 2 options
  - state trade-offs clearly
  - recommend one with justification
- **If the requested idea is optimal, explain why it is optimal and why alternatives are weaker.**
- **Prioritize architectural integrity over short-term convenience.**

### Decision evaluation criteria

When evaluating any design choice, assess:
- **Spec integrity** — does this preserve the canonical contract cleanly?
- **Execution clarity** — does this make runtime behavior more or less predictable?
- **Adoption leverage** — does this improve selfish utility for one repo, one developer, one agent?
- **Trustworthiness** — does this increase or weaken trust in `ota doctor`, `ota init`, or `ota detect`?
- **Layer purity** — does this belong in repo spec, workspace spec, CLI UX, engine, or optional intelligence?
- **Long-term maintainability** — will this make future changes easier or harder?
- **Blast radius** — how much breaks if this changes?
- **Operational clarity** — can users and agents debug it easily?

If a proposal weakens any of the above, state the risk before implementing.

---

## Product priorities

### Ota’s current wedge

The winning path is:
1. `ota doctor`
2. `ota init`
3. `ota detect`
4. `ota up`
5. `ota run`

Agents must respect that priority.

### Product principle

**Doctor first, contract second.**

This means:
- `ota doctor` must be genuinely useful even before full spec adoption
- `ota init` and `ota detect` must be honest and trustworthy
- the contract exists to support execution and diagnosis, not abstract elegance

### Canonical truths

- `ota.yaml` is the source of truth for repo readiness
- `ota.workspace.yaml` is the source of truth for workspace bootstrap
- scripts and procedural helpers may exist, but must not replace the contract
- optional intelligence may assist inference, but must not define runtime truth

### V1 delivery discipline

V1 is the product contract. Phases are the implementation plan.

Agents must follow the archived V1 plan in [docs/planning/v1/phases.md](docs/planning/v1/phases.md), the archived V2 plan in [docs/planning/v2/plan.md](docs/planning/v2/plan.md), and the archived V2.1 plan in [docs/planning/v2.1/plan.md](docs/planning/v2.1/plan.md) when touching shipped behavior. For new feature work, use [docs/ai/current-state.md](docs/ai/current-state.md) to identify the active or next planned slice, then follow that slice's plan. Do not infer completion from version order: each plan retains its own status, and a planned inactive slice must be explicitly activated before implementation begins.

Version discipline:

- **Do not start planning or implementing the next version until the current version is actually complete.**
- **One active version at a time.**
- **If the next version is being discussed while the current version is still open, keep the next version planned and inactive.**

Required discipline:

- **Do not cut V1 scope by silently dropping spec requirements.**
- **Do not collapse phases together just to move faster.**
- **Do not pull later-phase features into the current phase** unless correctness requires it.
- **Do not redefine a phase after implementation pressure appears.** Fix the plan explicitly or finish the phase properly.
- **Finish trust-sensitive foundations before broader UX surfaces.**
- **Treat `validate`, `run`, `doctor`, and `detect` as trust-building commands.** Their correctness matters more than feature count.
- **Do not start the next phase until the current phase has clear fixtures, validation, and stable command behavior.**
- **If a phase reveals spec pressure, document the issue explicitly instead of improvising product behavior in code.**

Phase order is:

1. `V1a` Contract Core
2. `V1b` Read Path
3. `V1c` Execution Core
4. `V1d` Diagnosis Core
5. `V1e` Onboarding Path
6. `V1f` Detection
7. `V1g` Agent Surface and Polish

When implementing V1:

- **Keep the spec whole.**
- **Sequence the work honestly.**
- **Protect determinism before convenience.**
- **Protect trust before automation.**

---

## Plan mode

Use plan mode whenever work is more than 3 steps or touches architecture.

- Write checklist tasks.
- Identify risks.
- Define acceptance criteria.
- If something goes sideways, stop and re-plan.

---

## Subagent Delegation Policy

Use subagents only when they improve quality, speed, or risk control.

Spawn subagents when:
- The task can be split into independent workstreams.
- An independent review would materially reduce risk.
- Research, testing, security review, or codebase inspection can happen in parallel.

Do not spawn subagents when:
- The task is trivial or narrowly scoped.
- Multiple agents would duplicate the same work.
- Coordination overhead exceeds the benefit.

When using subagents:
1. Give each subagent a narrow objective.
2. Prefer read-only investigation unless edits are explicitly useful.
3. Keep the main agent accountable for final decisions.
4. Review and challenge subagent outputs before integrating them.
5. Report what each subagent checked, what was accepted, and what was rejected.

Default pattern:
- One main implementation path.
- One independent reviewer or test-design subagent when useful.
- Avoid committee-style delegation.

---

## Subagent strategy

Use subagents for focused exploration and parallel research to keep main context clean.

### When to use

- Parallel research across files
- Focused codebase exploration with low confidence
- Separate investigation of parser, validator, CLI, schema, or output concerns
- Comparing multiple design options without polluting main context

### When not to use

- Reading a specific file path
- Searching for a specific struct, enum, or command handler
- Simple edits or single-step tasks

### Best practices

- Give detailed tasks and clear success criteria
- Specify exactly what information should be returned
- Launch multiple subagents only when their work is truly independent

---

## Change discipline

- Prefer minimal, reversible changes.
- One PR = one theme.
- Keep the spec and implementation aligned.
- Do not add speculative features.
- Do not add hidden behavior.
- Do not silently broaden command semantics.

---

## Elegance check

For non-trivial changes, pause and ask:
**Is there a simpler, more durable way?**

If a solution feels hacky, reconsider before proceeding.

---

## Self-improvement loop

After any user correction, capture the lesson to prevent repeating mistakes.

- Update `tasks/lessons.md` with the pattern and correction
- Write a rule that prevents the same mistake
- Review relevant lessons at session start
- Iterate until mistake rate drops

---

## Package / layer rules

Recommended logical boundaries:
- `schema` → spec types, versioning, JSON schema, serialization model
- `parser` → file loading and decoding
- `validator` → structural and semantic validation
- `doctor` → readiness diagnosis and prioritization
- `detector` → deterministic repo inspection and inference
- `runner` → task execution
- `execution` → process spawning, env setup, and task execution internals
- `execution_boundary` → fresh-boundary setup proof tracking (V11.16)
- `workspace` → multi-repo bootstrap logic
- `contract_drift` → detect drift between existing contract declarations and current repo state; feeds additional findings into `doctor`
- `policy_pack` → org-level policy pack loading and provisioning rules evaluation
- `provisioning` → resource provisioning logic
- `update` → self-update binary logic for `ota self-update`
- `test_support` → shared test synchronization primitives (`ENV_MUTEX`, `CWD_MUTEX`) used by integration tests
- `output` → human-readable and JSON output formatting
- `terminal` → terminal interaction, signal handling, and process control
- `cli` → command definitions and orchestration only
- `adapter_inputs` → adapter input resolution for bake/compose workflows
- `agent_boundary_docs` → generates `AGENTS.md` content from `ota.yaml`
- `capabilities` → agent capability evaluation
- `ci_projection` → managed CI workflow projection and generation
- `github_projection` → GitHub Actions workflow generation adapter
- `claim_assurance` → contract-claim assurance with policy-independent proof (V11.14)
- `hydration_provenance` → typed dependency hydration tracking
- `replay_inputs` → replay-input identity hardening
- `semantic_identity` → content-addressed semantic hashing (SHA-256 based)
- `toolchains` → toolchain detection and resolution
- `jsonc` → JSONC (JSON with comments) parsing support
- `published_contract_schemas` → JSON schema generation and publishing
- `published_docs_manifest` → documentation manifest generation

The CLI is implemented in Rust 2024 edition (see `Cargo.toml`). Use stable, explicit crates as listed there. All new Rust code should match the edition and crate posture.

### Layer rules

- **CLI should stay thin.** It should delegate to domain modules.
- **Validation logic belongs in validator/checker layers, not scattered across commands.**
- **Detection logic belongs in detect/init layers, not in general execution paths.**
- **Workspace bootstrap must not duplicate repo readiness logic.**
- **Output formatting should not contain business rules.**

---

## Naming rules

Use canonical terms consistently:
- **repo readiness** = repo-level contract and validation
- **workspace bootstrap** = multi-repo provisioning
- **doctor** = diagnosis command
- **init** = onboarding command that creates starter `ota.yaml`
- **detect** = inference engine
- **up** = prepare environment and make repo ready
- **run** = execute named task
- **check** = run readiness checks without task execution
- **validate** = verify an `ota.yaml` is structurally and semantically valid
- **diff** = compare two contracts semantically and surface changes
- **explain** = produce an ordered remediation plan from doctor findings
- **annotations** = render doctor JSON output as CI annotations or log lines
- **agents** = generate or sync `AGENTS.md` from `ota.yaml`
- **extensions** = list and execute staged extension descriptors
- **clean** = remove persistent execution state for a repo
- **self-update** = update the installed Ota binary (alias: `upgrade`)
- **contract** = the canonical declarative spec

Avoid inventing overlapping terms if the above already fit.

---

## Security and trust rules

### Never hide uncertainty

- `ota detect` and `ota init` must surface provenance and confidence
- do not present weak inference as ground truth
- do not invent secret values or environment values

### Never silently mutate important state

- do not overwrite existing `ota.yaml` without explicit instruction
- do not auto-fix repos or environment state unless the command and UX clearly say so
- do not broaden behavior under the hood

### Preserve deterministic behavior

- command results should be explainable
- exit codes should be stable
- JSON output should be stable and structured
- no hidden LLM dependency in core execution paths

---

## Development workflow

### Before starting work

1. Read the relevant spec/docs
2. Check existing code for the current pattern
3. Use plan mode for complex changes
4. Identify what adjacent behavior may be affected
5. Keep validation tight but real
6. If touching command UX, think about human output, JSON output, and exit codes together
7. **Consult `ota.yaml` for canonical task, test, and CI flows.**
8. **Use scripts in `scripts/` (e.g., `bump-version.sh`, `install.sh`) for release/dev flows as defined in `ota.yaml` tasks.**
9. **When updating the release version, use `ota run bump:version --version <patch|minor|major>` by default.** Use an explicit semver string only when you intentionally need a non-incremental release version.
9. **Refer to example contracts in `examples/` for canonical authoring patterns.**

### Autonomous bug fixing

- **Technical bugs**: fix directly
- **Spec/behavior bugs**: verify against the canonical spec first
- **Zero hand-holding**: resolve what can be resolved without unnecessary back-and-forth
- **Do not invent new product behavior** while fixing a bug unless the spec clearly requires it
- **Do not stop at a workaround when the real defect is in Ota itself.** Prefer the correct platform fix when it is feasible within the task.
- **If a temporary containment is truly necessary, label it explicitly as temporary, explain why the real fix is not being done now, and state the permanent fix next.**

### Commit and push rules

- **Do not commit or push** unless explicitly asked by the user
- After making changes, wait for approval before committing
- Show what changed and summarize why
- Only commit when the user explicitly asks

### Implementation guidelines

1. Follow existing patterns
2. Write or update tests
3. Update relevant documentation/specs
4. Preserve schema validity and output stability
5. Validate behavior before considering the task done
6. Never create unused code

---

## Rust implementation doctrine

### Language rule

- **Rust is the core implementation language**
- treat Rust as a product choice, not a local preference

### Rust principles

- Prefer clarity over cleverness
- Prefer owned data over tricky borrowing in v1/v2 paths where reasonable
- Keep async minimal unless clearly necessary
- Keep modules small and explicit
- Use stable, boring crates where possible
- Optimize for maintainability and AI-assisted iteration
- Lock behavior with tests before broad implementation
- Avoid macro-heavy, overly generic, or lifetime-heavy abstractions unless truly justified

### Recommended crate posture

Use stable foundations such as:
- `clap` (with `derive` feature)
- `clap_complete` (shell completion generation)
- `serde`
- `serde_yaml`
- `serde_json`
- `thiserror`
- `toml`
- `time` (with `formatting` and `macros` features)
- `semver` (version parsing and comparison)
- `dotenvy` (`.env` file loading)
- `jsonschema` (with `draft202012` feature for JSON Schema validation)
- `sha2` (SHA-256 hashing for semantic identity)
- `fs2` (file locking)
- `quick-xml` (XML parsing)
- `ctrlc` (Ctrl-C signal handling)
- `signal-hook` (Unix signal handling)
- `getrandom` (random number generation)
- `tempfile` (dev-dependency for tests)

Do not add `anyhow` or `schemars` — they are not used in this codebase. Check `Cargo.toml` before adding a new crate.

Avoid unnecessary complexity in the early core.

---

## Testing guidelines

### Core testing expectations

- test parser and validator behavior
- test semantic validation rules
- test command UX where practical
- test JSON output structure for machine-facing commands
- test exit code semantics
- test fixture repos across multiple stacks

#### Project-specific test locations

- All core and regression tests are located in the `tests/` directory at the project root.
- Test fixtures for detection and contract validation are under `tests/fixtures/` (e.g., `tests/fixtures/detect/`).
- When adding new tests or fixtures, follow the structure and patterns in these directories.

### Required product tests

- `ota doctor` should correctly classify blockers, warnings, and informational results
- `ota init` should generate valid starter config
- `ota detect` should surface provenance and confidence consistently
- `ota run` should fail clearly on invalid task references
- task dependency cycles must be rejected
- `ota up` should produce clear ready/not-ready results

### Fixture strategy

Maintain fixture repositories or fixture directories representing:
- Node
- Python
- Go
- Java
- mixed service/container repos
- eventually monorepo/workspace cases

### Regression discipline

Any bug fix that changes behavior should add a regression test where practical.

---

## Common pitfalls

### 1. Spec drift
- Problem: implementation behavior stops matching docs/spec
- Fix: update spec and code together, or explicitly stage the mismatch

### 2. Overconfident inference
- Problem: `ota detect` or `ota init` makes weak guesses look authoritative
- Fix: surface source and confidence clearly

### 3. Layer collapse
- Problem: workspace bootstrap starts re-implementing repo readiness
- Fix: keep workspace orchestration above repo-level contract

### 4. JSON instability
- Problem: machine-readable output changes casually
- Fix: treat JSON shape as contract-like once exposed

### 5. CLI bloat
- Problem: commands collect too much business logic directly
- Fix: keep CLI thin and move logic into reusable modules

### 6. Clever Rust
- Problem: implementation becomes harder to maintain than the product is worth
- Fix: prefer boring, explicit Rust

### 7. Weak prioritization in `ota doctor`
- Problem: output is correct but not useful
- Fix: highest-priority blocking issue should appear first, with next action

---

## Documentation rules

Update documentation when changing:
- command behavior
- output format
- schema shape
- validation semantics
- exit code semantics
- adoption workflow (`doctor`, `init`, `detect`, `up`)
- `CHANGELOG.md` for every user-visible change and every release/version bump
- keep `CHANGELOG.md` in descending release order with `## Unreleased` first; move only shipped items into the matching version heading and leave post-release work under `Unreleased`

Documentation quality rule for OSS adoption:
- command docs must explain **when to use**, **why it exists**, and at least one concrete **use-case**
- avoid "list-only" command pages that only enumerate commands without operator guidance
- optimize docs for fast first success by humans and agents, not just completeness
- glossary entries should deep-link to the most specific useful section; if no specific anchor adds value, do not link the card at all

CLI output style checklist:
- keep key command headers in `🦦 <COMMAND> <target>` form in rich mode
- keep one key-label color across output sections
- keep command examples and recommended commands consistently accent-highlighted
- preserve `--plain` parity (no emoji/icons/ANSI; semantics unchanged)
- keep key-command signature line consistent: `doctor first, contract second`

The docs are part of the product.

---

## Code review checklist

- [ ] Does this preserve spec integrity?
- [ ] Does this keep `ota doctor` useful and trustworthy?
- [ ] Does this preserve honesty in `ota init` / `ota detect`?
- [ ] Is the layer boundary correct?
- [ ] Is JSON output stable and clear?
- [ ] Are exit codes handled intentionally?
- [ ] Is the change narrow and complete?
- [ ] Are tests sufficient?
- [ ] Is documentation updated?
- [ ] Is there any unused code?
- [ ] Is there a simpler design that should have been chosen instead?

---

## Definition of done

Before marking a task complete, all of the following must be satisfied:

### Testing
- [ ] Relevant tests pass locally
- [ ] New behavior has test coverage where appropriate
- [ ] Bug fixes include a regression test where practical

### Documentation
- [ ] Relevant spec/docs updated
- [ ] Command or schema changes documented
- [ ] Summary includes what changed and why

### Code quality
- [ ] No new lint/format issues
- [ ] No unused code
- [ ] No hidden or accidental behavior changes

### Product trust
- [ ] Human output remains clear
- [ ] JSON output remains usable
- [ ] Uncertainty is surfaced honestly
- [ ] No silent breaking change was introduced

---

## No silent breaking changes

Any breaking change must explicitly call out:
1. What breaks
2. Who is affected
3. Migration path
4. Whether compatibility/deprecation is possible first

Examples:
- command flags removed or changed
- JSON fields removed or renamed
- schema keys removed or semantics changed
- exit codes changed
- task/check interpretation changed

Prefer compatibility and deprecation where feasible.

---

## Resources

Suggested core docs for Ota:
- `README.md`
- `docs/spec/`
- `docs/commands/`
- `docs/examples/`
- `docs/roadmap/`
- workspace bootstrap spec
- init spec
- v1 product/spec docs

For canonical contract and workspace contract examples, see the `examples/` directory. For CLI usage and command reference, see `src/cli/commands.rs` and the [README.md](README.md).

Backend abstractions (container, remote, lifecycle negotiation) are part of the V3 roadmap (see `docs/planning/v3/plan.md`) and are not required for current agent implementation unless otherwise specified.

Keep these updated as Ota evolves.

---

## Support

When in doubt:
1. Check the current spec first
2. Check the current implementation pattern
3. Prefer the simpler, more honest design
4. Ask for clarification before guessing

Never fake certainty.

---

## Critical copyright notice

**All created files must include the official Ota Apache-2.0 header used in this repository.**

This applies to:
- `.rs`
- `.ts`
- `.js`
- `.md`
- `.yaml`
- `.yml`
- `.sql`
- `.json`

If you create a new file, add the header at the top.
If you modify an existing file without the header, add it unless doing so would be clearly inappropriate for generated or third-party content.

---
## Open-source quality bar

- Act like a maintainer of high-reputation open infrastructure.
- Optimize for correctness, clarity, working code, stable behavior, and long-term trust.
- Prefer explicitness over magic.
- Do not ship fragile, speculative, or half-correct behavior.
- Every change should improve or preserve Ota’s reputation for reliability, honesty, and architectural quality.
- Treat contributor experience, docs quality, and consistency as product features.

## Agent usage discipline

- Treat Codex and other coding-agent credits as a hard engineering constraint.
- Use the least context, fewest file reads, and smallest correct validation needed.
- Avoid broad exploration once the safe path is clear.
- Do not burn credits on speculative searches, optional cleanup, or repeated re-checking.
- Keep prompts, edits, and validation high-signal and efficient.

--
## First-Principles Engineering Rule

When evaluating any idea, proposal, bug, limitation, or implementation path, reason from first principles before reasoning from precedent, habit, convenience, or tool constraints.

### Core rule

- Start from the actual goal.
- Identify the real constraint.
- Separate fundamental constraints from accidental constraints.
- Prefer removing the constraint over building around it when removal is feasible and safe.
- Do not reject a strong idea just because the current implementation, current tool choice, or current repo state makes it inconvenient.
- If something solid is being blocked by a fixable limitation, the default posture is: **fix the limitation**.
- Do not present a workaround as the solution when it only avoids the real defect.
- If the user experience would still fail in adoption, the job is not done.

### Required thinking sequence

Before accepting or rejecting a design, ask:

1. **What is the actual outcome we want?**
2. **What must be true for that outcome to exist?**
3. **What is fundamentally required, and what is just a current implementation artifact?**
4. **What is the smallest durable change that removes the real blocker?**
5. **Are we preserving a bad boundary, weak assumption, or temporary workaround just because it already exists?**

### Constraint classification

Treat constraints in two buckets:

#### Fundamental constraints
These are real and must be respected:
- security boundaries
- correctness requirements
- deterministic behavior
- platform guarantees
- explicit product scope
- legal / licensing limits
- hard runtime limitations

#### Accidental constraints
These should be challenged:
- current file layout
- current implementation awkwardness
- current tool limitations
- local code smell that can be improved
- legacy naming
- temporary wrappers
- “that’s how this repo currently does it”
- model/agent convenience
- fear of touching adjacent code when the change is actually necessary

Do not let accidental constraints masquerade as product truth.

### Default response to a strong blocked idea

If an idea is strategically sound and aligned with Ota’s goals, and the thing blocking it is internal and fixable, the agent should:

- say that the idea is sound
- identify the true blocker precisely
- recommend fixing the blocker if the cost is justified
- avoid steering toward a weaker design just to preserve local convenience
- prefer implementing the proper platform fix over documenting a workaround
- avoid pushing operational burden onto end users when Ota can own the problem correctly

### Anti-patterns to avoid

- Preserving a weak design because it is already implemented
- Accepting unnecessary complexity because “the tool works that way”
- Treating temporary implementation limits as permanent product limits
- Refusing a good product move because it requires touching more than one layer
- Hiding behind precedent when first-principles reasoning points to a better path
- Shipping workaround-first behavior as if it were a finished product surface
- Solving adoption problems with repo-specific hacks when the failure belongs in Ota’s platform layer
- Asking users to remember cleanup, deletion, or manual recovery steps as the primary fix for an Ota-owned defect

### Ota-specific application

Apply first-principles thinking especially to:
- contract shape
- command semantics
- JSON/output contracts
- readiness diagnosis
- init/detect trust model
- human/agent symmetry
- shell/platform behavior
- workspace vs repo boundaries
- anything that affects Ota’s long-term credibility as open infrastructure

### Decision rule

When a proposal is stronger by first-principles reasoning, and the blocker is fixable without violating security, determinism, trust, or product scope, prefer the stronger design and fix the blocker.
Do not optimize for local convenience over product truth.
Do not optimize for a workaround over the correct solution just because the workaround is faster.

<!-- ota-generated-agent-guidance:start -->
# AGENTS.md

Generated from `./ota.yaml` by `ota agents`.

## Repo

- `project`: `ota`
- `description`: `Ota CLI and Contract Engine`

## Agent Contract

Use declared `ota run <task>` paths before raw package-manager, compiler, or test commands when this contract already defines the task you need. Drop to raw commands only for narrow debugging or one-off checks that the contract does not model yet.

- `entrypoint`: `setup` (`ota run setup`)
- `default_task`: `ci` (`ota run ci`)
- `safe_tasks`:
  - `setup` (`ota run setup`)
  - `build` (`ota run build`)
  - `fmt` (`ota run fmt`)
  - `check` (`ota run check`)
  - `test` (`ota run test`)
  - `ci` (`ota run ci`)
  - `contract:validate` (`ota run contract:validate`)
  - `doctor-annotations` (`ota run doctor-annotations`)
- `verify_after_changes`:
  - `ci` (`ota run ci`)
- `writable_paths`: `src`, `docs`, `README.md`, `scripts`, `tests`, `CHANGELOG.md`, `Cargo.toml`
- `protected_paths`: `Cargo.lock`, `ota.yaml`, `LICENSE`

## Bootstrap

Only install ota if it is missing and installation is approved.

- `source.kind`: `git_rev`
- `source.rev`: `6538cc074a78b3a0471f976a415238d13307a699`
- `sh`: `curl -fsSL https://dist.ota.run/install.sh | OTA_GIT_REV=6538cc074a78b3a0471f976a415238d13307a699 sh -s -- --from-git`
- `powershell`: `$env:OTA_GIT_REV='6538cc074a78b3a0471f976a415238d13307a699'; & ([scriptblock]::Create((irm https://dist.ota.run/install.ps1))) -FromGit`

## Notes

Treat Ota as the default workflow, the way npm is the default in Node repos.
Use `ota validate` to verify the contract.
Use `ota doctor` to inspect readiness and agent guidance.
Use `ota tasks` to discover runnable repo actions.
When a repo already exposes a matching safe task, prefer `ota run <task>` over raw package-manager or language-tool commands.
Fall back to direct commands only when no truthful Ota task exists or when isolating an Ota defect.
Use `ota run setup` to prepare the repo.
Use `ota run contract:validate` to self-host the contract validator against this checkout.
Use `ota run ci` to run the canonical verification path.
Prefer narrow changes with regression tests.
Keep public docs and contracts aligned with implementation.
<!-- ota-generated-agent-guidance:end -->
