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
                "\n{}\n\n{}",
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
                    "\n{}\n\n{}",
                    format_command_header("WORKSPACE REFRESH PREVIEW", path),
                    format_mode_line("dry-run (no write)")
                )
            } else {
                format!(
                    "\n{}\n\n{}",
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
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
        },
    }
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
                    stdout.push_str(&format!("\n{} {target_ref}", paint_key("Target:")));
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
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: 0,
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
                "RUN SUMMARY",
            ));

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
        "READY" => paint("READY", "1;38;2;0;255;120"),
        "NOT READY" => paint("NOT READY", "1;38;2;255;235;59"),
        "NOT ACQUIRED" => paint("NOT ACQUIRED", "1;38;2;255;235;59"),
        "SKIPPED" => paint("SKIPPED", "1;38;2;180;180;180"),
        "BLOCKED" => paint("BLOCKED", "1;38;2;255;235;59"),
        "PREVIEW" => paint("PREVIEW", "1;38;2;0;255;255"),
        "WARN" => paint("WARN", "1;38;2;255;235;59"),
        value if value.contains("FAILED") => paint(value, "1;31"),
        other => paint(other, "1;37"),
    }
}

fn workspace_phase_output_label(phase: &str) -> &'static str {
    match phase {
        "acquisition" => "Acquire output:",
        "services" => "Service output:",
        "setup" => "Setup output:",
        "refresh" => "Refresh output:",
        _ => "Task output:",
    }
}

fn workspace_run_output_label(repo: &WorkspaceRepoRunReport) -> &'static str {
    if repo.status == "ACQUIRE FAILED" || (repo.source_url.is_some() && repo.task_command.is_none())
    {
        "Acquire output:"
    } else {
        "Task output:"
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
        "MATCH" => paint("MATCH", "1;38;2;0;255;120"),
        "DIFFERENT" => paint("DIFFERENT", "1;38;2;255;235;59"),
        "DIRTY" => paint("DIRTY", "1;38;2;255;214;79"),
        "MISSING" => paint("MISSING", "1;38;2;255;80;80"),
        "MISSING CONTRACT" => paint("MISSING CONTRACT", "1;38;2;255;80;80"),
        "UNRESOLVED" => paint("UNRESOLVED", "1;38;2;255;235;59"),
        other => paint(other, "1;37"),
    }
}

fn render_workspace_diff_summary(repos: &[WorkspaceRepoDiffReport]) -> String {
    let summary = workspace_diff_summary(repos);
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!(
        "{}:",
        paint_section_title("WORKSPACE DIFF SUMMARY")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Repos:", "1;38;2;102;217;255"),
        paint(&summary.repo_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Match:", "1;38;2;0;255;120"),
        paint(&summary.match_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Different:", "1;38;2;255;235;59"),
        paint(&summary.different_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Dirty:", "1;38;2;255;214;79"),
        paint(&summary.dirty_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Missing:", "1;38;2;255;80;80"),
        paint(&summary.missing_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Unresolved:", "1;38;2;255;235;59"),
        paint(&summary.unresolved_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout
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
