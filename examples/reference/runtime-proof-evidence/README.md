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

# Runtime proof evidence

Use this example when runtime readiness is not enough and the proof must distinguish three claims:

- `reachable`: Ota owned readiness for the selected dependency seam.
- `exercised`: a finite observer recovered a runner-issued marker through that dependency.
- `fault_tested`: a separate control proved the same obligation fails for the declared reason.

Run the complete proof with:

```sh
ota proof runtime --workflow app-proof --negative-control dependency-unavailable --json .
```

Read `proof_verdict` together with `not_proved[]`. Even after the seam reaches `fault_tested`, Ota
retains `dependency_output_shaping_not_proved`: this example proves dependency causality, not that
the dependency produced every application output correctly.

The producer receives `OTA_PROOF_DEPENDENCY_MARKER`. The observer does not; it must recover the
marker from the dependency and write the runner-owned attestation. The negative control writes a
separate same-obligation failure attestation and exits non-zero. A generic failure without that
validated attestation cannot become `fault_tested`.
