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

# Contributing

ota is maintainer-led and does not currently accept external code contributions.

That is a governance choice, not a signal that outside feedback is unwanted. The most useful
external input right now is issue-quality feedback that helps harden the public contract and the
trust-sensitive command surfaces.

## What to send

- bug reports with a concrete reproduction
- feature requests tied to a real repo or operator workflow
- docs feedback where wording, examples, or guidance are unclear
- fixture ideas or example repos that expose important edge cases

For suspected vulnerabilities, do not open a public issue. Use [SECURITY.md](SECURITY.md).

Use the GitHub issue templates when possible so the report is actionable.

## What not to send

- external code pull requests
- speculative refactors without a concrete operator problem
- product-positioning changes that are not grounded in the shipped contract and docs

## Why pull requests are closed

Ota treats the contract, diagnosis model, JSON output, and readiness semantics as trust-critical
product surfaces. The project is currently kept under strict maintainer stewardship so those
surfaces can evolve coherently while the public contract hardens.

External code pull requests remain closed during this stabilization period. Any future opening
will begin with bounded contribution lanes backed by explicit contribution terms, provenance,
ownership, and review controls; roadmap and trust-sensitive product decisions will remain
maintainer-led.

## If you want to help anyway

- open a bug with a failing repo shape
- file docs corrections or examples that are misleading
- propose a feature with exact operator value and trade-offs
- point out contract drift, output ambiguity, or trust leaks

## Related pages

- [docs/policy/governance.md](docs/policy/governance.md)
- [SECURITY.md](SECURITY.md)
- [docs/policy/commercial-policy.md](docs/policy/commercial-policy.md)
- [docs/policy/support-and-enterprise.md](docs/policy/support-and-enterprise.md)
