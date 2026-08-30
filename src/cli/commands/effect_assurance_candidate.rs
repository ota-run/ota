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

use super::*;

#[cfg(unix)]
struct FrozenContractSnapshot {
    root_directory: File,
    contract_file: File,
    bytes: Vec<u8>,
}

#[cfg(unix)]
impl FrozenContractSnapshot {
    fn capture(root: &Path) -> Result<Self, String> {
        use std::io::{Seek as _, SeekFrom};
        use std::os::unix::fs::OpenOptionsExt as _;

        let mut root_options = OpenOptions::new();
        root_options
            .read(true)
            .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW);
        let root_directory = root_options.open(root).map_err(|error| {
            format!(
                "failed to open repository root `{}` without following aliases: {error}",
                root.display()
            )
        })?;
        let contract_name =
            CString::new(DEFAULT_CONTRACT_FILE).expect("static contract name has no NUL");
        let fd = unsafe {
            libc::openat(
                root_directory.as_raw_fd(),
                contract_name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(format!(
                "failed to open current ota.yaml without following aliases: {}",
                io::Error::last_os_error()
            ));
        }
        let mut contract_file = unsafe { File::from_raw_fd(fd) };
        if !contract_file
            .metadata()
            .map_err(|error| format!("failed to inspect current ota.yaml: {error}"))?
            .is_file()
        {
            return Err(String::from("current ota.yaml is not a regular file"));
        }
        let mut bytes = Vec::new();
        contract_file
            .read_to_end(&mut bytes)
            .map_err(|error| format!("failed to read current ota.yaml: {error}"))?;
        contract_file
            .seek(SeekFrom::Start(0))
            .map_err(|error| format!("failed to retain current ota.yaml: {error}"))?;
        Ok(Self {
            root_directory,
            contract_file,
            bytes,
        })
    }

    fn revalidate(&mut self) -> Result<(), String> {
        use std::io::{Seek as _, SeekFrom};

        self.contract_file
            .seek(SeekFrom::Start(0))
            .map_err(|error| format!("failed to recheck retained ota.yaml: {error}"))?;
        let mut retained = Vec::new();
        self.contract_file
            .read_to_end(&mut retained)
            .map_err(|error| format!("failed to recheck retained ota.yaml: {error}"))?;
        let contract_name =
            CString::new(DEFAULT_CONTRACT_FILE).expect("static contract name has no NUL");
        let fd = unsafe {
            libc::openat(
                self.root_directory.as_raw_fd(),
                contract_name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(format!(
                "current ota.yaml changed or became aliased during reconciliation: {}",
                io::Error::last_os_error()
            ));
        }
        let mut current = unsafe { File::from_raw_fd(fd) };
        if !current
            .metadata()
            .map_err(|error| format!("failed to inspect re-opened ota.yaml: {error}"))?
            .is_file()
        {
            return Err(String::from(
                "current ota.yaml changed to a non-regular file during reconciliation",
            ));
        }
        let mut current_bytes = Vec::new();
        current
            .read_to_end(&mut current_bytes)
            .map_err(|error| format!("failed to re-read current ota.yaml: {error}"))?;
        if retained != self.bytes || current_bytes != self.bytes {
            return Err(String::from(
                "current ota.yaml changed during effect-assurance reconciliation",
            ));
        }
        Ok(())
    }
}

#[cfg(not(unix))]
struct FrozenContractSnapshot {
    bytes: Vec<u8>,
}

#[cfg(not(unix))]
impl FrozenContractSnapshot {
    fn capture(_root: &Path) -> Result<Self, String> {
        Err(String::from(
            "effect-assurance candidate reconciliation requires Unix no-follow descriptors",
        ))
    }

    fn revalidate(&mut self) -> Result<(), String> {
        Err(String::from(
            "effect-assurance candidate reconciliation requires Unix no-follow descriptors",
        ))
    }
}

/// Produces one review-only effect-refusal canary candidate from a verified private archive.
///
/// The archive proves a bounded negative event, so this command never creates an application
/// projection and `apply-candidate` refuses the resulting artifact.
pub(in crate::cli) fn effect_refusal_archive_candidate(
    path: Option<&Path>,
    archive: &Path,
    canary_id: &str,
    candidate_out: &Path,
    format: OutputFormat,
    debug: bool,
) -> CommandOutput {
    let root = resolve_repo_path(path);
    let root_display = compact_repo_path(&root);
    let candidate_path_display = candidate_out.display().to_string();
    let debug_lines = vec![
        String::from("DEBUG command=contract effect-refusal-candidate"),
        format!("DEBUG repo_root={}", root.display()),
        format!("DEBUG archive={}", archive.display()),
        format!("DEBUG canary_id={canary_id}"),
        format!("DEBUG candidate_out={candidate_path_display}"),
    ];
    let failure = |code: &'static str,
                   error: String,
                   next: Option<&str>,
                   candidate_published: bool,
                   candidate_publication: &'static str| {
        let output = match format {
            OutputFormat::Text => CommandOutput::failure(command_message_failure_text(
                "EFFECT REFUSAL CANDIDATE",
                &root_display,
                "Review-only candidate was not produced",
                &format!("{code}: {error}"),
                &next.into_iter().map(str::to_string).collect::<Vec<_>>(),
            )),
            OutputFormat::Json => {
                CommandOutput::failure(to_json(&EffectAssuranceCandidateFailure {
                    ok: false,
                    path: &root_display,
                    written: false,
                    candidate_path: &candidate_path_display,
                    candidate_published,
                    candidate_publication,
                    code,
                    error: &error,
                    next,
                }))
            }
        };
        finalize_debug(output, debug, debug_lines.clone())
    };
    let archive_path = if archive.is_absolute() {
        archive.to_path_buf()
    } else {
        root.join(archive)
    };
    let record = match read_repo_receipt_archive_record(&archive_path) {
        Ok(record) => record,
        Err(error) => {
            return failure(
                "effect_refusal_archive_invalid",
                error,
                Some("select one verified private workflow refusal archive under .ota/receipts"),
                false,
                "not_published",
            );
        }
    };
    let contract_path = root.join(DEFAULT_CONTRACT_FILE);
    let mut contract_snapshot = match FrozenContractSnapshot::capture(&root) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return failure(
                "effect_refusal_candidate_failed",
                error,
                None,
                false,
                "not_published",
            );
        }
    };
    let contract_text = match std::str::from_utf8(&contract_snapshot.bytes) {
        Ok(contents) => contents,
        Err(error) => {
            return failure(
                "effect_refusal_candidate_failed",
                format!("current ota.yaml is not valid UTF-8: {error}"),
                None,
                false,
                "not_published",
            );
        }
    };
    let contract = match parse_contract_str(&contract_path, contract_text) {
        Ok(contract) => contract,
        Err(error) => {
            return failure(
                "effect_refusal_candidate_failed",
                error.to_string(),
                None,
                false,
                "not_published",
            );
        }
    };
    if let Err(errors) = validate_contract_with_path(&contract, Some(&contract_path)) {
        return failure(
            "effect_refusal_candidate_failed",
            errors.to_string(),
            Some("repair ota.yaml before deriving a review candidate"),
            false,
            "not_published",
        );
    }
    let Some(context) = record
        .payload
        .archive_context
        .as_ref()
        .and_then(|context| context.effect_policy_refusal.as_ref())
    else {
        return failure(
            "effect_refusal_archive_inapplicable",
            String::from("the archive does not retain typed workflow refusal context"),
            Some("archive an explicit workflow typed-policy denial before requesting a candidate"),
            false,
            "not_published",
        );
    };
    let Some(evidence) = record.payload.receipt.typed_effect_policy_refusal.as_ref() else {
        return failure(
            "effect_refusal_archive_inapplicable",
            String::from("the archive does not retain typed refusal evidence"),
            None,
            false,
            "not_published",
        );
    };
    let matching_effects = evidence
        .policy_decision
        .effects
        .iter()
        .filter(|effect| {
            effect.eligible
                && effect.decision == PolicyEffectDecision::Deny
                && effect
                    .applicable_rules
                    .iter()
                    .any(|rule| rule.decision == PolicyEffectDecision::Deny)
        })
        .collect::<Vec<_>>();
    if matching_effects.len() != 1 {
        return failure(
            "effect_refusal_archive_ambiguous",
            format!(
                "the verified archive contains {} eligible explicit typed denials; exactly one is required",
                matching_effects.len()
            ),
            Some("select an archive with one exact eligible typed refusal"),
            false,
            "not_published",
        );
    }
    let Some(contract_snapshot_identity) = record.contract_snapshot_identity.as_ref() else {
        return failure(
            "effect_refusal_archive_invalid",
            String::from("the verified archive has no semantic contract snapshot identity"),
            None,
            false,
            "not_published",
        );
    };
    let archive_path = match archive_path.strip_prefix(&root) {
        Ok(path) => path.to_string_lossy().replace('\\', "/"),
        Err(_) => {
            return failure(
                "effect_refusal_archive_invalid",
                String::from("the archive must be inside the selected repository"),
                None,
                false,
                "not_published",
            );
        }
    };
    let effect = matching_effects[0];
    let adjusted_contract =
        contract_adjusted_for_selected_workflow_env_profile(&contract, Some(&context.workflow));
    let admission_contract = adjusted_contract.as_ref().unwrap_or(&contract);
    let roots = selected_workflow_phase_task_roots(admission_contract, Some(&context.workflow));
    if roots != context.roots {
        return failure(
            "effect_refusal_candidate_stale",
            String::from("current workflow roots no longer match the verified archive"),
            Some("regenerate the refusal archive from the current repository"),
            false,
            "not_published",
        );
    }
    let overrides = ExecutionOverrides {
        backend: context.backend,
        lifecycle: context.lifecycle,
        host_port: context.host_port,
        memory: context.memory,
        skip_deps: context.skip_dependencies,
    };
    let current_closure = match build_typed_effect_closure_admission(
        admission_contract,
        &contract_path,
        &roots,
        overrides,
    ) {
        Ok(closure) => closure,
        Err(error) => {
            return failure(
                "effect_refusal_candidate_stale",
                error.to_string(),
                Some("restore current typed-effect inputs or archive a new refusal"),
                false,
                "not_published",
            );
        }
    };
    let matching_current_plans = current_closure
        .application_plans
        .iter()
        .filter(|plan| {
            plan.effect_identity == effect.effect_identity
                && plan.attachment_identity == effect.attachment_identity
        })
        .collect::<Vec<_>>();
    if matching_current_plans.len() != 1 {
        return failure(
            "effect_refusal_candidate_stale",
            format!(
                "current workflow contains {} matching typed realizations; exactly one is required",
                matching_current_plans.len()
            ),
            Some("restore the archived realization or archive a new refusal"),
            false,
            "not_published",
        );
    }
    let current_realization_identity =
        match typed_effect_realization_identity(admission_contract, matching_current_plans[0]) {
            Ok(identity) => identity,
            Err(error) => {
                return failure(
                    "effect_refusal_candidate_stale",
                    error.to_string(),
                    Some("restore the archived realization or archive a new refusal"),
                    false,
                    "not_published",
                );
            }
        };
    let input = EffectAssuranceCandidateInput {
        archive_path,
        archive_identity: record.archive_identity,
        contract_snapshot_identity: contract_snapshot_identity.clone(),
        workflow: context.workflow.clone(),
        effect_identity: effect.effect_identity.clone(),
        attachment_identity: effect.attachment_identity.clone(),
        realization_identity: effect.realization_identity.clone(),
        current_realization_identity,
        current_contract_content_identity: contract_snapshot_hash(&contract_snapshot.bytes),
    };
    let reconciliation = match effect_assurance_candidate_reconciliation(&input, canary_id) {
        Ok(reconciliation) => serde_json::to_value(reconciliation)
            .expect("serializing effect-assurance reconciliation should not fail"),
        Err(error) => {
            return failure(
                "effect_refusal_candidate_stale",
                error.to_string(),
                None,
                false,
                "not_published",
            );
        }
    };
    let candidate = match derive_effect_assurance_candidate(&contract, &input, canary_id) {
        Ok(EffectAssuranceCandidateDerivation::Candidate(candidate)) => candidate,
        Ok(EffectAssuranceCandidateDerivation::AlreadyDeclared) => {
            if let Err(error) = contract_snapshot.revalidate() {
                return failure(
                    "effect_refusal_candidate_stale",
                    error,
                    Some("retry against a stable regular ota.yaml"),
                    false,
                    "not_published",
                );
            }
            let output = match format {
                OutputFormat::Text => CommandOutput::success(format!(
                    "{}\n\n{}\n{}",
                    format_command_header("EFFECT REFUSAL CANDIDATE", &root_display),
                    format_result_line("existing canary already matches the verified archive"),
                    "No candidate was published."
                )),
                OutputFormat::Json => {
                    CommandOutput::success(to_json(&EffectAssuranceCandidateSuccess {
                        ok: true,
                        path: &root_display,
                        written: false,
                        candidate_path: &candidate_path_display,
                        candidate_published: false,
                        candidate_publication: "not_published",
                        disposition: "already_declared",
                        no_op: true,
                        reconciliation: reconciliation.clone(),
                        candidate: None,
                    }))
                }
            };
            return finalize_debug(output, debug, debug_lines);
        }
        Ok(EffectAssuranceCandidateDerivation::Conflict) => {
            return failure(
                "effect_refusal_candidate_conflict",
                format!(
                    "effect-refusal canary `{canary_id}` already declares different contract truth"
                ),
                Some(
                    "review the existing canary; this command does not replace contract declarations",
                ),
                false,
                "not_published",
            );
        }
        Err(error) => {
            return failure(
                "effect_refusal_candidate_stale",
                error.to_string(),
                Some("regenerate only from a verified archive that matches current ota.yaml"),
                false,
                "not_published",
            );
        }
    };
    if let Err(error) = contract_snapshot.revalidate() {
        return failure(
            "effect_refusal_candidate_stale",
            error,
            Some("retry against a stable regular ota.yaml"),
            false,
            "not_published",
        );
    }
    if let Err(error) = write_candidate_create_new(&root, candidate_out, &candidate) {
        return failure(
            "effect_refusal_candidate_write_failed",
            error.message().to_string(),
            Some("choose a new in-repository candidate path and retry"),
            error.candidate_published(),
            error.posture(),
        );
    }
    let candidate_json = serde_json::to_value(&candidate)
        .expect("serializing verified effect-assurance candidate should not fail");
    let output = match format {
        OutputFormat::Text => CommandOutput::success(format!(
            "{}\n\n{}\n{}\n{}",
            format_command_header("EFFECT REFUSAL CANDIDATE", &root_display),
            format_result_line("review-only candidate produced"),
            format_result_line(&format!("identity {}", paint_code(&candidate.identity))),
            format!(
                "Review: {}\nThis archive-bound proposal is unknown and cannot apply ota.yaml.",
                paint_code(&candidate_path_display)
            )
        )),
        OutputFormat::Json => CommandOutput::success(to_json(&EffectAssuranceCandidateSuccess {
            ok: true,
            path: &root_display,
            written: false,
            candidate_path: &candidate_path_display,
            candidate_published: true,
            candidate_publication: "durable",
            disposition: "unknown",
            no_op: false,
            reconciliation,
            candidate: Some(candidate_json),
        })),
    };
    finalize_debug(output, debug, debug_lines)
}
