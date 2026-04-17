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

use serde_yaml::{Mapping, Value as YamlValue};

use crate::detector::{Inference, detect_repo};
use crate::doctor::{Finding, FindingSeverity};
use crate::output::{DetectComparisonChange, DetectComparisonRemoval};
use crate::schema::Contract;

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

    for change in
        collect_detect_changes(contract, &detect_report.contract, &detect_report.inferences)
            .into_iter()
            .filter(|change| {
                change.status == "update"
                    && change.owner_kind.as_deref() == Some(DETECT_OWNER_KIND_MERGED)
            })
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

    for removal in collect_detect_drift_removals(contract, &detect_report.contract) {
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

    for (name, service) in &detected.services {
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("services.{name}.provider"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.provider.as_deref()),
            service.provider.as_deref(),
        );
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("services.{name}.start"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.start.as_deref()),
            service.start.as_deref(),
        );
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("services.{name}.stop"),
            existing
                .services
                .get(name)
                .and_then(|existing_service| existing_service.stop.as_deref()),
            service.stop.as_deref(),
        );
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
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
            existing,
            &inference_index,
            &format!("tasks.{name}.run"),
            existing_value,
            Some(task.run.as_str()),
        );
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

    for (name, service) in &existing.services {
        let detected_service = detected.services.get(name);
        if service.provider.is_some()
            && detected_service
                .and_then(|value| value.provider.as_ref())
                .is_none()
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("services.{name}.provider"),
                service.provider.as_deref().unwrap_or_default().to_string(),
            );
        }
        if service.start.is_some()
            && detected_service
                .and_then(|value| value.start.as_ref())
                .is_none()
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("services.{name}.start"),
                service.start.as_deref().unwrap_or_default().to_string(),
            );
        }
        if service.stop.is_some()
            && detected_service
                .and_then(|value| value.stop.as_ref())
                .is_none()
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("services.{name}.stop"),
                service.stop.as_deref().unwrap_or_default().to_string(),
            );
        }
        if service.healthcheck.is_some()
            && detected_service
                .and_then(|value| value.healthcheck.as_ref())
                .is_none()
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("services.{name}.healthcheck"),
                service
                    .healthcheck
                    .as_deref()
                    .unwrap_or_default()
                    .to_string(),
            );
        }
    }

    for (name, task) in &existing.tasks {
        let detected_task = detected.tasks.get(name);
        if let Some(existing_run) = match task.default_execution_kind() {
            Some("run") => task.default_execution_body(),
            _ => None,
        } && detected_task.is_none()
        {
            push_detect_removal(
                &mut removals,
                existing,
                format!("tasks.{name}.run"),
                existing_run.to_string(),
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
    let inference = inference_index.get(field);

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
            confidence: inference.map(|value| value.confidence),
        }),
        Some(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::{DetectContract, DetectProject, DetectService, DetectTask};
    use crate::parser::parse_contract_str;
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
                provider: None,
                start: None,
                stop: None,
                healthcheck: None,
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
