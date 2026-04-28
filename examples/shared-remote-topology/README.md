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

# Shared remote topology dogfood

This example turns the shipped shared-remote `ssh` slice into a real local dogfood loop.

## Why this exists

- proves that the built-in `ssh` provider can drive shared-remote `ensure_ready`
- keeps the example truthful to the shipped slice:
  - shared remote backend binding
  - `address_view: internal`
  - TCP readiness
- uses a dockerized `sshd` target only as transport scaffolding for the test
- reuses one existing default local SSH key instead of inventing a provider-specific path
- installs one explicit `ota-remote-devbox` host alias through your normal `~/.ssh/config`

## What it proves

- ota can start or reuse the remote producer through the built-in `ssh` provider
- ota can probe remote readiness on the remote plane
- the consumer can target the producer through `address_view: internal`

## What it does not prove

- a real off-host network boundary
- remote host-view activation
- remote HTTP readiness
- provider behavior beyond the built-in `ssh` path

## Why docker is acceptable here

Yes, using a docker container for remote access is acceptable for this dogfood.

It does not pretend to be a true remote host. It only gives the built-in `ssh` provider a real
remote command surface so the shipped activation and target-resolution slice can be exercised
end to end without requiring separate infrastructure.

## Try it

1. Prepare the dockerized sshd target:

```bash
./prepare-dogfood.sh
```

2. Validate the example:

```bash
cargo run --quiet -- validate --json ./examples/shared-remote-topology
```

3. Build the current source tree once under your normal shell environment:

```bash
cargo build -q
```

4. Run the one-shot smoke task through the shared remote backend:

```bash
./target/debug/ota run smoke ./examples/shared-remote-topology --stream
```

5. Optionally run the long-running helper variant afterward:

```bash
./target/debug/ota run sandbox ./examples/shared-remote-topology --stream
```

## What to look for

- `smoke` should ask ota to make `dev` ready
- ota should start or reuse the remote producer through `ssh ota-remote-devbox`
- `smoke` should print the resolved `OTA_INPUT_BASE_URL`
- `smoke` should fetch the remote API successfully and exit `0`
- `sandbox` uses the same target path but keeps running as a service after the fetch succeeds

## What `prepare-dogfood.sh` changes locally

- starts the dockerized sshd container on local port `2222`
- copies one existing local public key into the example runtime as the remote `authorized_keys`
- writes `~/.ssh/ota-shared-remote-topology.conf`
- ensures your normal `~/.ssh/config` includes that file once

## Cleanup

```bash
docker compose -f ./examples/shared-remote-topology/compose.yaml down -v
```
