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

use std::collections::BTreeSet;
use std::path::Path;

use super::*;

pub(super) fn workspace_explain_summary(
    report: &crate::workspace::WorkspaceDoctorReport,
) -> WorkspaceExplainSummary {
    let mut summary = WorkspaceExplainSummary {
        repo_count: report.repos.len(),
        ..WorkspaceExplainSummary::default()
    };

    for repo in &report.repos {
        if repo.ok {
            summary.ready_count += 1;
        } else {
            summary.not_ready_count += 1;
        }

        let repo_summary = explain_summary_from_findings(&repo.findings);
        summary.error_count += repo_summary.error_count;
        summary.warn_count += repo_summary.warn_count;
        summary.info_count += repo_summary.info_count;
        summary.step_count += repo_summary.step_count;
    }

    summary
}

pub(super) fn workspace_explain_repos(
    report: &crate::workspace::WorkspaceDoctorReport,
) -> Vec<WorkspaceRepoExplainReport> {
    report
        .repos
        .iter()
        .map(|repo| WorkspaceRepoExplainReport {
            name: repo.name.clone(),
            path: repo.path.clone(),
            contract_path: repo.contract_path.clone(),
            required: repo.required,
            ok: repo.ok,
            summary: explain_summary_from_findings(&repo.findings),
            steps: explain_steps(&repo.findings),
        })
        .collect()
}

pub(super) fn render_explain_steps_text(findings: &[Finding], contract_path: &Path) -> String {
    let mut stdout = String::from("\n\n");
    stdout.push_str(&paint_section_title("Plan"));
    let groups = group_doctor_findings(findings.iter());
    if groups.is_empty() {
        if plain_mode() {
            stdout.push_str("\n* none");
        } else {
            stdout.push_str(&format!("\n{} none", paint("✦", "1;38;2;255;214;79")));
        }
        return stdout;
    }

    for (index, group) in groups.iter().enumerate() {
        let finding_count = group.findings.len();
        let per_finding_next = explain_group_uses_per_finding_nexts(group);
        let render_items = explain_group_renders_items(group);
        if index > 0 {
            stdout.push_str("\n\n");
        } else {
            stdout.push('\n');
        }
        stdout.push_str(&format!(
            " {}. {} {}",
            index + 1,
            render_finding_summary(group.severity, &explain_group_title(group)),
            paint_group_meta(&format!("({finding_count})"))
        ));
        if !concise_mode() {
            append_wrapped_labeled_text(
                &mut stdout,
                "Why:",
                &explain_group_why(group),
                "  ",
                84,
                false,
                |_| explain_why_key(),
                |value| render_backticked_text(value, Some(contract_path)),
            );
        }
        if render_items {
            for finding in &group.findings {
                append_wrapped_bullet_text(
                    &mut stdout,
                    summary_bullet(),
                    &explain_group_item_text(group, finding),
                    "  ",
                    84,
                    |value| render_backticked_text(value, Some(contract_path)),
                );
                if per_finding_next {
                    append_explain_next_text(&mut stdout, &finding.next, "    ", 84, contract_path);
                }
            }
        }
        if !per_finding_next {
            append_explain_next_text(
                &mut stdout,
                &explain_group_next(group),
                "  ",
                84,
                contract_path,
            );
        }
    }

    stdout
}

pub(super) fn explain_action_count(findings: &[Finding]) -> usize {
    group_doctor_findings(findings.iter()).len()
}

fn explain_group_uses_per_finding_nexts(group: &DoctorFindingGroup<'_>) -> bool {
    matches!(group.kind, DoctorFindingGroupKind::ToolingVersion)
        && group
            .findings
            .iter()
            .map(|finding| compact_backticked_paths(&finding.next))
            .collect::<BTreeSet<_>>()
            .len()
            > 1
}

fn explain_group_renders_items(group: &DoctorFindingGroup<'_>) -> bool {
    !(matches!(group.kind, DoctorFindingGroupKind::SharedAction(_)) && group.findings.len() == 1)
}

fn explain_group_title(group: &DoctorFindingGroup<'_>) -> String {
    match group.kind {
        DoctorFindingGroupKind::SharedAction(_) if group.findings.len() == 1 => {
            group.findings[0].summary.clone()
        }
        _ => doctor_finding_group_title(&group.kind, &group.findings),
    }
}

fn explain_group_why(group: &DoctorFindingGroup<'_>) -> String {
    match group.kind {
        DoctorFindingGroupKind::SharedAction(_) if group.findings.len() == 1 => {
            group.findings[0].why.clone()
        }
        _ => doctor_finding_group_why(&group.kind, &group.findings),
    }
}

fn explain_group_next(group: &DoctorFindingGroup<'_>) -> String {
    match group.kind {
        DoctorFindingGroupKind::SharedAction(_) if group.findings.len() == 1 => {
            compact_backticked_paths(&group.findings[0].next)
        }
        _ => doctor_finding_group_next(&group.kind, &group.findings, None),
    }
}

fn explain_group_item_text(group: &DoctorFindingGroup<'_>, finding: &Finding) -> String {
    match group.kind {
        DoctorFindingGroupKind::AdapterBootstrap => {
            doctor_finding_group_item_text(&group.kind, finding, false)
        }
        DoctorFindingGroupKind::ToolingVersion => {
            doctor_finding_group_item_text(&group.kind, finding, false)
        }
        DoctorFindingGroupKind::ContractDrift => {
            doctor_finding_group_item_text(&group.kind, finding, false)
        }
        DoctorFindingGroupKind::PolicySurface
        | DoctorFindingGroupKind::ServiceHealth
        | DoctorFindingGroupKind::CheckFailure
        | DoctorFindingGroupKind::EnvironmentValue
        | DoctorFindingGroupKind::ExecutionBackend
        | DoctorFindingGroupKind::SharedAction(_) => finding.summary.clone(),
    }
}

pub(super) fn explain_summary(report: &DoctorReport) -> ExplainSummary {
    explain_summary_from_findings(&report.findings)
}

fn explain_summary_from_findings(findings: &[Finding]) -> ExplainSummary {
    let mut summary = ExplainSummary {
        step_count: findings.len(),
        ..ExplainSummary::default()
    };

    for finding in findings {
        match finding.severity {
            FindingSeverity::Error => summary.error_count += 1,
            FindingSeverity::Warn => summary.warn_count += 1,
            FindingSeverity::Info => summary.info_count += 1,
        }
    }

    summary
}

pub(super) fn explain_steps(findings: &[Finding]) -> Vec<ExplainStep> {
    findings
        .iter()
        .enumerate()
        .map(|(index, finding)| ExplainStep {
            order: index + 1,
            code: finding.code(),
            severity: finding.severity,
            summary: finding.summary.clone(),
            why: finding.why.clone(),
            next: finding.next.clone(),
            provenance: finding.provenance(),
            provenance_key: finding.provenance_key(),
        })
        .collect()
}
