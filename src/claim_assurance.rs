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

/// Evaluates a declared-safe task without treating its own declaration as corroboration.
pub fn agent_safety_claim(
    task_name: &str,
    effective_safe: bool,
    unsafe_closure_tasks: Vec<String>,
) -> ClaimAssuranceRecord {
    ClaimAssuranceRecord {
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
    }
}

#[cfg(test)]
mod tests {
    use super::agent_safety_claim;

    #[test]
    fn declared_safe_closure_is_unknown_without_independent_evidence() {
        let claim = agent_safety_claim("verify", true, Vec::new());

        assert_eq!(claim.declaration.value, "safe");
        assert_eq!(claim.closure.status, "safe");
        assert_eq!(claim.assurance.status, "unknown");
        assert_eq!(claim.assurance.gaps, ["non_self_origin_evidence"]);
        assert_eq!(claim.policy.decision, "allow");
    }

    #[test]
    fn unsafe_closure_remains_separate_from_assurance() {
        let claim = agent_safety_claim("verify", false, vec![String::from("publish")]);

        assert_eq!(claim.closure.status, "unsafe");
        assert_eq!(claim.closure.blockers, ["publish"]);
        assert_eq!(claim.assurance.status, "unknown");
    }
}
