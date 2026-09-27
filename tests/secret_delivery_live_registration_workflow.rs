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
    use serde_yaml::{Mapping, Value};

    fn field<'a>(mapping: &'a Mapping, key: &str) -> &'a Value {
        mapping
            .get(&Value::String(key.into()))
            .unwrap_or_else(|| panic!("missing workflow field: {key}"))
    }

    let workflow: Value = serde_yaml::from_str(WORKFLOW).expect("registration workflow YAML");
    let root = workflow.as_mapping().expect("workflow mapping");
    assert_eq!(root.len(), 4);
    assert_eq!(
        field(root, "name").as_str(),
        Some("secret-delivery-github-oidc-live")
    );
    assert_eq!(
        field(root, "permissions").as_mapping(),
        Some(&Mapping::new())
    );

    let triggers = field(root, "on").as_mapping().expect("trigger mapping");
    assert_eq!(triggers.len(), 1);
    assert!(field(triggers, "workflow_dispatch").is_null());

    let jobs = field(root, "jobs").as_mapping().expect("jobs mapping");
    assert_eq!(jobs.len(), 1);
    let job = field(jobs, "registration-only")
        .as_mapping()
        .expect("registration job");
    assert_eq!(job.len(), 3);
    assert_eq!(field(job, "runs-on").as_str(), Some("ubuntu-latest"));
    assert_eq!(
        field(job, "permissions").as_mapping(),
        Some(&Mapping::new())
    );

    let steps = field(job, "steps").as_sequence().expect("steps sequence");
    assert_eq!(steps.len(), 1);
    let step = steps[0].as_mapping().expect("registration step");
    assert_eq!(step.len(), 2);
    assert_eq!(
        field(step, "name").as_str(),
        Some("Refuse execution from the registration stub")
    );
    assert_eq!(field(step, "run").as_str(), Some("exit 1"));

    assert!(!WORKFLOW.contains("if:"));
    assert!(!WORKFLOW.contains("id-token:"));
    assert!(!WORKFLOW.contains("uses:"));
    assert!(!WORKFLOW.contains("secrets."));
    assert!(!WORKFLOW.contains("ACTIONS_ID_TOKEN_REQUEST_"));
}
