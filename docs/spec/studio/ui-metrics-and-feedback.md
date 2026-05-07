<!--
               █████
              ░░███
      ██████  ███████    ██████
     ███░░███░░░███░    ░░░░░███
    ░███ ░███  ░███      ███████
    ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
     ░░░░░░     ░░░░░░   ░░░░░░░░

  Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

  Do NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

  Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
  You may not use this file except in compliance with that License.
  Unless required by applicable law or agreed to in writing, software distributed under the
  License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
  either express or implied. See the License for the specific language governing permissions
  and limitations under the License.

  If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio Metrics and Feedback

Status: planned.

Studio UX must use trust-first feedback. This document defines observability and feedback quality standards
before we implement UI.

## Feedback model

Every user-facing interaction should have one of four result classes:

- `pass` – completed successfully
- `warn` – completed with constraints or partial uncertainty
- `fail` – blocked or failed
- `blocked` – prevented from running for policy/safety reasons

## Global quality signals

### Trust-first status bar

Required top-level indicators:
- current repo / contract presence
- readiness state
- recent operation state
- active blocking reason (if any)

### Source honesty signals

- `source` field for operations: `studio|terminal|agent|automation`
- `requested_by` when non-local context is involved
- operation id visibility
- payload provenance for major actions

These signals should be visible in Overview or Run / Evidence and never hidden behind details.

## Metrics to capture in UI

Collect and display:
- operation duration (start to finish)
- phase count
- receipt write status
- exit code for completed operations
- stale-review count for draft refresh cycles
- action confirmation rate (how often action is attempted and approved)

The dashboard must display the following always:
- started time
- source
- status
- last update time

## Feedback states per operation

### Pending

- skeleton + explicit action label
- no ambiguous "processing" text; include operation kind and target

### Running

- phase timeline
- current phase text
- most recent event message

### Ready

- ready milestone marker for pre-final readiness transitions (not final success)

### Passed

- exit code
- receipt/log links
- next recommended action

### Failed

- fail reason summary
- step where it failed
- recovery action options
- direct path to logs/receipt

### Blocked

- why blocked
- required correction
- whether action can be retried automatically

## Evidence quality standards

Evidence surfaces should prefer:
1. operation timeline
2. machine-generated receipt summary
3. raw artifact path(s)
4. direct log chunks

If any level is missing, Studio should show `missing evidence` with explanation.

## Accessibility and clarity checks

- non-color status encoding:
  - label text + icon
  - status-specific microcopy
- screen-reader labels for action previews
- error surfaces should include actionable steps, not generic advice
- keyboard focus visible for each critical action control

## Dashboard anti-patterns to avoid

- silent success without proof
- showing only raw logs as proof
- hiding policy blocks behind retry buttons
- stale action buttons without state version context
- generic "operation failed" without recovery path

## Success criteria before implementation

- each page state must map to one feedback class
- every mutation action must show preview + required inputs
- every failed/blocked state must include recovery guidance
- every completed action must have retrievable evidence path
