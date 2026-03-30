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
                    render_status_word(&repo.status)
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
                if let Some(service) = &repo.service {
                    stdout.push_str(&format!("\n{} {service}", paint_key("Service:")));
                }
                if let Some(task) = &repo.task {
                    stdout.push_str(&format!("\n{} {task}", paint_key("Task:")));
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
                append_output_block(&mut stdout, "Stdout", repo.stdout.as_deref());
                append_output_block(&mut stdout, "Stderr", repo.stderr.as_deref());
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
                "WORKSPACE RUN SUMMARY",
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
                summary: report.receipt.summary,
                receipt: report.receipt.clone(),
                repos: &report.repos,
            }),
            stderr: None,
            exit_code: if report.ok { 0 } else { 1 },
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
                    render_status_word(&repo.status)
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
                append_output_block(&mut stdout, "Stdout", repo.stdout.as_deref());
                append_output_block(&mut stdout, "Stderr", repo.stderr.as_deref());
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
