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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
-->

# V11.19: Typed uv Local-Project Hydration

Status: active. This is the current implementation slice after V11.18. It narrows one real
package-hydration gap before any Dev Container adapter work begins.

## Problem

Ota's typed `prepare.source.kind: uv` hydration currently owns only:

- `uv sync`; and
- `uv pip install -r <requirements_file>`.

Many Python repositories instead install a checked-out local project in editable mode, select its
extras, and install one or more dependency groups. Dograh's API test lane is a concrete example:

```sh
uv pip install -e ./pipecat[openai,deepgram,...]
uv pip install --group pipecat/pyproject.toml:dev
```

Representing those operations as an opaque structured command preserves execution, but Ota cannot
then own the local project, extras, group, input identity, or resolved hydration provenance as one
canonical source. It also leaves `doctor`, dry-run, receipts, replay, and policy consumers to
infer too much from a command string.

## Product Principle

Local-project hydration is contract-owned dependency truth, not a shell escape hatch.

The contract declares the project path, editable posture, extras, groups, index posture, and
durable target. The runner resolves the exact ordered `uv` invocations, observes the local project
and declared dependency inputs, and emits the same typed hydration provenance used by V11.12. Ota
does not infer extras or groups from ambient project configuration, and it never rewrites a lock or
promotes an observed dependency set into a contract pin.

## Scope

V11.19 adds one additive canonical uv hydration mode for one local project per task:

```yaml
tasks:
  setup:pipecat:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: uv
        cwd: .
        mode: pip_local_project
        local_project:
          path: pipecat
          editable: true
          extras:
            - openai
            - deepgram
          groups:
            - dev
          lockfile: pipecat/uv.lock
        default_index: https://pypi.org/simple
    requirements:
      toolchains: [python]
    effects:
      writes: [.venv]
      network: true
      network_kind: dependency_hydration
```

`local_project.path` is relative to `source.cwd` and resolves to a directory containing `pyproject.toml`.
`editable` is explicit and defaults to `false`; Ota must not assume editable installation merely
because the path is local. `extras[]` and `groups[]` are ordered, non-empty string sets with stable
deduplication. `lockfile` is optional and relative to `source.cwd`. Its absence is honest but leaves the
affected dependency resolution replay posture unavailable; it never makes the hydration lockfile
backed by implication.

The typed renderer executes deterministic, declared operations in order:

1. `uv pip install <project>` or `uv pip install -e <project>` with the declared extras;
2. `uv pip install --group <project>/pyproject.toml:<group>` for each declared group in order.

The project install must happen before group installation. Ota does not add project dependencies,
groups, extras, `--upgrade`, `--reinstall`, or ambient indexes not present in the contract.
Existing `default_index`, ordered `indexes`, `offline`, and Compose-wrapper semantics remain
shared with the current typed uv source; V11.19 must reuse rather than duplicate them.

## Identity And Evidence

The runner owns observed provenance. Before hydration it must record:

- the normalized local-project path;
- the declared `pyproject.toml` identity;
- the declared lockfile identity when one is declared and present;
- declared extras and ordered groups;
- resolved index/offline posture through the existing hydration-provenance model; and
- the selected execution scope, mode, and source identity already bound by the receipt.

The receipt publishes the declared local-project posture separately from observed input identities.
An editable project is source-sensitive: the record must bind the project source identity available
at the selected boundary, not merely the directory name. A Git submodule remains a source input;
Ota records its resolved revision when available and otherwise reports source identity as
unavailable. The local project must never be represented as a witnessed execution observation.

For V11.10 replay, a declared `lockfile` is a pinned input only after its observed identity is
captured. A missing lockfile, source-identity failure, or undeclared ambient index leaves the
affected replay class unavailable or suspect; a successful install does not erase that boundary.

## Admission And Diagnostics

Validation must reject:

- a non-directory local project;
- a project without `pyproject.toml`;
- a `local_project` block with a non-`pip_local_project` mode;
- `pip_local_project` without one local project;
- duplicate extras or groups after normalization;
- an invalid or escaping `lockfile` path;
- group/extras on `sync` or `pip_requirements`; and
- missing Python toolchain, durable output, or dependency-hydration effects inherited from the
  existing uv source rules.

`ota doctor` must report missing project manifest, missing declared lockfile, unavailable source
identity, and index posture gaps before execution. Dry-run must render the exact typed steps and
their declared input identities without starting hydration. Run, `up`, receipt, and replay paths
must share one source evaluator; no surface may accept the declaration without checking the same
local-project inputs.

## Non-goals

V11.19 does not:

- accept arbitrary `uv pip` flags or command fragments;
- infer project extras/groups from `pyproject.toml`;
- support multiple local projects in one typed source;
- implement workspace-member package resolution beyond the declared local path;
- automatically update a lockfile, expected identity, or baseline;
- treat an unlocked local project as replay-pinned;
- govern Dev Container build, post-create, post-start, mounts, or VS Code lifecycle hooks; or
- normalize GitHub service-container and local Compose topologies into one false proof claim.

Those last two boundaries are distinct potential follow-ons. A future V11.20 Dev Container adapter
must own its own execution, image/config identity, lifecycle, mounts, and provider capabilities;
it is not a `pip_local_project` mode or a generic `--container` switch.

## Implementation Order

1. Add the typed contract model, parser, published schema, and semantic validator.
2. Add one shared uv local-project source evaluator and renderer; reuse existing index, offline,
   Compose-wrapper, execution-mode, and hydration-provenance paths.
3. Bind project manifest, optional lockfile, and source identity into dry-run, Doctor, receipt,
   replay, and policy-facing output.
4. Add regressions for editable and non-editable install, extras/groups ordering, missing/escaping
   inputs, absent lockfile replay boundaries, Compose wrapping, and no execution after invalid
   admission.
5. Update contract reference, JSON schemas/reference, command docs where output changes,
   changelog, a copy-ready example, canonical Ota skill, and site reference only where public
   behavior changes.
6. Pressure-test Dograh's API lane without the current opaque Pipecat commands, then pressure an
   independent Python repository that uses a different local-project/extras or group shape.

## Acceptance Bar

V11.19 is complete only when:

- one canonical contract source renders Dograh's editable project plus ordered group install with
  no shell-owned hydration command;
- validator, Doctor, dry-run, run, `up`, receipt, and replay use one shared input evaluator;
- output distinguishes declared local-project posture from runner-observed manifest, lockfile, and
  source identities;
- an unlocked or source-unavailable project cannot be over-read as replay-pinned;
- normal and Compose-wrapped execution preserve existing uv index/offline semantics;
- Dograh proves the typed lane on its exact pinned Core source and matrix;
- a second repository proves the shape is not Dograh-specific; and
- every material ungoverned behavior in both pressure repos is either explicitly bounded or
  recorded as a named Ota gap.
