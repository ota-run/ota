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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.
-->

# Trusted Replay Baseline Regeneration

Use this shape when a generated fixture, local store, or recorded evaluation baseline must be
reviewed as a produced artifact instead of pinned with a hand-edited digest.

The authority chain is explicit:

```text
baseline:record -> recorded attestation -> explicit promotion -> read-only replay
```

This example uses `consumption: read_only`, so its replay task requires an ephemeral-container
boundary for the full selected closure. Ota mounts a run-scoped snapshot outside the writable
workspace at the declared baseline paths. If a repo must retain native replay, use
`consumption: verify_unchanged`: Ota rechecks
the promoted output manifest after the task and reports `replay_artifact_mutation_detected` if it
changed. It never upgrades that posture into a read-only enforcement claim.

Record a candidate only when you intentionally want to change the baseline:

```bash
ota baseline record --artifact recorded-baseline --json
```

Recording requires a clean Git source tree. Ota captures that identity before the producer changes
the baseline outputs, so the reviewed recording remains bound to the source that generated it.

Review the generated artifact diff and the attestation, then select that exact record:

```bash
ota baseline promote --artifact recorded-baseline \
  --attestation .ota/replay-baselines/recorded-baseline/attestation-<sha>.json --json
```

Commit `replay/recorded-baseline.ota.json` with the reviewed baseline files. It embeds the selected
recorded attestation, so a fresh clone or CI runner can inspect producer provenance as well as verify
the baseline identities. Do not commit or hand-edit a digest in `ota.yaml`. Symlinked baseline outputs
must resolve within declared artifact paths; Ota refuses targets that escape into the mutable worktree.

`baseline:replay` has no dependency on `baseline:record`. Ota verifies the selected manifest and
the current output identities before execution, then mounts a runner-owned snapshot over the
declared output as read-only in an ephemeral container. Native, persistent-container, and deferred
container paths refuse rather than claiming protection they cannot enforce.
