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

# Generated Artifact Lineage

Use this shape when one task generates a named repo-local SDK or client and another task verifies
or packages that generated output.

`artifacts.<name>` owns the generated output identity. The producer task declares how it is made;
the consumer task directly depends on that producer and names the artifact under
`requires_artifacts`. Ota checks that the output exists after dependencies finish and before the
consumer runs.

This declares presence and lineage, not freshness. Do not use timestamps or a passing consumer to
claim that a generated artifact reflects every current source input.

## Replay inputs and witnessed observations

Use `replay_inputs` for immutable deterministic lane inputs and
`witnessed_observations.query_traces` for prior-run query evidence that must remain runner-witnessed
output instead of current-run input truth.

```yaml
tasks:
  sdk:verify:
    replay_inputs:
      - id: api_schema
        kind: static_file
        path: schema/api.graphql
        expected_identity: sha256:cf448e0a574c770db5d4562bd2d46a67b50fa02d16eb207febc2a613c399b27a
      - id: runtime-presentation
        kind: presentation_profile
        path: replay/presentation-profile.yaml
      - id: equivalence
        kind: comparator_profile
        path: replay/comparator-profile.yaml
    witnessed_observations:
      query_traces:
        - id: recorded_queries
          path: evidence/sdk-queries.jsonl
```

Use `static_file` for generic immutable repo files, `presentation_profile` for declared
output-shaping or normalization policy, and `comparator_profile` for declared equivalence or
tolerance policy. Keep the query trace separate: Ota emits it under receipt
`witnessed_observations`, not `evaluated_inputs`.

Use `expected_identity` when the lane must reject a changed input before execution. It is an
explicit reviewed SHA-256 pin: Ota records the observed identity but never updates the declared
value for you.
