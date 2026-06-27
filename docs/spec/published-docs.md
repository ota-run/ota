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

# ota Published Docs Manifest

This document describes the current machine-readable published docs ownership surface.

Use this when a downstream system needs to know which upstream `ota` source file owns a public docs
surface, without scraping site chrome or hard-coding repo paths.

Published canonical docs manifest:

- locally: [`published-docs/canonical-docs.json`](published-docs/canonical-docs.json)
- published: `https://dist.ota.run/spec/published-docs/latest/canonical-docs.json`

## Purpose

`canonical-docs.json` is the Rust-owned machine-readable map from key `ota` docs surfaces to:

- the canonical source file path in the `ota` repo
- the canonical source URL on GitHub
- the public docs URL when one exists
- the canonical responsibility of that doc surface

The initial shipped entries cover core reference surfaces such as:

- contract
- workspace
- command
- JSON output
- execution governance loop
- execution topology
- local service topology
- toolchains/runtimes/tools
- doctor finding catalog

## Top-level shape

```json
{
  "version": 1,
  "docs": [
    {
      "id": "contract-reference",
      "title": "ota Contract Reference",
      "kind": "reference",
      "source_path": "docs/spec/contract-reference.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/contract-reference.md",
      "public_url": "https://ota.run/docs/reference/contract",
      "canonical_for": [
        "ota.yaml field semantics",
        "repo contract validation truth"
      ]
    }
  ]
}
```

## Field meaning

- `version`: manifest schema version for this published surface
- `docs`: ordered list of canonical docs entries
- `docs[].id`: stable identifier for one doc surface
- `docs[].title`: human-facing title for that doc surface
- `docs[].kind`: current category of the doc surface; this shipped slice uses `reference`
- `docs[].source_path`: repo-relative path to the canonical upstream source file
- `docs[].source_url`: canonical GitHub source URL for that same file
- `docs[].public_url`: public `ota.run` URL for the corresponding rendered docs page, or `null`
  when the source currently has no public page
- `docs[].canonical_for`: ordered list of the responsibilities this doc owns

## Governance rules

- Treat `canonical-docs.json` as a published compatibility surface.
- Regenerate it with `cargo run --bin sync_published_doc_manifests`; do not hand-edit the file.
- The release gate and local `compat` task rerun the Rust-owned generator and fail on drift.
- Additive entries are preferred when ota learns a new authoritative docs surface.
- If a source file moves or a public URL changes, update the manifest in the same change.
