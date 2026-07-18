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

use crate::contract_drift::merge_check_id_for_lane_task;
use crate::schema::Contract;
use crate::semantic_identity::semantic_contract_identity;
use serde::Serialize;
use sha2::Digest;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CiProjection {
    pub schema_version: u8,
    pub semantic_contract_identity: String,
    pub workflow: String,
    pub task: String,
    pub mode: String,
    /// The operating system selected for this projection, independent of a provider runner label.
    pub target_os: String,
    pub merge_check_ids: Vec<String>,
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
    mode: &'a str,
    target_os: &'a str,
    merge_check_ids: &'a [String],
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
    let semantic_contract_identity = semantic_contract_identity(contract)?;
    let merge_check_ids = vec![merge_check_id_for_lane_task(&task)];
    let proof_required = workflow.proof.claim_value().is_some();
    let proof_claim = workflow.proof.claim_value().map(str::to_string);
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
        mode: mode.to_string(),
        target_os: target_os.to_string(),
        merge_check_ids,
        proof_required,
        proof_claim,
        bootstrap,
        governance: CiProjectionGovernance {
            agent_admission: CiProjectionAdmission {
                decision: String::from("unresolved"),
                basis: Vec::new(),
            },
            proof_assurance: None,
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
                mode: &projection.mode,
                target_os: &projection.target_os,
                merge_check_ids: &projection.merge_check_ids,
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
}
