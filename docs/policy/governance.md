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

# Governance

ota is maintainer-led open infrastructure.

The core is open source under Apache 2.0, but governance is not community-run in the usual
“open PRs drive the roadmap” sense. Ota stewards the contract, command semantics, JSON output,
diagnosis model, and release direction so the product stays coherent and trustworthy.

## What is open

- the CLI
- the repo and workspace contracts
- the JSON output and public docs
- examples, specs, and the contract-first readiness model

## Governance model

- roadmap and release direction are maintained by Ota
- schema, JSON, and command-surface changes are treated as product-level decisions
- trust-sensitive behavior such as `doctor`, `detect`, `init`, `up`, and agent guidance is kept under strict maintainer review
- the public core should stay useful on its own, without requiring a hosted control plane

## Participation that is welcome

- bug reports with concrete reproduction steps
- feature requests tied to real repo/operator pain
- docs feedback and clarity issues
- example repos and fixture ideas that expose real-world edge cases
- extension, adapter, and ecosystem feedback

## Participation that is not currently accepted

- external code pull requests
- drive-by schema changes without maintainer review
- unofficial forks presented as the upstream project

## Why the project is run this way

Ota is trying to become trusted infrastructure, not just a useful script. That means:

- contract and JSON stability matter
- diagnosis must stay honest
- onboarding flows must stay deterministic
- policy and enterprise boundaries must stay explicit

Maintainer-led governance is the current mechanism for keeping those boundaries clean while the
public contract hardens.

External code pull requests remain closed during this stabilization period. Any future opening
will begin with bounded contribution lanes backed by explicit contribution terms, provenance,
ownership, and review controls; it will not transfer roadmap or release authority away from Ota.

## Maintainer repo controls

The repository should enforce a small branch-protection baseline on `main` even though Ota does
not currently accept external code pull requests.

Recommended required checks:

- `Release Gate`
- `Smoke`
- `docs-quality`
- `cargo-deny`
- `codeql`

Required checks should run on maintainer branch pushes as well as `main`. If a required workflow is
path-filtered or only runs on `main`/pull requests, branch-first maintainer merges can become
impossible because GitHub cannot verify the exact branch commit before it lands on `main`.

This keeps maintainer pushes and release work aligned with the same public trust surfaces that Ota
expects users to rely on, without turning every specialist proof workflow into a permanent merge
gate.

## First-party consumer sync discipline

Contract-surface, governance-surface, and canonical-docs widening should not ship quietly while
first-party consumer repos still teach or render an older product shape.

The release gate therefore requires an explicit update to the affected first-party consumer status
record whenever maintainers change governed surfaces. That update must either:

- record the synced consumer commit, or
- record an explicit waiver reason

See [skills-sync-governance.md](skills-sync-governance.md).

## Related policy pages

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [../../SECURITY.md](../../SECURITY.md)
- [commercial-policy.md](commercial-policy.md)
- [support-and-enterprise.md](support-and-enterprise.md)
- [org-policy.md](org-policy.md)
- [brand-policy.md](brand-policy.md)
