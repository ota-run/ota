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

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
-->

# Policy-Governed Replay Input Identity

This example keeps the replay input declaration in `ota.yaml` and the organization requirement in
`.ota/org-policy.yaml`.

Run:

```bash
ota validate
ota doctor --json
ota run verify --dry-run --json
```

The selected task is admitted only while every declared replay input has a matching immutable
identity. Change `fixtures/baseline.txt` to see the unconditional hard-pin mismatch refuse before
task execution. In a scratch copy, remove `expected_identity` without changing the policy to
exercise the policy-specific `missing_expected_identity` refusal through Doctor, dry-run,
`ota run verify --receipt`, and `ota up --json --receipt`.
