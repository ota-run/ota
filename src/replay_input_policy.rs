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
//   You may not use this file except in compliance with that License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::policy_pack::{OrgPolicyPack, PolicyReplayInputIdentityRule};
use crate::replay_inputs::sha256_identity;
use crate::schema::Contract;
use crate::semantic_identity::semantic_contract_identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayInputPolicySubject<'a> {
    Task(&'a str),
    Workflow(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplayInputPolicySubjectRecord {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplayInputPolicyInputRecord {
    pub task: String,
    pub id: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_identity: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplayInputPolicyRuleRecord {
    pub identity: String,
    pub subject: ReplayInputPolicySubjectRecord,
    pub on_insufficient: String,
    pub closure_tasks: Vec<String>,
    pub input_keys: Vec<String>,
    pub coverage: String,
    pub decision: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplayInputPolicyUnknownSelector {
    pub identity: String,
    pub subject: ReplayInputPolicySubjectRecord,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplayInputPolicyEvaluation {
    pub policy_identity: String,
    pub subject: ReplayInputPolicySubjectRecord,
    pub required: bool,
    pub decision: String,
    pub coverage: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub applicable_rules: Vec<ReplayInputPolicyRuleRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<ReplayInputPolicyInputRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unknown_selectors: Vec<ReplayInputPolicyUnknownSelector>,
}

pub(crate) type ReplayInputPolicyObservations = BTreeMap<String, ReplayInputPolicyInputRecord>;

pub(crate) fn observe_replay_inputs(
    contract: &Contract,
    contract_root: &Path,
    roots: impl IntoIterator<Item = String>,
) -> ReplayInputPolicyObservations {
    let mut observations = BTreeMap::new();
    for task_name in contract.task_execution_closure_names(roots) {
        let Some(task) = contract.tasks.get(&task_name) else {
            continue;
        };
        for input in &task.replay_inputs {
            let key = replay_input_observation_key(&task_name, &input.id);
            let observed = fs::read(contract_root.join(input.path.trim()))
                .map(|bytes| sha256_identity(&bytes))
                .map_err(|error| error.to_string());
            let (observed_identity, status, error) = match observed {
                Ok(observed) if input.expected_identity.as_deref() == Some(observed.as_str()) => {
                    (Some(observed), "matched", None)
                }
                Ok(observed) if input.expected_identity.is_some() => {
                    (Some(observed), "mismatched", None)
                }
                Ok(observed) => (Some(observed), "unpinned", None),
                Err(error) if input.expected_identity.is_some() => (None, "missing", Some(error)),
                Err(error) => (None, "unpinned_unreadable", Some(error)),
            };
            observations.insert(
                key,
                ReplayInputPolicyInputRecord {
                    task: task_name.clone(),
                    id: input.id.clone(),
                    path: input.path.clone(),
                    expected_identity: input.expected_identity.clone(),
                    observed_identity,
                    status: status.to_string(),
                    error,
                },
            );
        }
    }
    observations
}

pub(crate) fn evaluate_replay_input_policy(
    contract: &Contract,
    contract_root: &Path,
    policy: &OrgPolicyPack,
    subject: ReplayInputPolicySubject<'_>,
) -> ReplayInputPolicyEvaluation {
    let observations = observe_replay_inputs(
        contract,
        contract_root,
        selected_replay_input_policy_closure(contract, subject),
    );
    evaluate_replay_input_policy_with_observations(contract, policy, subject, &observations)
}

pub(crate) fn evaluate_replay_input_policy_with_observations(
    contract: &Contract,
    policy: &OrgPolicyPack,
    subject: ReplayInputPolicySubject<'_>,
    observations: &ReplayInputPolicyObservations,
) -> ReplayInputPolicyEvaluation {
    evaluate_replay_input_policy_for_closure_with_observations(
        contract,
        policy,
        subject,
        selected_replay_input_policy_closure(contract, subject),
        observations,
    )
}

pub(crate) fn evaluate_replay_input_policy_for_closure_with_observations(
    contract: &Contract,
    policy: &OrgPolicyPack,
    subject: ReplayInputPolicySubject<'_>,
    selected_closure: Vec<String>,
    observations: &ReplayInputPolicyObservations,
) -> ReplayInputPolicyEvaluation {
    let selected_closure = contract.task_execution_closure_names(selected_closure);
    let subject_record = subject_record(subject);
    let policy_identity =
        semantic_contract_identity(policy).expect("org policy pack serialization must succeed");

    let rules = &policy.policies.replay_inputs.identity;
    let selected_names = selected_closure.iter().cloned().collect::<BTreeSet<_>>();
    let mut applicable = Vec::new();

    if let ReplayInputPolicySubject::Workflow(name) = subject
        && let Some(rule) = rules.workflows.get(name)
    {
        applicable.push((
            rule_identity("workflow", name),
            ReplayInputPolicySubject::Workflow(name),
            rule,
            selected_closure.clone(),
        ));
    }
    for (name, rule) in &rules.tasks {
        if selected_names.contains(name) {
            applicable.push((
                rule_identity("task", name),
                ReplayInputPolicySubject::Task(name),
                rule,
                contract.task_execution_closure_names([name.clone()]),
            ));
        }
    }
    applicable.sort_by(|left, right| left.0.cmp(&right.0));

    let unknown_selectors = unknown_selectors(contract, policy);
    if applicable.is_empty() {
        if !unknown_selectors.is_empty() {
            return ReplayInputPolicyEvaluation {
                policy_identity,
                subject: subject_record,
                required: true,
                decision: String::from("deny"),
                coverage: String::from("insufficient"),
                applicable_rules: Vec::new(),
                inputs: Vec::new(),
                unknown_selectors,
            };
        }
        return ReplayInputPolicyEvaluation {
            policy_identity,
            subject: subject_record,
            required: false,
            decision: String::from("allow"),
            coverage: String::from("not_required"),
            applicable_rules: Vec::new(),
            inputs: Vec::new(),
            unknown_selectors,
        };
    }

    let input_tasks = applicable
        .iter()
        .flat_map(|(_, _, _, closure)| closure.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut inputs_by_key = BTreeMap::new();
    for task_name in input_tasks {
        let Some(task) = contract.tasks.get(&task_name) else {
            continue;
        };
        for input in &task.replay_inputs {
            let key = replay_input_observation_key(&task_name, &input.id);
            let observation =
                observations
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| ReplayInputPolicyInputRecord {
                        task: task_name.clone(),
                        id: input.id.clone(),
                        path: input.path.clone(),
                        expected_identity: input.expected_identity.clone(),
                        observed_identity: None,
                        status: String::from("observation_unavailable"),
                        error: Some(String::from(
                            "replay input was not captured by the command preflight",
                        )),
                    });
            inputs_by_key.insert(key, observation);
        }
    }

    let rule_records = applicable
        .into_iter()
        .map(|(identity, rule_subject, rule, closure)| {
            evaluate_rule(identity, rule_subject, rule, closure, &inputs_by_key)
        })
        .collect::<Vec<_>>();
    let decision = if unknown_selectors.is_empty() {
        aggregate_decision(&rule_records)
    } else {
        "deny"
    };
    let coverage = if unknown_selectors.is_empty()
        && rule_records.iter().all(|rule| rule.coverage == "complete")
    {
        "complete"
    } else {
        "insufficient"
    };

    ReplayInputPolicyEvaluation {
        policy_identity,
        subject: subject_record,
        required: true,
        decision: decision.to_string(),
        coverage: coverage.to_string(),
        applicable_rules: rule_records,
        inputs: inputs_by_key.into_values().collect(),
        unknown_selectors,
    }
}

pub(crate) fn replay_input_observation_key(task_name: &str, input_id: &str) -> String {
    input_key(task_name, input_id)
}

pub(crate) fn replay_input_policy_unknown_selectors(
    contract: &Contract,
    policy: &OrgPolicyPack,
) -> Vec<ReplayInputPolicyUnknownSelector> {
    unknown_selectors(contract, policy)
}

fn evaluate_rule(
    identity: String,
    subject: ReplayInputPolicySubject<'_>,
    rule: &PolicyReplayInputIdentityRule,
    closure_tasks: Vec<String>,
    inputs_by_key: &BTreeMap<String, ReplayInputPolicyInputRecord>,
) -> ReplayInputPolicyRuleRecord {
    let closure = closure_tasks.iter().cloned().collect::<BTreeSet<_>>();
    let mut input_keys = inputs_by_key
        .iter()
        .filter(|(_, input)| closure.contains(&input.task))
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    input_keys.sort();

    let mut reasons = Vec::new();
    if input_keys.is_empty() {
        reasons.push(String::from("no_declared_replay_inputs"));
    }
    for key in &input_keys {
        let input = &inputs_by_key[key];
        match input.status.as_str() {
            "unpinned" => {
                reasons.push(format!("missing_expected_identity:{key}"));
            }
            "unpinned_unreadable" => {
                reasons.push(format!("missing_expected_identity:{key}"));
                reasons.push(format!("declared_input_unreadable:{key}"));
            }
            "missing" => reasons.push(format!("declared_input_unreadable:{key}")),
            "mismatched" => reasons.push(format!("declared_identity_mismatch:{key}")),
            "matched" => {}
            "observation_unavailable" => {
                reasons.push(format!("observation_unavailable:{key}"));
            }
            status => {
                reasons.push(format!("unknown_observation_status:{key}:{status}"));
            }
        }
    }
    reasons.sort();
    reasons.dedup();

    let coverage = if reasons.is_empty() {
        "complete"
    } else {
        "insufficient"
    };
    let hard_failure = reasons.iter().any(|reason| {
        reason.starts_with("declared_input_unreadable:")
            || reason.starts_with("declared_identity_mismatch:")
            || reason.starts_with("observation_unavailable:")
            || reason.starts_with("unknown_observation_status:")
    });
    let decision = if reasons.is_empty() {
        "allow"
    } else if hard_failure {
        "deny"
    } else {
        rule.on_insufficient.as_str()
    };

    ReplayInputPolicyRuleRecord {
        identity,
        subject: subject_record(subject),
        on_insufficient: rule.on_insufficient.as_str().to_string(),
        closure_tasks,
        input_keys,
        coverage: coverage.to_string(),
        decision: decision.to_string(),
        reasons,
    }
}

fn aggregate_decision(rules: &[ReplayInputPolicyRuleRecord]) -> &'static str {
    if rules.iter().any(|rule| rule.decision == "deny") {
        "deny"
    } else if rules.iter().any(|rule| rule.decision == "review") {
        "review"
    } else {
        "allow"
    }
}

pub(crate) fn selected_replay_input_policy_closure(
    contract: &Contract,
    subject: ReplayInputPolicySubject<'_>,
) -> Vec<String> {
    match subject {
        ReplayInputPolicySubject::Task(name) => {
            contract.task_execution_closure_names([name.to_string()])
        }
        ReplayInputPolicySubject::Workflow(name) => contract.task_execution_closure_names(
            contract.selected_workflow_task_closure_names(Some(name)),
        ),
    }
}

fn unknown_selectors(
    contract: &Contract,
    policy: &OrgPolicyPack,
) -> Vec<ReplayInputPolicyUnknownSelector> {
    let rules = &policy.policies.replay_inputs.identity;
    let mut unknown = rules
        .tasks
        .keys()
        .filter(|name| !contract.tasks.contains_key(*name))
        .map(|name| ReplayInputPolicyUnknownSelector {
            identity: rule_identity("task", name),
            subject: ReplayInputPolicySubjectRecord {
                kind: String::from("task"),
                name: name.clone(),
            },
            reason: String::from("unknown_task_selector"),
        })
        .chain(
            rules
                .workflows
                .keys()
                .filter(|name| {
                    contract
                        .workflows
                        .as_ref()
                        .is_none_or(|workflows| !workflows.items.contains_key(*name))
                })
                .map(|name| ReplayInputPolicyUnknownSelector {
                    identity: rule_identity("workflow", name),
                    subject: ReplayInputPolicySubjectRecord {
                        kind: String::from("workflow"),
                        name: name.clone(),
                    },
                    reason: String::from("unknown_workflow_selector"),
                }),
        )
        .collect::<Vec<_>>();
    unknown.sort_by(|left, right| left.identity.cmp(&right.identity));
    unknown
}

fn subject_record(subject: ReplayInputPolicySubject<'_>) -> ReplayInputPolicySubjectRecord {
    match subject {
        ReplayInputPolicySubject::Task(name) => ReplayInputPolicySubjectRecord {
            kind: String::from("task"),
            name: name.to_string(),
        },
        ReplayInputPolicySubject::Workflow(name) => ReplayInputPolicySubjectRecord {
            kind: String::from("workflow"),
            name: name.to_string(),
        },
    }
}

fn rule_identity(kind: &str, name: &str) -> String {
    format!("replay_inputs:identity:{kind}:{name}")
}

fn input_key(task: &str, input: &str) -> String {
    format!("task:{task}:input:{input}")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        ReplayInputPolicySubject, evaluate_replay_input_policy,
        evaluate_replay_input_policy_with_observations, observe_replay_inputs,
    };
    use crate::policy_pack::OrgPolicyPack;
    use crate::replay_inputs::sha256_identity;
    use crate::schema::Contract;

    fn contract(yaml: &str) -> Contract {
        serde_yaml::from_str(yaml).expect("contract should parse")
    }

    fn policy(yaml: &str) -> OrgPolicyPack {
        serde_yaml::from_str(yaml).expect("policy should parse")
    }

    #[test]
    fn parent_task_cannot_bypass_dependency_identity_rule() {
        let repo = tempdir().unwrap();
        fs::write(repo.path().join("fixture.txt"), "frozen").unwrap();
        let identity = sha256_identity(b"frozen");
        let contract = contract(&format!(
            "version: 1\nproject:\n  name: test\ntasks:\n  replay:\n    replay_inputs:\n      - id: fixture\n        kind: static_file\n        path: fixture.txt\n        expected_identity: {identity}\n    command:\n      exe: true\n  verify:\n    depends_on: [replay]\n    command:\n      exe: true\n"
        ));
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        replay:\n          on_insufficient: deny\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("verify"),
        );

        assert!(result.required);
        assert_eq!(result.decision, "allow");
        assert_eq!(result.coverage, "complete");
        assert_eq!(result.applicable_rules[0].subject.name, "replay");
        assert_eq!(result.inputs[0].status, "matched");
    }

    #[test]
    fn parent_task_cannot_bypass_outcome_hook_identity_rule() {
        let repo = tempdir().unwrap();
        fs::write(repo.path().join("fixture.txt"), "ambient").unwrap();
        let contract = contract(
            "version: 1\nproject:\n  name: test\ntasks:\n  report:\n    replay_inputs:\n      - id: fixture\n        kind: static_file\n        path: fixture.txt\n    command:\n      exe: true\n  verify:\n    command:\n      exe: true\n    after_success: [report]\n",
        );
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        report:\n          on_insufficient: deny\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("verify"),
        );

        assert!(result.required);
        assert_eq!(result.decision, "deny");
        assert_eq!(result.applicable_rules[0].subject.name, "report");
        assert_eq!(
            result.applicable_rules[0].closure_tasks,
            vec![String::from("report")]
        );
        assert_eq!(result.inputs[0].task, "report");
    }

    #[test]
    fn supplied_observations_are_the_only_policy_evidence_for_an_admission() {
        let repo = tempdir().unwrap();
        fs::write(repo.path().join("fixture.txt"), "frozen").unwrap();
        let identity = sha256_identity(b"frozen");
        let contract = contract(&format!(
            "version: 1\nproject:\n  name: test\ntasks:\n  replay:\n    replay_inputs:\n      - id: fixture\n        kind: static_file\n        path: fixture.txt\n        expected_identity: {identity}\n    command:\n      exe: true\n"
        ));
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        replay:\n          on_insufficient: deny\n",
        );
        let observations = observe_replay_inputs(&contract, repo.path(), [String::from("replay")]);
        fs::write(repo.path().join("fixture.txt"), "changed").unwrap();

        let result = evaluate_replay_input_policy_with_observations(
            &contract,
            &policy,
            ReplayInputPolicySubject::Task("replay"),
            &observations,
        );

        assert_eq!(result.decision, "allow");
        assert_eq!(result.inputs[0].status, "matched");
        assert_eq!(
            result.inputs[0].observed_identity.as_deref(),
            Some(identity.as_str())
        );
    }

    #[test]
    fn unavailable_preflight_observation_fails_closed() {
        let contract = contract(
            "version: 1\nproject:\n  name: test\ntasks:\n  replay:\n    replay_inputs:\n      - id: fixture\n        kind: static_file\n        path: fixture.txt\n    command:\n      exe: true\n",
        );
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        replay:\n          on_insufficient: review\n",
        );

        let result = evaluate_replay_input_policy_with_observations(
            &contract,
            &policy,
            ReplayInputPolicySubject::Task("replay"),
            &Default::default(),
        );

        assert_eq!(result.decision, "deny");
        assert_eq!(result.coverage, "insufficient");
        assert_eq!(result.inputs[0].status, "observation_unavailable");
        assert_eq!(
            result.applicable_rules[0].reasons,
            vec![String::from(
                "observation_unavailable:task:replay:input:fixture"
            )]
        );
    }

    #[test]
    fn each_rule_uses_its_own_closure_and_deny_outranks_review() {
        let repo = tempdir().unwrap();
        fs::write(repo.path().join("pinned.txt"), "frozen").unwrap();
        fs::write(repo.path().join("unpinned.txt"), "ambient").unwrap();
        let identity = sha256_identity(b"frozen");
        let contract = contract(&format!(
            "version: 1\nproject:\n  name: test\ntasks:\n  pinned:\n    replay_inputs:\n      - id: pinned\n        kind: static_file\n        path: pinned.txt\n        expected_identity: {identity}\n    command:\n      exe: true\n  unpinned:\n    replay_inputs:\n      - id: unpinned\n        kind: static_file\n        path: unpinned.txt\n    command:\n      exe: true\n  verify:\n    depends_on: [pinned, unpinned]\n    command:\n      exe: true\n"
        ));
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        pinned:\n          on_insufficient: review\n        unpinned:\n          on_insufficient: deny\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("verify"),
        );

        assert_eq!(result.decision, "deny");
        assert_eq!(result.applicable_rules.len(), 2);
        assert_eq!(result.applicable_rules[0].coverage, "complete");
        assert_eq!(result.applicable_rules[0].input_keys.len(), 1);
        assert_eq!(result.applicable_rules[1].coverage, "insufficient");
        assert_eq!(result.applicable_rules[1].input_keys.len(), 1);
    }

    #[test]
    fn declared_pin_mismatch_is_always_denied() {
        let repo = tempdir().unwrap();
        fs::write(repo.path().join("fixture.txt"), "changed").unwrap();
        let expected = sha256_identity(b"frozen");
        let contract = contract(&format!(
            "version: 1\nproject:\n  name: test\ntasks:\n  replay:\n    replay_inputs:\n      - id: fixture\n        kind: static_file\n        path: fixture.txt\n        expected_identity: {expected}\n    command:\n      exe: true\n"
        ));
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        replay:\n          on_insufficient: review\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("replay"),
        );

        assert_eq!(result.decision, "deny");
        assert_eq!(result.inputs[0].status, "mismatched");
        assert_eq!(
            result.applicable_rules[0].reasons,
            vec![String::from(
                "declared_identity_mismatch:task:replay:input:fixture"
            )]
        );
    }

    #[test]
    fn unreadable_declared_input_is_always_denied() {
        let repo = tempdir().unwrap();
        let contract = contract(
            "version: 1\nproject:\n  name: test\ntasks:\n  replay:\n    replay_inputs:\n      - id: fixture\n        kind: static_file\n        path: missing.txt\n    command:\n      exe: true\n",
        );
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        replay:\n          on_insufficient: review\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("replay"),
        );

        assert_eq!(result.decision, "deny");
        assert_eq!(result.inputs[0].status, "unpinned_unreadable");
        assert_eq!(
            result.applicable_rules[0].reasons,
            vec![
                String::from("declared_input_unreadable:task:replay:input:fixture"),
                String::from("missing_expected_identity:task:replay:input:fixture"),
            ]
        );
    }

    #[test]
    fn unselected_task_preserves_optional_pin_compatibility() {
        let repo = tempdir().unwrap();
        let contract = contract(
            "version: 1\nproject:\n  name: test\ntasks:\n  governed:\n    command:\n      exe: true\n  ordinary:\n    command:\n      exe: true\n",
        );
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        governed:\n          on_insufficient: deny\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("ordinary"),
        );

        assert!(!result.required);
        assert_eq!(result.decision, "allow");
        assert_eq!(result.coverage, "not_required");
    }

    #[test]
    fn unknown_selectors_remain_contextual_policy_evidence() {
        let repo = tempdir().unwrap();
        let contract = contract(
            "version: 1\nproject:\n  name: test\ntasks:\n  verify:\n    command:\n      exe: true\n",
        );
        let policy = policy(
            "policies:\n  replay_inputs:\n    identity:\n      tasks:\n        missing:\n          on_insufficient: deny\n      workflows:\n        absent:\n          on_insufficient: review\n",
        );

        let result = evaluate_replay_input_policy(
            &contract,
            repo.path(),
            &policy,
            ReplayInputPolicySubject::Task("verify"),
        );

        assert!(result.required);
        assert_eq!(result.decision, "deny");
        assert_eq!(result.coverage, "insufficient");
        assert_eq!(result.unknown_selectors.len(), 2);
        assert_eq!(result.unknown_selectors[0].reason, "unknown_task_selector");
        assert_eq!(
            result.unknown_selectors[1].reason,
            "unknown_workflow_selector"
        );
    }
}
