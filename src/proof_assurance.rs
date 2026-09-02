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
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Semantic reconciliation for runtime-proof negative-control evidence.

use crate::output::{
    ExecutionEvidenceClass, ProofRuntimeDependencyEvidence, ProofRuntimeNegativeControl,
    ProofRuntimeNegativeControlFailureMode, ProofRuntimeNegativeControlOutcome,
    ProofRuntimeNegativeControlStatus,
};

/// Contract-derived negative-control selection for one runtime-proof transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NegativeControlSelection {
    pub id: String,
    pub dependency_id: String,
    pub obligation_id: String,
}

/// Ensures every fault-tested dependency projection names and exactly matches the canonical,
/// attested negative-control record in this proof output.
pub(crate) fn reconcile_negative_control_projection(
    dependency_evidence: &[ProofRuntimeDependencyEvidence],
    canonical: Option<&ProofRuntimeNegativeControl>,
    selection: Option<&NegativeControlSelection>,
) -> Result<(), &'static str> {
    let mut validated_projection_count = 0;
    let mut selected_projection_count = 0;
    for evidence in dependency_evidence {
        let fault_tested = evidence.level.as_deref() == Some("fault_tested");
        let Some(projection) = evidence.negative_control.as_ref() else {
            if fault_tested {
                return Err("fault-tested dependency evidence has no negative-control projection");
            }
            continue;
        };
        let validated = projection.status == ProofRuntimeNegativeControlStatus::Validated;
        if fault_tested && !validated {
            return Err("fault-tested dependency evidence has a non-validated negative control");
        }
        if !validated {
            if projection.same_obligation
                || projection.negative_control_id.is_some()
                || projection.failure_attestation_digest.is_some()
            {
                return Err("non-validated negative-control projection carries canonical linkage");
            }
            if selection.is_some_and(|selected| {
                evidence.dependency_id == selected.dependency_id
                    && evidence.proof_obligation_id.as_deref()
                        == Some(selected.obligation_id.as_str())
            }) {
                selected_projection_count += 1;
            }
            continue;
        }
        if !fault_tested {
            return Err("validated negative-control projection is not fault-tested");
        }
        let canonical =
            canonical.ok_or("validated negative-control projection has no canonical control")?;
        if canonical.status != ProofRuntimeNegativeControlStatus::Validated
            || canonical.outcome != ProofRuntimeNegativeControlOutcome::ExpectedObligationFailed
            || canonical.failure_mode
                != Some(ProofRuntimeNegativeControlFailureMode::ExpectedMissingEffect)
            || canonical.evidence_class != ExecutionEvidenceClass::Attested
        {
            return Err(
                "canonical negative control is not an attested expected obligation failure",
            );
        }
        let digest = canonical
            .failure_attestation_digest
            .as_deref()
            .ok_or("canonical validated negative control has no attestation digest")?;
        if projection.evidence_class != ExecutionEvidenceClass::Derived
            || !projection.same_obligation
            || projection.negative_control_id.as_deref() != Some(canonical.id.as_str())
            || projection.failure_mode
                != Some(ProofRuntimeNegativeControlFailureMode::ExpectedMissingEffect)
            || projection.failure_attestation_digest.as_deref() != Some(digest)
            || evidence.dependency_id != canonical.dependency_id
            || evidence.proof_obligation_id.as_deref() != Some(canonical.obligation_id.as_str())
        {
            return Err("negative-control projection does not match its canonical control");
        }
        validated_projection_count += 1;
        if selection.is_some_and(|selected| {
            evidence.dependency_id == selected.dependency_id
                && evidence.proof_obligation_id.as_deref() == Some(selected.obligation_id.as_str())
        }) {
            selected_projection_count += 1;
        }
    }
    match (selection, canonical) {
        (Some(selected), Some(canonical)) => {
            if canonical.id != selected.id
                || canonical.dependency_id != selected.dependency_id
                || canonical.obligation_id != selected.obligation_id
            {
                return Err("canonical negative control does not match the selected control");
            }
            if selected_projection_count != 1 {
                return Err("selected negative control does not have exactly one projection");
            }
        }
        (Some(_), None) => return Err("selected negative control has no canonical record"),
        (None, Some(_)) => return Err("unselected negative control has a canonical record"),
        (None, None) => {}
    }
    if canonical
        .is_some_and(|control| control.status == ProofRuntimeNegativeControlStatus::Validated)
        && validated_projection_count != 1
    {
        return Err("validated canonical negative control does not have exactly one projection");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{
        ProofRuntimeDependencyNegativeControl, ProofRuntimeDependencyObservation,
        ProofRuntimeNegativeControlIntervention,
    };

    fn valid_evidence_and_control() -> (
        Vec<ProofRuntimeDependencyEvidence>,
        ProofRuntimeNegativeControl,
    ) {
        let control = ProofRuntimeNegativeControl {
            id: String::from("negative-control"),
            dependency_id: String::from("database"),
            obligation_id: String::from("database-ready"),
            transaction_id: None,
            control_task: String::from("test:without-database"),
            intervention: ProofRuntimeNegativeControlIntervention {
                kind: String::from("environment"),
                id: String::from("database-url"),
            },
            expected_failure: String::from("database readiness must fail"),
            outcome: ProofRuntimeNegativeControlOutcome::ExpectedObligationFailed,
            status: ProofRuntimeNegativeControlStatus::Validated,
            failure_mode: Some(ProofRuntimeNegativeControlFailureMode::ExpectedMissingEffect),
            proof_scope_ref: String::from("workflow:verify"),
            evidence_class: ExecutionEvidenceClass::Attested,
            failure_attestation_digest: Some(format!("sha256:{}", "a".repeat(64))),
            exit_code: Some(1),
            detail: None,
        };
        let evidence = ProofRuntimeDependencyEvidence {
            dependency_id: control.dependency_id.clone(),
            proof_obligation_id: Some(control.obligation_id.clone()),
            level: Some(String::from("fault_tested")),
            interaction_attempted: false,
            observation: ProofRuntimeDependencyObservation {
                origin: String::from("negative_control"),
                evidence_class: ExecutionEvidenceClass::Derived,
            },
            declared_by_tasks: vec![control.control_task.clone()],
            declared_by_workflows: vec![String::from("verify")],
            negative_control: Some(ProofRuntimeDependencyNegativeControl {
                evidence_class: ExecutionEvidenceClass::Derived,
                status: ProofRuntimeNegativeControlStatus::Validated,
                same_obligation: true,
                negative_control_id: Some(control.id.clone()),
                failure_mode: Some(ProofRuntimeNegativeControlFailureMode::ExpectedMissingEffect),
                failure_attestation_digest: control.failure_attestation_digest.clone(),
            }),
        };
        (vec![evidence], control)
    }

    fn selection(control: &ProofRuntimeNegativeControl) -> NegativeControlSelection {
        NegativeControlSelection {
            id: control.id.clone(),
            dependency_id: control.dependency_id.clone(),
            obligation_id: control.obligation_id.clone(),
        }
    }

    fn reconcile(
        evidence: &[ProofRuntimeDependencyEvidence],
        control: Option<&ProofRuntimeNegativeControl>,
    ) -> Result<(), &'static str> {
        let selection = control.map(selection);
        reconcile_negative_control_projection(evidence, control, selection.as_ref())
    }

    #[test]
    fn accepts_exact_canonical_negative_control_projection() {
        let (evidence, control) = valid_evidence_and_control();
        assert!(reconcile(&evidence, Some(&control)).is_ok());
    }

    #[test]
    fn rejects_tampered_negative_control_projection_or_canonical_control() {
        let (evidence, control) = valid_evidence_and_control();

        let mut changed_digest = evidence.clone();
        changed_digest[0]
            .negative_control
            .as_mut()
            .expect("fixture has projection")
            .failure_attestation_digest = Some(String::from("sha256:changed"));
        assert!(reconcile(&changed_digest, Some(&control)).is_err());

        let mut changed_well_formed_digest = evidence.clone();
        changed_well_formed_digest[0]
            .negative_control
            .as_mut()
            .expect("fixture has projection")
            .failure_attestation_digest = Some(format!("sha256:{}", "0".repeat(64)));
        assert!(reconcile(&changed_well_formed_digest, Some(&control)).is_err());

        let mut changed_control_id = evidence.clone();
        changed_control_id[0]
            .negative_control
            .as_mut()
            .expect("fixture has projection")
            .negative_control_id = Some(String::from("other-control"));
        assert!(reconcile(&changed_control_id, Some(&control)).is_err());

        let mut changed_dependency = evidence.clone();
        changed_dependency[0].dependency_id = String::from("other-dependency");
        assert!(reconcile(&changed_dependency, Some(&control)).is_err());

        let mut changed_obligation = evidence.clone();
        changed_obligation[0].proof_obligation_id = Some(String::from("other-obligation"));
        assert!(reconcile(&changed_obligation, Some(&control)).is_err());

        let mut invalid_canonical = control.clone();
        invalid_canonical.status = ProofRuntimeNegativeControlStatus::Invalid;
        assert!(reconcile(&evidence, Some(&invalid_canonical)).is_err());

        let mut wrong_outcome = control.clone();
        wrong_outcome.outcome = ProofRuntimeNegativeControlOutcome::UnexpectedSuccess;
        assert!(reconcile(&evidence, Some(&wrong_outcome)).is_err());

        let mut wrong_failure_mode = control;
        wrong_failure_mode.failure_mode = Some(ProofRuntimeNegativeControlFailureMode::Timeout);
        assert!(reconcile(&evidence, Some(&wrong_failure_mode)).is_err());
    }

    #[test]
    fn rejects_fault_tested_evidence_without_a_validated_projection() {
        let (mut evidence, control) = valid_evidence_and_control();
        evidence[0].negative_control = None;
        assert!(reconcile(&evidence, Some(&control)).is_err());

        let (mut evidence, control) = valid_evidence_and_control();
        evidence[0]
            .negative_control
            .as_mut()
            .expect("fixture has projection")
            .status = ProofRuntimeNegativeControlStatus::Invalid;
        assert!(reconcile(&evidence, Some(&control)).is_err());

        let (evidence, mut control) = valid_evidence_and_control();
        control.status = ProofRuntimeNegativeControlStatus::Invalid;
        assert!(reconcile(&evidence, Some(&control)).is_err());

        let (mut evidence, control) = valid_evidence_and_control();
        evidence[0].negative_control = None;
        evidence[0].level = Some(String::from("exercised"));
        assert!(reconcile(&evidence, Some(&control)).is_err());
    }

    #[test]
    fn accepts_one_unlinked_projection_for_an_invalid_selected_control() {
        let (mut evidence, mut control) = valid_evidence_and_control();
        evidence[0].level = Some(String::from("exercised"));
        let projection = evidence[0]
            .negative_control
            .as_mut()
            .expect("fixture has projection");
        projection.status = ProofRuntimeNegativeControlStatus::Invalid;
        projection.same_obligation = false;
        projection.negative_control_id = None;
        projection.failure_attestation_digest = None;
        control.status = ProofRuntimeNegativeControlStatus::Invalid;
        control.outcome = ProofRuntimeNegativeControlOutcome::NonzeroExitObserved;
        control.failure_mode = Some(ProofRuntimeNegativeControlFailureMode::Timeout);

        assert!(reconcile(&evidence, Some(&control)).is_ok());
    }
}
