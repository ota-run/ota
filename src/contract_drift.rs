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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_yaml::{Mapping, Value as YamlValue};

use crate::detector::{
    CiVerificationTaskSignal, Confidence, DetectService, Inference,
    collect_github_actions_verification_tasks, detect_repo, infer_ci_verification_task_line,
};
use crate::doctor::{Finding, FindingSeverity};
use crate::output::{DetectComparisonChange, DetectComparisonRemoval};
use crate::schema::{
    AgentBootstrapOtaSource, Backend, Contract, ServiceSpec, ToolchainFulfillmentMode,
    ToolchainProvider,
};

const DETECT_COMPARISON_REPO_CONTRACT_OWNERSHIP: &str = "repo_contract";
const DETECT_COMPARISON_REPO_SIGNALS_OWNERSHIP: &str = "repo_signals";
const DETECT_COMPARISON_REPO_SIGNALS_PROVENANCE: &str = "repo_signals";
const DETECT_COMPARISON_REPO_SIGNALS_PROVENANCE_KEY: &str = "repo_signals";
pub(crate) const DETECT_OWNER_KIND_DETECTED: &str = "detected";
pub(crate) const DETECT_OWNER_KIND_MANUAL: &str = "manual";
pub(crate) const DETECT_OWNER_KIND_MERGED: &str = "merged";
pub(crate) const DETECT_OWNER_KIND_POLICY: &str = "policy";

pub(crate) fn append_contract_drift_findings(
    contract: &Contract,
    contract_path: &Path,
    findings: &mut Vec<Finding>,
) {
    let root = contract_path.parent().unwrap_or(contract_path);
    let Ok(detect_report) = detect_repo(root) else {
        return;
    };

    let detect_changes =
        collect_detect_changes(contract, &detect_report.contract, &detect_report.inferences);

    for change in detect_changes.iter().filter(|change| {
        change.status == "update" && change.owner_kind.as_deref() == Some(DETECT_OWNER_KIND_MERGED)
    }) {
        let existing = change.existing.as_deref().unwrap_or_default();
        findings.push(Finding::identified(
            "OTA_CONTRACT_DRIFT",
            "contract",
            "repo_contract",
            FindingSeverity::Warn,
            format!("Contract drift: `{}` differs from repo signals", change.field),
            format!(
                "`ota.yaml` still declares `{}` = `{}`, but repo inspection under `{}` now detects `{}`",
                change.field,
                existing,
                compact_display_path(root),
                change.detected
            ),
            format!(
                "run `ota detect --merge --dry-run {}` to review the comparison, then `ota detect --merge {}` to apply matching updates",
                compact_display_path(root),
                compact_display_path(root)
            ),
        ));
    }

    for change in detect_changes.iter().filter(|change| {
        should_surface_external_source_governance_drift(change)
            && change.source_class.as_deref() != Some("ci_verification")
    }) {
        let existing = change.existing.as_deref().unwrap_or_default();
        let source = change.source.as_deref().unwrap_or_default();
        let (finding_id, summary, why, next) = if change.source_class.as_deref()
            == Some("ci_verification")
        {
            (
                "OTA_CI_VERIFICATION_DRIFT",
                format!(
                    "CI verification drift: `{}` differs from enforced workflow lane",
                    change.field
                ),
                format!(
                    "`ota.yaml` still declares `{}` = `{}`, but workflow verification in `{}` runs `{}`",
                    change.field, existing, source, change.detected
                ),
                format!(
                    "review whether `{}` reflects the canonical verification lane or whether the workflow is carrying repo-specific drift, then run `ota detect --dry-run {}` or `ota detect --merge --dry-run {}` before changing either side",
                    source,
                    compact_display_path(root),
                    compact_display_path(root)
                ),
            )
        } else {
            (
                "OTA_EXTERNAL_SOURCE_DRIFT",
                format!(
                    "External source drift: `{}` differs from high-confidence repo source",
                    change.field
                ),
                format!(
                    "`ota.yaml` still declares `{}` = `{}`, but `{}` now detects `{}`",
                    change.field, existing, source, change.detected
                ),
                format!(
                    "review whether `{}` or the repo contract is canonical, then run `ota detect --dry-run {}` or `ota detect --merge --dry-run {}` before changing either side",
                    source,
                    compact_display_path(root),
                    compact_display_path(root)
                ),
            )
        };
        findings.push(Finding::identified(
            finding_id,
            "contract",
            "repo_contract",
            FindingSeverity::Warn,
            summary,
            why,
            next,
        ));
    }

    if let Ok(ci_signals) = collect_github_actions_verification_tasks(root) {
        for change in collect_ci_verification_governance_changes(contract, &ci_signals) {
            let (code, summary, why) = match change.kind {
                CiVerificationGovernanceChangeKind::Update => (
                    "OTA_CI_VERIFICATION_DRIFT",
                    format!(
                        "CI verification drift: `{}` differs from enforced workflow lane",
                        change.field
                    ),
                    format!(
                        "`ota.yaml` still declares `{}` = `{}`, but workflow verification in `{}` runs `{}`",
                        change.field, change.existing, change.source, change.detected
                    ),
                ),
                CiVerificationGovernanceChangeKind::Removed => (
                    "OTA_CI_VERIFICATION_REMOVED",
                    format!(
                        "CI verification drift: `{}` is no longer detected from enforced workflow verification",
                        change.field
                    ),
                    format!(
                        "`ota.yaml` still declares `{}` = `{}`, but workflow verification under `.github/workflows/` no longer detects that verifier lane; current detected verifier tasks: {}",
                        change.field, change.existing, change.detected
                    ),
                ),
            };
            findings.push(Finding::identified(
                code,
                "contract",
                "repo_contract",
                FindingSeverity::Warn,
                summary,
                why,
                format!(
                    "review whether workflow verification or `{}` is canonical, then run `ota detect --dry-run {}` or `ota detect --merge --dry-run {}` before changing either side",
                    change.field,
                    compact_display_path(root),
                    compact_display_path(root)
                ),
            ));
        }

        for change in collect_ci_verification_aggregate_changes(contract, &ci_signals) {
            let (code, summary, why) = match change.kind {
                CiVerificationAggregateChangeKind::Update => (
                    "OTA_CI_VERIFICATION_DRIFT",
                    format!(
                        "CI verification drift: `{}` differs from enforced workflow verification set",
                        change.field
                    ),
                    format!(
                        "`ota.yaml` still declares `{}` = `{}`, but workflow verification under `{}` currently detects verifier tasks `{}`",
                        change.field, change.existing, change.source, change.detected
                    ),
                ),
            };
            findings.push(Finding::identified(
                code,
                "contract",
                "repo_contract",
                FindingSeverity::Warn,
                summary,
                why,
                format!(
                    "review whether workflow verification or `{}` is canonical, then run `ota detect --dry-run {}` or `ota detect --merge --dry-run {}` before changing either side",
                    change.field,
                    compact_display_path(root),
                    compact_display_path(root)
                ),
            ));
        }
    }

    if let Some(contract_source) = contract
        .agent
        .as_ref()
        .and_then(|agent| agent.bootstrap.as_ref())
        .and_then(|bootstrap| bootstrap.ota.as_ref())
        .and_then(|ota| ota.effective_source())
        && let Ok(signals) = collect_github_actions_ota_bootstrap_signals(root)
    {
        for signal in signals {
            match signal.mode {
                CiOtaBootstrapSignalMode::ContractConsumer => {}
                CiOtaBootstrapSignalMode::WorkflowOwned {
                    install_source,
                    surface,
                } => {
                    let contract_display = describe_ota_bootstrap_source(&contract_source);
                    if let Some(workflow_source) = install_source.as_ref() {
                        if *workflow_source != contract_source {
                            findings.push(Finding::identified(
                                "OTA_CI_BOOTSTRAP_TRUTH_DRIFT",
                                "contract",
                                "repo_contract",
                                FindingSeverity::Warn,
                                String::from(
                                    "CI bootstrap drift: workflow install truth conflicts with `agent.bootstrap.ota.source`",
                                ),
                                format!(
                                    "`ota.yaml` declares `agent.bootstrap.ota.source` = `{contract_display}`, but workflow install in `{}` uses `{}` through `{surface}`",
                                    signal.source,
                                    describe_ota_bootstrap_source(workflow_source),
                                ),
                                format!(
                                    "prefer `ota-run/setup@v1 source: contract` or `ota-run/action@v1 source: contract` so CI consumes repo-owned bootstrap truth from `{}` instead of restating it in workflow YAML",
                                    compact_display_path(root),
                                ),
                            ));
                            continue;
                        }
                    }

                    findings.push(Finding::identified(
                        "OTA_CI_BOOTSTRAP_TRUTH_DUPLICATED",
                        "contract",
                        "repo_contract",
                        FindingSeverity::Warn,
                        String::from(
                            "CI bootstrap drift: workflow restates ota install truth outside `agent.bootstrap.ota.source`",
                        ),
                        format!(
                            "`ota.yaml` already declares `agent.bootstrap.ota.source` = `{contract_display}`, but workflow install in `{}` still owns ota bootstrap through `{surface}`",
                            signal.source,
                        ),
                        format!(
                            "prefer `ota-run/setup@v1 source: contract` or `ota-run/action@v1 source: contract` so CI consumes repo-owned bootstrap truth from `{}` instead of duplicating version, git revision, branch, or source-install refs in workflow YAML",
                            compact_display_path(root),
                        ),
                    ));
                }
            }
        }
    }

    for removal in collect_detect_drift_removals(contract, &detect_report.contract) {
        findings.push(Finding::identified(
            "OTA_CONTRACT_DRIFT",
            "contract",
            "repo_contract",
            FindingSeverity::Warn,
            format!("Contract drift: `{}` is no longer detected", removal.field),
            format!(
                "`ota.yaml` still declares `{}` = `{}`, but repo inspection under `{}` no longer detects it",
                removal.field,
                removal.existing,
                compact_display_path(root)
            ),
            format!(
                "run `ota detect --merge --dry-run {}` to review the comparison, then `ota detect --merge {}` to apply matching updates",
                compact_display_path(root),
                compact_display_path(root)
            ),
        ));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CiOtaBootstrapSignalMode {
    ContractConsumer,
    WorkflowOwned {
        install_source: Option<AgentBootstrapOtaSource>,
        surface: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CiOtaBootstrapSignal {
    source: String,
    mode: CiOtaBootstrapSignalMode,
}

fn collect_github_actions_ota_bootstrap_signals(
    root: &Path,
) -> Result<Vec<CiOtaBootstrapSignal>, crate::detector::DetectError> {
    let workflows_dir = root.join(".github").join("workflows");
    if !workflows_dir.exists() {
        return Ok(Vec::new());
    }

    let mut workflow_files = fs::read_dir(&workflows_dir)
        .map_err(|source| crate::detector::DetectError::Read {
            path: workflows_dir.display().to_string(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect::<Vec<_>>();
    workflow_files.sort();

    let mut signals = Vec::new();
    let mut active_workflows = BTreeSet::new();
    for workflow_path in workflow_files {
        collect_github_actions_ota_bootstrap_signals_from_workflow(
            root,
            &workflow_path,
            None,
            &mut active_workflows,
            &mut signals,
        )?;
    }

    Ok(signals)
}

fn collect_github_actions_ota_bootstrap_signals_from_workflow(
    root: &Path,
    workflow_path: &Path,
    caller_job_source: Option<&str>,
    active_workflows: &mut BTreeSet<String>,
    signals: &mut Vec<CiOtaBootstrapSignal>,
) -> Result<(), crate::detector::DetectError> {
    let workflow_key = workflow_path
        .strip_prefix(root)
        .unwrap_or(workflow_path)
        .display()
        .to_string();
    if !active_workflows.insert(workflow_key.clone()) {
        return Ok(());
    }

    let contents =
        fs::read_to_string(workflow_path).map_err(|source| crate::detector::DetectError::Read {
            path: workflow_path.display().to_string(),
            source,
        })?;
    let workflow: YamlValue = serde_yaml::from_str(&contents).map_err(|source_error| {
        crate::detector::DetectError::Parse {
            path: workflow_path.display().to_string(),
            message: source_error.to_string(),
        }
    })?;

    let Some(jobs) = workflow.get("jobs").and_then(YamlValue::as_mapping) else {
        active_workflows.remove(&workflow_key);
        return Ok(());
    };

    for (job_name, job_value) in jobs
        .iter()
        .filter_map(|(name, value)| Some((name.as_str()?, value)))
    {
        let direct_job_source = format!("{workflow_key}#jobs.{job_name}");
        let job_source = caller_job_source.unwrap_or(&direct_job_source).to_string();

        if let Some(reusable_path) = job_value
            .get("uses")
            .and_then(YamlValue::as_str)
            .and_then(local_reusable_github_workflow_path)
        {
            collect_github_actions_ota_bootstrap_signals_from_workflow(
                root,
                &root.join(reusable_path),
                Some(&job_source),
                active_workflows,
                signals,
            )?;
            continue;
        }

        let Some(steps) = job_value.get("steps").and_then(YamlValue::as_sequence) else {
            continue;
        };

        for (step_index, step) in steps.iter().enumerate() {
            let step_source_prefix = if caller_job_source.is_some() {
                format!("{job_source}::{workflow_key}#jobs.{job_name}.steps[{step_index}]")
            } else {
                format!("{job_source}.steps[{step_index}]")
            };

            if let Some(signal) =
                github_actions_ota_bootstrap_signal_for_step(step, &step_source_prefix)
            {
                signals.push(signal);
            }
        }
    }

    active_workflows.remove(&workflow_key);
    Ok(())
}

fn github_actions_ota_bootstrap_signal_for_step(
    step: &YamlValue,
    source_prefix: &str,
) -> Option<CiOtaBootstrapSignal> {
    if let Some(uses) = step.get("uses").and_then(YamlValue::as_str) {
        let uses_lower = uses.to_ascii_lowercase();
        let with = step.get("with").and_then(YamlValue::as_mapping);

        if uses_lower.starts_with("ota-run/setup@") || uses_lower.starts_with("ota-run/action@") {
            let mode = with
                .and_then(|mapping| yaml_mapping_string(mapping, "source"))
                .unwrap_or("explicit");
            if mode.eq_ignore_ascii_case("contract") {
                return Some(CiOtaBootstrapSignal {
                    source: format!("{source_prefix}.uses"),
                    mode: CiOtaBootstrapSignalMode::ContractConsumer,
                });
            }

            let install_source = with
                .and_then(|mapping| yaml_mapping_string(mapping, "ota-version"))
                .map(|version| AgentBootstrapOtaSource::Version {
                    version: version.to_string(),
                });
            return Some(CiOtaBootstrapSignal {
                source: format!("{source_prefix}.uses"),
                mode: CiOtaBootstrapSignalMode::WorkflowOwned {
                    install_source,
                    surface: format!("{uses} ({mode})"),
                },
            });
        }

        if uses_lower.starts_with("ota-run/ota/.github/actions/install-ota-from-source@") {
            let install_source = with
                .and_then(|mapping| yaml_mapping_string(mapping, "ref"))
                .and_then(parse_workflow_owned_ota_ref);
            return Some(CiOtaBootstrapSignal {
                source: format!("{source_prefix}.uses"),
                mode: CiOtaBootstrapSignalMode::WorkflowOwned {
                    install_source,
                    surface: String::from("ota-run/ota/.github/actions/install-ota-from-source"),
                },
            });
        }
    }

    if let Some(run) = step.get("run").and_then(YamlValue::as_str)
        && workflow_run_mentions_ota_installer(run)
    {
        let install_source = parse_ota_installer_source_from_workflow_run(run);
        return Some(CiOtaBootstrapSignal {
            source: format!("{source_prefix}.run"),
            mode: CiOtaBootstrapSignalMode::WorkflowOwned {
                install_source,
                surface: String::from("official ota installer command"),
            },
        });
    }

    None
}

fn workflow_run_mentions_ota_installer(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    normalized.contains("dist.ota.run/install.sh")
        || normalized.contains("dist.ota.run/install.ps1")
}

fn parse_ota_installer_source_from_workflow_run(command: &str) -> Option<AgentBootstrapOtaSource> {
    let normalized = command.trim();
    if normalized.is_empty() {
        return None;
    }

    extract_bootstrap_marker_value(normalized, &["OTA_GIT_BRANCH=", "$env:OTA_GIT_BRANCH="])
        .map(|branch| AgentBootstrapOtaSource::Branch { branch })
        .or_else(|| {
            extract_bootstrap_marker_value(normalized, &["OTA_GIT_REV=", "$env:OTA_GIT_REV="])
                .map(|rev| AgentBootstrapOtaSource::GitRev { rev })
        })
        .or_else(|| {
            extract_bootstrap_marker_value(normalized, &["OTA_VERSION=", "$env:OTA_VERSION="])
                .map(|version| AgentBootstrapOtaSource::Version { version })
        })
}

fn extract_bootstrap_marker_value(command: &str, markers: &[&str]) -> Option<String> {
    markers.iter().find_map(|marker| {
        let index = command.find(marker)?;
        let remainder = &command[index + marker.len()..];
        let trimmed = remainder.trim_start();
        let quoted = trimmed
            .strip_prefix('\'')
            .and_then(|value| value.split('\'').next())
            .or_else(|| {
                trimmed
                    .strip_prefix('"')
                    .and_then(|value| value.split('"').next())
            });
        if let Some(value) = quoted {
            let normalized = value.trim();
            return (!normalized.is_empty()).then(|| normalized.to_string());
        }

        let value = trimmed
            .split(|character: char| {
                character.is_whitespace()
                    || character == ';'
                    || character == '|'
                    || character == '&'
                    || character == ')'
            })
            .next()
            .unwrap_or_default()
            .trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn parse_workflow_owned_ota_ref(value: &str) -> Option<AgentBootstrapOtaSource> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains("${{") {
        return None;
    }
    if trimmed.len() == 40
        && trimmed
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Some(AgentBootstrapOtaSource::GitRev {
            rev: trimmed.to_string(),
        });
    }
    if trimmed.starts_with('v')
        && trimmed
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_digit() || character == '.')
    {
        return Some(AgentBootstrapOtaSource::Version {
            version: trimmed.to_string(),
        });
    }
    Some(AgentBootstrapOtaSource::Branch {
        branch: trimmed.to_string(),
    })
}

fn yaml_mapping_string<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a str> {
    mapping
        .iter()
        .find_map(|(mapping_key, value)| (mapping_key.as_str() == Some(key)).then_some(value))
        .and_then(YamlValue::as_str)
}

fn local_reusable_github_workflow_path(uses: &str) -> Option<&str> {
    uses.strip_prefix("./")
        .filter(|path| path.starts_with(".github/workflows/"))
}

fn describe_ota_bootstrap_source(source: &AgentBootstrapOtaSource) -> String {
    match source {
        AgentBootstrapOtaSource::Version { version } => format!("version {version}"),
        AgentBootstrapOtaSource::GitRev { rev } => format!("git_rev {rev}"),
        AgentBootstrapOtaSource::Branch { branch } => format!("branch {branch}"),
    }
}

fn compact_display_path(path: &Path) -> String {
    let Ok(current_dir) = std::env::current_dir() else {
        return path.display().to_string();
    };

    path.strip_prefix(&current_dir)
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                String::from(".")
            } else {
                relative.display().to_string()
            }
        })
        .unwrap_or_else(|_| path.display().to_string())
}

fn should_surface_external_source_governance_drift(change: &DetectComparisonChange) -> bool {
    change.status == "update"
        && change.owner_kind.as_deref() == Some(DETECT_OWNER_KIND_MANUAL)
        && change.source.as_deref().is_some_and(|source| {
            matches_governed_external_source(
                change.source_class.as_deref(),
                source,
                &change.field,
                change.confidence,
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CiVerificationGovernanceChangeKind {
    Update,
    Removed,
}

struct CiVerificationGovernanceChange {
    field: String,
    existing: String,
    detected: String,
    source: String,
    kind: CiVerificationGovernanceChangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CiVerificationAggregateChangeKind {
    Update,
}

struct CiVerificationAggregateChange {
    field: String,
    existing: String,
    detected: String,
    source: String,
    kind: CiVerificationAggregateChangeKind,
}

fn collect_ci_verification_governance_changes(
    existing: &Contract,
    signals: &[CiVerificationTaskSignal],
) -> Vec<CiVerificationGovernanceChange> {
    let recovered_signals = recover_ci_verification_task_signals(existing, signals);
    let ci_tasks = recovered_signals
        .iter()
        .filter(|signal| {
            signal.exact_command
                && is_task_command_truth_field(&signal.field)
                && is_verification_task_truth_field(&signal.field)
        })
        .map(|signal| {
            (
                signal.field.clone(),
                (signal.command.clone(), signal.source.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();

    if ci_tasks.is_empty() {
        return Vec::new();
    }

    let detected_rendered = ci_tasks
        .iter()
        .map(|(field, _)| field.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let detected_fields = recovered_signals
        .iter()
        .map(|signal| signal.field.as_str())
        .collect::<Vec<_>>();
    let detected_verifier_tasks = detected_fields
        .iter()
        .filter_map(|field| verification_task_name_from_field(field))
        .collect::<Vec<_>>();

    let mut changes = Vec::new();
    for (task_name, task) in &existing.tasks {
        let field = format!("tasks.{task_name}.run");
        let Some(existing_command) = existing_task_detectable_command_truth(task) else {
            continue;
        };
        let owner_kind = detect_existing_field_owner_kind(existing, &field);
        if owner_kind != DETECT_OWNER_KIND_MANUAL || !is_verification_task_truth_field(&field) {
            continue;
        }

        if let Some((detected_command, source)) = ci_tasks.get(&field) {
            if !equivalent_ci_verification_commands(&existing_command, detected_command) {
                changes.push(CiVerificationGovernanceChange {
                    field,
                    existing: existing_command,
                    detected: detected_command.clone(),
                    source: source.clone(),
                    kind: CiVerificationGovernanceChangeKind::Update,
                });
            }
            continue;
        }

        if detected_fields
            .iter()
            .any(|detected_field| *detected_field == field)
        {
            continue;
        }

        if infer_ci_verification_task_line(existing_command.as_str())
            .is_some_and(|(inferred, _)| inferred == *task_name)
        {
            if should_treat_ci_verifier_family_root_as_covered(task_name, &detected_verifier_tasks)
            {
                continue;
            }
            changes.push(CiVerificationGovernanceChange {
                field,
                existing: existing_command,
                detected: detected_rendered.clone(),
                source: String::from(".github/workflows/"),
                kind: CiVerificationGovernanceChangeKind::Removed,
            });
        }
    }

    changes
}

fn collect_ci_verification_aggregate_changes(
    existing: &Contract,
    signals: &[CiVerificationTaskSignal],
) -> Vec<CiVerificationAggregateChange> {
    let workflow_candidates = collect_ci_verification_workflow_task_sequences(existing, signals);
    if workflow_candidates.is_empty() {
        return Vec::new();
    }
    let mut changes = Vec::new();
    for (task_name, task) in &existing.tasks {
        let Some(aggregate) = task.aggregate.as_ref() else {
            continue;
        };
        if !is_verifier_task_name(task_name) {
            continue;
        }

        let field = format!("tasks.{task_name}.aggregate.tasks");
        let owner_kind = detect_existing_field_owner_kind(existing, &field);
        if owner_kind != DETECT_OWNER_KIND_MANUAL {
            continue;
        }

        let existing_tasks = aggregate
            .tasks
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        if workflow_candidates.iter().any(|(_, detected_tasks)| {
            detected_tasks
                .iter()
                .map(String::as_str)
                .eq(existing_tasks.iter().copied())
        }) {
            continue;
        }
        if workflow_file_union_matches_exact_verifier_aggregate(
            &existing_tasks,
            &workflow_candidates,
        ) {
            continue;
        }

        let Some((source, detected_tasks)) =
            best_matching_ci_verification_workflow_candidate(&existing_tasks, &workflow_candidates)
        else {
            continue;
        };

        changes.push(CiVerificationAggregateChange {
            field,
            existing: aggregate.tasks.join(", "),
            detected: render_string_list(detected_tasks),
            source: source.clone(),
            kind: CiVerificationAggregateChangeKind::Update,
        });
    }

    changes
}

fn workflow_file_union_matches_exact_verifier_aggregate(
    existing_tasks: &[&str],
    workflow_candidates: &[(String, Vec<String>)],
) -> bool {
    let expected = existing_tasks.iter().copied().collect::<BTreeSet<_>>();
    let workflow_file_candidates = workflow_candidates
        .iter()
        .filter(|(source, _)| !source.contains('#'))
        .collect::<Vec<_>>();
    if workflow_file_candidates.len() < 2 {
        return false;
    }

    let mut union = BTreeSet::<&str>::new();
    let mut contributing_files = 0usize;
    for (_, detected_tasks) in workflow_file_candidates {
        if detected_tasks
            .iter()
            .all(|task| expected.contains(task.as_str()))
        {
            let before = union.len();
            union.extend(detected_tasks.iter().map(String::as_str));
            if union.len() > before {
                contributing_files += 1;
            }
        }
    }

    contributing_files >= 2 && union == expected
}

fn collect_ci_verification_workflow_task_sequences(
    existing: &Contract,
    signals: &[CiVerificationTaskSignal],
) -> Vec<(String, Vec<String>)> {
    let recovered_signals = recover_ci_verification_task_signals(existing, signals);
    let mut signals_by_workflow = BTreeMap::<String, Vec<CiVerificationTaskSignal>>::new();
    for signal in recovered_signals {
        for scope in [
            ci_verification_signal_workflow_lane_source(&signal.source),
            ci_verification_signal_workflow_source(&signal.source),
        ] {
            signals_by_workflow
                .entry(scope)
                .or_default()
                .push(signal.clone());
        }
    }

    signals_by_workflow
        .into_iter()
        .filter_map(|(source, workflow_signals)| {
            let detected_tasks = recover_ci_verification_aggregate_task_sequence_from_signals(
                existing,
                &workflow_signals,
            );
            (!detected_tasks.is_empty()).then_some((source, detected_tasks))
        })
        .collect()
}

fn recover_ci_verification_aggregate_task_sequence_from_signals(
    existing: &Contract,
    recovered_signals: &[CiVerificationTaskSignal],
) -> Vec<String> {
    let mut detected_tasks = Vec::new();
    let mut seen = BTreeMap::<String, ()>::new();

    let script_matchers = existing
        .tasks
        .iter()
        .filter_map(|(task_name, task)| {
            let field = format!("tasks.{task_name}.run");
            if !is_verification_task_truth_field(&field) {
                return None;
            }
            let execution = task.resolved_execution(current_os())?;
            if execution.kind != "script" {
                return None;
            }

            let body = execution.shell_body()?;
            let commands = ci_script_command_sequence(body)?;
            Some((task_name.clone(), commands))
        })
        .collect::<Vec<_>>();

    let mut index = 0usize;
    while index < recovered_signals.len() {
        let signal = &recovered_signals[index];
        if !signal.exact_command {
            if let Some(task_name) = verification_task_name_from_field(&signal.field) {
                if seen.insert(task_name.to_string(), ()).is_none() {
                    detected_tasks.push(task_name.to_string());
                }
            }
            index += 1;
            continue;
        }

        if let Some((task_name, width)) =
            longest_matching_ci_script_task(&script_matchers, &recovered_signals[index..])
        {
            if seen.insert(task_name.clone(), ()).is_none() {
                detected_tasks.push(task_name);
            }
            index += width;
            continue;
        }

        if let Some(task_name) = verification_task_name_from_field(&signal.field) {
            if seen.insert(task_name.to_string(), ()).is_none() {
                detected_tasks.push(task_name.to_string());
            }
        }
        index += 1;
    }
    detected_tasks
}

fn ci_verification_signal_workflow_lane_source(source: &str) -> String {
    source.split("::").next().unwrap_or(source).to_string()
}

fn ci_verification_signal_workflow_source(source: &str) -> String {
    ci_verification_signal_workflow_lane_source(source)
        .split('#')
        .next()
        .unwrap_or(source)
        .to_string()
}

fn best_matching_ci_verification_workflow_candidate<'a>(
    existing_tasks: &[&str],
    workflow_candidates: &'a [(String, Vec<String>)],
) -> Option<&'a (String, Vec<String>)> {
    workflow_candidates
        .iter()
        .max_by_key(|(_, detected_tasks)| {
            let overlap = existing_tasks
                .iter()
                .filter(|task| detected_tasks.iter().any(|detected| detected == **task))
                .count();
            let normalized_overlap = existing_tasks
                .iter()
                .filter(|task| {
                    let existing_family = ci_verifier_task_family(task);
                    detected_tasks.iter().any(|detected| {
                        ci_verifier_task_family(detected.as_str()) == existing_family
                    })
                })
                .count();
            let extras = detected_tasks
                .iter()
                .filter(|detected| !existing_tasks.iter().any(|task| *task == detected.as_str()))
                .count();
            let missing = existing_tasks
                .iter()
                .filter(|task| !detected_tasks.iter().any(|detected| detected == **task))
                .count();
            let ordered_prefix = existing_tasks
                .iter()
                .zip(detected_tasks.iter().map(String::as_str))
                .take_while(|(left, right)| **left == *right)
                .count();
            (
                normalized_overlap,
                overlap,
                ordered_prefix,
                usize::MAX - extras,
                usize::MAX - missing,
            )
        })
}

fn ci_verifier_task_family<'a>(name: &'a str) -> &'a str {
    let normalized = name.trim();
    if normalized.starts_with("lint") {
        "lint"
    } else if normalized.starts_with("typecheck") || normalized.starts_with("check-types") {
        "typecheck"
    } else if normalized.starts_with("test") {
        "test"
    } else if normalized.starts_with("format")
        || normalized.starts_with("prettier")
        || normalized.starts_with("spotless")
    {
        "format"
    } else if normalized.starts_with("validate") {
        "validate"
    } else {
        normalized.split(':').next().unwrap_or(normalized)
    }
}

fn should_treat_ci_verifier_family_root_as_covered(
    task_name: &str,
    detected_verifier_tasks: &[&str],
) -> bool {
    let family = ci_verifier_task_family(task_name);
    task_name == family
        && detected_verifier_tasks.iter().any(|detected| {
            let detected = detected.trim();
            detected != task_name && ci_verifier_task_family(detected) == family
        })
}

fn recover_ci_verification_task_signals(
    existing: &Contract,
    signals: &[CiVerificationTaskSignal],
) -> Vec<CiVerificationTaskSignal> {
    #[derive(Clone)]
    struct VerificationCommandCandidate {
        field: String,
        has_task_env: bool,
    }

    let mut command_index = BTreeMap::<String, Vec<VerificationCommandCandidate>>::new();
    for (task_name, task) in &existing.tasks {
        let field = format!("tasks.{task_name}.run");
        if !is_verification_task_truth_field(&field) {
            continue;
        }
        let Some(command) = existing_task_detectable_command_truth(task) else {
            continue;
        };
        let Some(command_key) = canonicalize_ci_verification_command(&command) else {
            continue;
        };
        command_index
            .entry(command_key)
            .or_default()
            .push(VerificationCommandCandidate {
                field,
                has_task_env: !task.env.is_empty(),
            });
    }

    signals
        .iter()
        .map(|signal| {
            if command_index
                .values()
                .flatten()
                .any(|candidate| candidate.field == signal.field)
            {
                if let Some(command_key) = canonicalize_ci_verification_command(&signal.command)
                    && command_index.get(&command_key).is_some_and(|fields| {
                        fields
                            .iter()
                            .any(|candidate| candidate.field == signal.field)
                    })
                {
                    return signal.clone();
                }
            }

            let Some(command_key) = canonicalize_ci_verification_command(&signal.command) else {
                return signal.clone();
            };
            let Some(fields) = command_index.get(&command_key) else {
                return signal.clone();
            };
            let matched_field = if fields.len() == 1 {
                Some(fields[0].field.clone())
            } else if ci_command_declares_inline_env(&signal.command) {
                let with_env = fields
                    .iter()
                    .filter(|candidate| candidate.has_task_env)
                    .collect::<Vec<_>>();
                (with_env.len() == 1).then(|| with_env[0].field.clone())
            } else {
                let without_env = fields
                    .iter()
                    .filter(|candidate| !candidate.has_task_env)
                    .collect::<Vec<_>>();
                (without_env.len() == 1).then(|| without_env[0].field.clone())
            };
            let Some(matched_field) = matched_field else {
                return signal.clone();
            };

            let mut recovered = signal.clone();
            recovered.field = matched_field;
            recovered
        })
        .collect()
}

fn ci_command_declares_inline_env(command: &str) -> bool {
    let trimmed = command.trim();
    let Some((prefix, _)) = trimmed.split_once(' ') else {
        return false;
    };
    prefix.contains('=')
        && prefix
            .split_once('=')
            .is_some_and(|(name, _)| !name.is_empty() && is_shell_identifier(name))
}

fn is_shell_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn equivalent_ci_verification_commands(existing: &str, detected: &str) -> bool {
    existing == detected
        || canonicalize_ci_verification_command(existing)
            .zip(canonicalize_ci_verification_command(detected))
            .is_some_and(|(left, right)| left == right)
}

fn canonicalize_ci_verification_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty()
        || trimmed.contains("&&")
        || trimmed.contains("||")
        || trimmed.contains(';')
    {
        return None;
    }

    let mut normalized = trimmed;
    loop {
        let next = normalized
            .strip_prefix("corepack ")
            .or_else(|| normalized.strip_prefix("uv run "))
            .or_else(|| normalized.strip_prefix("bundle exec "))
            .or_else(|| normalized.strip_prefix("python -m "))
            .or_else(|| normalized.strip_prefix("python3 -m "));
        let Some(next) = next else {
            break;
        };
        normalized = next.trim_start();
    }

    Some(normalized.to_string())
}

fn ci_script_command_sequence(body: &str) -> Option<Vec<String>> {
    let mut commands = Vec::new();
    for raw_line in body.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let canonical = canonicalize_ci_verification_command(line)?;
        commands.push(canonical);
    }
    (!commands.is_empty()).then_some(commands)
}

fn longest_matching_ci_script_task<'a>(
    script_matchers: &'a [(String, Vec<String>)],
    signals: &[CiVerificationTaskSignal],
) -> Option<(String, usize)> {
    let exact_prefix = signals
        .iter()
        .take_while(|signal| signal.exact_command)
        .collect::<Vec<_>>();
    let command_slice = exact_prefix
        .iter()
        .map(|signal| canonicalize_ci_verification_command(&signal.command))
        .collect::<Option<Vec<_>>>()?;

    script_matchers
        .iter()
        .filter(|(_, matcher)| matcher.len() <= command_slice.len())
        .filter(|(_, matcher)| {
            matcher
                .iter()
                .zip(command_slice.iter())
                .all(|(left, right)| left == right)
        })
        .max_by_key(|(_, matcher)| matcher.len())
        .map(|(task_name, matcher)| (task_name.clone(), matcher.len()))
}

fn existing_task_detectable_command_truth(task: &crate::schema::TaskSpec) -> Option<String> {
    let execution = task.resolved_execution(current_os())?;
    matches!(execution.kind, "run" | "script" | "command").then(|| execution.preview())
}

fn matches_governed_external_source(
    source_class: Option<&str>,
    source: &str,
    field: &str,
    confidence: Option<Confidence>,
) -> bool {
    match source_class {
        Some("environment_toolchain") => {
            confidence == Some(Confidence::High)
                && is_governed_external_environment_source(source)
                && is_runtime_or_toolchain_truth_field(field)
        }
        Some("task_command") => {
            confidence == Some(Confidence::High)
                && is_governed_task_command_source(source)
                && is_task_command_truth_field(field)
        }
        Some("ci_verification") => {
            confidence == Some(Confidence::Medium)
                && is_governed_ci_verification_source(source)
                && is_task_command_truth_field(field)
                && is_verification_task_truth_field(field)
        }
        _ => false,
    }
}

fn is_governed_external_environment_source(source: &str) -> bool {
    let source_file = source.split('#').next().unwrap_or(source);
    matches!(source_file, "mise.toml" | "devbox.json" | "devenv.nix")
}

fn is_governed_task_command_source(source: &str) -> bool {
    source.starts_with("package.json#scripts.")
        || source.starts_with("devbox.json#shell.scripts.")
        || source.starts_with("Taskfile.yml#tasks.")
        || source.starts_with("Taskfile.yaml#tasks.")
        || source.starts_with("justfile#")
}

fn is_governed_ci_verification_source(source: &str) -> bool {
    source.starts_with(".github/workflows/") && source.contains("#jobs.")
}

fn is_runtime_or_toolchain_truth_field(field: &str) -> bool {
    field.starts_with("runtimes.")
        || field.starts_with("toolchains.")
        || matches!(
            field,
            "tools.pnpm" | "tools.npm" | "tools.yarn" | "tools.bun"
        )
}

fn is_task_command_truth_field(field: &str) -> bool {
    field.starts_with("tasks.") && field.ends_with(".run")
}

fn is_verification_task_truth_field(field: &str) -> bool {
    verification_task_name_from_field(field).is_some_and(is_verifier_task_name)
}

fn is_verifier_task_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| {
            matches!(
                token,
                "test"
                    | "tests"
                    | "lint"
                    | "typecheck"
                    | "check"
                    | "verify"
                    | "fmt"
                    | "format"
                    | "ci"
            )
        })
}

fn verification_task_name_from_field(field: &str) -> Option<&str> {
    field
        .strip_prefix("tasks.")
        .and_then(|value| value.strip_suffix(".run"))
}

fn render_string_list<T: AsRef<str>>(values: &[T]) -> String {
    values
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(", ")
}

fn detect_change_ownership(existing: Option<&str>) -> String {
    if existing.is_some() {
        String::from(DETECT_COMPARISON_REPO_CONTRACT_OWNERSHIP)
    } else {
        String::from(DETECT_COMPARISON_REPO_SIGNALS_OWNERSHIP)
    }
}

fn detect_change_provenance() -> String {
    String::from(DETECT_COMPARISON_REPO_SIGNALS_PROVENANCE)
}

fn detect_change_provenance_key() -> String {
    String::from(DETECT_COMPARISON_REPO_SIGNALS_PROVENANCE_KEY)
}

fn detect_existing_field_owner_kind(contract: &Contract, field: &str) -> &'static str {
    match detect_field_owner_kind_metadata(contract, field).as_deref() {
        Some(DETECT_OWNER_KIND_MERGED) => DETECT_OWNER_KIND_MERGED,
        Some(DETECT_OWNER_KIND_POLICY) => DETECT_OWNER_KIND_POLICY,
        Some(DETECT_OWNER_KIND_MANUAL) => DETECT_OWNER_KIND_MANUAL,
        _ => DETECT_OWNER_KIND_MANUAL,
    }
}

pub(crate) fn detect_change_owner_kind(
    existing: &Contract,
    field: &str,
    has_existing: bool,
) -> String {
    if has_existing {
        detect_existing_field_owner_kind(existing, field).to_string()
    } else {
        String::from(DETECT_OWNER_KIND_DETECTED)
    }
}

fn detect_field_owner_kind_metadata(contract: &Contract, field: &str) -> Option<String> {
    let ota = contract.metadata.get("ota")?.as_mapping()?;
    let detect = mapping_child(ota, "detect")?;
    let field_ownership = mapping_child(detect, "field_ownership")?;
    mapping_string(field_ownership, field).map(ToString::to_string)
}

fn mapping_child<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Mapping> {
    mapping
        .get(&YamlValue::String(key.to_string()))
        .and_then(YamlValue::as_mapping)
}

fn mapping_string<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a str> {
    mapping
        .get(&YamlValue::String(key.to_string()))
        .and_then(YamlValue::as_str)
}

fn push_detect_removal(
    removals: &mut Vec<DetectComparisonRemoval>,
    existing_contract: &Contract,
    field: String,
    existing: String,
) {
    removals.push(DetectComparisonRemoval {
        owner_kind: Some(detect_existing_field_owner_kind(existing_contract, &field).to_string()),
        field,
        existing,
        ownership: Some(String::from(DETECT_COMPARISON_REPO_CONTRACT_OWNERSHIP)),
        provenance: Some(detect_change_provenance()),
        provenance_key: Some(detect_change_provenance_key()),
    });
}

pub(crate) fn collect_detect_drift_removals(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
) -> Vec<DetectComparisonRemoval> {
    collect_detect_removals(existing, detected)
        .into_iter()
        .filter(|removal| {
            removal
                .owner_kind
                .as_deref()
                .is_some_and(|owner_kind| owner_kind == DETECT_OWNER_KIND_MERGED)
        })
        .collect()
}

pub(crate) fn collect_detect_changes(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
    inferences: &[Inference],
) -> Vec<DetectComparisonChange> {
    let mut changes = Vec::new();
    let inference_index = inferences
        .iter()
        .map(|inference| (inference.field.as_str(), inference))
        .collect::<std::collections::BTreeMap<_, _>>();

    if let Some(project) = detected.project.as_ref() {
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            "project.name",
            Some(existing.project.name.as_str()),
            Some(project.name.as_str()),
        );
    }

    let existing_execution_fields = existing_execution_field_values(existing);
    for (field, detected_value) in detect_execution_field_values(detected) {
        if !should_surface_detect_execution_change(
            existing,
            field.as_str(),
            existing_execution_fields.contains_key(&field),
        ) {
            continue;
        }
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &field,
            existing_execution_fields.get(&field).map(String::as_str),
            Some(detected_value.as_str()),
        );
    }

    for (name, value) in &detected.runtimes {
        let existing_version = match existing.runtimes.get(name) {
            Some(requirement) if !requirement.required_for_os(current_os()) => continue,
            Some(requirement) => Some(requirement.version_for_os(current_os())),
            None => None,
        };
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("runtimes.{name}"),
            existing_version,
            Some(value.as_str()),
        );
    }

    for (name, value) in &detected.tools {
        let existing_version = match existing.tools.get(name) {
            Some(requirement) if !requirement.required_for_os(current_os()) => continue,
            Some(requirement) => Some(requirement.version_for_os(current_os())),
            None => None,
        };
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("tools.{name}"),
            existing_version,
            Some(value.as_str()),
        );
    }

    for (name, toolchain) in &detected.toolchains {
        let existing_toolchain = match existing.toolchains.get(name) {
            Some(existing_toolchain) if !existing_toolchain.active_for_os(current_os()) => continue,
            value => value,
        };
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("toolchains.{name}.version"),
            existing_toolchain.map(|value| value.version_for_os(current_os())),
            Some(toolchain.version.as_str()),
        );
        for (package_manager, version) in &toolchain.package_managers {
            push_detect_change(
                &mut changes,
                existing,
                &inference_index,
                &format!("toolchains.{name}.package_managers.{package_manager}"),
                existing_toolchain.and_then(|value| {
                    value
                        .package_managers
                        .get(package_manager)
                        .map(String::as_str)
                }),
                Some(version.as_str()),
            );
        }
        if let Some(fulfillment) = toolchain.fulfillment.as_ref().map(|value| value.mode) {
            push_detect_change(
                &mut changes,
                existing,
                &inference_index,
                &format!("toolchains.{name}.fulfillment"),
                existing_toolchain
                    .map(|value| value.fulfillment_mode())
                    .map(toolchain_fulfillment_name),
                Some(toolchain_fulfillment_name(fulfillment)),
            );
        }
    }

    let mut next_env_source_add_index = existing.env.sources.len();
    for (detected_index, source) in detected.env.sources.iter().enumerate() {
        if existing.env.sources.iter().any(|existing_source| {
            existing_source.kind == source.kind && existing_source.path == source.path
        }) {
            continue;
        }

        if let Some(existing_index) = existing
            .env
            .sources
            .iter()
            .position(|existing_source| existing_source.path == source.path)
        {
            let existing_kind = existing.env.sources[existing_index].kind.to_string();
            let detected_kind = source.kind.to_string();
            let detected_field = format!("env.sources.{detected_index}.kind");
            push_detect_change_with_inference(
                &mut changes,
                existing,
                inference_index.get(detected_field.as_str()).copied(),
                &format!("env.sources.{existing_index}.kind"),
                Some(existing_kind.as_str()),
                Some(detected_kind.as_str()),
            );
            continue;
        }

        let change_kind = format!("env.sources.{next_env_source_add_index}.kind");
        let change_path = format!("env.sources.{next_env_source_add_index}.path");
        let detected_kind = source.kind.to_string();
        let detected_kind_field = format!("env.sources.{detected_index}.kind");
        let detected_path_field = format!("env.sources.{detected_index}.path");
        push_detect_change_with_inference(
            &mut changes,
            existing,
            inference_index.get(detected_kind_field.as_str()).copied(),
            &change_kind,
            None,
            Some(detected_kind.as_str()),
        );
        push_detect_change_with_inference(
            &mut changes,
            existing,
            inference_index.get(detected_path_field.as_str()).copied(),
            &change_path,
            None,
            Some(source.path.as_str()),
        );
        if source.must_exist {
            let detected_must_exist_field = format!("env.sources.{detected_index}.must_exist");
            push_detect_change_with_inference(
                &mut changes,
                existing,
                inference_index
                    .get(detected_must_exist_field.as_str())
                    .copied(),
                &format!("env.sources.{next_env_source_add_index}.must_exist"),
                None,
                Some("true"),
            );
        }
        next_env_source_add_index += 1;
    }

    for (name, service) in &detected.services {
        let existing_fields = existing
            .services
            .get(name)
            .map(|service| existing_service_field_values(name, service))
            .unwrap_or_default();
        for (field, detected_value) in detect_service_field_values(name, service) {
            if !should_surface_detect_service_change(
                existing,
                detected,
                field.as_str(),
                existing_fields.contains_key(&field),
            ) {
                continue;
            }
            push_detect_change(
                &mut changes,
                existing,
                &inference_index,
                &field,
                existing_fields.get(&field).map(String::as_str),
                Some(detected_value.as_str()),
            );
        }
    }

    for (name, task) in &detected.tasks {
        let existing_value = existing
            .tasks
            .get(name)
            .and_then(existing_task_detectable_command_truth);
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("tasks.{name}.run"),
            existing_value.as_deref(),
            Some(task.run.as_str()),
        );
        if task.internal {
            let existing_internal = existing.tasks.get(name).map(|task| task.internal);
            push_detect_change(
                &mut changes,
                existing,
                &inference_index,
                &format!("tasks.{name}.internal"),
                existing_internal.map(|value| if value { "true" } else { "false" }),
                Some("true"),
            );
        }
        if task.safe_for_agent {
            let existing_safe = existing.tasks.get(name).map(|task| task.safe_for_agent);
            push_detect_change(
                &mut changes,
                existing,
                &inference_index,
                &format!("tasks.{name}.safe_for_agent"),
                existing_safe.map(|value| if value { "true" } else { "false" }),
                Some("true"),
            );
        }
    }

    changes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExistingExecutionMergeMode {
    None,
    ContextMode,
    Shorthand,
}

fn existing_execution_merge_mode(existing: &Contract) -> ExistingExecutionMergeMode {
    let Some(execution) = existing.execution.as_ref() else {
        return ExistingExecutionMergeMode::None;
    };
    if execution.default_context.is_some() || !execution.contexts.is_empty() {
        ExistingExecutionMergeMode::ContextMode
    } else {
        ExistingExecutionMergeMode::Shorthand
    }
}

fn should_surface_detect_execution_change(
    existing: &Contract,
    field: &str,
    existing_field_present: bool,
) -> bool {
    match field {
        "execution.default_context" => existing.execution.is_none(),
        "execution.contexts.host.backend" => {
            !existing_field_present
                && !matches!(
                    existing_execution_merge_mode(existing),
                    ExistingExecutionMergeMode::Shorthand
                )
        }
        _ => true,
    }
}

fn should_surface_detect_service_change(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
    field: &str,
    existing_field_present: bool,
) -> bool {
    if !is_detected_host_topology_service_field(detected, field) {
        return true;
    }
    if existing_field_present {
        return false;
    }

    match existing_execution_merge_mode(existing) {
        ExistingExecutionMergeMode::None => true,
        ExistingExecutionMergeMode::Shorthand => false,
        ExistingExecutionMergeMode::ContextMode => {
            existing_host_context_is_native_or_missing(existing)
        }
    }
}

fn is_detected_host_topology_service_field(
    detected: &crate::detector::DetectContract,
    field: &str,
) -> bool {
    let segments = field.split('.').collect::<Vec<_>>();
    match segments.as_slice() {
        [
            "services",
            service_name,
            "endpoints",
            "host",
            "address" | "port",
        ] => detected
            .services
            .get(*service_name)
            .is_some_and(|service| service.endpoints.contains_key("host")),
        ["services", service_name, "readiness", "from"]
        | ["services", service_name, "readiness", "kind"] => detected
            .services
            .get(*service_name)
            .and_then(|service| service.readiness.as_ref())
            .is_some_and(|readiness| readiness.from.as_deref() == Some("host")),
        _ => false,
    }
}

fn existing_host_context_is_native_or_missing(existing: &Contract) -> bool {
    existing
        .execution
        .as_ref()
        .and_then(|execution| execution.contexts.get("host"))
        .is_none_or(|context| context.backend == Backend::Native)
}

pub(crate) fn collect_detect_removals(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
) -> Vec<DetectComparisonRemoval> {
    let mut removals = Vec::new();

    if detected.project.is_none() {
        push_detect_removal(
            &mut removals,
            existing,
            String::from("project.name"),
            existing.project.name.clone(),
        );
    }

    let existing_execution_fields = existing_execution_field_values(existing);
    let detected_execution_fields = detect_execution_field_values(detected)
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    for (field, existing_value) in existing_execution_fields {
        if !detected_execution_fields.contains_key(&field) {
            push_detect_removal(&mut removals, existing, field, existing_value);
        }
    }

    for (name, requirement) in &existing.runtimes {
        if !requirement.required_for_os(current_os()) {
            continue;
        }
        if !detected.runtimes.contains_key(name) {
            push_detect_removal(
                &mut removals,
                existing,
                format!("runtimes.{name}"),
                requirement.version_for_os(current_os()).to_string(),
            );
        }
    }

    for (name, requirement) in &existing.tools {
        if !requirement.required_for_os(current_os()) {
            continue;
        }
        if !detected.tools.contains_key(name) {
            push_detect_removal(
                &mut removals,
                existing,
                format!("tools.{name}"),
                requirement.version_for_os(current_os()).to_string(),
            );
        }
    }

    for (name, toolchain) in &existing.toolchains {
        if !toolchain.active_for_os(current_os()) {
            continue;
        }
        let detected_toolchain = detected.toolchains.get(name);
        if let Some(provider) = toolchain.provider {
            push_detect_removal(
                &mut removals,
                existing,
                format!("toolchains.{name}.provider"),
                toolchain_provider_name(provider).to_string(),
            );
        }
        if detected_toolchain.is_none() {
            push_detect_removal(
                &mut removals,
                existing,
                format!("toolchains.{name}.version"),
                toolchain.version_for_os(current_os()).to_string(),
            );
        }
        for (package_manager, version) in &toolchain.package_managers {
            if !detected_toolchain
                .is_some_and(|value| value.package_managers.contains_key(package_manager))
            {
                push_detect_removal(
                    &mut removals,
                    existing,
                    format!("toolchains.{name}.package_managers.{package_manager}"),
                    version.to_string(),
                );
            }
        }
        let fulfillment = toolchain.fulfillment_mode();
        if fulfillment != ToolchainFulfillmentMode::None
            && !detected_toolchain.is_some_and(|value| value.fulfillment.is_some())
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("toolchains.{name}.fulfillment"),
                toolchain_fulfillment_name(fulfillment).to_string(),
            );
        }
    }

    for (index, source) in existing.env.sources.iter().enumerate() {
        if !detected.env.sources.iter().any(|detected_source| {
            detected_source.kind == source.kind && detected_source.path == source.path
        }) {
            push_detect_removal(
                &mut removals,
                existing,
                format!("env.sources.{index}.kind"),
                source.kind.to_string(),
            );
            push_detect_removal(
                &mut removals,
                existing,
                format!("env.sources.{index}.path"),
                source.path.clone(),
            );
            if source.must_exist {
                push_detect_removal(
                    &mut removals,
                    existing,
                    format!("env.sources.{index}.must_exist"),
                    String::from("true"),
                );
            }
        }
    }

    for (name, service) in &existing.services {
        let existing_fields = existing_service_field_values(name, service);
        let detected_fields = detected
            .services
            .get(name)
            .map(|value| detect_service_field_values(name, value))
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        for (field, existing_value) in existing_fields {
            if !detected_fields.contains_key(&field) {
                push_detect_removal(&mut removals, existing, field, existing_value);
            }
        }
    }

    for (name, task) in &existing.tasks {
        let detected_task = detected.tasks.get(name);
        if let Some(existing_run) = existing_task_detectable_command_truth(task)
            && detected_task.is_none()
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("tasks.{name}.run"),
                existing_run,
            );
        }
        if task.safe_for_agent && !detected_task.is_some_and(|task| task.safe_for_agent) {
            push_detect_removal(
                &mut removals,
                existing,
                format!("tasks.{name}.safe_for_agent"),
                String::from("true"),
            );
        }
    }

    removals
}

fn existing_execution_field_values(existing: &Contract) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    let Some(execution) = existing.execution.as_ref() else {
        return fields;
    };

    if let Some(default_context) = execution.default_context.as_ref() {
        fields.insert(
            String::from("execution.default_context"),
            default_context.clone(),
        );
    }
    for (context_name, context) in &execution.contexts {
        fields.insert(
            format!("execution.contexts.{context_name}.backend"),
            backend_name(context.backend).to_string(),
        );
    }
    fields
}

fn detect_execution_field_values(
    detected: &crate::detector::DetectContract,
) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let Some(execution) = detected.execution.as_ref() else {
        return fields;
    };

    if let Some(default_context) = execution.default_context.as_ref() {
        fields.push((
            String::from("execution.default_context"),
            default_context.clone(),
        ));
    }
    for (context_name, context) in &execution.contexts {
        fields.push((
            format!("execution.contexts.{context_name}.backend"),
            context.backend.clone(),
        ));
    }
    fields
}

fn existing_service_field_values(name: &str, service: &ServiceSpec) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    if let Some(manager) = service.manager.as_ref() {
        fields.insert(
            format!("services.{name}.manager.kind"),
            manager.kind.as_str().to_string(),
        );
        if let Some(manager_name) = manager.name.as_ref() {
            fields.insert(
                format!("services.{name}.manager.name"),
                manager_name.clone(),
            );
        }
        if let Some(file) = manager.file.as_ref() {
            fields.insert(format!("services.{name}.manager.file"), file.clone());
        }
        for (index, file) in manager.files.iter().enumerate() {
            fields.insert(
                format!("services.{name}.manager.files.{index}"),
                file.clone(),
            );
        }
        if let Some(env_file) = manager.env_file.as_ref() {
            fields.insert(
                format!("services.{name}.manager.env_file"),
                env_file.clone(),
            );
        }
        for (index, env_file) in manager.env_files.iter().enumerate() {
            fields.insert(
                format!("services.{name}.manager.env_files.{index}"),
                env_file.clone(),
            );
        }
        if let Some(service_name) = manager.service.as_ref() {
            fields.insert(
                format!("services.{name}.manager.service"),
                service_name.clone(),
            );
        }
    }
    if let Some(provider) = service.provider.as_ref() {
        fields.insert(format!("services.{name}.provider"), provider.clone());
    }
    if let Some(start) = service.start.as_ref() {
        fields.insert(format!("services.{name}.start"), start.clone());
    }
    if let Some(stop) = service.stop.as_ref() {
        fields.insert(format!("services.{name}.stop"), stop.clone());
    }
    if let Some(healthcheck) = service.healthcheck.as_ref() {
        fields.insert(format!("services.{name}.healthcheck"), healthcheck.clone());
    }
    for (endpoint_name, endpoint) in &service.endpoints {
        if let Some(context) = endpoint.context.as_ref() {
            fields.insert(
                format!("services.{name}.endpoints.{endpoint_name}.context"),
                context.clone(),
            );
        }
        fields.insert(
            format!("services.{name}.endpoints.{endpoint_name}.address"),
            endpoint.address.clone(),
        );
        fields.insert(
            format!("services.{name}.endpoints.{endpoint_name}.port"),
            endpoint.port.to_string(),
        );
    }
    if let Some(readiness) = service.readiness.as_ref() {
        if let Some(from) = readiness.from.as_ref() {
            fields.insert(format!("services.{name}.readiness.from"), from.clone());
        }
        if let Some(endpoint) = readiness.endpoint.as_ref() {
            fields.insert(
                format!("services.{name}.readiness.endpoint"),
                endpoint.clone(),
            );
        }
        if let Some(kind) = readiness.kind {
            fields.insert(
                format!("services.{name}.readiness.kind"),
                kind.as_str().to_string(),
            );
        }
    }

    fields
}

fn detect_service_field_values(name: &str, service: &DetectService) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    if let Some(manager) = service.manager.as_ref() {
        fields.push((
            format!("services.{name}.manager.kind"),
            manager.kind.as_str().to_string(),
        ));
        if let Some(manager_name) = manager.name.as_ref() {
            fields.push((
                format!("services.{name}.manager.name"),
                manager_name.clone(),
            ));
        }
        if let Some(file) = manager.file.as_ref() {
            fields.push((format!("services.{name}.manager.file"), file.clone()));
        }
        for (index, file) in manager.files.iter().enumerate() {
            fields.push((
                format!("services.{name}.manager.files.{index}"),
                file.clone(),
            ));
        }
        if let Some(env_file) = manager.env_file.as_ref() {
            fields.push((
                format!("services.{name}.manager.env_file"),
                env_file.clone(),
            ));
        }
        for (index, env_file) in manager.env_files.iter().enumerate() {
            fields.push((
                format!("services.{name}.manager.env_files.{index}"),
                env_file.clone(),
            ));
        }
        if let Some(service_name) = manager.service.as_ref() {
            fields.push((
                format!("services.{name}.manager.service"),
                service_name.clone(),
            ));
        }
    }
    if let Some(provider) = service.provider.as_ref() {
        fields.push((format!("services.{name}.provider"), provider.clone()));
    }
    if let Some(start) = service.start.as_ref() {
        fields.push((format!("services.{name}.start"), start.clone()));
    }
    if let Some(stop) = service.stop.as_ref() {
        fields.push((format!("services.{name}.stop"), stop.clone()));
    }
    if let Some(healthcheck) = service.healthcheck.as_ref() {
        fields.push((format!("services.{name}.healthcheck"), healthcheck.clone()));
    }
    for (endpoint_name, endpoint) in &service.endpoints {
        if let Some(context) = endpoint.context.as_ref() {
            fields.push((
                format!("services.{name}.endpoints.{endpoint_name}.context"),
                context.clone(),
            ));
        }
        fields.push((
            format!("services.{name}.endpoints.{endpoint_name}.address"),
            endpoint.address.clone(),
        ));
        fields.push((
            format!("services.{name}.endpoints.{endpoint_name}.port"),
            endpoint.port.to_string(),
        ));
    }
    if let Some(readiness) = service.readiness.as_ref() {
        if let Some(from) = readiness.from.as_ref() {
            fields.push((format!("services.{name}.readiness.from"), from.clone()));
        }
        if let Some(endpoint) = readiness.endpoint.as_ref() {
            fields.push((
                format!("services.{name}.readiness.endpoint"),
                endpoint.clone(),
            ));
        }
        if let Some(kind) = readiness.kind {
            fields.push((
                format!("services.{name}.readiness.kind"),
                kind.as_str().to_string(),
            ));
        }
    }
    fields
}

fn toolchain_provider_name(provider: ToolchainProvider) -> &'static str {
    match provider {
        ToolchainProvider::Rustup => "rustup",
        ToolchainProvider::Corepack => "corepack",
        ToolchainProvider::Sdkman => "sdkman",
        ToolchainProvider::Uv => "uv",
        ToolchainProvider::Go => "go",
        ToolchainProvider::Ruby => "ruby",
        ToolchainProvider::Dotnet => "dotnet",
    }
}

fn toolchain_fulfillment_name(fulfillment: ToolchainFulfillmentMode) -> &'static str {
    match fulfillment {
        ToolchainFulfillmentMode::None => "none",
        ToolchainFulfillmentMode::Run => "run",
    }
}

fn backend_name(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

#[cfg(target_os = "windows")]
fn current_os() -> &'static str {
    "windows"
}

#[cfg(target_os = "macos")]
fn current_os() -> &'static str {
    "macos"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn current_os() -> &'static str {
    "linux"
}

fn push_detect_change(
    changes: &mut Vec<DetectComparisonChange>,
    existing_contract: &Contract,
    inference_index: &std::collections::BTreeMap<&str, &Inference>,
    field: &str,
    existing: Option<&str>,
    detected: Option<&str>,
) {
    let Some(detected) = detected else {
        return;
    };
    let inference = inference_index.get(field).copied();
    push_detect_change_with_inference(
        changes,
        existing_contract,
        inference,
        field,
        existing,
        Some(detected),
    );
}

fn push_detect_change_with_inference(
    changes: &mut Vec<DetectComparisonChange>,
    existing_contract: &Contract,
    inference: Option<&Inference>,
    field: &str,
    existing: Option<&str>,
    detected: Option<&str>,
) {
    let Some(detected) = detected else {
        return;
    };

    match existing {
        None => changes.push(DetectComparisonChange {
            field: field.to_string(),
            status: "add",
            existing: None,
            detected: detected.to_string(),
            owner_kind: Some(detect_change_owner_kind(existing_contract, field, false)),
            ownership: Some(detect_change_ownership(None)),
            provenance: Some(detect_change_provenance()),
            provenance_key: Some(detect_change_provenance_key()),
            source: inference.map(|value| value.source.clone()),
            source_class: inference.map(|value| value.source_class.to_string()),
            confidence: inference.map(|value| value.confidence),
        }),
        Some(existing) if existing != detected => changes.push(DetectComparisonChange {
            field: field.to_string(),
            status: "update",
            existing: Some(existing.to_string()),
            detected: detected.to_string(),
            owner_kind: Some(detect_change_owner_kind(existing_contract, field, true)),
            ownership: Some(detect_change_ownership(Some(existing))),
            provenance: Some(detect_change_provenance()),
            provenance_key: Some(detect_change_provenance_key()),
            source: inference.map(|value| value.source.clone()),
            source_class: inference.map(|value| value.source_class.to_string()),
            confidence: inference.map(|value| value.confidence),
        }),
        Some(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::{
        Confidence, DetectContract, DetectExecution, DetectExecutionContext, DetectProject,
        DetectService, DetectTask, DetectToolchainSpec, Inference,
    };
    use crate::parser::parse_contract_str;
    use crate::schema::{EnvSource, EnvSourceKind, ToolchainProvider};
    use std::collections::BTreeMap;
    use std::path::Path;

    #[test]
    fn collect_detect_removals_keeps_manual_task_fields_visible_for_rewrite_only() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo fetch
    safe_for_agent: true
  ci:
    run: cargo test
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            ..DetectContract::default()
        };

        let removals = collect_detect_removals(&existing, &detected);
        assert_eq!(removals.len(), 3);
        assert_eq!(removals[0].field, "tasks.ci.run");
        assert_eq!(removals[0].owner_kind.as_deref(), Some("manual"));
        assert_eq!(removals[1].field, "tasks.setup.run");
        assert_eq!(removals[1].owner_kind.as_deref(), Some("manual"));
        assert_eq!(removals[2].field, "tasks.setup.safe_for_agent");
        assert_eq!(removals[2].owner_kind.as_deref(), Some("manual"));

        let drift = collect_detect_drift_removals(&existing, &detected);
        assert!(drift.is_empty());
    }

    #[test]
    fn collect_detect_removals_keeps_positive_non_task_removals() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    provider: docker-compose
"#,
        )
        .unwrap();

        let mut services = BTreeMap::new();
        services.insert(
            String::from("postgres"),
            DetectService {
                manager: None,
                provider: None,
                start: None,
                stop: None,
                endpoints: BTreeMap::new(),
                healthcheck: None,
                readiness: None,
            },
        );
        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            services,
            tasks: BTreeMap::<String, DetectTask>::new(),
            ..DetectContract::default()
        };

        let removals = collect_detect_removals(&existing, &detected);
        assert_eq!(removals.len(), 1);
        assert_eq!(removals[0].field, "services.postgres.provider");
        assert_eq!(removals[0].owner_kind.as_deref(), Some("manual"));
        assert_eq!(removals[0].ownership.as_deref(), Some("repo_contract"));
        assert_eq!(removals[0].provenance.as_deref(), Some("repo_signals"));
    }

    #[test]
    fn collect_detect_removals_keeps_execution_topology_fields_visible() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            execution: Some(DetectExecution {
                default_context: Some(String::from("host")),
                contexts: BTreeMap::from([(
                    String::from("host"),
                    DetectExecutionContext {
                        backend: String::from("native"),
                    },
                )]),
            }),
            ..DetectContract::default()
        };

        assert!(collect_detect_removals(&existing, &detected).is_empty());

        let detected_without_execution = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            ..DetectContract::default()
        };

        let removals = collect_detect_removals(&existing, &detected_without_execution);
        assert_eq!(removals.len(), 2);
        assert_eq!(removals[0].field, "execution.contexts.host.backend");
        assert_eq!(removals[1].field, "execution.default_context");
    }

    #[test]
    fn collect_detect_changes_skips_host_default_context_update_for_existing_context_mode() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: node:22
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            execution: Some(DetectExecution {
                default_context: Some(String::from("host")),
                contexts: BTreeMap::from([(
                    String::from("host"),
                    DetectExecutionContext {
                        backend: String::from("native"),
                    },
                )]),
            }),
            ..DetectContract::default()
        };
        let inferences = vec![
            Inference::new(
                String::from("execution.default_context"),
                String::from("host"),
                String::from("docker-compose.yml#services.web.ports[0]"),
                Confidence::High,
            ),
            Inference::new(
                String::from("execution.contexts.host.backend"),
                String::from("native"),
                String::from("docker-compose.yml#services.web.ports[0]"),
                Confidence::High,
            ),
        ];

        let changes = collect_detect_changes(&existing, &detected, &inferences);
        assert!(
            !changes
                .iter()
                .any(|change| change.field == "execution.default_context")
        );
        assert!(
            changes
                .iter()
                .any(|change| change.field == "execution.contexts.host.backend"
                    && change.status == "add")
        );
    }

    #[test]
    fn collect_detect_changes_skip_host_topology_additions_for_shorthand_execution() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: node:22
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            execution: Some(DetectExecution {
                default_context: Some(String::from("host")),
                contexts: BTreeMap::from([(
                    String::from("host"),
                    DetectExecutionContext {
                        backend: String::from("native"),
                    },
                )]),
            }),
            services: BTreeMap::from([(
                String::from("web"),
                DetectService {
                    manager: Some(crate::detector::DetectServiceManagerSpec {
                        kind: crate::schema::ServiceManagerKind::Compose,
                        engine: crate::schema::ComposeCliEngine::Docker,
                        name: Some(String::from("ota")),
                        file: Some(String::from("docker-compose.yml")),
                        files: vec![String::from("docker-compose.yml")],
                        env_file: None,
                        env_files: Vec::new(),
                        profiles: Vec::new(),
                        service: Some(String::from("web")),
                        start: None,
                        stop: None,
                    }),
                    provider: None,
                    start: None,
                    stop: None,
                    endpoints: BTreeMap::from([(
                        String::from("host"),
                        crate::detector::DetectServiceEndpointSpec {
                            context: None,
                            address: String::from("127.0.0.1"),
                            port: 3000,
                        },
                    )]),
                    healthcheck: None,
                    readiness: Some(crate::detector::DetectServiceReadinessSpec {
                        from: Some(String::from("host")),
                        endpoint: None,
                        kind: Some(crate::schema::ServiceReadinessKind::Tcp),
                    }),
                },
            )]),
            ..DetectContract::default()
        };

        let changes = collect_detect_changes(&existing, &detected, &[]);
        let fields = changes
            .iter()
            .map(|change| change.field.as_str())
            .collect::<Vec<_>>();

        assert!(!fields.contains(&"execution.default_context"));
        assert!(!fields.contains(&"execution.contexts.host.backend"));
        assert!(!fields.contains(&"services.web.endpoints.host.address"));
        assert!(!fields.contains(&"services.web.endpoints.host.port"));
        assert!(!fields.contains(&"services.web.readiness.from"));
        assert!(!fields.contains(&"services.web.readiness.kind"));
        assert!(fields.contains(&"services.web.manager.kind"));
    }

    #[test]
    fn collect_detect_drift_removals_only_keeps_ota_managed_fields() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  cargo: "1.78"
tasks:
  setup:
    run: cargo fetch
metadata:
  ota:
    detect:
      field_ownership:
        tools.cargo: merged
        tasks.setup.run: merged
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            ..DetectContract::default()
        };

        let removals = collect_detect_removals(&existing, &detected);
        assert_eq!(removals.len(), 2);
        assert_eq!(removals[0].owner_kind.as_deref(), Some("merged"));
        assert_eq!(removals[1].owner_kind.as_deref(), Some("merged"));

        let drift = collect_detect_drift_removals(&existing, &detected);
        assert_eq!(drift.len(), 2);
        assert_eq!(drift[0].field, "tools.cargo");
        assert_eq!(drift[1].field, "tasks.setup.run");
    }

    #[test]
    fn collect_detect_changes_surfaces_toolchain_package_manager_fields() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            toolchains: BTreeMap::from([(
                String::from("node"),
                DetectToolchainSpec {
                    provider: ToolchainProvider::Corepack,
                    version: String::from("22"),
                    package_managers: BTreeMap::from([(
                        String::from("pnpm"),
                        String::from("10.27.0"),
                    )]),
                    fulfillment: None,
                },
            )]),
            ..DetectContract::default()
        };
        let inferences = vec![
            Inference::new(
                String::from("toolchains.node.version"),
                String::from("22"),
                String::from(".nvmrc"),
                Confidence::High,
            ),
            Inference::new(
                String::from("toolchains.node.package_managers.pnpm"),
                String::from("10.27.0"),
                String::from("package.json#packageManager"),
                Confidence::High,
            ),
        ];

        let changes = collect_detect_changes(&existing, &detected, &inferences);
        let fields = changes
            .iter()
            .map(|change| change.field.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            fields,
            vec![
                "toolchains.node.version",
                "toolchains.node.package_managers.pnpm",
            ]
        );
        assert!(changes.iter().all(|change| change.status == "add"));
        assert!(
            changes
                .iter()
                .all(|change| change.owner_kind.as_deref() == Some("detected"))
        );
        assert_eq!(
            changes[1].source.as_deref(),
            Some("package.json#packageManager")
        );
    }

    #[test]
    fn collect_detect_drift_removals_tracks_ota_managed_toolchain_fields() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    package_managers:
      pnpm: "10.27.0"
metadata:
  ota:
    detect:
      field_ownership:
        toolchains.node.version: merged
        toolchains.node.package_managers.pnpm: merged
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            ..DetectContract::default()
        };

        let drift = collect_detect_drift_removals(&existing, &detected);
        let fields = drift
            .iter()
            .map(|removal| removal.field.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            fields,
            vec![
                "toolchains.node.version",
                "toolchains.node.package_managers.pnpm",
            ]
        );
        assert!(
            drift
                .iter()
                .all(|removal| removal.owner_kind.as_deref() == Some("merged"))
        );
    }

    #[test]
    fn collect_detect_removals_skips_platform_scoped_tool_outside_current_os() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project:
  name: ota
tools:
  pwsh:
    version: "7.6.0"
    only_on:
      - {}
"#,
                match super::current_os() {
                    "windows" => "linux",
                    _ => "windows",
                }
            ),
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            ..DetectContract::default()
        };

        let removals = collect_detect_removals(&existing, &detected);
        assert!(removals.is_empty());
    }

    #[test]
    fn collect_detect_changes_skips_matching_declared_env_sources() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  sources:
    - kind: dotenv
      path: .env.local
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            env: crate::schema::EnvConfig {
                vars: BTreeMap::new(),
                sources: vec![EnvSource {
                    kind: EnvSourceKind::Dotenv,
                    path: String::from(".env.local"),
                    must_exist: false,
                }],
                profiles: BTreeMap::new(),
            },
            ..DetectContract::default()
        };

        let changes = collect_detect_changes(&existing, &detected, &[]);
        assert!(changes.is_empty());
    }

    #[test]
    fn collect_detect_changes_surfaces_env_source_kind_conflicts() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  sources:
    - kind: dotenv
      path: appsettings.json
"#,
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            env: crate::schema::EnvConfig {
                vars: BTreeMap::new(),
                sources: vec![EnvSource {
                    kind: EnvSourceKind::Json,
                    path: String::from("appsettings.json"),
                    must_exist: false,
                }],
                profiles: BTreeMap::new(),
            },
            ..DetectContract::default()
        };
        let inferences = vec![Inference::new(
            String::from("env.sources.0.kind"),
            String::from("json"),
            String::from("appsettings.json"),
            Confidence::High,
        )];

        let changes = collect_detect_changes(&existing, &detected, &inferences);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "env.sources.0.kind");
        assert_eq!(changes[0].status, "update");
        assert_eq!(changes[0].existing.as_deref(), Some("dotenv"));
        assert_eq!(changes[0].detected, "json");
    }

    #[test]
    fn collect_detect_changes_skips_platform_scoped_tool_outside_current_os() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project:
  name: ota
tools:
  pwsh:
    version: "7.6.0"
    only_on:
      - {}
"#,
                match super::current_os() {
                    "windows" => "linux",
                    _ => "windows",
                }
            ),
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            tools: BTreeMap::from([(String::from("pwsh"), String::from("*"))]),
            ..DetectContract::default()
        };

        let changes = collect_detect_changes(&existing, &detected, &[]);
        assert!(changes.is_empty());
    }

    #[test]
    fn collect_detect_changes_skips_platform_scoped_runtime_outside_current_os() {
        let existing = parse_contract_str(
            Path::new("ota.yaml"),
            &format!(
                r#"
version: 1
project:
  name: ota
runtimes:
  powershell:
    version: "7.6.0"
    only_on:
      - {}
"#,
                match super::current_os() {
                    "windows" => "linux",
                    _ => "windows",
                }
            ),
        )
        .unwrap();

        let detected = DetectContract {
            version: 1,
            project: Some(DetectProject {
                name: String::from("ota"),
            }),
            runtimes: BTreeMap::from([(String::from("powershell"), String::from("*"))]),
            ..DetectContract::default()
        };

        let changes = collect_detect_changes(&existing, &detected, &[]);
        assert!(changes.is_empty());
    }
}
