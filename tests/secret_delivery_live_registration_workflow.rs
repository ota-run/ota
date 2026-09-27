//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//     ░░██████   ░░█████ ░░████████
//      ░░░░░░     ░░░░░   ░░░░░░░░
//
//   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
//
//   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
//
//   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.

const WORKFLOW: &str = include_str!("../.github/workflows/secret-delivery-github-oidc-live.yml");

#[test]
fn default_branch_live_oidc_workflow_is_registration_only() {
    assert!(WORKFLOW.contains("name: secret-delivery-github-oidc-live"));
    assert!(WORKFLOW.contains("  workflow_dispatch:"));
    assert!(WORKFLOW.contains("permissions: {}"));
    assert!(WORKFLOW.contains("    if: ${{ github.ref != 'refs/heads/main' }}"));
    assert!(WORKFLOW.contains("        run: exit 1"));
    assert!(!WORKFLOW.contains("id-token:"));
    assert!(!WORKFLOW.contains("uses:"));
    assert!(!WORKFLOW.contains("secrets."));
    assert!(!WORKFLOW.contains("ACTIONS_ID_TOKEN_REQUEST_"));
}
