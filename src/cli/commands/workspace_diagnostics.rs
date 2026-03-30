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

pub(crate) fn apply_workspace_doctor_filters(
    report: crate::workspace::WorkspaceDoctorReport,
    filters: &WorkspaceDoctorFilters,
) -> crate::workspace::WorkspaceDoctorReport {
    let mut repos = Vec::new();

    for mut repo in report.repos {
        if let Some(target_repo) = filters.repo.as_deref() {
            if repo.name != target_repo {
                continue;
            }
        }

        match filters.status {
            WorkspaceDoctorStatusFilter::All => {}
            WorkspaceDoctorStatusFilter::Ready if !repo.ok => continue,
            WorkspaceDoctorStatusFilter::NotReady if repo.ok => continue,
            _ => {}
        }

        repo.findings = repo
            .findings
            .into_iter()
            .filter(|finding| match filters.severity {
                WorkspaceDoctorSeverityFilter::All => true,
                WorkspaceDoctorSeverityFilter::Error => finding.severity == FindingSeverity::Error,
                WorkspaceDoctorSeverityFilter::Warn => finding.severity == FindingSeverity::Warn,
                WorkspaceDoctorSeverityFilter::Info => finding.severity == FindingSeverity::Info,
            })
            .collect();

        if !matches!(filters.severity, WorkspaceDoctorSeverityFilter::All)
            && repo.findings.is_empty()
        {
            continue;
        }

        repo.ok = !repo
            .findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error);
        repos.push(repo);
    }

    let ok = repos.iter().all(|repo| !repo.required || repo.ok);

    crate::workspace::WorkspaceDoctorReport { ok, repos }
}

pub(crate) fn render_workspace_doctor_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let summary = workspace_doctor_summary(report);
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE DOCTOR", path),
        render_readiness_status(report.ok)
    );
    if let Some(primary_blocker) = summary.primary_blocker.as_ref() {
        stdout.push_str(&render_primary_blocker_text(
            "Primary Blocker",
            &format!(
                "{} [{}]",
                primary_blocker.repo,
                render_finding_summary(primary_blocker.severity, &primary_blocker.summary)
            ),
            &primary_blocker.why,
            &primary_blocker.next,
        ));
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
            render_status_word(if repo.ok { "READY" } else { "NOT READY" })
        ));
        if !concise_mode() {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Path:"),
                compact_repo_path(Path::new(&repo.path))
            ));
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Contract:"),
                compact_contract_path(Path::new(&repo.contract_path))
            ));
        }

        if let Some(execution) = repo.execution.as_ref() {
            stdout.push_str(&render_workspace_execution_text(execution));
        }

        if !repo.extensions.is_empty() {
            stdout.push_str(&render_extensions_text(&repo.extensions));
        }

        for finding in &repo.findings {
            let next = compact_backticked_paths(&finding.next);
            if concise_mode() {
                stdout.push_str(&format!(
                    "\n\n{}  {}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            } else {
                let why = compact_backticked_paths(&finding.why);
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
    }
    stdout.push_str(&render_workspace_summary_text(&summary));

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

pub(crate) fn render_workspace_explain_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE EXPLAIN", path),
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
            render_status_word(if repo.ok { "READY" } else { "NOT READY" })
        ));
        if !concise_mode() {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Path:"),
                compact_repo_path(Path::new(&repo.path))
            ));
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Contract:"),
                compact_contract_path(Path::new(&repo.contract_path))
            ));
        }

        let steps = explain_steps(&repo.findings);
        stdout.push_str(&render_explain_steps_text(&steps));
    }
    stdout.push_str(&render_workspace_explain_summary_text(
        &workspace_explain_summary(report),
    ));

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

pub(crate) fn render_check_summary_text(summary: &DoctorSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("Verdict")));
    stdout.push_str(&format!(
        " {}  {} {}",
        paint("➡", "1;38;2;255;214;95"),
        paint_key("Repo:"),
        render_doctor_verdict(summary.verdict)
    ));
    stdout.push_str(&format!(
        "\n {}  {} {}",
        paint("➡", "1;38;2;255;214;95"),
        paint_key("Agent:"),
        render_doctor_verdict(summary.agent_verdict)
    ));
    stdout.push_str("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("SUMMARY")));
    stdout.push_str(&format!(
        "{} {}",
        paint("Errors:", "1;38;2;255;255;255"),
        paint(&summary.error_count.to_string(), "1;31")
    ));
    stdout.push_str(&format!(
        "\n{} {}",
        paint("Warnings:", "1;38;2;255;255;255"),
        paint(&summary.warn_count.to_string(), "1;33")
    ));
    stdout.push_str(&format!(
        "\n{} {}",
        paint("Info:", "1;38;2;255;255;255"),
        paint(&summary.info_count.to_string(), "1;36")
    ));
    stdout
}

fn render_workspace_summary_text(summary: &WorkspaceDoctorSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("Verdict")));
    stdout.push_str(&format!(
        " {}  {} {}",
        paint("➡", "1;38;2;255;214;95"),
        paint_key("Repo:"),
        render_doctor_verdict(summary.verdict)
    ));
    stdout.push_str(&format!(
        "\n {}  {} {}",
        paint("➡", "1;38;2;255;214;95"),
        paint_key("Agent:"),
        render_doctor_verdict(summary.agent_verdict)
    ));
    stdout.push_str("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("SUMMARY")));
    stdout.push_str(&format!(
        "{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Repos:", "1;38;2;102;217;255"),
        paint(&summary.repo_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Ready:", "1;38;2;0;255;120"),
        paint(&summary.ready_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Not Ready:", "1;38;2;255;235;59"),
        paint(&summary.not_ready_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Errors:", "1;31"),
        paint(&summary.error_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Warnings:", "1;33"),
        paint(&summary.warn_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Info:", "1;36"),
        paint(&summary.info_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout
}

fn render_workspace_explain_summary_text(summary: &WorkspaceExplainSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!("{}:", paint_section_title("SUMMARY")));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Repos:", "1;38;2;102;217;255"),
        paint(&summary.repo_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Ready:", "1;38;2;0;255;120"),
        paint(&summary.ready_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Not Ready:", "1;38;2;255;235;59"),
        paint(&summary.not_ready_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Errors:", "1;31"),
        paint(&summary.error_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Warnings:", "1;33"),
        paint(&summary.warn_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Info:", "1;36"),
        paint(&summary.info_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout.push_str(&format!(
        "\n{} {} {}",
        paint("»", "1;38;2;255;214;79"),
        paint("Steps:", "1;38;2;102;217;255"),
        paint(&summary.step_count.to_string(), "1;38;2;255;255;255")
    ));
    stdout
}

pub(crate) fn render_workspace_check_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE CHECK", path),
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
            render_status_word(if repo.ok { "READY" } else { "NOT READY" })
        ));
        if !concise_mode() {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Path:"),
                compact_repo_path(Path::new(&repo.path))
            ));
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Contract:"),
                compact_contract_path(Path::new(&repo.contract_path))
            ));
        }
        if let Some(execution) = repo.execution.as_ref() {
            stdout.push_str(&render_workspace_execution_text(execution));
        }
        if !repo.extensions.is_empty() {
            stdout.push_str(&render_extensions_text(&repo.extensions));
        }
        for finding in &repo.findings {
            let next = compact_backticked_paths(&finding.next);
            if concise_mode() {
                stdout.push_str(&format!(
                    "\n\n{}  {}\n{} {}",
                    render_severity(finding.severity),
                    render_finding_summary(finding.severity, &finding.summary),
                    finding_detail_key(finding.severity, "Next:"),
                    next
                ));
            } else {
                let why = compact_backticked_paths(&finding.why);
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
    }
    stdout.push_str(&render_workspace_summary_text(&workspace_doctor_summary(
        report,
    )));

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}
