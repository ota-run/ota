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
use crate::cli::commands::workspace_output::render_workspace_repo_findings_text;

pub(crate) fn normalize_workspace_doctor_report(
    mut report: crate::workspace::WorkspaceDoctorReport,
) -> crate::workspace::WorkspaceDoctorReport {
    for repo in &mut report.repos {
        repo.primary_blocker = workspace_repo_primary_blocker(&repo.findings);
    }

    report
}

pub(crate) fn apply_workspace_doctor_filters(
    report: crate::workspace::WorkspaceDoctorReport,
    filters: &WorkspaceDoctorFilters,
) -> crate::workspace::WorkspaceDoctorReport {
    let report = normalize_workspace_doctor_report(report);
    let mut repos = Vec::new();

    for mut repo in report.repos {
        if let Some(target_repo) = filters.repo.as_deref()
            && repo.name != target_repo
        {
            continue;
        }

        match filters.status {
            WorkspaceDoctorStatusFilter::All => {}
            WorkspaceDoctorStatusFilter::Ready if !repo.ok => continue,
            WorkspaceDoctorStatusFilter::NotReady if repo.ok => continue,
            _ => {}
        }

        repo.findings.retain(|finding| match filters.severity {
            WorkspaceDoctorSeverityFilter::All => true,
            WorkspaceDoctorSeverityFilter::Error => finding.severity == FindingSeverity::Error,
            WorkspaceDoctorSeverityFilter::Warn => finding.severity == FindingSeverity::Warn,
            WorkspaceDoctorSeverityFilter::Info => finding.severity == FindingSeverity::Info,
        });

        if !matches!(filters.severity, WorkspaceDoctorSeverityFilter::All)
            && repo.findings.is_empty()
        {
            continue;
        }

        repo.ok = !repo
            .findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error);
        repo.primary_blocker = workspace_repo_primary_blocker(&repo.findings);
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
        if !stdout.ends_with("\n\n") {
            stdout.push_str("\n\n");
        }
        stdout.push_str(&render_primary_finding_text(
            primary_blocker.severity,
            &format!(
                "{} [{}]",
                primary_blocker.repo,
                render_finding_summary(primary_blocker.severity, &primary_blocker.summary)
            ),
            &primary_blocker.why,
            &primary_blocker.next,
            primary_blocker.provenance.clone(),
            None,
            None,
        ));
    }
    if report.ok && report.repos.iter().all(|repo| repo.findings.is_empty()) {
        stdout.push_str(&format_next_timeline(&[
            String::from("run `ota workspace up` to prepare the workspace end to end"),
            String::from("run `ota workspace tasks` to inspect runnable workspace tasks"),
        ]));
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
                compact_repo_path(Path::new(&repo.contract_path))
            ));
        }

        if let Some(execution) = repo.execution.as_ref() {
            stdout.push_str(&render_workspace_execution_text(execution));
        }

        if !repo.extensions.is_empty() {
            stdout.push_str(&render_extensions_text(&repo.extensions));
        }
        if repo.findings.len() > 1
            && let Some(primary_blocker) = repo.primary_blocker.as_ref()
        {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Primary next:"),
                compact_backticked_paths(&primary_blocker.next)
            ));
        }
        stdout.push_str(&render_workspace_repo_findings_text(&repo.findings));
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
    let action_count = report
        .repos
        .iter()
        .map(|repo| explain_action_count(&repo.findings))
        .sum::<usize>();
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE EXPLAIN", path),
        render_explain_status(report.ok, action_count)
    );
    let actions = workspace_explain_actions(report);

    if !actions.is_empty() {
        stdout.push_str("\n\n");
        stdout.push_str(&paint_section_title("Plan"));
        for action in &actions {
            stdout.push_str(&format!(
                "\n {}. {} [{}] {}",
                action.action.order,
                paint(&action.repo, "1"),
                if action.required {
                    "required"
                } else {
                    "optional"
                },
                render_finding_summary(action.action.severity, &action.action.action_title)
            ));
            stdout.push_str(&format!(
                "\n{} {}",
                finding_detail_key(action.action.severity, "Why:"),
                compact_backticked_paths(&action.action.why)
            ));
            let next_steps = finding_next_steps(&action.action.next);
            append_error_detail_section(&mut stdout, "Next:", &next_steps, None);
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
                compact_repo_path(Path::new(&repo.contract_path))
            ));
        }

        stdout.push_str(&render_explain_steps_text(
            &repo.findings,
            Path::new(&repo.contract_path),
        ));
    }
    stdout.push_str(&render_workspace_explain_summary_text(
        &workspace_explain_summary(report),
        actions.len(),
    ));

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}

fn workspace_repo_primary_blocker(
    findings: &[Finding],
) -> Option<crate::workspace::WorkspaceRepoPrimaryBlocker> {
    findings
        .first()
        .map(|finding| crate::workspace::WorkspaceRepoPrimaryBlocker {
            severity: finding.severity,
            summary: finding.summary.clone(),
            why: finding.why.clone(),
            next: finding.next.clone(),
            provenance: finding.provenance(),
            provenance_key: finding.provenance_key(),
        })
}

pub(crate) fn render_check_summary_text(summary: &DoctorSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("Verdict")));
    stdout.push_str(&format!(
        " {}  {} {}",
        paint("●", "1;38;2;255;214;95"),
        paint_key("Repo:"),
        render_doctor_verdict(summary.verdict)
    ));
    stdout.push_str(&format!(
        "\n {}  {} {}",
        paint("●", "1;38;2;255;214;95"),
        paint_key("Agent:"),
        render_doctor_verdict(summary.agent_verdict)
    ));
    stdout.push_str("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("Overview")));
    stdout.push_str(&section_list_row(
        &summary_bullet(),
        &paint("Errors:", "1;31"),
        &paint(&summary.error_count.to_string(), "1;31"),
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Warnings:", "1;33"),
            &paint(&summary.warn_count.to_string(), "1;33"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Info:", "1;36"),
            &paint(&summary.info_count.to_string(), "1;36"),
        )
    ));
    stdout
}

fn render_workspace_summary_text(summary: &WorkspaceDoctorSummary) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("Verdict")));
    stdout.push_str(&format!(
        " {}  {} {}",
        paint("●", "1;38;2;255;214;95"),
        paint_key("Repo:"),
        render_doctor_verdict(summary.verdict)
    ));
    stdout.push_str(&format!(
        "\n {}  {} {}",
        paint("●", "1;38;2;255;214;95"),
        paint_key("Agent:"),
        render_doctor_verdict(summary.agent_verdict)
    ));
    stdout.push_str("\n\n");
    stdout.push_str(&format!("{}\n", paint_section_title("Overview")));
    stdout.push_str(&section_list_row(
        &summary_bullet(),
        &paint("Repos:", "1;38;2;102;217;255"),
        &paint(&summary.repo_count.to_string(), "1;38;2;255;255;255"),
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
            &paint("Not Ready:", "1;38;2;255;235;59"),
            &paint(&summary.not_ready_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Errors:", "1;31"),
            &paint(&summary.error_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Warnings:", "1;33"),
            &paint(&summary.warn_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Info:", "1;36"),
            &paint(&summary.info_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout
}

fn render_workspace_explain_summary_text(
    summary: &WorkspaceExplainSummary,
    action_count: usize,
) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Overview"));
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
            &paint("Not Ready:", "1;38;2;255;235;59"),
            &paint(&summary.not_ready_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Actions:", "1;38;2;102;217;255"),
            &paint(&action_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Errors:", "1;31"),
            &paint(&summary.error_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Warnings:", "1;33"),
            &paint(&summary.warn_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout.push_str(&format!(
        "\n{}",
        section_list_row(
            &summary_bullet(),
            &paint("Info:", "1;36"),
            &paint(&summary.info_count.to_string(), "1;38;2;255;255;255"),
        )
    ));
    stdout
}

pub(crate) fn render_workspace_check_text(
    path: &str,
    report: &crate::workspace::WorkspaceDoctorReport,
) -> CommandOutput {
    let summary = workspace_doctor_summary(report);
    let mut stdout = format!(
        "{}\n\n{}",
        format_command_header("WORKSPACE CHECK", path),
        render_readiness_status(report.ok)
    );
    if let Some(primary_blocker) = summary.primary_blocker.as_ref() {
        if !stdout.ends_with("\n\n") {
            stdout.push_str("\n\n");
        }
        stdout.push_str(&render_primary_finding_text(
            primary_blocker.severity,
            &format!(
                "{} [{}]",
                primary_blocker.repo,
                render_finding_summary(primary_blocker.severity, &primary_blocker.summary)
            ),
            &primary_blocker.why,
            &primary_blocker.next,
            primary_blocker.provenance.clone(),
            None,
            None,
        ));
    }
    if report.ok && report.repos.iter().all(|repo| repo.findings.is_empty()) {
        stdout.push_str(&format_next_timeline(&[
            String::from("run `ota workspace up` to prepare the workspace end to end"),
            String::from("run `ota workspace tasks` to inspect runnable workspace tasks"),
        ]));
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
                compact_repo_path(Path::new(&repo.contract_path))
            ));
        }
        if let Some(execution) = repo.execution.as_ref() {
            stdout.push_str(&render_workspace_execution_text(execution));
        }
        if !repo.extensions.is_empty() {
            stdout.push_str(&render_extensions_text(&repo.extensions));
        }
        if repo.findings.len() > 1
            && let Some(primary_blocker) = repo.primary_blocker.as_ref()
        {
            stdout.push_str(&format!(
                "\n{} {}",
                paint_key("Primary next:"),
                compact_backticked_paths(&primary_blocker.next)
            ));
        }
        stdout.push_str(&render_workspace_repo_findings_text(&repo.findings));
    }
    stdout.push_str(&render_workspace_summary_text(&summary));

    CommandOutput {
        stdout,
        stderr: None,
        exit_code: if report.ok { 0 } else { 1 },
    }
}
