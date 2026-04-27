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
    let mut note = match (mode.as_str(), receipt.lifecycle.as_deref()) {
        ("container", Some("persistent")) => persistent_container_note_from_receipt(receipt, task)
            .unwrap_or_else(|| String::from("reusing persistent container backend")),
        ("container", Some("ephemeral")) => {
            String::from("using a fresh container image for this run")
        }
        ("native", Some(lifecycle)) => format!(
            "running on the host environment; requested `--lifecycle {lifecycle}` is advisory in native mode only"
        ),
        ("native", _) => String::from("running on the host environment"),
        (other, _) => format!("executing through the `{other}` backend"),
    };
    if let Some(internal_note) = internal_task_note_from_receipt(receipt, task)
        && !note.contains(internal_note.as_str())
    {
        note = format!("{note}; {internal_note}");
    }
    if let Some(requested_note) = requested_task_note_from_receipt(receipt, task)
        && !note.contains(requested_note.as_str())
    {
        note = format!("{note}; {requested_note}");
    }
    if let Some(service_termination) = receipt.service_termination.as_ref() {
        let service_note = service_termination_summary_note(service_termination);
        if !note.contains(service_note.as_str()) {
            note = format!("{note}; {service_note}");
        }
    }
    let log_capture_warning = receipt_log_capture_warning(receipt).map(str::to_string);
    lines.push(summary_detail_line("Scope:", &receipt.scope));
    lines.push(summary_detail_line("Path:", &path_display));
    lines.push(summary_detail_line("Contract:", &contract_display));
    if let Some(workspace) = receipt.workspace.as_deref() {
        lines.push(summary_detail_line("Workspace:", workspace));
    }
    if let Some(lifecycle) = receipt.lifecycle.as_deref() {
        lines.push(summary_detail_line("Lifecycle:", lifecycle));
    }
    lines.push(summary_detail_line("Mode:", &mode));
    if let Some(context) = receipt.context.as_deref() {
        lines.push(summary_detail_line("Context:", context));
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
            &format!(
                "service({}.{}) -> {} ({})",
                resolution.service_ref.task,
                resolution.service_ref.listener,
                resolution.effective_url,
                crate::runner::render_target_resolution_source_and_activation_label(resolution)
            ),
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
            &format!(
                "{} ({}) -> {}",
                backend_fulfillment.backend_unit,
                backend_fulfillment_mode_label(backend_fulfillment.mode),
                backend_fulfillment_result_label(backend_fulfillment.result),
            ),
        ));
        if !backend_fulfillment.missing.is_empty() {
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
    lines.push(summary_detail_line("Task:", task));
    let status = aggregate_execution_summary_status(receipt.ok, &receipt.steps, &receipt.blocked);
    lines.push(summary_detail_line(
        "Status:",
        &render_execution_summary_status_value(&status),
    ));
    lines.push(summary_detail_line("Note:", &note));
    if let Some(log_warning) = log_capture_warning.as_deref() {
        lines.push(summary_detail_line("Warning:", log_warning));
    }
    lines.join("\n")
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

fn receipt_log_capture_warning(receipt: &ExecutionReceipt) -> Option<&str> {
    receipt.next.as_deref().and_then(|next| {
        next.split("; ")
            .find(|part| part.starts_with("log capture failed:"))
    })
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

fn backend_fulfillment_mode_label(mode: crate::runner::BackendFulfillmentMode) -> &'static str {
    match mode {
        crate::runner::BackendFulfillmentMode::None => "none",
        crate::runner::BackendFulfillmentMode::Run => "run",
    }
}

fn backend_fulfillment_result_label(
    result: crate::runner::BackendFulfillmentResult,
) -> &'static str {
    match result {
        crate::runner::BackendFulfillmentResult::RequirementsSatisfied => "requirements_satisfied",
        crate::runner::BackendFulfillmentResult::MissingRequirements => "missing_requirements",
        crate::runner::BackendFulfillmentResult::Fulfilled => "fulfilled",
        crate::runner::BackendFulfillmentResult::Failed => "failed",
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
    if !receipt.ok
        && !receipt
            .service_termination
            .as_ref()
            .is_some_and(|termination| termination.after_readiness)
    {
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
    if !receipt.ok
        && !receipt
            .service_termination
            .as_ref()
            .is_some_and(|termination| termination.after_readiness)
    {
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
