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
use crate::runner::TaskTargetResolutionSource;
pub(super) fn render_execution_receipt_summary_block(
    receipt: &ExecutionReceipt,
    task: Option<&str>,
    title: &str,
) -> String {
    let title = if plain_mode() {
        title.to_string()
    } else if title.starts_with("WORKSPACE ") {
        paint(&format!("🦦 {title}"), "1;37")
    } else {
        paint(&format!("🦦 {title}"), "1")
    };
    let mut lines = vec![String::new(), title, String::new()];
    let mode = receipt
        .backend
        .as_deref()
        .unwrap_or("native")
        .trim()
        .to_string();
    let task = task.unwrap_or_else(|| {
        receipt
            .steps
            .first()
            .map(|step| step.label.as_str())
            .unwrap_or("setup")
    });
    let backend_summary = backend_summary_from_receipt(receipt, task, mode.as_str());
    let note =
        build_execution_summary_note(receipt, task, backend_summary.as_deref(), mode.as_str());
    let log_capture_warning = receipt_log_capture_warning(receipt).map(str::to_string);
    let status = aggregate_execution_summary_status(receipt.ok, &receipt.steps, &receipt.blocked);

    let path_display = if receipt.scope == "repo" {
        Path::new(receipt.path.as_str())
            .parent()
            .map(|parent| compact_path(parent, "."))
            .unwrap_or_else(|| compact_path(Path::new(receipt.path.as_str()), "."))
    } else if receipt.scope == "workspace" {
        receipt.path.clone()
    } else {
        compact_path(Path::new(receipt.path.as_str()), ".")
    };
    let contract_display = compact_path(Path::new(receipt.contract.as_str()), ".");

    lines.push(summary_detail_line(
        "Status:",
        &render_execution_summary_status_value(&status),
    ));
    lines.push(summary_detail_line("Scope:", &receipt.scope));
    lines.push(summary_detail_line("Path:", &path_display));
    lines.push(summary_detail_line("Contract:", &contract_display));
    lines.push(summary_detail_line("Mode:", &mode));
    lines.push(summary_detail_line("Task:", task));
    if let Some(workspace) = receipt.workspace.as_deref() {
        lines.push(summary_detail_line("Workspace:", workspace));
    }
    if let Some(context) = receipt.context.as_deref() {
        lines.push(summary_detail_line("Context:", context));
    }
    if let Some(lifecycle) = receipt.lifecycle.as_deref() {
        lines.push(summary_detail_line("Lifecycle:", lifecycle));
    }
    if let Some(image) = receipt.image.as_deref() {
        lines.push(summary_detail_line("Image:", image));
    }
    if let Some(memory_bytes) = receipt.container_memory_bytes {
        let memory_display = format_memory_size_bytes(memory_bytes);
        lines.push(summary_detail_line("Memory:", memory_display.as_str()));
    }
    if let Some(target) = receipt.target.as_deref() {
        if receipt.backend.as_deref() == Some("container") {
            lines.push(summary_detail_line("Container:", target));
        } else if !matches!(
            (receipt.backend.as_deref(), receipt.lifecycle.as_deref()),
            (Some("container"), Some("ephemeral"))
        ) {
            lines.push(summary_detail_line("Target:", target));
        }
    }
    if let Some(provider) = receipt.provider.as_deref() {
        lines.push(summary_detail_line("Provider:", provider));
    }
    if let Some(cwd) = receipt.cwd.as_deref() {
        lines.push(summary_detail_line("Cwd:", cwd));
    }
    if let Some(endpoint) = primary_receipt_endpoint(receipt) {
        lines.push(summary_detail_line("External:", &endpoint));
        if let Some(internal) = primary_receipt_internal_endpoint(receipt)
            && internal != endpoint
        {
            lines.push(summary_detail_line("Internal:", &internal));
        }
        let secondary_count = secondary_receipt_endpoint_count(receipt);
        if secondary_count > 0 {
            lines.push(summary_detail_line(
                "Secondary:",
                &format!("{secondary_count} additional endpoint(s)"),
            ));
        }
    }
    for resolution in requested_task_target_resolutions(receipt, task) {
        lines.push(summary_detail_line(
            &format!("Target {}:", resolution.target),
            &human_target_resolution_summary(resolution),
        ));
        lines.push(summary_detail_line(
            target_resolution_value_label(resolution),
            &resolution.effective_url,
        ));
    }
    if let Some(shared_backend) = requested_task_shared_local_backend(receipt, task) {
        let mut shared_value = format!(
            "{} -> {} ({})",
            shared_backend.name, shared_backend.effective_identity, shared_backend.lifecycle
        );
        if let Some(reuse) = shared_backend.reuse {
            let reuse_label = match reuse {
                crate::runner::SharedLocalBackendReuse::Created => "created",
                crate::runner::SharedLocalBackendReuse::Reused => "reused",
                crate::runner::SharedLocalBackendReuse::Recreated => "recreated",
            };
            shared_value.push_str(format!(", {reuse_label}").as_str());
        }
        lines.push(summary_detail_line("Shared:", &shared_value));
        if let Some(environment) = shared_backend.environment.as_ref() {
            let mut declared = Vec::new();
            if let Some(profile) = environment.declared_profile.as_deref() {
                declared.push(format!("profile={profile}"));
            }
            if let Some(alias) = environment.declared_image_alias.as_deref() {
                declared.push(format!("alias={alias}"));
            }
            if let Some(image) = environment.declared_image.as_deref() {
                declared.push(format!("image={image}"));
            }
            if let Some(source) = environment.declared_source.as_deref() {
                declared.push(format!("source={source}"));
            }
            let declared_value = if declared.is_empty() {
                String::from("implicit")
            } else {
                declared.join(", ")
            };
            lines.push(summary_detail_line("Env intent:", &declared_value));

            let mut effective = vec![format!("image={}", environment.effective_image)];
            if let Some(profile) = environment.effective_profile.as_deref() {
                effective.push(format!("profile={profile}"));
            }
            if let Some(alias) = environment.effective_image_alias.as_deref() {
                effective.push(format!("alias={alias}"));
            }
            if let Some(source) = environment.effective_source.as_deref() {
                effective.push(format!("source={source}"));
            }
            if let Some(registry) = environment.effective_registry.as_deref() {
                effective.push(format!("registry={registry}"));
            }
            if let Some(policy) = environment.policy.as_deref() {
                effective.push(format!("policy={policy}"));
            }
            lines.push(summary_detail_line("Env effective:", &effective.join(", ")));
        }
    }
    if let Some(backend_fulfillment) = requested_task_backend_fulfillment(receipt, task) {
        lines.push(summary_detail_line(
            "Fulfillment:",
            &human_backend_fulfillment_summary(backend_fulfillment),
        ));
        if matches!(
            backend_fulfillment.result,
            crate::runner::BackendFulfillmentResult::MissingRequirements
                | crate::runner::BackendFulfillmentResult::Failed
        ) && !backend_fulfillment.missing.is_empty()
        {
            lines.push(summary_detail_line(
                "Missing:",
                &backend_fulfillment.missing.join("; "),
            ));
        }
        if !backend_fulfillment.actions.is_empty() {
            lines.push(summary_detail_line(
                "Actions:",
                &backend_fulfillment.actions.join("; "),
            ));
        }
    }
    if let Some(logs) = receipt.logs.as_ref() {
        lines.push(summary_detail_line("Logs:", &logs.dir));
    }
    if let Some(note_value) = note.as_deref() {
        lines.push(summary_detail_line("Note:", note_value));
    }
    if let Some(log_warning) = log_capture_warning.as_deref() {
        lines.push(summary_detail_line("Warning:", log_warning));
    }
    lines.join("\n")
}

fn backend_summary_from_receipt(
    receipt: &ExecutionReceipt,
    task: &str,
    mode: &str,
) -> Option<String> {
    match (mode, receipt.lifecycle.as_deref()) {
        ("container", Some("persistent")) => Some(
            persistent_container_note_from_receipt(receipt, task)
                .unwrap_or_else(|| String::from("persistent container reused")),
        ),
        ("container", Some("ephemeral")) => {
            Some(String::from("fresh container image for this run"))
        }
        ("native", _) => Some(String::from("host environment")),
        (other, _) => Some(format!("executing through the `{other}` backend")),
    }
}

fn build_execution_summary_note(
    receipt: &ExecutionReceipt,
    task: &str,
    backend_summary: Option<&str>,
    mode: &str,
) -> Option<String> {
    let mut parts = Vec::new();

    // Add base backend context note based on mode and lifecycle
    let base_note = match (mode, receipt.lifecycle.as_deref()) {
        ("container", Some("persistent")) => Some(
            persistent_container_note_from_receipt(receipt, task)
                .unwrap_or_else(|| String::from("reusing persistent container backend")),
        ),
        ("container", Some("ephemeral")) => {
            Some(String::from("using a fresh container image for this run"))
        }
        ("native", Some(lifecycle)) => Some(format!(
            "running on the host environment; requested `--lifecycle {lifecycle}` is advisory in native mode only"
        )),
        ("native", _) => Some(String::from("running on the host environment")),
        (other, _) => Some(format!("executing through the `{other}` backend")),
    };

    if let Some(note) = base_note {
        parts.push(note);
    }

    if let Some(internal_note) = internal_task_note_from_receipt(receipt, task) {
        push_unique_summary_note_part(&mut parts, internal_note);
    }
    if let Some(requested_note) = requested_task_note_from_receipt(receipt, task) {
        for part in requested_note.split("; ") {
            let trimmed = part.trim();
            if trimmed.is_empty()
                || trimmed == "requested task"
                || backend_summary.is_some_and(|backend| trimmed == backend)
                || trimmed.starts_with("target `")
                || trimmed.starts_with("backend `")
                || trimmed.starts_with("activation ")
            {
                continue;
            }
            push_unique_summary_note_part(&mut parts, trimmed.to_string());
        }
    }
    if let Some(failed_dependency_note) = failed_dependency_note_from_receipt(receipt, task) {
        push_unique_summary_note_part(&mut parts, failed_dependency_note);
    }
    if let Some(service_termination) = receipt.service_termination.as_ref() {
        push_unique_summary_note_part(
            &mut parts,
            service_termination_summary_note(service_termination),
        );
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn persistent_container_note_from_receipt(
    receipt: &ExecutionReceipt,
    task: &str,
) -> Option<String> {
    receipt.steps.iter().find_map(|step| {
        if step.label != task {
            return None;
        }
        let detail = step.detail.as_deref()?;
        let note = detail.strip_prefix("requested task; ").unwrap_or(detail);
        note.contains("persistent container")
            .then(|| note.to_string())
    })
}

fn internal_task_note_from_receipt(receipt: &ExecutionReceipt, task: &str) -> Option<String> {
    receipt.steps.iter().find_map(|step| {
        if step.label != task {
            return None;
        }
        let detail = step.detail.as_deref()?;
        let note = detail.strip_prefix("requested task; ").unwrap_or(detail);
        note.split("; ")
            .find(|part| part.contains("marked internal"))
            .map(str::to_string)
    })
}

fn requested_task_note_from_receipt(receipt: &ExecutionReceipt, task: &str) -> Option<String> {
    receipt.steps.iter().find_map(|step| {
        if step.label != task {
            return None;
        }
        let detail = step.detail.as_deref()?;
        let note = detail.strip_prefix("requested task; ").unwrap_or(detail);
        (!note.is_empty()).then(|| note.to_string())
    })
}

fn failed_dependency_note_from_receipt(receipt: &ExecutionReceipt, task: &str) -> Option<String> {
    if receipt.ok {
        return None;
    }

    receipt.steps.iter().find_map(|step| {
        if step.exit_code == Some(0) {
            return None;
        }
        match step.detail.as_deref() {
            Some(detail) if detail.starts_with(&format!("depends_on for `{task}`")) => {
                Some(format!(
                    "depends_on task `{}` failed for requested task `{task}`",
                    step.label
                ))
            }
            _ => None,
        }
    })
}

fn receipt_log_capture_warning(receipt: &ExecutionReceipt) -> Option<&str> {
    receipt.next.as_deref().and_then(|next| {
        next.split("; ")
            .find(|part| part.starts_with("log capture failed:"))
    })
}

pub(super) fn execution_receipt_next_steps(receipt: &ExecutionReceipt) -> Vec<String> {
    receipt
        .next
        .as_deref()
        .map(|next| {
            next.split("; ")
                .map(str::trim)
                .filter(|part| !part.is_empty() && !part.starts_with("log capture failed:"))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(super) fn summary_detail_line(label: &str, value: &str) -> String {
    const SUMMARY_LABEL_WIDTH: usize = 12;
    format!("{label:<width$} {value}", width = SUMMARY_LABEL_WIDTH)
}

pub(super) fn summary_has_status(summary_block: &str, expected_status: &str) -> bool {
    summary_block.lines().any(|line| {
        line.strip_prefix("Status:")
            .is_some_and(|status| status.trim().eq_ignore_ascii_case(expected_status.trim()))
    })
}

fn requested_task_target_resolutions<'a>(
    receipt: &'a ExecutionReceipt,
    requested_task: &str,
) -> &'a [TaskTargetResolutionEvidence] {
    receipt
        .steps
        .iter()
        .find(|step| {
            step.label == requested_task
                && matches!(
                    step.detail.as_deref(),
                    Some(detail) if detail.starts_with("requested task")
                )
        })
        .map(|step| step.target_resolutions.as_slice())
        .unwrap_or(&[])
}

fn requested_task_shared_local_backend<'a>(
    receipt: &'a ExecutionReceipt,
    requested_task: &str,
) -> Option<&'a SharedLocalBackendEvidence> {
    receipt
        .steps
        .iter()
        .find(|step| {
            step.label == requested_task
                && matches!(
                    step.detail.as_deref(),
                    Some(detail) if detail.starts_with("requested task")
                )
        })
        .and_then(|step| step.shared_local_backend.as_ref())
}

fn requested_task_backend_fulfillment<'a>(
    receipt: &'a ExecutionReceipt,
    requested_task: &str,
) -> Option<&'a crate::runner::BackendFulfillmentEvidence> {
    receipt
        .steps
        .iter()
        .find(|step| {
            step.label == requested_task
                && matches!(
                    step.detail.as_deref(),
                    Some(detail) if detail.starts_with("requested task")
                )
        })
        .and_then(|step| step.backend_fulfillment.as_ref())
        .or(receipt.backend_fulfillment.as_ref())
}

fn human_target_resolution_summary(resolution: &TaskTargetResolutionEvidence) -> String {
    let service_task = resolution
        .service_ref
        .as_ref()
        .map(|service_ref| match service_ref.member.as_deref() {
            Some(member) => format!("{member}:{}", service_ref.task),
            None => service_ref.task.clone(),
        })
        .unwrap_or_else(|| String::from("producer"));
    let service_edge =
        resolution
            .service_ref
            .as_ref()
            .map(|service_ref| match service_ref.member.as_deref() {
                Some(member) => format!("{member}:{}.{}", service_ref.task, service_ref.listener),
                None => format!("{}.{}", service_ref.task, service_ref.listener),
            });
    match resolution
        .activation
        .as_ref()
        .map(|activation| activation.status)
    {
        Some(crate::runner::TaskTargetActivationStatus::StartedStarted) => format!(
            "started producer `{}` without waiting for reachability or readiness",
            service_task.as_str()
        ),
        Some(crate::runner::TaskTargetActivationStatus::RestartedReady) => format!(
            "restarted producer `{}` and waited for readiness",
            service_task.as_str()
        ),
        Some(crate::runner::TaskTargetActivationStatus::ReusedStarted) => {
            format!(
                "reused already-started producer `{}`",
                service_task.as_str()
            )
        }
        Some(crate::runner::TaskTargetActivationStatus::StartedReady) => format!(
            "started producer `{}` and waited for readiness",
            service_task.as_str()
        ),
        Some(crate::runner::TaskTargetActivationStatus::ReusedReady) => {
            format!("reused ready producer `{}`", service_task.as_str())
        }
        Some(crate::runner::TaskTargetActivationStatus::StartedRunning) => format!(
            "started producer `{}` and waited for the declared listener",
            service_task.as_str()
        ),
        Some(crate::runner::TaskTargetActivationStatus::ReusedRunning) => {
            format!(
                "reused producer `{}` because the declared listener was already reachable",
                service_task.as_str()
            )
        }
        Some(crate::runner::TaskTargetActivationStatus::SkippedExplicitOverride) => {
            String::from("skipped activation because an explicit override was provided")
        }
        _ => match resolution.source {
            TaskTargetResolutionSource::ExplicitOverride => String::from("used explicit override"),
            TaskTargetResolutionSource::TargetBinding => {
                if let Some(service_edge) = service_edge {
                    format!("resolved from producer `{service_edge}`")
                } else if let Some(url_ref) = resolution.url_ref.as_ref() {
                    format!("resolved from declared url `{}`", url_ref.url)
                } else {
                    String::from("resolved from declared target")
                }
            }
            TaskTargetResolutionSource::CompatibilityLiteralDefault => {
                String::from("used compatibility literal default")
            }
        },
    }
}

fn target_resolution_value_label(resolution: &TaskTargetResolutionEvidence) -> &'static str {
    match resolution.override_input.as_deref() {
        Some("base_url") => "Base URL:",
        _ => "Resolved:",
    }
}

fn human_backend_fulfillment_summary(
    backend_fulfillment: &crate::runner::BackendFulfillmentEvidence,
) -> String {
    match backend_fulfillment.result {
        crate::runner::BackendFulfillmentResult::RequirementsSatisfied => format!(
            "requirements already satisfied for `{}`",
            backend_fulfillment.backend_unit
        ),
        crate::runner::BackendFulfillmentResult::MissingRequirements => format!(
            "missing requirements for `{}`",
            backend_fulfillment.backend_unit
        ),
        crate::runner::BackendFulfillmentResult::Fulfilled => {
            format!("prepared `{}`", backend_fulfillment.backend_unit)
        }
        crate::runner::BackendFulfillmentResult::Failed => format!(
            "failed while preparing `{}`",
            backend_fulfillment.backend_unit
        ),
    }
}

fn push_unique_summary_note_part(parts: &mut Vec<String>, part: String) {
    if !parts.iter().any(|existing| existing == &part) {
        parts.push(part);
    }
}

pub(super) fn render_execution_receipt_status(status: &str) -> String {
    match status.trim() {
        "READY" => paint("READY", "1;38;2;0;255;120"),
        "INTERRUPTED" => paint("INTERRUPTED", "1;38;2;0;255;255"),
        "NOT READY" | "BLOCKED" | "WARN" => paint(status.trim(), "1;38;2;255;235;59"),
        value if value.contains("FAILED") => render_failed_status_label(value),
        other => paint(other, "1;37"),
    }
}

pub(super) fn append_runtime_listener_lines(
    stdout: &mut String,
    runtime: &crate::runner::ResolvedTaskRuntime,
    indent: &str,
) {
    let endpoint_index = runtime
        .exposed_endpoints
        .iter()
        .map(|endpoint| (endpoint.listener.as_str(), endpoint))
        .collect::<BTreeMap<_, _>>();
    for (listener_name, listener) in &runtime.listeners {
        let role = if runtime.primary_listener.as_deref() == Some(listener_name.as_str()) {
            Some("primary")
        } else if endpoint_index.contains_key(listener_name.as_str()) {
            Some("secondary")
        } else {
            None
        };
        let listener_label = role
            .map(|role| format!("{listener_name} ({role})"))
            .unwrap_or_else(|| listener_name.to_string());
        stdout.push_str(&format!(
            "\n{indent}{} {}:{}",
            paint_key(&listener_label),
            listener.bind.address,
            listener.bind.port
        ));
        if let Some(endpoint) = endpoint_index.get(listener_name.as_str()) {
            stdout.push_str(&format!(
                "\n{indent}  {} {}",
                paint_key("External:"),
                runtime_host_endpoint_text(&endpoint.host)
            ));
            let internal = runtime_internal_endpoint_text(endpoint);
            let external = runtime_host_endpoint_text(&endpoint.host);
            if internal != external {
                stdout.push_str(&format!(
                    "\n{indent}  {} {}",
                    paint_key("Internal:"),
                    internal
                ));
            }
            continue;
        }
        if let Some(host) = listener
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.host.as_ref())
        {
            stdout.push_str(&format!(
                "\n{indent}  {} {}",
                paint_key("External:"),
                runtime_host_endpoint_text(host)
            ));
        }
    }
}

fn runtime_host_endpoint_text(host: &crate::runner::ResolvedTaskRuntimeHost) -> String {
    host.url
        .clone()
        .unwrap_or_else(|| format!("{}:{}", host.address, host.port))
}

pub(super) fn primary_runtime_endpoint(
    runtime: &crate::runner::ResolvedTaskRuntime,
) -> Option<String> {
    runtime
        .primary_endpoint
        .as_ref()
        .map(|endpoint| runtime_host_endpoint_text(&endpoint.host))
        .or_else(|| {
            runtime
                .exposed_endpoints
                .first()
                .map(|endpoint| runtime_host_endpoint_text(&endpoint.host))
        })
}

fn runtime_internal_endpoint_text(endpoint: &crate::runner::ResolvedTaskRuntimeEndpoint) -> String {
    crate::runner::resolved_runtime_internal_endpoint_text(endpoint)
}

fn secondary_runtime_endpoint_count(runtime: &crate::runner::ResolvedTaskRuntime) -> usize {
    runtime.exposed_endpoints.len().saturating_sub(1)
}

fn primary_runtime_internal_endpoint(
    runtime: &crate::runner::ResolvedTaskRuntime,
) -> Option<String> {
    runtime
        .primary_endpoint
        .as_ref()
        .map(runtime_internal_endpoint_text)
        .or_else(|| {
            runtime
                .exposed_endpoints
                .first()
                .map(runtime_internal_endpoint_text)
        })
}

fn primary_receipt_endpoint(receipt: &ExecutionReceipt) -> Option<String> {
    if !receipt_allows_endpoint_summary(receipt) {
        return None;
    }

    receipt
        .runtime
        .as_ref()
        .and_then(primary_runtime_endpoint)
        .or_else(|| {
            receipt
                .workloads
                .values()
                .find_map(primary_runtime_endpoint)
        })
}

fn primary_receipt_internal_endpoint(receipt: &ExecutionReceipt) -> Option<String> {
    if !receipt_allows_endpoint_summary(receipt) {
        return None;
    }

    receipt
        .runtime
        .as_ref()
        .and_then(primary_runtime_internal_endpoint)
        .or_else(|| {
            receipt
                .workloads
                .values()
                .find_map(primary_runtime_internal_endpoint)
        })
}

fn receipt_allows_endpoint_summary(receipt: &ExecutionReceipt) -> bool {
    if receipt.ok {
        return true;
    }

    matches!(
        receipt
            .service_termination
            .as_ref()
            .map(|termination| (termination.after_readiness, &termination.cause)),
        Some((true, _)) | Some((false, crate::runner::ServiceTerminationCause::Interrupted))
    )
}

fn secondary_receipt_endpoint_count(receipt: &ExecutionReceipt) -> usize {
    if let Some(runtime) = receipt
        .runtime
        .as_ref()
        .filter(|runtime| primary_runtime_endpoint(runtime).is_some())
    {
        return secondary_runtime_endpoint_count(runtime);
    }

    receipt
        .workloads
        .values()
        .find(|runtime| primary_runtime_endpoint(runtime).is_some())
        .map(secondary_runtime_endpoint_count)
        .unwrap_or(0)
}

pub(super) fn render_execution_summary_status_value(status: &str) -> String {
    match status.trim() {
        "success" => paint("success", "1;38;2;0;255;120"),
        "interrupted" => paint("interrupted", "1;38;2;0;255;255"),
        "blocked" => paint("blocked", "1;38;2;255;235;59"),
        "skipped" => paint("skipped", "1;38;2;180;180;180"),
        "preview" => paint("preview", "1;38;2;0;255;255"),
        "failed" => paint("failed", "1;31"),
        other => paint(other, "1;37"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{ExecutionReceipt, ExecutionReceiptSummary};
    use crate::runner::{
        ResolvedTaskRuntime, ResolvedTaskRuntimeBind, ResolvedTaskRuntimeEndpoint,
        ResolvedTaskRuntimeHost, ServiceTermination, ServiceTerminationCause,
        ServiceTerminationKind,
    };

    fn sample_service_runtime(external_port: u16) -> ResolvedTaskRuntime {
        ResolvedTaskRuntime {
            kind: crate::schema::TaskRuntimeKind::Service,
            listeners: BTreeMap::new(),
            primary_listener: Some(String::from("http")),
            primary_endpoint: Some(ResolvedTaskRuntimeEndpoint {
                listener: String::from("http"),
                protocol: crate::schema::TaskRuntimeProtocol::Http,
                bind: ResolvedTaskRuntimeBind {
                    address: String::from("0.0.0.0"),
                    port: 3000,
                },
                host: ResolvedTaskRuntimeHost {
                    address: String::from("127.0.0.1"),
                    port: external_port,
                    url: Some(format!("http://127.0.0.1:{external_port}/")),
                },
                primary: true,
            }),
            exposed_endpoints: Vec::new(),
        }
    }

    fn sample_receipt() -> ExecutionReceipt {
        ExecutionReceipt {
            ok: false,
            path: String::from("./ota.yaml"),
            scope: String::from("repo"),
            contract: String::from("./ota.yaml"),
            contract_identity: None,
            workspace: None,
            backend: Some(String::from("container")),
            context: Some(String::from("development")),
            lifecycle: Some(String::from("ephemeral")),
            image: Some(String::from("node:24-bookworm")),
            container_memory_bytes: None,
            target: Some(String::from("ota-ephemeral-deadbeef")),
            provider: None,
            cwd: None,
            acquired: Vec::new(),
            env: BTreeMap::new(),
            env_sources: Vec::new(),
            workflow_env_artifacts: Vec::new(),
            native_prerequisites: Vec::new(),
            toolchains: Vec::new(),
            runtime: None,
            logs: None,
            service_termination: None,
            backend_fulfillment: None,
            workloads: BTreeMap::new(),
            policy: Vec::new(),
            steps: Vec::new(),
            blocked: Vec::new(),
            status: None,
            failed_task: None,
            failed_dependency: None,
            failure_origin: None,
            summary: ExecutionReceiptSummary::default(),
            next: None,
        }
    }

    #[test]
    fn run_summary_never_includes_next_lines() {
        let mut receipt = sample_receipt();
        receipt.ok = true;
        receipt.next = Some(String::from(
            "inspect task `dev` output and rerun `ota run dev`; log capture failed: permission denied",
        ));

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            Some("dev"),
            "RUN SUMMARY",
        ));

        assert!(!rendered.contains("\nNext:"), "{rendered}");
        assert!(rendered.contains("Warning:"), "{rendered}");
    }

    #[test]
    fn interrupted_pre_confirmation_summary_keeps_projected_endpoints() {
        let mut receipt = sample_receipt();
        receipt.runtime = Some(sample_service_runtime(3001));
        receipt.service_termination = Some(ServiceTermination {
            kind: ServiceTerminationKind::ServiceStopped,
            cause: ServiceTerminationCause::Interrupted,
            after_readiness: false,
            target: String::from("container"),
            container: String::from("ota-ephemeral-deadbeef"),
            exit_code: Some(130),
            readiness: None,
        });

        let rendered = strip_ansi_codes(&render_execution_receipt_summary_block(
            &receipt,
            Some("dev"),
            "RUN SUMMARY",
        ));

        assert!(
            rendered.contains("External:    http://127.0.0.1:3001/"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Internal:    http://0.0.0.0:3000/"),
            "{rendered}"
        );
    }

    #[test]
    fn execution_receipt_next_steps_filters_log_capture_warning() {
        let mut receipt = sample_receipt();
        receipt.next = Some(String::from(
            "repair task `dev` and rerun `ota run dev`; run `ota tasks --use`; log capture failed: permission denied",
        ));

        let next_steps = execution_receipt_next_steps(&receipt);

        assert_eq!(
            next_steps,
            vec![
                String::from("repair task `dev` and rerun `ota run dev`"),
                String::from("run `ota tasks --use`"),
            ]
        );
    }
}
