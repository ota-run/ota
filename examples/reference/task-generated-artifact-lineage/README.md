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
