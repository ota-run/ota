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

use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml::{Mapping, Value as YamlValue};

use crate::detector::{DetectService, Inference, detect_repo};
use crate::doctor::{Finding, FindingSeverity};
use crate::output::{DetectComparisonChange, DetectComparisonRemoval};
use crate::schema::{Backend, Contract, ServiceSpec, ToolchainFulfillmentMode, ToolchainProvider};

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
            identity: None,
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
            identity: None,
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
        let provider = toolchain_provider_name(toolchain.provider);
        push_detect_change(
            &mut changes,
            existing,
            &inference_index,
            &format!("toolchains.{name}.provider"),
            existing_toolchain.and_then(|value| value.provider.map(toolchain_provider_name)),
            Some(provider),
        );
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
        if detected_toolchain.is_none() {
            if let Some(provider) = toolchain.provider {
                push_detect_removal(
                    &mut removals,
                    existing,
                    format!("toolchains.{name}.provider"),
                    toolchain_provider_name(provider).to_string(),
                );
            }
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
                        name: Some(String::from("ota")),
                        file: Some(String::from("docker-compose.yml")),
                        service: Some(String::from("web")),
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
                String::from("toolchains.node.provider"),
                String::from("corepack"),
                String::from("ota.detect#toolchains.node.provider"),
                Confidence::High,
            ),
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
                "toolchains.node.provider",
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
            changes[2].source.as_deref(),
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
        toolchains.node.provider: merged
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
                "toolchains.node.provider",
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
