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

//! Runner-authored prerequisite provenance for runtime proof.
//!
//! This module intentionally owns semantic evaluation only. JSON schema can validate the record
//! shape, but cannot establish causal producer-to-assertion bindings across graph edges.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::semantic_identity::semantic_contract_identity;

pub const EXECUTION_BOUNDARY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrerequisiteClass {
    Filesystem,
    DependencyCache,
    Service,
    Volume,
    Environment,
    ImageRuntime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreconditionState {
    Absent,
    Present,
    Cleared,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryEvidenceClass {
    Derived,
    Attested,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrerequisiteState {
    CreatedThisRun,
    DeclaredImmutableInput,
    VerifiedReused,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetFreshness {
    ColdStartVerified,
    PersistentStateReused,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationPosture {
    FullyDerived,
    CacheAssisted,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImmutableInputPosture {
    None,
    InheritedImmutable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivationPosture {
    pub materialization: MaterializationPosture,
    pub immutable_inputs: ImmutableInputPosture,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryMaterialization {
    pub edge_id: String,
    pub producer_execution: String,
    pub identity: String,
    pub evidence_class: BoundaryEvidenceClass,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryAssertion {
    pub edge_id: String,
    pub consumer_execution: String,
    pub identity: String,
    pub evidence_class: BoundaryEvidenceClass,
    pub sequence: u64,
    pub established_by_edge_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryPrerequisite {
    pub id: String,
    pub class: PrerequisiteClass,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_producers: Vec<String>,
    pub precondition: PreconditionState,
    /// Identity observed at the selected boundary before a producer ran or reuse was admitted.
    /// It lets later workflow phases verify a reused boundary without treating it as produced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precondition_identity: Option<String>,
    pub state: PrerequisiteState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub materializations: Vec<BoundaryMaterialization>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertions: Vec<BoundaryAssertion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_established_by_edge_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient_boundary: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryEdgeKind {
    Produced,
    AssertedAt,
    Consumed,
    DerivedFrom,
    ReusedFrom,
    ClearedBefore,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryEdge {
    pub id: String,
    pub kind: BoundaryEdgeKind,
    pub prerequisite_id: String,
    pub execution_id: String,
    pub boundary_id: String,
    pub scope: String,
    pub identity: String,
    pub evidence_class: BoundaryEvidenceClass,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionBoundaryRecord {
    pub schema_version: u32,
    pub identity: String,
    pub asserted_target_closure: Vec<String>,
    pub derivation_input_closure: Vec<String>,
    pub prerequisites: Vec<BoundaryPrerequisite>,
    pub edges: Vec<BoundaryEdge>,
    pub target_freshness: TargetFreshness,
    pub derivation_posture: DerivationPosture,
}

/// Validate the causal graph, derive its summaries, and content-address the semantic record.
pub fn evaluate_execution_boundary(
    mut record: ExecutionBoundaryRecord,
) -> Result<ExecutionBoundaryRecord, String> {
    if record.schema_version != EXECUTION_BOUNDARY_SCHEMA_VERSION {
        return Err(format!(
            "execution boundary schema version `{}` is unsupported",
            record.schema_version
        ));
    }

    normalize_closure(
        &mut record.asserted_target_closure,
        "asserted_target_closure",
    )?;
    normalize_closure(
        &mut record.derivation_input_closure,
        "derivation_input_closure",
    )?;
    normalize_prerequisites(&mut record.prerequisites)?;
    validate_edges(&record.edges)?;
    validate_prerequisites(&record)?;

    record.target_freshness = derive_target_freshness(&record);
    record.derivation_posture = derive_derivation_posture(&record);
    record.identity.clear();
    record.identity = semantic_contract_identity(&record)
        .map_err(|error| format!("failed to derive execution boundary identity: {error}"))?;
    Ok(record)
}

fn normalize_prerequisites(prerequisites: &mut [BoundaryPrerequisite]) -> Result<(), String> {
    for prerequisite in &mut *prerequisites {
        normalize_closure(&mut prerequisite.declared_artifacts, "declared_artifacts")?;
        normalize_closure(&mut prerequisite.declared_producers, "declared_producers")?;
    }
    prerequisites.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(())
}

fn normalize_closure(values: &mut Vec<String>, label: &str) -> Result<(), String> {
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(format!(
            "execution boundary `{label}` must not contain an empty identity"
        ));
    }
    values.sort();
    values.dedup();
    Ok(())
}

fn validate_edges(edges: &[BoundaryEdge]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    for edge in edges {
        if edge.id.trim().is_empty() || !ids.insert(edge.id.as_str()) {
            return Err(format!(
                "execution boundary edge id `{}` is not unique",
                edge.id
            ));
        }
        if !sequences.insert(edge.sequence) {
            return Err(format!(
                "execution boundary edge sequence `{}` is not unique",
                edge.sequence
            ));
        }
    }
    Ok(())
}

fn validate_prerequisites(record: &ExecutionBoundaryRecord) -> Result<(), String> {
    let prerequisites = record
        .prerequisites
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    if prerequisites.len() != record.prerequisites.len() {
        return Err(String::from(
            "execution boundary prerequisite ids must be unique",
        ));
    }

    let edges = record
        .edges
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    for id in &record.asserted_target_closure {
        let prerequisite = prerequisites
            .get(id.as_str())
            .ok_or_else(|| format!("asserted target prerequisite `{id}` has no evidence record"))?;
        validate_prerequisite_assertions(prerequisite, &edges)?;
    }
    Ok(())
}

fn validate_prerequisite_assertions(
    prerequisite: &BoundaryPrerequisite,
    edges: &BTreeMap<&str, &BoundaryEdge>,
) -> Result<(), String> {
    if prerequisite.state == PrerequisiteState::VerifiedReused
        && (prerequisite.precondition != PreconditionState::Present
            || prerequisite.precondition_identity.is_none())
    {
        return Err(format!(
            "execution boundary prerequisite `{}` cannot claim verified reuse without a present precondition identity",
            prerequisite.id
        ));
    }
    let materializations = prerequisite
        .materializations
        .iter()
        .map(|item| (item.edge_id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    if materializations.len() != prerequisite.materializations.len() {
        return Err(format!(
            "prerequisite `{}` materialization edge ids must be unique",
            prerequisite.id
        ));
    }
    for materialization in &prerequisite.materializations {
        let edge = edges.get(materialization.edge_id.as_str()).ok_or_else(|| {
            format!(
                "prerequisite `{}` materialization references unknown edge `{}`",
                prerequisite.id, materialization.edge_id
            )
        })?;
        if edge.kind != BoundaryEdgeKind::Produced
            || edge.prerequisite_id != prerequisite.id
            || edge.execution_id != materialization.producer_execution
            || edge.identity != materialization.identity
            || edge.sequence != materialization.sequence
        {
            return Err(format!(
                "prerequisite `{}` materialization does not match its produced edge",
                prerequisite.id
            ));
        }
    }
    for assertion in &prerequisite.assertions {
        let producer = materializations
            .get(assertion.established_by_edge_id.as_str())
            .ok_or_else(|| {
                format!(
                    "prerequisite `{}` assertion references unknown producer edge `{}`",
                    prerequisite.id, assertion.established_by_edge_id
                )
            })?;
        if producer.identity != assertion.identity || producer.sequence >= assertion.sequence {
            return Err(format!(
                "prerequisite `{}` assertion is not causally bound to its producer identity",
                prerequisite.id
            ));
        }
        let edge = edges.get(assertion.edge_id.as_str()).ok_or_else(|| {
            format!(
                "prerequisite `{}` assertion references unknown edge `{}`",
                prerequisite.id, assertion.edge_id
            )
        })?;
        if edge.kind != BoundaryEdgeKind::AssertedAt
            || edge.prerequisite_id != prerequisite.id
            || edge.execution_id != assertion.consumer_execution
            || edge.identity != assertion.identity
            || edge.sequence != assertion.sequence
        {
            return Err(format!(
                "prerequisite `{}` assertion does not match its asserted-at edge",
                prerequisite.id
            ));
        }
    }
    if let Some(terminal) = prerequisite.terminal_established_by_edge_id.as_ref() {
        let latest = prerequisite
            .materializations
            .iter()
            .max_by_key(|materialization| materialization.sequence)
            .map(|materialization| materialization.edge_id.as_str());
        if latest != Some(terminal.as_str()) {
            return Err(format!(
                "prerequisite `{}` terminal producer does not match the latest materialization",
                prerequisite.id
            ));
        }
    }
    Ok(())
}

fn derive_target_freshness(record: &ExecutionBoundaryRecord) -> TargetFreshness {
    let asserted = record
        .prerequisites
        .iter()
        .filter(|item| {
            record
                .asserted_target_closure
                .binary_search(&item.id)
                .is_ok()
        })
        .collect::<Vec<_>>();
    if asserted
        .iter()
        .any(|item| item.state == PrerequisiteState::VerifiedReused)
    {
        return TargetFreshness::PersistentStateReused;
    }
    if asserted.is_empty()
        || asserted.iter().any(|item| {
            item.state == PrerequisiteState::Unknown
                || (item.state == PrerequisiteState::CreatedThisRun
                    && !matches!(
                        item.precondition,
                        PreconditionState::Absent | PreconditionState::Cleared
                    ))
                || (item.state == PrerequisiteState::CreatedThisRun && item.assertions.is_empty())
                || (item.state == PrerequisiteState::DeclaredImmutableInput
                    && item.assertions.is_empty())
        })
    {
        return TargetFreshness::Unknown;
    }
    TargetFreshness::ColdStartVerified
}

fn derive_derivation_posture(record: &ExecutionBoundaryRecord) -> DerivationPosture {
    let cache_assisted = record
        .edges
        .iter()
        .any(|edge| edge.kind == BoundaryEdgeKind::ReusedFrom);
    let unknown = record
        .prerequisites
        .iter()
        .any(|item| item.state == PrerequisiteState::Unknown);
    DerivationPosture {
        materialization: if unknown {
            MaterializationPosture::Unknown
        } else if cache_assisted {
            MaterializationPosture::CacheAssisted
        } else {
            MaterializationPosture::FullyDerived
        },
        immutable_inputs: if record
            .prerequisites
            .iter()
            .any(|item| item.state == PrerequisiteState::Unknown)
        {
            ImmutableInputPosture::Unknown
        } else if record
            .prerequisites
            .iter()
            .any(|item| item.state == PrerequisiteState::DeclaredImmutableInput)
        {
            ImmutableInputPosture::InheritedImmutable
        } else {
            ImmutableInputPosture::None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_record() -> ExecutionBoundaryRecord {
        ExecutionBoundaryRecord {
            schema_version: EXECUTION_BOUNDARY_SCHEMA_VERSION,
            identity: String::new(),
            asserted_target_closure: vec![String::from("filesystem:.venv")],
            derivation_input_closure: vec![String::from("dependency_cache:pip")],
            prerequisites: vec![BoundaryPrerequisite {
                id: String::from("filesystem:.venv"),
                class: PrerequisiteClass::Filesystem,
                declared_artifacts: Vec::new(),
                declared_producers: Vec::new(),
                precondition: PreconditionState::Cleared,
                precondition_identity: None,
                state: PrerequisiteState::CreatedThisRun,
                materializations: vec![BoundaryMaterialization {
                    edge_id: String::from("edge:produce"),
                    producer_execution: String::from("task:setup"),
                    identity: String::from("sha256:venv"),
                    evidence_class: BoundaryEvidenceClass::Attested,
                    sequence: 2,
                }],
                assertions: vec![BoundaryAssertion {
                    edge_id: String::from("edge:assert"),
                    consumer_execution: String::from("task:run"),
                    identity: String::from("sha256:venv"),
                    evidence_class: BoundaryEvidenceClass::Attested,
                    sequence: 3,
                    established_by_edge_id: String::from("edge:produce"),
                }],
                terminal_established_by_edge_id: Some(String::from("edge:produce")),
                ambient_boundary: None,
            }],
            edges: vec![
                BoundaryEdge {
                    id: String::from("edge:cleared"),
                    kind: BoundaryEdgeKind::ClearedBefore,
                    prerequisite_id: String::from("filesystem:.venv"),
                    execution_id: String::from("task:setup"),
                    boundary_id: String::from("native:host"),
                    scope: String::from("workflow:verify"),
                    identity: String::from("empty"),
                    evidence_class: BoundaryEvidenceClass::Attested,
                    sequence: 1,
                },
                BoundaryEdge {
                    id: String::from("edge:produce"),
                    kind: BoundaryEdgeKind::Produced,
                    prerequisite_id: String::from("filesystem:.venv"),
                    execution_id: String::from("task:setup"),
                    boundary_id: String::from("native:host"),
                    scope: String::from("workflow:verify"),
                    identity: String::from("sha256:venv"),
                    evidence_class: BoundaryEvidenceClass::Attested,
                    sequence: 2,
                },
                BoundaryEdge {
                    id: String::from("edge:assert"),
                    kind: BoundaryEdgeKind::AssertedAt,
                    prerequisite_id: String::from("filesystem:.venv"),
                    execution_id: String::from("task:run"),
                    boundary_id: String::from("native:host"),
                    scope: String::from("workflow:verify"),
                    identity: String::from("sha256:venv"),
                    evidence_class: BoundaryEvidenceClass::Attested,
                    sequence: 3,
                },
                BoundaryEdge {
                    id: String::from("edge:cache"),
                    kind: BoundaryEdgeKind::ReusedFrom,
                    prerequisite_id: String::from("dependency_cache:pip"),
                    execution_id: String::from("task:setup"),
                    boundary_id: String::from("native:host"),
                    scope: String::from("workflow:verify"),
                    identity: String::from("sha256:cache"),
                    evidence_class: BoundaryEvidenceClass::Attested,
                    sequence: 4,
                },
            ],
            target_freshness: TargetFreshness::Unknown,
            derivation_posture: DerivationPosture {
                materialization: MaterializationPosture::Unknown,
                immutable_inputs: ImmutableInputPosture::Unknown,
            },
        }
    }

    #[test]
    fn cache_assisted_derivation_does_not_downgrade_a_fresh_target() {
        let record = evaluate_execution_boundary(base_record()).expect("record should evaluate");
        assert_eq!(record.target_freshness, TargetFreshness::ColdStartVerified);
        assert_eq!(
            record.derivation_posture.materialization,
            MaterializationPosture::CacheAssisted
        );
    }

    #[test]
    fn rejects_assertions_that_do_not_match_their_producer_identity() {
        let mut record = base_record();
        record.prerequisites[0].assertions[0].identity = String::from("sha256:other");
        assert!(
            evaluate_execution_boundary(record)
                .expect_err("mismatched assertion must fail")
                .contains("not causally bound")
        );
    }

    #[test]
    fn declared_immutable_inputs_require_an_identity_assertion_for_cold_proof() {
        let mut record = base_record();
        record.prerequisites[0].state = PrerequisiteState::DeclaredImmutableInput;
        record.prerequisites[0].assertions.clear();
        let record = evaluate_execution_boundary(record).expect("record should evaluate");
        assert_eq!(record.target_freshness, TargetFreshness::Unknown);
    }

    #[test]
    fn declared_producer_ownership_contributes_to_graph_identity() {
        let baseline =
            evaluate_execution_boundary(base_record()).expect("baseline should evaluate");
        let mut changed = base_record();
        changed.prerequisites[0]
            .declared_producers
            .push(String::from("task:other-producer"));
        let changed = evaluate_execution_boundary(changed).expect("changed record should evaluate");
        assert_ne!(baseline.identity, changed.identity);
    }
}
