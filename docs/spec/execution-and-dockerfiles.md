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
   License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Execution and Dockerfiles

A Dockerfile builds a runnable image. An ota contract tells ota what the repo needs, what is safe, and how readiness is checked.

Use both when they fit:

- Dockerfile = environment build recipe
- `ota.yaml` = readiness and execution contract
- `ota run` / `ota up` = the command path that uses the contract

Boundary note:

- Docker is optional, not a universal prerequisite for ota
- ota can use container mode when the repo or org already allows it
- host bootstrap for Docker is a separate concern and should not be assumed by default

When a repo uses `execution.preferred: container`, ota runs the task inside the configured image. If that
image comes from a Dockerfile, the Dockerfile is what made the environment runnable; ota is what
points at it, diagnoses readiness, and keeps the repo contract explicit.

Simple model:

```text
Dockerfile  -> builds the image
ota.yaml    -> declares what the repo needs
ota run/up  -> executes against the declared environment
```

What each one is good at:

- Dockerfile: OS packages, runtimes, and reproducible image setup
- ota contract: tasks, checks, safe AI-agent guidance, provisioning policy, and execution mode
- container mode: a repeatable runtime boundary for `ota run` and `ota up`
- `doctor`: the command that explains whether the repo is ready inside that boundary

Example:

```dockerfile
FROM eclipse-temurin:21-jdk
WORKDIR /workspace
COPY . .
RUN ./mvnw -q dependency:go-offline
```

```yaml
version: 1
project:
  name: qredex-core
runtimes:
  java: ">=21"
tools:
  maven: "*"
tasks:
  setup:
    run: mvn -q dependency:go-offline
  test:
    run: mvn test
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: eclipse-temurin:21-jdk
```

Rule of thumb:

- use the Dockerfile to make the image runnable
- use the ota contract to make the repo explainable, diagnosable, and safe
- use container mode when you want ota to run against the image instead of the host
- use `ota up` when you want ota to prepare the repo inside that image before running tasks
- do not replace the contract with the Dockerfile, because the Dockerfile does not tell ota what is safe to run or what should be provisioned
- do not make Docker the default adoption requirement unless the repo or org already uses it
