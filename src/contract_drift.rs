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

use crate::detector::detect_repo;
use crate::doctor::{Finding, FindingSeverity};
use crate::output::{DetectComparisonChange, DetectComparisonRemoval};
use crate::schema::Contract;

pub(crate) fn append_contract_drift_findings(
    contract: &Contract,
    contract_path: &Path,
    findings: &mut Vec<Finding>,
) {
    let root = contract_path.parent().unwrap_or(contract_path);
    let Ok(detect_report) = detect_repo(root) else {
        return;
    };

    for change in collect_detect_changes(contract, &detect_report.contract)
        .into_iter()
        .filter(|change| change.status == "update")
    {
        let existing = change.existing.unwrap_or_default();
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: format!("Contract drift: `{}` differs from repo signals", change.field),
            why: format!(
                "`ota.yaml` still declares `{}` = `{}`, but repo inspection under `{}` now detects `{}`",
                change.field,
                existing,
                compact_display_path(root),
                change.detected
            ),
            next: format!(
                "run `ota detect --merge --dry-run {}` to review the comparison, then `ota detect --merge {}` to apply matching updates",
                compact_display_path(root),
                compact_display_path(root)
            ),
        });
    }

    for removal in collect_detect_removals(contract, &detect_report.contract) {
        if removal.field.starts_with("tasks.") {
            continue;
        }
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: format!("Contract drift: `{}` is no longer detected", removal.field),
            why: format!(
                "`ota.yaml` still declares `{}` = `{}`, but repo inspection under `{}` no longer detects it",
                removal.field,
                removal.existing,
                compact_display_path(root)
            ),
            next: format!(
                "run `ota detect --merge --dry-run {}` to review the comparison, then `ota detect --merge {}` to apply matching updates",
                compact_display_path(root),
                compact_display_path(root)
            ),
        });
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

pub(crate) fn collect_detect_changes(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
) -> Vec<DetectComparisonChange> {
    let mut changes = Vec::new();

    if let Some(project) = detected.project.as_ref() {
        push_detect_change(
            &mut changes,
            "project.name",
            Some(existing.project.name.as_str()),
            Some(project.name.as_str()),
        );
    }

    for (name, value) in &detected.runtimes {
        push_detect_change(
            &mut changes,
            &format!("runtimes.{name}"),
            existing
                .runtimes
                .get(name)
                .map(|requirement| requirement.version()),
            Some(value.as_str()),
        );
    }

    for (name, value) in &detected.tools {
        push_detect_change(
            &mut changes,
            &format!("tools.{name}"),
            existing
                .tools
                .get(name)
                .map(|requirement| requirement.version()),
            Some(value.as_str()),
        );
    }

    for (name, service) in &detected.services {
        push_detect_change(
            &mut changes,
            &format!("services.{name}.provider"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.provider.as_deref()),
            service.provider.as_deref(),
        );
        push_detect_change(
            &mut changes,
            &format!("services.{name}.start"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.start.as_deref()),
            service.start.as_deref(),
        );
        push_detect_change(
            &mut changes,
            &format!("services.{name}.stop"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.stop.as_deref()),
            service.stop.as_deref(),
        );
        push_detect_change(
            &mut changes,
            &format!("services.{name}.healthcheck"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.healthcheck.as_deref()),
            service.healthcheck.as_deref(),
        );
    }

    for (name, task) in &detected.tasks {
        let existing_value =
            existing
                .tasks
                .get(name)
                .and_then(|task| match task.default_execution_kind() {
                    Some("run") => task.default_execution_body(),
                    _ => None,
                });
        push_detect_change(
            &mut changes,
            &format!("tasks.{name}.run"),
            existing_value,
            Some(task.run.as_str()),
        );
        if task.safe_for_agent {
            let existing_safe = existing.tasks.get(name).map(|task| task.safe_for_agent);
            push_detect_change(
                &mut changes,
                &format!("tasks.{name}.safe_for_agent"),
                existing_safe.map(|value| if value { "true" } else { "false" }),
                Some("true"),
            );
        }
    }

    changes
}

pub(crate) fn collect_detect_removals(
    existing: &Contract,
    detected: &crate::detector::DetectContract,
) -> Vec<DetectComparisonRemoval> {
    let mut removals = Vec::new();

    if detected.project.is_none() {
        removals.push(DetectComparisonRemoval {
            field: String::from("project.name"),
            existing: existing.project.name.clone(),
        });
    }

    for (name, requirement) in &existing.runtimes {
        if !detected.runtimes.contains_key(name) {
            removals.push(DetectComparisonRemoval {
                field: format!("runtimes.{name}"),
                existing: requirement.version().to_string(),
            });
        }
    }

    for (name, requirement) in &existing.tools {
        if !detected.tools.contains_key(name) {
            removals.push(DetectComparisonRemoval {
                field: format!("tools.{name}"),
                existing: requirement.version().to_string(),
            });
        }
    }

    for (name, service) in &existing.services {
        let detected_service = detected.services.get(name);
        if service.provider.is_some()
            && detected_service
                .and_then(|value| value.provider.as_ref())
                .is_none()
        {
            removals.push(DetectComparisonRemoval {
                field: format!("services.{name}.provider"),
                existing: service.provider.as_deref().unwrap_or_default().to_string(),
            });
        }
        if service.start.is_some()
            && detected_service
                .and_then(|value| value.start.as_ref())
                .is_none()
        {
            removals.push(DetectComparisonRemoval {
                field: format!("services.{name}.start"),
                existing: service.start.as_deref().unwrap_or_default().to_string(),
            });
        }
        if service.stop.is_some()
            && detected_service
                .and_then(|value| value.stop.as_ref())
                .is_none()
        {
            removals.push(DetectComparisonRemoval {
                field: format!("services.{name}.stop"),
                existing: service.stop.as_deref().unwrap_or_default().to_string(),
            });
        }
        if service.healthcheck.is_some()
            && detected_service
                .and_then(|value| value.healthcheck.as_ref())
                .is_none()
        {
            removals.push(DetectComparisonRemoval {
                field: format!("services.{name}.healthcheck"),
                existing: service
                    .healthcheck
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
            });
        }
    }

    for (name, task) in &existing.tasks {
        let detected_task = detected.tasks.get(name);
        let existing_run = task.default_execution_body().map(str::to_string);
        if let Some(existing_run) = existing_run {
            if detected_task.is_none() {
                removals.push(DetectComparisonRemoval {
                    field: format!("tasks.{name}.run"),
                    existing: existing_run,
                });
            }
        }

        if task.safe_for_agent
            && detected_task.is_none_or(|detected_task| !detected_task.safe_for_agent)
        {
            removals.push(DetectComparisonRemoval {
                field: format!("tasks.{name}.safe_for_agent"),
                existing: String::from("true"),
            });
        }
    }

    removals
}

fn push_detect_change(
    changes: &mut Vec<DetectComparisonChange>,
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
        }),
        Some(existing) if existing != detected => changes.push(DetectComparisonChange {
            field: field.to_string(),
            status: "update",
            existing: Some(existing.to_string()),
            detected: detected.to_string(),
        }),
        Some(_) => {}
    }
}
