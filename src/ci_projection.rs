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
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use crate::contract_drift::{merge_check_id_for_lane_task, merge_check_id_for_refusal_canary};
use crate::schema::{Backend, Contract, TaskRuntimeKind, ToolchainFulfillmentSource};
use crate::semantic_identity::semantic_contract_identity;
use serde::Serialize;
use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjection {
    pub schema_version: u8,
    pub semantic_contract_identity: String,
    pub workflow: String,
    pub task: String,
    /// The selected workflow run task's execution shape. Provider adapters use this to preserve
    /// the runner's finite-task versus service-runtime boundary without reinterpreting the contract.
    pub run_execution: CiProjectionRunExecution,
    pub mode: String,
    /// The operating system selected for this projection, independent of a provider runner label.
    pub target_os: String,
    pub merge_check_ids: Vec<String>,
    pub refusal_canaries: Vec<CiProjectionRefusalCanary>,
    /// Contract-owned toolchains required by the selected workflow closure.
    pub toolchains: Vec<CiProjectionToolchain>,
    pub proof_required: bool,
    pub proof_claim: Option<String>,
    pub bootstrap: CiProjectionBootstrap,
    pub governance: CiProjectionGovernance,
    pub ownership: CiProjectionOwnership,
    pub identity: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionBootstrap {
    pub source_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CiProjectionRunExecution {
    FiniteTask,
    ServiceRuntime,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionRefusalCanary {
    pub kind: String,
    pub target: String,
    pub merge_check_id: String,
}

/// A provider-neutral toolchain requirement. `execution_scopes` keeps ownership explicit so an
/// adapter only provisions toolchains that execute on the provider runner.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionToolchain {
    pub name: String,
    pub source: String,
    pub version: String,
    pub execution_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionOwnership {
    pub ota_owned: Vec<String>,
    pub provider_owned: Vec<String>,
    pub required_bindings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionGovernance {
    pub agent_admission: CiProjectionAdmission,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_assurance: Option<CiProjectionProofAssurance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_input_policy: Option<CiProjectionReplayInputPolicyRequirement>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionReplayInputPolicyRequirement {
    pub policy_identity: String,
    pub rule_identities: Vec<String>,
    pub selected_closure: Vec<String>,
    pub unknown_selector_identities: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionAdmission {
    pub decision: String,
    pub basis: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjectionProofAssurance {
    pub status: String,
    pub policy_decision: String,
    pub basis: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CiProjectionIdentity<'a> {
    version: u8,
    semantic_contract_identity: &'a str,
    workflow: &'a str,
    task: &'a str,
    run_execution: &'a CiProjectionRunExecution,
    mode: &'a str,
    target_os: &'a str,
    merge_check_ids: &'a [String],
    refusal_canaries: &'a [CiProjectionRefusalCanary],
    toolchains: &'a [CiProjectionToolchain],
    proof_required: bool,
    proof_claim: &'a Option<String>,
    bootstrap: &'a CiProjectionBootstrap,
    governance: &'a CiProjectionGovernance,
}

pub(crate) fn build_ci_projection(
    contract: &Contract,
    workflow_name: &str,
    mode: &str,
    target_os: &str,
) -> Result<CiProjection, String> {
    if !matches!(mode, "native" | "container" | "remote") {
        return Err(format!("unsupported execution mode `{mode}`"));
    }
    if !matches!(target_os, "linux" | "macos" | "windows") {
        return Err(format!("unsupported projection target OS `{target_os}`"));
    }
    let workflows = contract
        .workflows
        .as_ref()
        .ok_or_else(|| String::from("contract declares no workflows"))?;
    let workflow = workflows
        .items
        .get(workflow_name)
        .ok_or_else(|| format!("workflow `{workflow_name}` is not declared"))?;
    let task = workflow
        .run
        .as_ref()
        .map(|run| run.task.clone())
        .ok_or_else(|| format!("workflow `{workflow_name}` does not declare `run.task`"))?;
    if !contract.tasks.contains_key(&task) {
        return Err(format!(
            "workflow `{workflow_name}` references missing task `{task}`"
        ));
    }
    let backend = match mode {
        "native" => Backend::Native,
        "container" => Backend::Container,
        "remote" => Backend::Remote,
        _ => unreachable!("projection mode was validated above"),
    };
    let run_execution = contract
        .tasks
        .get(task.as_str())
        .and_then(|task| task.service_runtime_for_backend(backend))
        .is_some_and(|runtime| runtime.kind == TaskRuntimeKind::Service)
        .then_some(CiProjectionRunExecution::ServiceRuntime)
        .unwrap_or(CiProjectionRunExecution::FiniteTask);
    let unsupported_task = contract
        .selected_workflow_task_closure_names(Some(workflow_name))
        .into_iter()
        .filter_map(|task_name| {
            contract
                .tasks
                .get(task_name.as_str())
                .map(|candidate| (task_name, candidate))
        })
        // Aggregate nodes orchestrate their concrete closure; they do not execute directly.
        .find(|(_, candidate)| {
            candidate.aggregate.is_none()
                && (!candidate.active_for_os(target_os)
                    || !candidate.supports_execution_backend(
                        contract.execution.as_ref(),
                        backend,
                        target_os,
                    )
                    || !contract.task_active_for_backend_on_os(candidate, backend, target_os))
        })
        .map(|(task_name, _)| task_name);
    if let Some(unsupported_task) = unsupported_task {
        return Err(format!(
            "workflow `{workflow_name}` task closure member `{unsupported_task}` does not support `{mode}` execution on `{target_os}`"
        ));
    }
    let semantic_contract_identity = semantic_contract_identity(contract)?;
    let mut refusal_canaries = contract
        .agent
        .as_ref()
        .map(|agent| {
            agent
                .refusal_canaries
                .iter()
                .filter_map(|canary| {
                    canary
                        .task
                        .as_ref()
                        .map(|target| CiProjectionRefusalCanary {
                            kind: String::from("task"),
                            target: target.clone(),
                            merge_check_id: merge_check_id_for_refusal_canary("task", target),
                        })
                        .or_else(|| {
                            canary
                                .workflow
                                .as_ref()
                                .map(|target| CiProjectionRefusalCanary {
                                    kind: String::from("workflow"),
                                    target: target.clone(),
                                    merge_check_id: merge_check_id_for_refusal_canary(
                                        "workflow", target,
                                    ),
                                })
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    refusal_canaries.sort_by(|left, right| {
        (&left.kind, &left.target, &left.merge_check_id).cmp(&(
            &right.kind,
            &right.target,
            &right.merge_check_id,
        ))
    });
    refusal_canaries.dedup_by(|left, right| left.kind == right.kind && left.target == right.target);
    let mut refusal_check_ids = BTreeSet::new();
    for canary in &refusal_canaries {
        if !refusal_check_ids.insert(canary.merge_check_id.clone()) {
            return Err(format!(
                "refusal canaries produce the same merge check identity `{}`; rename one target to avoid a normalized identity collision",
                canary.merge_check_id
            ));
        }
    }
    let mut merge_check_ids = vec![merge_check_id_for_lane_task(&task)];
    merge_check_ids.extend(
        refusal_canaries
            .iter()
            .map(|canary| canary.merge_check_id.clone()),
    );
    let proof_required = workflow.proof.claim_value().is_some();
    let proof_claim = workflow.proof.claim_value().map(str::to_string);
    let toolchains = selected_projection_toolchains(contract, workflow_name, backend, target_os)?;
    let bootstrap = contract
        .agent
        .as_ref()
        .and_then(|agent| agent.bootstrap.as_ref())
        .and_then(|bootstrap| bootstrap.ota.as_ref())
        .and_then(|ota| ota.effective_source())
        .map(|source| match source {
            crate::schema::AgentBootstrapOtaSource::Version { version } => CiProjectionBootstrap {
                source_kind: String::from("version"),
                source_identity: Some(version),
            },
            crate::schema::AgentBootstrapOtaSource::GitRev { rev } => CiProjectionBootstrap {
                source_kind: String::from("git_rev"),
                source_identity: Some(rev),
            },
            crate::schema::AgentBootstrapOtaSource::Branch { branch } => CiProjectionBootstrap {
                source_kind: String::from("branch"),
                source_identity: Some(branch),
            },
        })
        .unwrap_or(CiProjectionBootstrap {
            source_kind: String::from("unspecified"),
            source_identity: None,
        });
    let mut projection = CiProjection {
        schema_version: 1,
        semantic_contract_identity,
        workflow: workflow_name.to_string(),
        task,
        run_execution,
        mode: mode.to_string(),
        target_os: target_os.to_string(),
        merge_check_ids,
        refusal_canaries,
        toolchains,
        proof_required,
        proof_claim,
        bootstrap,
        governance: CiProjectionGovernance {
            agent_admission: CiProjectionAdmission {
                decision: String::from("unresolved"),
                basis: Vec::new(),
            },
            proof_assurance: None,
            replay_input_policy: None,
        },
        ownership: CiProjectionOwnership {
            ota_owned: vec![
                String::from("bootstrap"),
                String::from("governance_lane"),
                String::from("merge_check_identity"),
                String::from("proof_boundaries"),
            ],
            provider_owned: vec![
                String::from("scheduling"),
                String::from("credentials"),
                String::from("execution_infrastructure"),
                String::from("delivery"),
            ],
            required_bindings: vec![
                String::from("projection_identity"),
                String::from("target_os"),
            ],
        },
        identity: String::new(),
    };
    refresh_ci_projection_identity(&mut projection)?;
    Ok(projection)
}

pub(crate) fn refresh_ci_projection_identity(projection: &mut CiProjection) -> Result<(), String> {
    projection.identity = format!(
        "sha256:{:x}",
        sha2::Sha256::digest(
            serde_json::to_vec(&CiProjectionIdentity {
                version: 1,
                semantic_contract_identity: &projection.semantic_contract_identity,
                workflow: &projection.workflow,
                task: &projection.task,
                run_execution: &projection.run_execution,
                mode: &projection.mode,
                target_os: &projection.target_os,
                merge_check_ids: &projection.merge_check_ids,
                refusal_canaries: &projection.refusal_canaries,
                toolchains: &projection.toolchains,
                proof_required: projection.proof_required,
                proof_claim: &projection.proof_claim,
                bootstrap: &projection.bootstrap,
                governance: &projection.governance,
            })
            .map_err(|error| format!("could not serialize CI projection identity: {error}"))?
        )
    );
    Ok(())
}

fn selected_projection_toolchains(
    contract: &Contract,
    workflow_name: &str,
    backend: Backend,
    target_os: &str,
) -> Result<Vec<CiProjectionToolchain>, String> {
    let execution_scope = match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    };
    let mut scopes = BTreeMap::<String, BTreeSet<String>>::new();
    for task_name in contract.selected_workflow_task_closure_names(Some(workflow_name)) {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        if task.aggregate.is_some() {
            continue;
        }
        let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
        for toolchain_name in contract.task_toolchain_names_for_execution_for_os(
            task,
            backend,
            context_name,
            target_os,
        ) {
            scopes
                .entry(toolchain_name)
                .or_default()
                .insert(execution_scope.to_string());
        }
    }
    scopes
        .into_iter()
        .filter_map(|(name, execution_scopes)| {
            let toolchain = contract.toolchains.get(name.as_str())?;
            toolchain.required_for_os(target_os).then(|| {
                Ok(CiProjectionToolchain {
                    name,
                    source: toolchain
                        .fulfillment_source()
                        .map(toolchain_source_label)
                        .unwrap_or("unspecified")
                        .to_string(),
                    version: toolchain.version_for_os(target_os).to_string(),
                    execution_scopes: execution_scopes.into_iter().collect(),
                })
            })
        })
        .collect()
}

fn toolchain_source_label(source: ToolchainFulfillmentSource) -> &'static str {
    match source {
        ToolchainFulfillmentSource::Rustup => "rustup",
        ToolchainFulfillmentSource::Corepack => "corepack",
        ToolchainFulfillmentSource::Sdkman => "sdkman",
        ToolchainFulfillmentSource::Uv => "uv",
        ToolchainFulfillmentSource::Go => "go",
        ToolchainFulfillmentSource::Ruby => "ruby",
        ToolchainFulfillmentSource::Dotnet => "dotnet",
        ToolchainFulfillmentSource::Mise => "mise",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_reuses_the_canonical_semantic_snapshot_identity() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: identity-fixture
tasks:
  verify:
    run: echo verify
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        )
        .expect("fixture contract should parse");
        let projection = build_ci_projection(&contract, "verify", "native", "linux")
            .expect("projection should build");
        assert_eq!(
            projection.semantic_contract_identity,
            semantic_contract_identity(&contract).expect("semantic identity should resolve")
        );
    }

    #[test]
    fn projection_carries_declared_refusal_canaries_with_stable_check_identity() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: canary-fixture
tasks:
  verify:
    run: echo verify
  publish:
    run: echo publish
workflows:
  default: verify
  verify:
    run:
      task: verify
  release:
    run:
      task: publish
agent:
  refusal_canaries:
    - task: publish
    - workflow: release
"#,
        )
        .expect("fixture contract should parse");
        let projection = build_ci_projection(&contract, "verify", "native", "linux")
            .expect("projection should build");

        assert_eq!(projection.refusal_canaries.len(), 2);
        assert_eq!(
            projection.refusal_canaries[0].merge_check_id,
            "ota.refusal-canary.task.publish"
        );
        assert_eq!(
            projection.refusal_canaries[1].merge_check_id,
            "ota.refusal-canary.workflow.release"
        );
        assert!(
            projection
                .merge_check_ids
                .contains(&String::from("ota.refusal-canary.task.publish"))
        );
        assert!(
            projection
                .merge_check_ids
                .contains(&String::from("ota.refusal-canary.workflow.release"))
        );
    }

    #[test]
    fn projection_carries_selected_closure_toolchains_with_target_os_version() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: toolchain-fixture
toolchains:
  go:
    version: ">=1.26,<1.27"
    fulfillment:
      source: go
      mode: none
    platforms:
      macos:
        version: "1.26.1"
tasks:
  setup:
    run: go mod download
    requirements:
      toolchains: [go]
  verify:
    run: go test ./...
    depends_on: [setup]
    requirements:
      toolchains: [go]
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        )
        .expect("fixture contract should parse");

        let projection = build_ci_projection(&contract, "verify", "native", "macos")
            .expect("projection should build");
        assert_eq!(projection.toolchains.len(), 1);
        assert_eq!(projection.toolchains[0].name, "go");
        assert_eq!(projection.toolchains[0].source, "go");
        assert_eq!(projection.toolchains[0].version, "1.26.1");
        assert_eq!(projection.toolchains[0].execution_scopes, ["native"]);
    }

    #[test]
    fn projection_keeps_container_owned_toolchains_out_of_provider_scope() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: container-toolchain-fixture
execution:
  contexts:
    app:
      backend: container
      container:
        image: ruby:3.3
toolchains:
  ruby:
    version: "3.3.11"
    fulfillment:
      source: ruby
      mode: run
tasks:
  verify:
    context: app
    run: ruby -v
    execution:
      default_mode: container
    requirements:
      toolchains: [ruby]
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        )
        .expect("fixture contract should parse");

        let projection = build_ci_projection(&contract, "verify", "container", "linux")
            .expect("container projection should build");
        assert_eq!(projection.toolchains[0].execution_scopes, ["container"]);
    }

    #[test]
    fn projection_rejects_a_target_os_outside_the_selected_context_scope() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: context-platform-fixture
execution:
  contexts:
    host:
      backend: native
      only_on: [linux, macos]
tasks:
  verify:
    context: host
    run: echo verify
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        )
        .expect("fixture contract should parse");

        let error = build_ci_projection(&contract, "verify", "native", "windows")
            .expect_err("Windows projection must reject a Linux/macOS-only context");
        assert!(error.contains("does not support `native` execution on `windows`"));
    }

    #[test]
    fn projection_rejects_normalized_refusal_canary_identity_collisions() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: collision-fixture
tasks:
  verify:
    run: echo verify
  publish-release:
    run: echo publish
  publish_release:
    run: echo publish
workflows:
  default: verify
  verify:
    run:
      task: verify
agent:
  refusal_canaries:
    - task: publish-release
    - task: publish_release
"#,
        )
        .expect("fixture contract should parse");

        let error = build_ci_projection(&contract, "verify", "native", "linux")
            .expect_err("normalized identity collisions must be rejected");
        assert!(error.contains("ota.refusal-canary.task.publish-release"));
    }
}
