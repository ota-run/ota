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

fn workspace_follow_up_artifact_routing(path: &str) -> Vec<crate::output::ArtifactRoute> {
    vec![artifact_route(
        "follow_up",
        "workspace_receipt_json",
        "receipt",
        Some(workspace_receipt_command_for_workspace(path, true)),
        None,
    )]
}

fn workspace_receipt_artifact_routing(
    path: &str,
    archive_path: Option<&str>,
) -> Vec<crate::output::ArtifactRoute> {
    let mut routes = vec![artifact_route(
        "inspect",
        "workspace_receipt_json",
        "receipt",
        Some(workspace_receipt_command_for_workspace(path, false)),
        None,
    )];
    if let Some(archive_path) = archive_path {
        routes.push(artifact_route(
            "keep",
            "workspace_receipt_archive",
            "receipt",
            None,
            Some(archive_path.to_string()),
        ));
    }
    routes
}

pub(crate) fn render_workspace_up(
    path: &str,
    report: &WorkspaceUpReport,
    format: OutputFormat,
    show_receipt: bool,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let workspace_root = Path::new(path).parent();
            let mut stdout = format!(
                "{}\n\n{}",
                format_command_header("WORKSPACE UP", path),
                render_readiness_status(report.ok)
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    workspace_status_word(&repo.status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Path:"),
                    compact_repo_path(Path::new(&repo.path))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Contract:"),
                    compact_contract_file_path_relative_to(
                        Path::new(&repo.contract_path),
                        DEFAULT_CONTRACT_FILE,
                        workspace_root,
                    )
                ));
                stdout.push_str(&format!("\n{} {}", paint_key("Phase:"), repo.phase));
                if let Some(source_url) = &repo.source_url {
                    stdout.push_str(&format!("\n{} {source_url}", paint_key("Source:")));
                }
                if let Some(source_ref) = &repo.source_ref {
                    stdout.push_str(&format!("\n{} {source_ref}", paint_key("Ref:")));
                }
                if let Some(service) = &repo.service {
                    stdout.push_str(&format!("\n{} {service}", paint_key("Service:")));
                }
                if let Some(service_command) = &repo.service_command {
                    stdout.push_str(&format!(
                        "\n{} {service_command}",
                        paint_key("Service command:")
                    ));
                }
                if let Some(task) = &repo.task {
                    stdout.push_str(&format!("\n{} {task}", paint_key("Task:")));
                }
                if let Some(task_command) = &repo.task_command {
                    stdout.push_str(&format!(
                        "\n{} {task_command}",
                        paint_key(workspace_phase_command_label(&repo.phase))
                    ));
                }
                if let Some(exit_code) = repo.exit_code {
                    stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
                }
                stdout.push_str(&render_workspace_repo_findings_text(&repo.findings));
                append_primary_output_block(
                    &mut stdout,
                    workspace_phase_output_label(&repo.phase),
                    repo.stdout.as_deref(),
                    repo.stderr.as_deref(),
                );
            }
            if show_receipt {
                stdout.push_str(&render_execution_receipt_text(&report.receipt));
            }
            stdout.push('\n');
            stdout.push('\n');
            stdout.push_str(&render_execution_receipt_summary_block(
                &report.receipt,
                report
                    .repos
                    .first()
                    .and_then(|repo| repo.task.as_deref())
                    .or(report.receipt.steps.first().map(|step| step.label.as_str())),
                "WORKSPACE UP SUMMARY",
            ));
            append_receipt_next_block(&mut stdout, &report.receipt);

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceUpSuccess {
                ok: report.ok,
                path,
                mode: None,
                summary: report.receipt.summary,
                receipt: report.receipt.clone(),
                artifact_routing: workspace_follow_up_artifact_routing(path),
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
        },
    }
}

pub(crate) fn render_workspace_refresh(
    path: &str,
    report: &WorkspaceUpReport,
    format: OutputFormat,
    show_receipt: bool,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let workspace_root = Path::new(path).parent();
            let mut stdout = if report.dry_run {
                format!(
                    "{}\n\n{}",
                    format_command_header("WORKSPACE REFRESH PREVIEW", path),
                    format_mode_line("dry-run (no write)")
                )
            } else {
                format!(
                    "{}\n\n{}",
                    format_command_header("WORKSPACE REFRESH", path),
                    render_readiness_status(report.ok)
                )
            };

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    workspace_status_word(&repo.status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Path:"),
                    compact_repo_path(Path::new(&repo.path))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Contract:"),
                    compact_contract_file_path_relative_to(
                        Path::new(&repo.contract_path),
                        DEFAULT_CONTRACT_FILE,
                        workspace_root,
                    )
                ));
                stdout.push_str(&format!("\n{} {}", paint_key("Phase:"), repo.phase));
                if let Some(source_url) = &repo.source_url {
                    stdout.push_str(&format!("\n{} {source_url}", paint_key("Source:")));
                }
                if let Some(source_ref) = &repo.source_ref {
                    stdout.push_str(&format!("\n{} {source_ref}", paint_key("Ref:")));
                }
                if let Some(task_command) = &repo.task_command {
                    stdout.push_str(&format!(
                        "\n{} {task_command}",
                        paint_key(workspace_phase_command_label(&repo.phase))
                    ));
                }
                if let Some(exit_code) = repo.exit_code {
                    stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
                }
                stdout.push_str(&render_workspace_repo_findings_text(&repo.findings));
                append_primary_output_block(
                    &mut stdout,
                    workspace_phase_output_label(&repo.phase),
                    repo.stdout.as_deref(),
                    repo.stderr.as_deref(),
                );
            }
            if show_receipt {
                stdout.push_str(&render_execution_receipt_text(&report.receipt));
            }
            stdout.push('\n');
            stdout.push('\n');
            stdout.push_str(&render_execution_receipt_summary_block(
                &report.receipt,
                report
                    .repos
                    .first()
                    .and_then(|repo| repo.task_command.as_deref())
                    .or(report.receipt.steps.first().map(|step| step.label.as_str())),
                if report.dry_run {
                    "WORKSPACE REFRESH PREVIEW SUMMARY"
                } else {
                    "WORKSPACE REFRESH SUMMARY"
                },
            ));
            append_receipt_next_block(&mut stdout, &report.receipt);

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceUpSuccess {
                ok: report.ok,
                path,
                mode: Some(if report.dry_run { "preview" } else { "refresh" }),
                summary: report.receipt.summary,
                receipt: report.receipt.clone(),
                artifact_routing: workspace_follow_up_artifact_routing(path),
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_up_text_groups_shared_actions_by_remediation() {
        let report = WorkspaceUpReport {
            ok: false,
            dry_run: false,
            receipt: ExecutionReceipt {
                ok: false,
                path: String::from("./ota.workspace.yaml"),
                scope: String::from("workspace"),
                contract: String::from("./ota.workspace.yaml"),
                contract_identity: None,
                contract_snapshot_hash: None,
                contract_snapshot_ref: None,
                assumption_set_hash: None,
                workspace: Some(String::from("demo")),
                backend: None,
                context: None,
                lifecycle: None,
                image: None,
                container_memory_bytes: None,
                target: None,
                provider: None,
                cwd: None,
                acquired: Vec::new(),
                env: BTreeMap::new(),
                env_sources: Vec::new(),
                workflow_env_artifacts: Vec::new(),
                native_prerequisites: Vec::new(),
                toolchains: Vec::new(),
                runtime: None,
                logs: None,
                service_termination: None,
                host_service_cleanup: Vec::new(),
                backend_fulfillment: None,
                workloads: BTreeMap::new(),
                policy: Vec::new(),
                dependency_steps: Vec::new(),
                steps: Vec::new(),
                blocked: Vec::new(),
                status: None,
                failed_task: None,
                failed_dependency: None,
                failure_origin: None,
                summary: ExecutionReceiptSummary::default(),
                next: None,
            },
            repos: vec![WorkspaceRepoUpReport {
                name: String::from("api"),
                path: String::from("api"),
                contract_path: String::from("api/ota.yaml"),
                workflow: None,
                required: true,
                ok: false,
                status: String::from("PROVISION FAILED"),
                phase: String::from("provisioning"),
                findings: vec![
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Warn,
                        summary: String::from("Contract drift: `tools.maven`"),
                        why: String::from(
                            "`ota.yaml` still declares `tools.maven` = `3.9.9`, but repo inspection under `.` now detects `*`",
                        ),
                        next: String::from(
                            "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                        ),
                    },
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Warn,
                        summary: String::from("Contract drift: `tools.node`"),
                        why: String::from(
                            "`ota.yaml` still declares `tools.node` = `22`, but repo inspection under `.` now detects `*`",
                        ),
                        next: String::from(
                            "run `ota detect --merge --dry-run .` to review the comparison, then `ota detect --merge .`",
                        ),
                    },
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Info,
                        summary: String::from("Policy-backed provisioning sources are declared"),
                        why: String::from(
                            "`.ota/org-policy.yaml` declares approved provisioning sources: curl via brew (versions 8.7.1)",
                        ),
                        next: String::from(
                            "use this policy surface when repo prerequisites need an approved source",
                        ),
                    },
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Info,
                        summary: String::from("Adapter bootstrap sources are declared"),
                        why: String::from(
                            "`.ota/org-policy.yaml` declares approved adapter bootstrap sources: brew via brew-bootstrap (any approved version)",
                        ),
                        next: String::from(
                            "use this policy surface when repo prerequisites need an approved source",
                        ),
                    },
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Info,
                        summary: String::from("Adapter bootstrap sources are declared"),
                        why: String::from(
                            "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap",
                        ),
                        next: String::from(
                            "use this policy surface when adapter bootstrap needs to be approved or audited",
                        ),
                    },
                ],
                source_url: None,
                source_ref: None,
                service: None,
                service_command: None,
                task: None,
                task_command: None,
                exit_code: Some(127),
                stdout: None,
                stderr: Some(String::from("sh: 1: sdk: not found")),
                env_sources: Vec::new(),
                next: None,
                next_steps: Vec::new(),
            }],
        };

        let text = strip_ansi_codes(
            &render_workspace_up("./ota.workspace.yaml", &report, OutputFormat::Text, false).stdout,
        );

        assert!(text.contains("Review contract drift (2)"));
        assert!(text.contains("Review active policy surfaces (2)"));
        assert_eq!(text.matches("Next:").count(), 2);
        assert!(text.contains("» Approved provisioning sources are configured"));
        assert!(text.contains("» Approved adapter bootstrap is configured"));
        assert!(text.contains("Task output:"));
        assert!(text.contains("» sh: 1: sdk: not found"));
    }

    #[test]
    fn workspace_up_groups_version_mismatches_under_one_action() {
        let report = WorkspaceUpReport {
            ok: false,
            dry_run: false,
            receipt: ExecutionReceipt {
                ok: false,
                path: String::from("./ota.workspace.yaml"),
                scope: String::from("workspace"),
                contract: String::from("./ota.workspace.yaml"),
                contract_identity: None,
                contract_snapshot_hash: None,
                contract_snapshot_ref: None,
                assumption_set_hash: None,
                workspace: Some(String::from("demo")),
                backend: None,
                context: None,
                lifecycle: None,
                image: None,
                container_memory_bytes: None,
                target: None,
                provider: None,
                cwd: None,
                acquired: Vec::new(),
                env: BTreeMap::new(),
                env_sources: Vec::new(),
                workflow_env_artifacts: Vec::new(),
                native_prerequisites: Vec::new(),
                toolchains: Vec::new(),
                runtime: None,
                logs: None,
                service_termination: None,
                host_service_cleanup: Vec::new(),
                backend_fulfillment: None,
                workloads: BTreeMap::new(),
                policy: Vec::new(),
                dependency_steps: Vec::new(),
                steps: Vec::new(),
                blocked: Vec::new(),
                status: None,
                failed_task: None,
                failed_dependency: None,
                failure_origin: None,
                summary: ExecutionReceiptSummary::default(),
                next: None,
            },
            repos: vec![WorkspaceRepoUpReport {
                name: String::from("api"),
                path: String::from("api"),
                contract_path: String::from("api/ota.yaml"),
                workflow: None,
                required: true,
                ok: false,
                status: String::from("PROVISION FAILED"),
                phase: String::from("provisioning"),
                findings: vec![
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Error,
                        summary: String::from("Version mismatch for runtime: java"),
                        why: String::from(
                            "java resolved to `25.0.2` but the contract requires `21`",
                        ),
                        next: String::from(
                            "install a compatible java version that satisfies `21`, then rerun `ota doctor`",
                        ),
                    },
                    Finding {
                        identity: None,
                        severity: FindingSeverity::Error,
                        summary: String::from("Version mismatch for tool: curl"),
                        why: String::from(
                            "curl resolved to `8.13.0` but the contract requires `8.7.1`",
                        ),
                        next: String::from(
                            "install a compatible curl version that satisfies `8.7.1`, then rerun `ota doctor`",
                        ),
                    },
                ],
                source_url: None,
                source_ref: None,
                service: None,
                service_command: None,
                task: None,
                task_command: None,
                exit_code: Some(127),
                stdout: None,
                stderr: Some(String::from("sh: 1: sdk: not found")),
                env_sources: Vec::new(),
                next: None,
                next_steps: Vec::new(),
            }],
        };

        let text = strip_ansi_codes(
            &render_workspace_up("./ota.workspace.yaml", &report, OutputFormat::Text, false).stdout,
        );

        assert!(text.contains("Fix version mismatches (2)"));
        assert!(text.contains("» java resolved `25.0.2`, requires `21`"));
        assert!(text.contains("» curl resolved `8.13.0`, requires `8.7.1`"));
        assert_eq!(text.matches("Next:").count(), 1);
    }

    #[test]
    fn workspace_diff_surfaces_top_level_refresh_lane_in_text_and_json() {
        let report = WorkspaceDiffReport {
            repos: vec![WorkspaceRepoDiffReport {
                name: String::from("api"),
                path: String::from("api"),
                contract_path: String::from("api/ota.yaml"),
                required: true,
                acquired: true,
                drift_kind: String::from("commit_divergence"),
                target_source: Some(String::from("declared_ref")),
                status: String::from("DIFFERENT"),
                source_url: Some(String::from("https://example.com/api.git")),
                source_ref: Some(String::from("main")),
                branch: Some(String::from("main")),
                head: Some(String::from("abc123")),
                target_ref: Some(String::from("origin/main")),
                ahead: Some(1),
                behind: Some(2),
                dirty: false,
                findings: vec![Finding {
                    identity: None,
                    severity: FindingSeverity::Warn,
                    summary: String::from("Repo drift detected: api"),
                    why: String::from(
                        "workspace repo `api` is 1 commit(s) ahead and 2 commit(s) behind of `origin/main`",
                    ),
                    next: String::from(
                        "run `ota workspace refresh` to reconcile the repo, or `ota workspace refresh --dry-run` to preview the sync",
                    ),
                }],
                next: Some(String::from(
                    "run `ota workspace refresh` to reconcile the repo, or `ota workspace refresh --dry-run` to preview the sync",
                )),
                next_steps: vec![
                    String::from(
                        "run `ota workspace refresh --dry-run` to preview the sync commands before mutating repo state",
                    ),
                    String::from(
                        "run `ota workspace refresh` when you are ready to reconcile workspace drift",
                    ),
                    String::from(
                        "run `ota workspace status` after workspace changes to confirm readiness and drift together",
                    ),
                ],
            }],
            next: Some(String::from(
                "run `ota workspace refresh --dry-run` to preview the sync commands before mutating repo state; run `ota workspace refresh` when you are ready to reconcile workspace drift; run `ota workspace status` after workspace changes to confirm readiness and drift together",
            )),
            next_steps: vec![
                String::from(
                    "run `ota workspace refresh --dry-run` to preview the sync commands before mutating repo state",
                ),
                String::from(
                    "run `ota workspace refresh` when you are ready to reconcile workspace drift",
                ),
                String::from(
                    "run `ota workspace status` after workspace changes to confirm readiness and drift together",
                ),
            ],
        };

        let text = strip_ansi_codes(
            &render_workspace_diff("./ota.workspace.yaml", &report, OutputFormat::Text).stdout,
        );
        assert!(text.contains("WORKSPACE DIFF"));
        assert!(text.contains("Next:"));
        assert!(text.contains("ota workspace refresh --dry-run"));
        assert!(text.contains("ota workspace status"));
        assert!(text.contains("Target: origin/main (declared source ref)"));

        let json =
            render_workspace_diff("./ota.workspace.yaml", &report, OutputFormat::Json).stdout;
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            value["next_steps"][0],
            "run `ota workspace refresh --dry-run` to preview the sync commands before mutating repo state"
        );
        assert_eq!(
            value["repos"][0]["next_steps"][2],
            "run `ota workspace status` after workspace changes to confirm readiness and drift together"
        );
        assert_eq!(value["repos"][0]["drift_kind"], "commit_divergence");
        assert_eq!(value["repos"][0]["target_source"], "declared_ref");
    }

    #[test]
    fn workspace_status_surfaces_doctor_then_refresh_lane_in_text_and_json() {
        let report = WorkspaceStatusReport {
            repos: vec![WorkspaceRepoStatusReport {
                name: String::from("api"),
                path: String::from("api"),
                contract_path: String::from("api/ota.yaml"),
                workflow: None,
                required: true,
                acquired: true,
                drift_kind: String::from("local_dirty"),
                target_source: Some(String::from("declared_ref")),
                ready: false,
                readiness_status: String::from("NOT READY"),
                drift_status: String::from("DIRTY"),
                source_url: Some(String::from("https://example.com/api.git")),
                source_ref: Some(String::from("main")),
                branch: Some(String::from("main")),
                head: Some(String::from("abc123")),
                target_ref: Some(String::from("origin/main")),
                ahead: None,
                behind: None,
                dirty: true,
                findings: vec![Finding {
                    identity: None,
                    severity: FindingSeverity::Error,
                    summary: String::from("Missing setup task"),
                    why: String::from("repo is not ready"),
                    next: String::from(
                        "run `ota workspace doctor` to inspect readiness blockers before retrying workspace execution",
                    ),
                }],
                next: Some(String::from(
                    "run `ota workspace doctor` to inspect readiness blockers before retrying workspace execution",
                )),
                next_steps: vec![String::from(
                    "run `ota workspace doctor` to inspect readiness blockers before retrying workspace execution",
                )],
            }],
            next: Some(String::from(
                "run `ota workspace doctor` to inspect the readiness blockers before retrying workspace execution; run `ota workspace refresh --dry-run` to preview the sync commands before reconciling workspace drift; run `ota workspace refresh` when you are ready to reconcile workspace drift",
            )),
            next_steps: vec![
                String::from(
                    "run `ota workspace doctor` to inspect the readiness blockers before retrying workspace execution",
                ),
                String::from(
                    "run `ota workspace refresh --dry-run` to preview the sync commands before reconciling workspace drift",
                ),
                String::from(
                    "run `ota workspace refresh` when you are ready to reconcile workspace drift",
                ),
            ],
        };

        let text = strip_ansi_codes(
            &render_workspace_status("./ota.workspace.yaml", &report, OutputFormat::Text).stdout,
        );
        assert!(text.contains("WORKSPACE STATUS"));
        assert!(text.contains("Next:"));
        assert!(text.contains("ota workspace doctor"));
        assert!(text.contains("ota workspace refresh --dry-run"));
        assert!(text.contains("Target: origin/main (declared source ref)"));

        let json =
            render_workspace_status("./ota.workspace.yaml", &report, OutputFormat::Json).stdout;
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            value["next_steps"][0],
            "run `ota workspace doctor` to inspect the readiness blockers before retrying workspace execution"
        );
        assert_eq!(
            value["repos"][0]["next"],
            "run `ota workspace doctor` to inspect readiness blockers before retrying workspace execution"
        );
        assert_eq!(value["repos"][0]["drift_kind"], "local_dirty");
        assert_eq!(value["repos"][0]["target_source"], "declared_ref");
    }

    #[test]
    fn workspace_diff_summary_breaks_out_missing_and_unresolved_subkinds() {
        let report = WorkspaceDiffReport {
            repos: vec![
                WorkspaceRepoDiffReport {
                    name: String::from("missing-contract"),
                    path: String::from("missing-contract"),
                    contract_path: String::from("missing-contract/ota.yaml"),
                    required: true,
                    acquired: true,
                    status: String::from("MISSING CONTRACT"),
                    drift_kind: String::from("missing_contract"),
                    target_source: None,
                    source_url: None,
                    source_ref: None,
                    branch: None,
                    head: None,
                    target_ref: None,
                    ahead: None,
                    behind: None,
                    dirty: false,
                    findings: Vec::new(),
                    next: None,
                    next_steps: Vec::new(),
                },
                WorkspaceRepoDiffReport {
                    name: String::from("no-target"),
                    path: String::from("no-target"),
                    contract_path: String::from("no-target/ota.yaml"),
                    required: true,
                    acquired: true,
                    status: String::from("UNRESOLVED"),
                    drift_kind: String::from("target_unavailable"),
                    target_source: None,
                    source_url: Some(String::from("https://example.com/no-target.git")),
                    source_ref: None,
                    branch: Some(String::from("main")),
                    head: Some(String::from("abc123")),
                    target_ref: None,
                    ahead: None,
                    behind: None,
                    dirty: false,
                    findings: Vec::new(),
                    next: None,
                    next_steps: Vec::new(),
                },
            ],
            next: None,
            next_steps: Vec::new(),
        };

        let text = strip_ansi_codes(
            &render_workspace_diff("./ota.workspace.yaml", &report, OutputFormat::Text).stdout,
        );
        assert!(text.contains("Missing contract: 1"));
        assert!(text.contains("Target unavailable: 1"));

        let json =
            render_workspace_diff("./ota.workspace.yaml", &report, OutputFormat::Json).stdout;
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["summary"]["missing_contract_count"], 1);
        assert_eq!(value["summary"]["target_unavailable_count"], 1);
    }

    #[test]
    fn workspace_status_summary_breaks_out_missing_and_unresolved_subkinds() {
        let report = WorkspaceStatusReport {
            repos: vec![
                WorkspaceRepoStatusReport {
                    name: String::from("missing-contract"),
                    path: String::from("missing-contract"),
                    contract_path: String::from("missing-contract/ota.yaml"),
                    workflow: None,
                    required: true,
                    acquired: true,
                    ready: false,
                    readiness_status: String::from("NOT READY"),
                    drift_status: String::from("MISSING CONTRACT"),
                    drift_kind: String::from("missing_contract"),
                    target_source: None,
                    source_url: None,
                    source_ref: None,
                    branch: None,
                    head: None,
                    target_ref: None,
                    ahead: None,
                    behind: None,
                    dirty: false,
                    findings: Vec::new(),
                    next: None,
                    next_steps: Vec::new(),
                },
                WorkspaceRepoStatusReport {
                    name: String::from("no-target"),
                    path: String::from("no-target"),
                    contract_path: String::from("no-target/ota.yaml"),
                    workflow: None,
                    required: true,
                    acquired: true,
                    ready: false,
                    readiness_status: String::from("NOT READY"),
                    drift_status: String::from("UNRESOLVED"),
                    drift_kind: String::from("target_unavailable"),
                    target_source: None,
                    source_url: Some(String::from("https://example.com/no-target.git")),
                    source_ref: None,
                    branch: Some(String::from("main")),
                    head: Some(String::from("abc123")),
                    target_ref: None,
                    ahead: None,
                    behind: None,
                    dirty: false,
                    findings: Vec::new(),
                    next: None,
                    next_steps: Vec::new(),
                },
            ],
            next: None,
            next_steps: Vec::new(),
        };

        let text = strip_ansi_codes(
            &render_workspace_status("./ota.workspace.yaml", &report, OutputFormat::Text).stdout,
        );
        assert!(text.contains("Missing contract: 1"));
        assert!(text.contains("Target unavailable: 1"));

        let json =
            render_workspace_status("./ota.workspace.yaml", &report, OutputFormat::Json).stdout;
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["summary"]["missing_contract_count"], 1);
        assert_eq!(value["summary"]["target_unavailable_count"], 1);
    }
}

pub(crate) fn render_workspace_repo_findings_text(findings: &[Finding]) -> String {
    let mut stdout = String::new();
    for group in group_doctor_findings(findings.iter()) {
        if group.findings.len() == 1 {
            let finding = group.findings[0];
            let next = compact_backticked_paths(&finding.next);
            let source_line = policy_finding_source(&finding.summary, &finding.why).map(|value| {
                format!(
                    "{} {}",
                    finding_detail_key(finding.severity, "Source:"),
                    value
                )
            });
            let source_block = source_line
                .as_ref()
                .map(|value| format!("\n{}", value))
                .unwrap_or_default();
            if concise_mode() {
                stdout.push_str("\n\n");
                stdout.push_str(&format!(
                    "{}  {}{}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    source_block,
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            } else {
                let why = compact_backticked_paths(&finding.why);
                stdout.push_str("\n\n");
                stdout.push_str(&format!(
                    "{}  {}\n{} {}{}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Why:"),
                    why,
                    source_block,
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            }
            continue;
        }

        stdout.push_str(&render_grouped_doctor_findings(&group, None, None));
    }
    stdout
}

pub(crate) fn render_workspace_diff(
    path: &str,
    report: &WorkspaceDiffReport,
    format: OutputFormat,
) -> CommandOutput {
    let match_state = report.repos.iter().all(|repo| repo.status == "MATCH");
    match format {
        OutputFormat::Text => {
            let workspace_root = Path::new(path).parent();
            let mut stdout = format!(
                "\n{}\n\n{}",
                format_command_header("WORKSPACE DIFF", path),
                workspace_diff_status_word(if match_state { "MATCH" } else { "DIFFERENT" })
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    workspace_diff_status_word(&repo.status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Path:"),
                    compact_repo_path(Path::new(&repo.path))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Contract:"),
                    compact_contract_file_path_relative_to(
                        Path::new(&repo.contract_path),
                        DEFAULT_CONTRACT_FILE,
                        workspace_root,
                    )
                ));
                if let Some(source_url) = &repo.source_url {
                    stdout.push_str(&format!("\n{} {source_url}", paint_key("Source:")));
                }
                if let Some(source_ref) = &repo.source_ref {
                    stdout.push_str(&format!("\n{} {source_ref}", paint_key("Ref:")));
                }
                if let Some(branch) = &repo.branch {
                    stdout.push_str(&format!("\n{} {branch}", paint_key("Branch:")));
                }
                if let Some(head) = &repo.head {
                    stdout.push_str(&format!("\n{} {head}", paint_key("Head:")));
                }
                if let Some(target_ref) = &repo.target_ref {
                    stdout.push_str(&format!(
                        "\n{} {}",
                        paint_key("Target:"),
                        render_workspace_target_label(target_ref, repo.target_source.as_deref())
                    ));
                }
                if let Some(ahead) = repo.ahead {
                    stdout.push_str(&format!("\n{} {ahead}", paint_key("Ahead:")));
                }
                if let Some(behind) = repo.behind {
                    stdout.push_str(&format!("\n{} {behind}", paint_key("Behind:")));
                }
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Dirty:"),
                    if repo.dirty { "yes" } else { "no" }
                ));
                for finding in &repo.findings {
                    let why = compact_backticked_paths(&finding.why);
                    let next = compact_backticked_paths(&finding.next);
                    stdout.push_str(&format!(
                        "\n\n{}  {}\n{} {}\n{} {}",
                        render_severity(finding.severity),
                        render_finding_summary(finding.severity, &finding.summary),
                        finding_detail_key(finding.severity, "Why:"),
                        why,
                        finding_detail_key(finding.severity, "Next:"),
                        next
                    ));
                }
            }

            stdout.push('\n');
            stdout.push('\n');
            stdout.push_str(&render_workspace_diff_summary(&report.repos));
            append_post_summary_next_block(&mut stdout, &report.next_steps);

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: 0,
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceDiffSuccess {
                ok: true,
                path,
                mode: "diff",
                summary: workspace_diff_summary(&report.repos),
                next: report.next.as_deref(),
                next_steps: &report.next_steps,
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: 0,
        },
    }
}

pub(crate) fn render_workspace_status(
    path: &str,
    report: &WorkspaceStatusReport,
    format: OutputFormat,
) -> CommandOutput {
    let summary = workspace_status_summary(&report.repos);
    let ok = report.repos.iter().all(|repo| !repo.required || repo.ready);

    match format {
        OutputFormat::Text => {
            let workspace_root = Path::new(path).parent();
            let mut stdout = format!(
                "\n{}\n\n{}",
                format_command_header("WORKSPACE STATUS", path),
                render_readiness_status(ok)
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}]",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    }
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Status:"),
                    workspace_status_word(&repo.readiness_status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Drift:"),
                    workspace_diff_status_word(&repo.drift_status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Path:"),
                    compact_repo_path(Path::new(&repo.path))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Contract:"),
                    compact_contract_file_path_relative_to(
                        Path::new(&repo.contract_path),
                        DEFAULT_CONTRACT_FILE,
                        workspace_root,
                    )
                ));
                if let Some(source_url) = &repo.source_url {
                    stdout.push_str(&format!("\n{} {source_url}", paint_key("Source:")));
                }
                if let Some(source_ref) = &repo.source_ref {
                    stdout.push_str(&format!("\n{} {source_ref}", paint_key("Ref:")));
                }
                if let Some(branch) = &repo.branch {
                    stdout.push_str(&format!("\n{} {branch}", paint_key("Branch:")));
                }
                if let Some(head) = &repo.head {
                    stdout.push_str(&format!("\n{} {head}", paint_key("Head:")));
                }
                if let Some(target_ref) = &repo.target_ref {
                    stdout.push_str(&format!(
                        "\n{} {}",
                        paint_key("Target:"),
                        render_workspace_target_label(target_ref, repo.target_source.as_deref())
                    ));
                }
                if let Some(ahead) = repo.ahead {
                    stdout.push_str(&format!("\n{} {ahead}", paint_key("Ahead:")));
                }
                if let Some(behind) = repo.behind {
                    stdout.push_str(&format!("\n{} {behind}", paint_key("Behind:")));
                }
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Dirty:"),
                    if repo.dirty { "yes" } else { "no" }
                ));
                for finding in &repo.findings {
                    let why = compact_backticked_paths(&finding.why);
                    let next = compact_backticked_paths(&finding.next);
                    stdout.push_str(&format!(
                        "\n\n{}  {}\n{} {}\n{} {}",
                        render_severity(finding.severity),
                        render_finding_summary(finding.severity, &finding.summary),
                        finding_detail_key(finding.severity, "Why:"),
                        why,
                        finding_detail_key(finding.severity, "Next:"),
                        next
                    ));
                }
            }

            stdout.push('\n');
            stdout.push('\n');
            stdout.push_str(&render_workspace_status_summary(&summary));
            append_post_summary_next_block(&mut stdout, &report.next_steps);

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceStatusSuccess {
                ok,
                path,
                mode: "status",
                summary,
                next: report.next.as_deref(),
                next_steps: &report.next_steps,
                artifact_routing: workspace_follow_up_artifact_routing(path),
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if ok { 0 } else { 1 },
        },
    }
}

pub(crate) fn render_workspace_receipt(
    path: &str,
    report: &WorkspaceReceiptReport,
    format: OutputFormat,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let mut stdout = format!(
                "{}\n{}",
                format_command_header("WORKSPACE RECEIPT", path),
                render_execution_receipt_text(&report.receipt)
            );
            if let Some(archive_path) = report.archive_path.as_ref() {
                stdout.push_str(&summary_detail_line(
                    "Archive:",
                    &compact_path(archive_path, "."),
                ));
                stdout.push('\n');
            }

            stdout.push('\n');

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.receipt.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: {
                let archive_path = report
                    .archive_path
                    .as_ref()
                    .map(|path| path.display().to_string());
                let payload = WorkspaceReceiptSuccess {
                    ok: report.receipt.ok,
                    path,
                    mode: "receipt",
                    summary: report.receipt.summary,
                    receipt: report.receipt.clone(),
                    archive_path: archive_path.as_deref(),
                    artifact_routing: workspace_receipt_artifact_routing(
                        path,
                        archive_path.as_deref(),
                    ),
                    repos: &report.repos,
                };
                to_json(&payload)
            },
            stderr: None,
            exit_code: if report.receipt.ok { 0 } else { 1 },
        },
    }
}

pub(crate) fn render_workspace_execution_plan(
    path: &str,
    report: &WorkspaceExecutionPlanReport,
    format: OutputFormat,
    overrides: Option<ExecutionPlanOverrides>,
) -> CommandOutput {
    let summary = workspace_execution_plan_summary(&report.repos);
    let ok = summary.unresolved_count == 0;

    match format {
        OutputFormat::Text => {
            let workspace_root = Path::new(path).parent();
            let mut stdout = format!(
                "{}\n\n{}",
                format_command_header("WORKSPACE EXECUTION PLAN", path),
                render_readiness_status(ok)
            );

            if let Some(overrides) = overrides.as_ref() {
                stdout.push_str(&format!("\n\n{}\n", paint_section_title("Overrides")));
                let mut first = true;
                if let Some(backend) = overrides.backend.as_deref() {
                    stdout.push_str(&format!(
                        "{} {} {}",
                        if first { "" } else { "\n" },
                        paint_key("Backend:"),
                        paint_backticked_code(backend)
                    ));
                    first = false;
                }
                if let Some(lifecycle) = overrides.lifecycle.as_deref() {
                    stdout.push_str(&format!(
                        "{} {} {}",
                        if first { "" } else { "\n" },
                        paint_key("Lifecycle:"),
                        paint_backticked_code(lifecycle)
                    ));
                }
            }

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    workspace_execution_plan_status_word(&repo.status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Path:"),
                    compact_repo_path(Path::new(&repo.path))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Contract:"),
                    compact_contract_file_path_relative_to(
                        Path::new(&repo.contract_path),
                        DEFAULT_CONTRACT_FILE,
                        workspace_root,
                    )
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Acquired:"),
                    if repo.acquired { "yes" } else { "no" }
                ));
                if let Some(workflow) = repo.workflow.as_deref() {
                    stdout.push_str(&format!(
                        "\n{} {}",
                        paint_key("Workflow:"),
                        paint_backticked_code(workflow)
                    ));
                }
                if let Some(task) = repo.task.as_deref() {
                    stdout.push_str(&format!(
                        "\n{} {}",
                        paint_key("Task:"),
                        paint_backticked_code(task)
                    ));
                }

                if let Some(resolved) = repo.resolved.as_ref() {
                    stdout.push_str(&format!(
                        "\n{} {} {} ({})",
                        summary_bullet(),
                        paint_key("Backend:"),
                        paint_backticked_code(&resolved.backend),
                        resolved.backend_source
                    ));
                    if let Some(lifecycle) = resolved.lifecycle.as_deref() {
                        let source = resolved.lifecycle_source.as_deref().unwrap_or("implicit");
                        stdout.push_str(&format!(
                            "\n{} {} {} ({})",
                            summary_bullet(),
                            paint_key("Lifecycle:"),
                            paint_backticked_code(lifecycle),
                            source
                        ));
                    }
                    if let Some(image) = resolved.image.as_deref() {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            summary_bullet(),
                            paint_key("Image:"),
                            paint_backticked_code(image)
                        ));
                    }
                    if let Some(engine) = resolved.engine.as_deref() {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            summary_bullet(),
                            paint_key("Engine:"),
                            paint_backticked_code(engine)
                        ));
                    }
                    if !resolved.engine_candidates.is_empty() {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            summary_bullet(),
                            paint_key("Engine candidates:"),
                            render_inline_code_list(&resolved.engine_candidates)
                        ));
                    }
                    if let Some(provider) = resolved.provider.as_deref() {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            summary_bullet(),
                            paint_key("Provider:"),
                            paint_backticked_code(provider)
                        ));
                    }
                    if let Some(target) = resolved.target.as_deref() {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            summary_bullet(),
                            paint_key("Target:"),
                            paint_backticked_code(target)
                        ));
                    }
                    stdout.push_str(&format!(
                        "\n{} {} {}",
                        summary_bullet(),
                        paint_key("Target strategy:"),
                        resolved.target_strategy
                    ));
                    if let Some(cwd) = resolved.cwd.as_deref() {
                        stdout.push_str(&format!(
                            "\n{} {} {}",
                            summary_bullet(),
                            paint_key("Cwd:"),
                            paint_backticked_code(cwd)
                        ));
                    }
                } else if let Some(error) = repo.error.as_deref() {
                    stdout.push_str(&format!(
                        "\n{} {}",
                        error_key("Why:"),
                        compact_backticked_paths(error)
                    ));
                    if let Some(next) = repo.next.as_deref() {
                        stdout.push_str(&format!(
                            "\n{} {}",
                            error_next_key("Next:"),
                            compact_backticked_paths(next)
                        ));
                    }
                }

                if let Some(contract_identity) = repo.contract_identity.as_ref() {
                    stdout.push_str(&format!(
                        "\n\n{}",
                        render_contract_identity_text(contract_identity)
                    ));
                }
                if let Some(declared_execution) = repo.declared_execution.as_ref() {
                    stdout.push_str(&render_workspace_execution_text(declared_execution));
                }
            }

            stdout.push_str(&render_workspace_execution_plan_summary(&summary));

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceExecutionPlanSuccess {
                ok,
                path,
                mode: "execution-plan",
                summary,
                repos: &report.repos,
                overrides,
            }),
            stderr: None,
            exit_code: if ok { 0 } else { 1 },
        },
    }
}

pub(crate) fn render_workspace_run(
    task: &str,
    path: &str,
    report: &WorkspaceRunReport,
    format: OutputFormat,
    show_receipt: bool,
) -> CommandOutput {
    match format {
        OutputFormat::Text => {
            let workspace_root = Path::new(path).parent();
            let mut stdout = format!(
                "{}\n\n{}",
                format_command_header("WORKSPACE RUN", &format!("{task} {path}")),
                render_readiness_status(report.ok)
            );

            for repo in &report.repos {
                stdout.push_str(&format!(
                    "\n\n{} {} [{}] ({})",
                    list_bullet(),
                    paint(&repo.name, "1"),
                    if repo.required {
                        "required"
                    } else {
                        "optional"
                    },
                    workspace_status_word(&repo.status)
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Path:"),
                    compact_repo_path(Path::new(&repo.path))
                ));
                stdout.push_str(&format!(
                    "\n{} {}",
                    paint_key("Contract:"),
                    compact_contract_file_path_relative_to(
                        Path::new(&repo.contract_path),
                        DEFAULT_CONTRACT_FILE,
                        workspace_root,
                    )
                ));
                stdout.push_str(&format!("\n{} {}", paint_key("Task:"), repo.task));
                if let Some(source_url) = &repo.source_url {
                    stdout.push_str(&format!("\n{} {source_url}", paint_key("Source:")));
                }
                if let Some(source_ref) = &repo.source_ref {
                    stdout.push_str(&format!("\n{} {source_ref}", paint_key("Ref:")));
                }
                if let Some(task_command) = &repo.task_command {
                    stdout.push_str(&format!("\n{} {task_command}", paint_key("Command:")));
                }
                if let Some(exit_code) = repo.exit_code {
                    stdout.push_str(&format!("\n{} {exit_code}", paint_key("Exit code:")));
                }
                for finding in &repo.findings {
                    let why = compact_backticked_paths(&finding.why);
                    let next = compact_backticked_paths(&finding.next);
                    stdout.push_str(&format!(
                        "\n\n{}  {}\n{} {}\n{} {}",
                        render_severity(finding.severity),
                        render_finding_summary(finding.severity, &finding.summary),
                        finding_detail_key(finding.severity, "Why:"),
                        why,
                        finding_detail_key(finding.severity, "Next:"),
                        next
                    ));
                }
                append_primary_output_block(
                    &mut stdout,
                    workspace_run_output_label(repo),
                    repo.stdout.as_deref(),
                    repo.stderr.as_deref(),
                );
            }
            if show_receipt {
                stdout.push_str(&render_execution_receipt_text(&report.receipt));
            }
            stdout.push('\n');
            stdout.push('\n');
            stdout.push_str(&render_execution_receipt_summary_block(
                &report.receipt,
                Some(task),
                "WORKSPACE RUN SUMMARY",
            ));
            append_receipt_next_block(&mut stdout, &report.receipt);

            CommandOutput {
                stdout,
                stderr: None,
                exit_code: if report.ok { 0 } else { 1 },
            }
        }
        OutputFormat::Json => CommandOutput {
            stdout: to_json(&WorkspaceRunSuccess {
                ok: report.ok,
                path,
                task,
                summary: report.receipt.summary,
                receipt: report.receipt.clone(),
                artifact_routing: workspace_follow_up_artifact_routing(path),
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
        },
    }
}

fn workspace_status_word(status: &str) -> String {
    let trimmed = status.trim();
    if plain_mode() {
        return trimmed.to_string();
    }
    match trimmed {
        "READY" => paint("READY", "1;38;2;102;217;255"),
        "NOT READY" => paint("NOT READY", "1;38;2;255;214;79"),
        "NOT ACQUIRED" => paint("NOT ACQUIRED", "1;38;2;0;180;255"),
        "SKIPPED" => paint("SKIPPED", "1;38;2;180;180;180"),
        "BLOCKED" => paint("BLOCKED", "1;38;2;183;134;255"),
        "PREVIEW" => paint("PREVIEW", "1;38;2;0;255;255"),
        "WARN" => paint("WARN", "1;38;2;0;180;255"),
        value if value.contains("FAILED") => paint(value, "1;31"),
        other => paint(other, "1;37"),
    }
}

fn workspace_phase_output_label(phase: &str) -> &'static str {
    match phase {
        "acquisition" => "Acquire output",
        "services" => "Service output",
        "setup" => "Setup output",
        "refresh" => "Refresh output",
        _ => "Task output",
    }
}

fn workspace_run_output_label(repo: &WorkspaceRepoRunReport) -> &'static str {
    if repo.status == "ACQUIRE FAILED" || (repo.source_url.is_some() && repo.task_command.is_none())
    {
        "Acquire output"
    } else {
        "Task output"
    }
}

fn workspace_phase_command_label(phase: &str) -> &'static str {
    match phase {
        "services" => "Service command:",
        "refresh" => "Refresh command:",
        _ => "Command:",
    }
}

fn workspace_diff_status_word(status: &str) -> String {
    let trimmed = status.trim();
    if plain_mode() {
        return match trimmed {
            "MATCH" => String::from("MATCH"),
            "DIFFERENT" => String::from("DIFFERENT"),
            "DIRTY" => String::from("DIRTY"),
            "MISSING" => String::from("MISSING"),
            "MISSING CONTRACT" => String::from("MISSING CONTRACT"),
            "UNRESOLVED" => String::from("UNRESOLVED"),
            other => other.to_string(),
        };
    }

    match trimmed {
        "MATCH" => paint("MATCH", "1;38;2;102;217;255"),
        "DIFFERENT" => paint("DIFFERENT", "1;38;2;255;235;59"),
        "DIRTY" => paint("DIRTY", "1;38;2;0;255;200"),
        "MISSING" => paint("MISSING", "1;38;2;255;80;80"),
        "MISSING CONTRACT" => paint("MISSING CONTRACT", "1;38;2;255;80;80"),
        "UNRESOLVED" => paint("UNRESOLVED", "1;38;2;183;134;255"),
        other => paint(other, "1;37"),
    }
}

fn workspace_execution_plan_status_word(status: &str) -> String {
    let trimmed = status.trim();
    if plain_mode() {
        return trimmed.to_string();
    }

    match trimmed {
        "RESOLVED" => paint("RESOLVED", "1;38;2;102;217;255"),
        "NOT ACQUIRED" => paint("NOT ACQUIRED", "1;38;2;0;180;255"),
        "MISSING CONTRACT" => paint("MISSING CONTRACT", "1;38;2;255;80;80"),
        "INVALID CONTRACT" => paint("INVALID CONTRACT", "1;38;2;255;80;80"),
        "UNREADABLE CONTRACT" => paint("UNREADABLE CONTRACT", "1;38;2;255;80;80"),
        "UNRESOLVED" => paint("UNRESOLVED", "1;38;2;183;134;255"),
        other => paint(other, "1;37"),
    }
}

fn render_workspace_diff_summary(repos: &[WorkspaceRepoDiffReport]) -> String {
    let summary = workspace_diff_summary(repos);
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Summary"));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Repos:", "1;38;2;102;217;255"),
            &paint(&summary.repo_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Match:", "1;38;2;102;217;255"),
            &paint(&summary.match_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Different:", "1;38;2;0;180;255"),
            &paint(&summary.different_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Dirty:", "1;38;2;0;255;200"),
            &paint(&summary.dirty_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Missing:", "1;38;2;255;80;80"),
            &paint(&summary.missing_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    if summary.missing_repo_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Missing repo:", "1;38;2;255;80;80"),
                &paint(
                    &summary.missing_repo_count.to_string(),
                    "1;38;2;255;255;255"
                ),
            )
        ));
    }
    if summary.missing_contract_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Missing contract:", "1;38;2;255;80;80"),
                &paint(
                    &summary.missing_contract_count.to_string(),
                    "1;38;2;255;255;255",
                ),
            )
        ));
    }
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Unresolved:", "1;38;2;183;134;255"),
            &paint(&summary.unresolved_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    if summary.target_unavailable_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Target unavailable:", "1;38;2;183;134;255"),
                &paint(
                    &summary.target_unavailable_count.to_string(),
                    "1;38;2;255;255;255",
                ),
            )
        ));
    }
    if summary.comparison_unresolved_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Comparison unavailable:", "1;38;2;183;134;255"),
                &paint(
                    &summary.comparison_unresolved_count.to_string(),
                    "1;38;2;255;255;255",
                ),
            )
        ));
    }
    stdout
}

fn render_workspace_target_label(target_ref: &str, target_source: Option<&str>) -> String {
    match target_source {
        Some("declared_ref") => format!("{target_ref} (declared source ref)"),
        Some("upstream_branch") => format!("{target_ref} (upstream branch)"),
        _ => target_ref.to_string(),
    }
}

fn workspace_diff_summary(repos: &[WorkspaceRepoDiffReport]) -> WorkspaceDiffSummary {
    let mut summary = WorkspaceDiffSummary {
        repo_count: repos.len(),
        ..WorkspaceDiffSummary::default()
    };

    for repo in repos {
        match repo.status.as_str() {
            "MATCH" => summary.match_count += 1,
            "DIFFERENT" => summary.different_count += 1,
            "DIRTY" => summary.dirty_count += 1,
            "MISSING" => summary.missing_count += 1,
            "MISSING CONTRACT" => summary.missing_count += 1,
            "UNRESOLVED" => summary.unresolved_count += 1,
            _ => {}
        }
        match repo.drift_kind.as_str() {
            "missing_repo" => summary.missing_repo_count += 1,
            "missing_contract" => summary.missing_contract_count += 1,
            "target_unavailable" => summary.target_unavailable_count += 1,
            "comparison_unresolved" => summary.comparison_unresolved_count += 1,
            _ => {}
        }
        for finding in &repo.findings {
            match finding.severity {
                FindingSeverity::Error => summary.error_count += 1,
                FindingSeverity::Warn => summary.warn_count += 1,
                FindingSeverity::Info => summary.info_count += 1,
            }
        }
    }

    summary
}

fn render_workspace_status_summary(summary: &WorkspaceStatusSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Summary"));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Repos:", "1;38;2;102;217;255"),
            &paint(&summary.repo_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Ready:", "1;38;2;0;255;120"),
            &paint(&summary.ready_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Not ready:", "1;38;2;255;214;79"),
            &paint(&summary.not_ready_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Match:", "1;38;2;0;255;120"),
            &paint(&summary.match_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Different:", "1;38;2;255;235;59"),
            &paint(&summary.different_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Dirty:", "1;38;2;61;174;255"),
            &paint(&summary.dirty_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Missing:", "1;38;2;255;80;80"),
            &paint(&summary.missing_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    if summary.missing_repo_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Missing repo:", "1;38;2;255;80;80"),
                &paint(
                    &summary.missing_repo_count.to_string(),
                    "1;38;2;255;255;255"
                ),
            )
        ));
    }
    if summary.missing_contract_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Missing contract:", "1;38;2;255;80;80"),
                &paint(
                    &summary.missing_contract_count.to_string(),
                    "1;38;2;255;255;255",
                ),
            )
        ));
    }
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Unresolved:", "1;38;2;183;134;255"),
            &paint(&summary.unresolved_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    if summary.target_unavailable_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Target unavailable:", "1;38;2;183;134;255"),
                &paint(
                    &summary.target_unavailable_count.to_string(),
                    "1;38;2;255;255;255",
                ),
            )
        ));
    }
    if summary.comparison_unresolved_count > 0 {
        stdout.push_str(&format!(
            "\n{}",
            section_list_row(
                &summary_bullet(),
                &paint("Comparison unavailable:", "1;38;2;183;134;255"),
                &paint(
                    &summary.comparison_unresolved_count.to_string(),
                    "1;38;2;255;255;255",
                ),
            )
        ));
    }
    stdout
}

fn render_workspace_execution_plan_summary(summary: &WorkspaceExecutionPlanSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Summary"));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Repos:", "1;38;2;102;217;255"),
            &paint(&summary.repo_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Resolved:", "1;38;2;102;217;255"),
            &paint(&summary.resolved_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Unresolved:", "1;38;2;183;134;255"),
            &paint(&summary.unresolved_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Required unresolved:", "1;38;2;255;80;80"),
            &paint(
                &summary.required_unresolved_count.to_string(),
                "1;38;2;255;255;255",
            ),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Not acquired:", "1;38;2;0;180;255"),
            &paint(
                &summary.not_acquired_count.to_string(),
                "1;38;2;255;255;255"
            ),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Missing contract:", "1;38;2;255;80;80"),
            &paint(
                &summary.missing_contract_count.to_string(),
                "1;38;2;255;255;255",
            ),
        )
    ));
    stdout
}

fn workspace_status_summary(repos: &[WorkspaceRepoStatusReport]) -> WorkspaceStatusSummary {
    let mut summary = WorkspaceStatusSummary {
        repo_count: repos.len(),
        ..WorkspaceStatusSummary::default()
    };

    for repo in repos {
        if repo.ready {
            summary.ready_count += 1;
        } else {
            summary.not_ready_count += 1;
        }
        match repo.drift_status.as_str() {
            "MATCH" => summary.match_count += 1,
            "DIFFERENT" => summary.different_count += 1,
            "DIRTY" => summary.dirty_count += 1,
            "MISSING" => summary.missing_count += 1,
            "MISSING CONTRACT" => summary.missing_count += 1,
            "UNRESOLVED" => summary.unresolved_count += 1,
            _ => {}
        }
        match repo.drift_kind.as_str() {
            "missing_repo" => summary.missing_repo_count += 1,
            "missing_contract" => summary.missing_contract_count += 1,
            "target_unavailable" => summary.target_unavailable_count += 1,
            "comparison_unresolved" => summary.comparison_unresolved_count += 1,
            _ => {}
        }
        for finding in &repo.findings {
            match finding.severity {
                FindingSeverity::Error => summary.error_count += 1,
                FindingSeverity::Warn => summary.warn_count += 1,
                FindingSeverity::Info => summary.info_count += 1,
            }
        }
    }

    summary
}

fn workspace_execution_plan_summary(
    repos: &[WorkspaceRepoExecutionPlanReport],
) -> WorkspaceExecutionPlanSummary {
    let mut summary = WorkspaceExecutionPlanSummary {
        repo_count: repos.len(),
        ..WorkspaceExecutionPlanSummary::default()
    };

    for repo in repos {
        match repo.status.as_str() {
            "RESOLVED" => summary.resolved_count += 1,
            "NOT ACQUIRED" => {
                summary.unresolved_count += 1;
                summary.not_acquired_count += 1;
                if repo.required {
                    summary.required_unresolved_count += 1;
                }
            }
            "MISSING CONTRACT" => {
                summary.unresolved_count += 1;
                summary.missing_contract_count += 1;
                if repo.required {
                    summary.required_unresolved_count += 1;
                }
            }
            _ => {
                summary.unresolved_count += 1;
                if repo.required {
                    summary.required_unresolved_count += 1;
                }
            }
        }
    }

    summary
}

fn append_primary_output_block(
    buffer: &mut String,
    label: &str,
    stdout: Option<&str>,
    stderr: Option<&str>,
) {
    let primary = stderr
        .and_then(|stderr| {
            let stderr = stderr.trim_end();
            (!stderr.is_empty()).then_some(stderr)
        })
        .or_else(|| {
            stdout.and_then(|stdout| {
                let stdout = stdout.trim_end();
                (!stdout.is_empty()).then_some(stdout)
            })
        });

    if let Some(contents) = primary {
        append_output_block(buffer, label, Some(contents));
    }
}
