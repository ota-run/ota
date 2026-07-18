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
//   Licensed under the Apache License, Version 2.0. See LICENSE for the full license.
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.

//! Canonical, policy-independent assessment of contract claims.

use crate::schema::{TaskActionSpec, TaskSpec};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimAssuranceRecord {
    pub subject: ClaimSubject,
    pub family: String,
    pub declaration: ClaimDeclaration,
    pub closure: ClaimClosure,
    pub assurance: ClaimAssurance,
    pub policy: ClaimPolicyDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimSubject {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimDeclaration {
    pub value: String,
    pub evidence_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimClosure {
    pub status: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    pub evidence_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimAssurance {
    pub status: String,
    pub coverage: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<ClaimEvidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contradictions: Vec<ClaimContradiction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimEvidence {
    pub id: String,
    pub source: String,
    pub evidence_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimContradiction {
    pub id: String,
    pub source: String,
    pub evidence_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimPolicyDecision {
    pub decision: String,
    pub basis: Vec<String>,
    pub evidence_class: String,
}

/// The execution boundary a runtime-proof archive must match before it can support a claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofArchiveScope {
    pub workflow: Option<String>,
    pub task: Option<String>,
    pub backend: String,
    pub provider: Option<String>,
    pub lifecycle: Option<String>,
    pub target: Option<String>,
}

/// A content-verified runtime-proof archive supplied by the persistence boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofArchiveCandidate {
    pub identity: String,
    pub contract_snapshot_hash: String,
    pub source_identity: Option<String>,
    pub scope: ProofArchiveScope,
    pub proof_ok: bool,
    pub proof_verdict: String,
}

/// Evaluates a declared-safe task without treating its own declaration as corroboration.
pub fn agent_safety_claim(
    task_name: &str,
    task: &TaskSpec,
    effective_safe: bool,
    unsafe_closure_tasks: Vec<String>,
) -> ClaimAssuranceRecord {
    let mut claim = ClaimAssuranceRecord {
        subject: ClaimSubject {
            kind: String::from("task"),
            name: task_name.to_string(),
        },
        family: String::from("agent_safety"),
        declaration: ClaimDeclaration {
            value: String::from("safe"),
            evidence_class: String::from("asserted"),
        },
        closure: ClaimClosure {
            status: if effective_safe {
                String::from("safe")
            } else {
                String::from("unsafe")
            },
            blockers: unsafe_closure_tasks,
            evidence_class: String::from("derived"),
        },
        // Closure is necessary for admission but is contract-derived. It cannot corroborate the
        // maintainer's safety assertion without independently inspected evidence.
        assurance: ClaimAssurance {
            status: String::from("unknown"),
            coverage: vec![
                String::from("contract_declaration"),
                String::from("execution_closure"),
            ],
            gaps: vec![String::from("non_self_origin_evidence")],
            evidence: vec![ClaimEvidence {
                id: String::from("agent_safe_closure"),
                source: String::from("execution_closure"),
                evidence_class: String::from("derived"),
            }],
            contradictions: Vec::new(),
        },
        policy: ClaimPolicyDecision {
            decision: String::from("allow"),
            basis: vec![String::from("default_compatibility")],
            evidence_class: String::from("derived"),
        },
    };

    apply_typed_action_contradictions(&mut claim, task);
    claim
}

/// Evaluates a workflow's qualified runtime-proof claim from immutable archive candidates.
/// A bounded proof can support this qualified claim; policy still consumes V11.11 boundaries when
/// it needs stronger completion coverage.
pub fn proof_breadth_claim(
    workflow_name: &str,
    declaration_value: &str,
    expected_scope: ProofArchiveScope,
    current_contract_snapshot_hash: Option<&str>,
    current_source_identity: Option<&str>,
    archives: &[ProofArchiveCandidate],
) -> ClaimAssuranceRecord {
    let mut claim = ClaimAssuranceRecord {
        subject: ClaimSubject {
            kind: String::from("workflow"),
            name: workflow_name.to_string(),
        },
        family: String::from("proof_breadth"),
        declaration: ClaimDeclaration {
            value: String::from(declaration_value),
            evidence_class: String::from("asserted"),
        },
        closure: ClaimClosure {
            status: String::from("resolved"),
            blockers: Vec::new(),
            evidence_class: String::from("derived"),
        },
        assurance: ClaimAssurance {
            status: String::from("unknown"),
            coverage: vec![String::from("workflow_proof_declaration")],
            gaps: vec![String::from("immutable_scope_matching_proof_archive")],
            evidence: Vec::new(),
            contradictions: Vec::new(),
        },
        policy: ClaimPolicyDecision {
            decision: String::from("allow"),
            basis: vec![String::from("default_compatibility")],
            evidence_class: String::from("derived"),
        },
    };

    let Some(contract_snapshot_hash) = current_contract_snapshot_hash else {
        claim
            .assurance
            .gaps
            .push(String::from("current_contract_snapshot"));
        return claim;
    };
    let Some(source_identity) = current_source_identity else {
        claim
            .assurance
            .gaps
            .push(String::from("clean_source_identity"));
        return claim;
    };

    let matching_archive = archives.iter().find(|archive| {
        archive.contract_snapshot_hash == contract_snapshot_hash
            && archive.source_identity.as_deref() == Some(source_identity)
            && archive.scope == expected_scope
    });
    let Some(archive) = matching_archive else {
        claim.assurance.gaps = proof_archive_gaps(
            archives,
            contract_snapshot_hash,
            source_identity,
            &expected_scope,
        );
        return claim;
    };

    claim.assurance.coverage.extend([
        String::from("immutable_proof_archive"),
        String::from("contract_snapshot"),
        String::from("clean_source_identity"),
        String::from("resolved_execution_scope"),
        String::from("proof_verdict"),
    ]);
    claim.assurance.evidence.push(ClaimEvidence {
        id: format!("proof_archive:{}", archive.identity),
        source: String::from(".ota/proof/archives"),
        evidence_class: String::from("attested"),
    });
    if archive.proof_ok
        && matches!(
            archive.proof_verdict.as_str(),
            "passed" | "passed_with_unproven_boundaries"
        )
    {
        claim.assurance.status = String::from("supported");
        claim.assurance.gaps.clear();
    } else {
        claim.assurance.status = String::from("contradicted");
        claim.assurance.gaps.clear();
        claim.assurance.contradictions.push(ClaimContradiction {
            id: format!("proof_verdict:{}", archive.proof_verdict),
            source: String::from(".ota/proof/archives"),
            evidence_class: String::from("attested"),
        });
    }
    claim
}

fn proof_archive_gaps(
    archives: &[ProofArchiveCandidate],
    contract_snapshot_hash: &str,
    source_identity: &str,
    expected_scope: &ProofArchiveScope,
) -> Vec<String> {
    if archives.is_empty() {
        return vec![String::from("immutable_proof_archive")];
    }
    if !archives
        .iter()
        .any(|archive| archive.contract_snapshot_hash == contract_snapshot_hash)
    {
        return vec![String::from("matching_contract_snapshot")];
    }
    if !archives.iter().any(|archive| {
        archive.contract_snapshot_hash == contract_snapshot_hash
            && archive.source_identity.as_deref() == Some(source_identity)
    }) {
        return vec![String::from("matching_clean_source_identity")];
    }
    if !archives.iter().any(|archive| {
        archive.contract_snapshot_hash == contract_snapshot_hash
            && archive.source_identity.as_deref() == Some(source_identity)
            && archive.scope == *expected_scope
    }) {
        return vec![String::from("matching_execution_scope")];
    }
    vec![String::from("usable_terminal_proof_verdict")]
}

// Typed actions are structured execution facts. They can contradict an omitted matching effect
// without trying to infer intent from an opaque shell command.
fn apply_typed_action_contradictions(claim: &mut ClaimAssuranceRecord, task: &TaskSpec) {
    let Some(TaskActionSpec::ResetComposeServiceVolume(action)) = task.action.as_ref() else {
        return;
    };
    let volume = action.volume.trim();
    let required_effect = format!("compose_volume:{volume}");
    if task
        .effects
        .adapter_state
        .iter()
        .any(|effect| effect.trim() == required_effect)
    {
        return;
    }

    claim.assurance.status = String::from("contradicted");
    claim
        .assurance
        .coverage
        .push(String::from("structured_action"));
    claim.assurance.gaps.clear();
    claim.assurance.evidence.push(ClaimEvidence {
        id: format!("typed_action:reset_compose_service_volume:{}", volume),
        source: String::from("task.action"),
        evidence_class: String::from("derived"),
    });
    claim.assurance.contradictions.push(ClaimContradiction {
        id: format!("missing_adapter_state:{required_effect}"),
        source: String::from("task.action"),
        evidence_class: String::from("derived"),
    });
}

#[cfg(test)]
mod tests {
    use super::{
        ProofArchiveCandidate, ProofArchiveScope, agent_safety_claim, proof_breadth_claim,
    };
    use crate::schema::TaskSpec;

    #[test]
    fn declared_safe_closure_is_unknown_without_independent_evidence() {
        let task: TaskSpec = serde_yaml::from_str("run: printf verify").unwrap();
        let claim = agent_safety_claim("verify", &task, true, Vec::new());

        assert_eq!(claim.declaration.value, "safe");
        assert_eq!(claim.closure.status, "safe");
        assert_eq!(claim.assurance.status, "unknown");
        assert_eq!(claim.assurance.gaps, ["non_self_origin_evidence"]);
        assert_eq!(claim.policy.decision, "allow");
    }

    #[test]
    fn unsafe_closure_remains_separate_from_assurance() {
        let task: TaskSpec = serde_yaml::from_str("run: printf verify").unwrap();
        let claim = agent_safety_claim("verify", &task, false, vec![String::from("publish")]);

        assert_eq!(claim.closure.status, "unsafe");
        assert_eq!(claim.closure.blockers, ["publish"]);
        assert_eq!(claim.assurance.status, "unknown");
    }

    #[test]
    fn typed_volume_reset_without_matching_effect_is_contradicted() {
        let task: TaskSpec = serde_yaml::from_str(
            r#"
action:
  kind: reset_compose_service_volume
  service: web
  volume: node_modules
"#,
        )
        .unwrap();
        let claim = agent_safety_claim("reset", &task, true, Vec::new());

        assert_eq!(claim.assurance.status, "contradicted");
        assert_eq!(
            claim.assurance.contradictions[0].id,
            "missing_adapter_state:compose_volume:node_modules"
        );
    }

    #[test]
    fn typed_volume_reset_with_matching_effect_remains_unknown() {
        let task: TaskSpec = serde_yaml::from_str(
            r#"
action:
  kind: reset_compose_service_volume
  service: web
  volume: " node_modules "
effects:
  adapter_state:
    - compose_volume:node_modules
"#,
        )
        .unwrap();
        let claim = agent_safety_claim("reset", &task, true, Vec::new());

        assert_eq!(claim.assurance.status, "unknown");
        assert!(claim.assurance.contradictions.is_empty());
    }

    fn proof_scope() -> ProofArchiveScope {
        ProofArchiveScope {
            workflow: Some(String::from("verify")),
            task: Some(String::from("gate")),
            backend: String::from("native"),
            provider: None,
            lifecycle: None,
            target: None,
        }
    }

    fn matching_archive() -> ProofArchiveCandidate {
        ProofArchiveCandidate {
            identity: String::from("sha256:archive"),
            contract_snapshot_hash: String::from("sha256:contract"),
            source_identity: Some(String::from("git:source")),
            scope: proof_scope(),
            proof_ok: true,
            proof_verdict: String::from("passed_with_unproven_boundaries"),
        }
    }

    #[test]
    fn qualified_proof_requires_immutable_matching_archive() {
        let claim = proof_breadth_claim(
            "verify",
            "qualified_runtime_proof",
            proof_scope(),
            Some("sha256:contract"),
            Some("git:source"),
            &[matching_archive()],
        );

        assert_eq!(claim.assurance.status, "supported");
        assert!(claim.assurance.gaps.is_empty());
        assert_eq!(claim.assurance.evidence[0].evidence_class, "attested");
    }

    #[test]
    fn qualified_proof_rejects_stale_or_scope_mismatched_archive() {
        let mut stale = matching_archive();
        stale.contract_snapshot_hash = String::from("sha256:old");
        let claim = proof_breadth_claim(
            "verify",
            "qualified_runtime_proof",
            proof_scope(),
            Some("sha256:contract"),
            Some("git:source"),
            &[stale],
        );
        assert_eq!(claim.assurance.status, "unknown");
        assert_eq!(claim.assurance.gaps, ["matching_contract_snapshot"]);

        let mut wrong_scope = matching_archive();
        wrong_scope.scope.backend = String::from("container");
        let claim = proof_breadth_claim(
            "verify",
            "qualified_runtime_proof",
            proof_scope(),
            Some("sha256:contract"),
            Some("git:source"),
            &[wrong_scope],
        );
        assert_eq!(claim.assurance.status, "unknown");
        assert_eq!(claim.assurance.gaps, ["matching_execution_scope"]);

        let mut stale_source = matching_archive();
        stale_source.source_identity = Some(String::from("git:old-source"));
        let claim = proof_breadth_claim(
            "verify",
            "qualified_runtime_proof",
            proof_scope(),
            Some("sha256:contract"),
            Some("git:source"),
            &[stale_source],
        );
        assert_eq!(claim.assurance.status, "unknown");
        assert_eq!(claim.assurance.gaps, ["matching_clean_source_identity"]);
    }

    #[test]
    fn qualified_proof_marks_a_matching_failed_archive_contradicted() {
        let mut failed = matching_archive();
        failed.proof_ok = false;
        failed.proof_verdict = String::from("failed");
        let claim = proof_breadth_claim(
            "verify",
            "qualified_runtime_proof",
            proof_scope(),
            Some("sha256:contract"),
            Some("git:source"),
            &[failed],
        );

        assert_eq!(claim.assurance.status, "contradicted");
        assert_eq!(claim.assurance.contradictions[0].id, "proof_verdict:failed");
    }
}
