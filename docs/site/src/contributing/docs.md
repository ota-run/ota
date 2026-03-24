<!-- markdownlint-disable MD013 -->
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

   Licensed under the Apache License, Version 2.0.
   See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing,
   software distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions,
   please email: os@ota.run
-->
<!-- markdownlint-enable MD013 -->

# Docs Authoring

Docs site source path:

- `docs/site/book.toml`
- `docs/site/src/SUMMARY.md`
- `docs/site/src/**/*.md`

Rules:

- keep pages short and deterministic
- prefer command examples that are already covered by tests
- do not introduce behavior claims that are not in `src/` and tests
- keep canonical specs in `docs/spec/` as the normative product surface

Local workflow:

```bash
mdbook build docs/site
```

CI workflow:

- `.github/workflows/docs-check.yml` builds the book on PRs and pushes
- `.github/workflows/docs-quality.yml` checks markdown quality and links
- `.github/workflows/docs-pages.yml` publishes to GitHub Pages from `main`
