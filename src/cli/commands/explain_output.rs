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

use std::path::Path;

use crate::output::{ExplainAction, WorkspaceExplainAction};

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
            actions: explain_actions(&repo.findings),
            steps: explain_steps(&repo.findings),
        })
        .collect()
}

pub(super) fn workspace_explain_actions(
    report: &crate::workspace::WorkspaceDoctorReport,
) -> Vec<WorkspaceExplainAction> {
    let mut actions = report
        .repos
        .iter()
        .flat_map(|repo| {
            explain_actions(&repo.findings)
                .into_iter()
                .map(move |action| WorkspaceExplainAction {
                    repo: repo.name.clone(),
                    path: repo.path.clone(),
                    contract_path: repo.contract_path.clone(),
                    required: repo.required,
                    action,
                })
        })
        .collect::<Vec<_>>();

    for (index, action) in actions.iter_mut().enumerate() {
        action.action.order = index + 1;
    }

    actions
}

pub(super) fn render_explain_steps_text(findings: &[Finding], contract_path: &Path) -> String {
    let groups = explain_groups(findings);
    let (blockers, notes): (Vec<_>, Vec<_>) = groups
        .into_iter()
        .partition(|group| group.severity != FindingSeverity::Info);

    let mut stdout = String::from("\n\n");
    if blockers.is_empty() && notes.is_empty() {
        stdout.push_str(&paint_section_title("Plan"));
        if plain_mode() {
            stdout.push_str("\n* none");
        } else {
            stdout.push_str(&format!("\n{} none", paint("✦", "1;38;2;255;214;79")));
        }
        return stdout;
    }

    let mut next_index = 1usize;
    if !blockers.is_empty() {
        render_explain_group_section(
            &mut stdout,
            "Plan",
            &blockers,
            &mut next_index,
            contract_path,
        );
    }

    if !notes.is_empty() {
        if !stdout.ends_with("\n\n") {
            stdout.push_str("\n\n");
        }
        render_explain_context_section(&mut stdout, &notes, contract_path);
    }

    stdout
}

fn render_explain_group_section(
    output: &mut String,
    section_title: &str,
    groups: &[DoctorFindingGroup<'_>],
    next_index: &mut usize,
    contract_path: &Path,
) {
    output.push_str(&paint_section_title(section_title));
    for (offset, group) in groups.iter().enumerate() {
        if offset > 0 {
            output.push_str("\n\n");
        } else {
            output.push('\n');
        }
        render_explain_group(output, group, *next_index, contract_path);
        *next_index += 1;
    }
}

fn render_explain_context_section(
    output: &mut String,
    groups: &[DoctorFindingGroup<'_>],
    contract_path: &Path,
) {
    output.push_str(&paint_section_title("Context"));

    let shared_provenance = shared_group_provenance(groups);
    let shared_next = shared_group_next(groups);

    for (offset, group) in groups.iter().enumerate() {
        if offset > 0 {
            output.push_str("\n\n");
        } else {
            output.push('\n');
        }
        render_explain_context_group(
            output,
            group,
            contract_path,
            shared_provenance.is_none(),
            shared_next.is_none(),
        );
    }

    if let Some(provenance) = shared_provenance {
        append_wrapped_labeled_text(
            output,
            "Provenance:",
            &provenance,
            "",
            84,
            false,
            paint_key,
            |value| render_backticked_text(value, Some(contract_path)),
        );
    }
    if let Some(next) = shared_next {
        append_explain_next_text(output, &next, "", 84, contract_path);
    }
}

fn render_explain_group(
    output: &mut String,
    group: &DoctorFindingGroup<'_>,
    index: usize,
    contract_path: &Path,
) {
    let finding_count = group.findings.len();
    output.push_str(&format!(
        " {}. {} {}",
        index,
        render_finding_summary(group.severity, &explain_group_title(group)),
        paint_group_meta(&format!("({finding_count})"))
    ));

    if !concise_mode() {
        let why_lines = explain_group_why_lines(group);
        if !why_lines.is_empty() {
            output.push_str("\n  ");
            output.push_str(&explain_why_key());
            for line in why_lines {
                append_wrapped_bullet_text(output, summary_bullet(), &line, "    ", 84, |value| {
                    render_backticked_text(value, Some(contract_path))
                });
            }
        }
    }

    if let Some(provenance) = explain_group_provenance(group) {
        append_wrapped_labeled_text(
            output,
            "Provenance:",
            &provenance,
            "  ",
            84,
            false,
            paint_key,
            |value| render_backticked_text(value, Some(contract_path)),
        );
    }
    append_explain_next_text(
        output,
        &doctor_finding_group_next(&group.kind, &group.findings, None),
        "  ",
        84,
        contract_path,
    );
}

fn render_explain_context_group(
    output: &mut String,
    group: &DoctorFindingGroup<'_>,
    contract_path: &Path,
    show_provenance: bool,
    show_next: bool,
) {
    let finding_count = group.findings.len();
    output.push_str(&format!(
        " {} {} {}",
        list_bullet(),
        render_finding_summary(group.severity, &explain_group_title(group)),
        paint_group_meta(&format!("({finding_count})"))
    ));

    if !concise_mode() {
        let why_lines = explain_group_why_lines(group);
        if !why_lines.is_empty() {
            output.push_str("\n  ");
            output.push_str(&explain_why_key());
            for line in why_lines {
                append_wrapped_bullet_text(output, summary_bullet(), &line, "    ", 84, |value| {
                    render_backticked_text(value, Some(contract_path))
                });
            }
        }
    }

    if show_provenance {
        if let Some(provenance) = explain_group_provenance(group) {
            append_wrapped_labeled_text(
                output,
                "Provenance:",
                &provenance,
                "  ",
                84,
                false,
                paint_key,
                |value| render_backticked_text(value, Some(contract_path)),
            );
        }
    }

    if show_next {
        append_explain_next_text(
            output,
            &doctor_finding_group_next(&group.kind, &group.findings, None),
            "  ",
            84,
            contract_path,
        );
    }
}

fn explain_group_provenance(group: &DoctorFindingGroup<'_>) -> Option<String> {
    let mut entries = group.findings.iter().map(|finding| {
        finding
            .provenance()
            .zip(finding.provenance_key())
            .map(|(provenance, key)| (provenance, key))
    });
    let first = entries.next().flatten()?;
    if entries.all(|entry| entry == Some((first.0.clone(), first.1.clone()))) {
        Some(first.0)
    } else {
        None
    }
}

fn shared_group_provenance(groups: &[DoctorFindingGroup<'_>]) -> Option<String> {
    let mut entries = groups.iter().map(explain_group_provenance);
    let first = entries.next().flatten()?;
    if entries.all(|entry| entry == Some(first.clone())) {
        Some(first)
    } else {
        None
    }
}

fn shared_group_next(groups: &[DoctorFindingGroup<'_>]) -> Option<String> {
    let mut entries = groups.iter().map(|group| {
        compact_backticked_paths(&doctor_finding_group_next(
            &group.kind,
            &group.findings,
            None,
        ))
    });
    let first = entries.next()?;
    if first.trim().is_empty() {
        return None;
    }
    if entries.all(|entry| entry == first) {
        Some(first)
    } else {
        None
    }
}

pub(super) fn explain_action_count(findings: &[Finding]) -> usize {
    explain_groups(findings)
        .into_iter()
        .filter(|group| group.severity != FindingSeverity::Info)
        .count()
}

fn explain_group_title(group: &DoctorFindingGroup<'_>) -> String {
    match group.kind {
        DoctorFindingGroupKind::SharedAction(_) if group.findings.len() == 1 => {
            group.findings[0].summary.clone()
        }
        _ => doctor_finding_group_title(&group.kind, &group.findings),
    }
}

fn explain_group_why_lines(group: &DoctorFindingGroup<'_>) -> Vec<String> {
    group
        .findings
        .iter()
        .flat_map(|finding| {
            finding
                .why
                .split(';')
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn explain_groups(findings: &[Finding]) -> Vec<DoctorFindingGroup<'_>> {
    let mut groups = group_doctor_findings(findings.iter());
    groups.sort_by(|left, right| {
        explain_group_sort_key(left)
            .cmp(&explain_group_sort_key(right))
            .then_with(|| left.action_key.cmp(&right.action_key))
            .then_with(|| explain_group_title(left).cmp(&explain_group_title(right)))
    });
    groups
}

fn explain_group_sort_key(group: &DoctorFindingGroup<'_>) -> (usize, usize) {
    (
        explain_severity_priority(group.severity),
        explain_action_priority(group),
    )
}

fn explain_severity_priority(severity: FindingSeverity) -> usize {
    match severity {
        FindingSeverity::Error => 0,
        FindingSeverity::Warn => 1,
        FindingSeverity::Info => 2,
    }
}

fn explain_action_priority(group: &DoctorFindingGroup<'_>) -> usize {
    let next = doctor_finding_group_next(&group.kind, &group.findings, None);
    if next.contains("`ota detect --dry-run") || next.contains("`ota init --dry-run") {
        0
    } else if next.contains("`ota assist ") {
        1
    } else if next.contains("`ota env") {
        2
    } else {
        match group.kind {
            DoctorFindingGroupKind::ToolingVersion
            | DoctorFindingGroupKind::ExecutionBackend
            | DoctorFindingGroupKind::AdapterBootstrap => 3,
            DoctorFindingGroupKind::ServiceHealth => 4,
            DoctorFindingGroupKind::CheckFailure => 5,
            DoctorFindingGroupKind::ContractDrift => 6,
            DoctorFindingGroupKind::PolicySurface => 7,
            DoctorFindingGroupKind::EnvironmentValue => 2,
            DoctorFindingGroupKind::SharedAction(_) => 8,
        }
    }
}

pub(super) fn explain_summary(report: &DoctorReport) -> ExplainSummary {
    explain_summary_from_findings(&report.findings)
}

fn explain_summary_from_findings(findings: &[Finding]) -> ExplainSummary {
    let groups = explain_groups(findings);
    let mut summary = ExplainSummary {
        step_count: groups.len(),
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

pub(super) fn explain_actions(findings: &[Finding]) -> Vec<ExplainAction> {
    explain_groups(findings)
        .into_iter()
        .enumerate()
        .map(|(index, group)| ExplainAction {
            order: index + 1,
            action_key: group.action_key.clone(),
            action_title: explain_group_title(&group),
            severity: group.severity,
            count: group.findings.len(),
            why: explain_group_why_lines(&group).join("; "),
            next: doctor_finding_group_next(&group.kind, &group.findings, None),
            provenance: explain_group_provenance(&group),
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_steps_split_plan_and_notes_and_break_why_into_bullets() {
        set_plain_mode(true);

        let findings = vec![
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Service readiness failed: postgres"),
                why: String::from(
                    "service `postgres` did not pass its configured readiness probe from context `app`; projected endpoint is `postgres:5432`",
                ),
                next: String::from("run `ota up` and rerun `ota explain`"),
            },
            Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Policy-backed version rules are declared"),
                why: String::from(
                    "`.ota/org-policy.yaml` declares approved repo version rules: runtime java (versions >=21), tool node (versions 24.14.1)",
                ),
                next: String::from("run `ota policy review`"),
            },
        ];

        let text = render_explain_steps_text(&findings, Path::new("./ota.yaml"));
        set_plain_mode(false);

        assert!(text.contains("\nPlan\n"));
        assert!(text.contains("\nContext\n"));
        assert!(text.contains("Why:\n    - service `postgres` did not pass its configured readiness probe from context `app`"));
        assert!(text.contains("    - projected endpoint is `postgres:5432`"));
        assert!(text.contains("Policy-backed version rules are declared"));
    }

    #[test]
    fn explain_context_section_shares_policy_footer_once() {
        set_plain_mode(true);

        let first = Finding {
            severity: FindingSeverity::Info,
            summary: String::from("Policy-backed provisioning sources are declared"),
            why: String::from(
                "`.ota/org-policy.yaml` declares approved provisioning sources: curl via apt",
            ),
            next: String::from(
                "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when provisioning or bootstrap needs a governed path",
            ),
        };
        let second = Finding {
            severity: FindingSeverity::Info,
            summary: String::from("Policy-backed provisioning sources are declared"),
            why: String::from(
                "`.ota/org-policy.yaml` can bootstrap missing adapter binaries through: brew via brew-bootstrap",
            ),
            next: String::from(
                "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when provisioning or bootstrap needs a governed path",
            ),
        };

        let groups = vec![
            DoctorFindingGroup {
                group_key: String::from("policy-surface-provisioning"),
                action_key: String::from("policy-surface-provisioning"),
                kind: DoctorFindingGroupKind::PolicySurface,
                severity: FindingSeverity::Info,
                findings: vec![&first],
            },
            DoctorFindingGroup {
                group_key: String::from("policy-surface-bootstrap"),
                action_key: String::from("policy-surface-bootstrap"),
                kind: DoctorFindingGroupKind::PolicySurface,
                severity: FindingSeverity::Info,
                findings: vec![&second],
            },
        ];

        let mut stdout = String::new();
        render_explain_context_section(&mut stdout, &groups, Path::new("./ota.yaml"));
        let text = strip_ansi_codes(&stdout);

        assert!(text.contains("Context"));
        assert!(text.contains("Review active policy surfaces"));
        assert!(text.contains("approved provisioning sources"));
        assert!(text.contains("can bootstrap missing adapter binaries"));
        assert_eq!(text.matches("Provenance: org policy").count(), 1);
        assert_eq!(text.matches("ota policy review").count(), 1);
    }

    #[test]
    fn explain_actions_prioritize_preview_and_assist_before_runtime_followups() {
        let findings = vec![
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Check failed: api-health"),
                why: String::from("the health check failed"),
                next: String::from("run `ota run smoke` and rerun `ota doctor`"),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("No tasks defined in contract"),
                why: String::from("the contract cannot run anything yet"),
                next: String::from(
                    "run `ota detect --dry-run` to review inferred tasks before writing, or run `ota assist add-task --name dev --kind command` when you want one explicit runnable task",
                ),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("The required service `postgres` is not verifiable"),
                why: String::from("the service has no readiness probe"),
                next: String::from(
                    "declare readiness with `ota assist declare-readiness --service postgres --style tcp` or `--style http`, then rerun `ota doctor`",
                ),
            },
            Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Required environment variable `DATABASE_URL` is missing"),
                why: String::from("the current precedence did not resolve a value"),
                next: String::from(
                    "run `ota env` to inspect the current precedence, set the listed environment variables, then rerun `ota doctor`",
                ),
            },
        ];

        let actions = explain_actions(&findings);

        assert_eq!(actions.len(), 4);
        let detect_index = actions
            .iter()
            .position(|action| action.next.contains("`ota detect --dry-run`"))
            .unwrap();
        let assist_index = actions
            .iter()
            .position(|action| action.next.contains("`ota assist declare-readiness"))
            .unwrap();
        let env_index = actions
            .iter()
            .position(|action| action.next.contains("`ota env`"))
            .unwrap();
        let runtime_index = actions
            .iter()
            .position(|action| action.next.contains("`ota run smoke`"))
            .unwrap();

        assert!(detect_index < assist_index);
        assert!(assist_index < env_index);
        assert!(env_index < runtime_index);
    }

    #[test]
    fn explain_group_renders_staged_commands_when_multiple_ota_commands_exist() {
        set_plain_mode(true);

        let finding = Finding {
            severity: FindingSeverity::Error,
            summary: String::from("No tasks defined in contract"),
            why: String::from("the contract cannot run anything yet"),
            next: String::from(
                "run `ota detect --dry-run` to review inferred tasks before writing, or run `ota assist add-task --name dev --kind command` when you want one explicit runnable task",
            ),
        };
        let group = DoctorFindingGroup {
            group_key: String::from("tasks-missing"),
            action_key: String::from("tasks-missing"),
            kind: DoctorFindingGroupKind::SharedAction(compact_backticked_paths(&finding.next)),
            severity: FindingSeverity::Error,
            findings: vec![&finding],
        };

        let mut output = String::new();
        render_explain_group(&mut output, &group, 1, Path::new("./ota.yaml"));
        let text = strip_ansi_codes(&output);
        set_plain_mode(false);

        assert!(!text.contains("Commands:"));
    }

    #[test]
    fn workspace_explain_actions_use_one_global_order() {
        let report = crate::workspace::WorkspaceDoctorReport {
            ok: false,
            repos: vec![
                crate::workspace::WorkspaceRepoDoctorReport {
                    name: String::from("api"),
                    path: String::from("./api"),
                    contract_path: String::from("./api/ota.yaml"),
                    required: true,
                    ok: false,
                    agent_verdict: DoctorVerdict::NotReady,
                    primary_blocker: None,
                    execution: None,
                    provisioning: None,
                    adapter_bootstrap: None,
                    extensions: BTreeMap::new(),
                    findings: vec![Finding {
                        severity: FindingSeverity::Error,
                        summary: String::from("No tasks defined in contract"),
                        why: String::from("the contract cannot run anything yet"),
                        next: String::from(
                            "run `ota detect --dry-run` to review inferred tasks before writing",
                        ),
                    }],
                },
                crate::workspace::WorkspaceRepoDoctorReport {
                    name: String::from("web"),
                    path: String::from("./web"),
                    contract_path: String::from("./web/ota.yaml"),
                    required: true,
                    ok: false,
                    agent_verdict: DoctorVerdict::NotReady,
                    primary_blocker: None,
                    execution: None,
                    provisioning: None,
                    adapter_bootstrap: None,
                    extensions: BTreeMap::new(),
                    findings: vec![Finding {
                        severity: FindingSeverity::Error,
                        summary: String::from("Required environment variable `DATABASE_URL` is missing"),
                        why: String::from("the current precedence did not resolve a value"),
                        next: String::from(
                            "run `ota env` to inspect the current precedence, then rerun `ota doctor`",
                        ),
                    }],
                },
            ],
        };

        let actions = workspace_explain_actions(&report);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].repo, "api");
        assert_eq!(actions[0].action.order, 1);
        assert_eq!(actions[1].repo, "web");
        assert_eq!(actions[1].action.order, 2);
    }
}
