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

use crate::schema::{
    Backend, TaskAdapterInputsSpec, TaskBakeAdapterInputsSpec, TaskCommandSpec,
    TaskComposeAdapterInputsSpec, TaskLaunchSpec, TaskModeBranchSpec, TaskSpec, WorkflowEnvSpec,
    WorkflowSpec,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkflowAdapterOverlay {
    adapter_inputs: TaskAdapterInputsSpec,
}

impl WorkflowAdapterOverlay {
    pub(crate) fn from_workflow(workflow: &WorkflowSpec) -> Self {
        let mut adapter_inputs = workflow.adapter_inputs.clone();
        let Some(env) = workflow.env.as_ref() else {
            return Self { adapter_inputs };
        };
        merge_legacy_workflow_adapter_inputs(&mut adapter_inputs, &env.adapter_inputs);
        if !env.compose_files.is_empty() {
            let compose = adapter_inputs
                .compose
                .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
            if compose.files.is_empty() {
                compose.files = env.compose_files.clone();
            }
        }
        if let Some(project_name) = env
            .compose_project_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let compose = adapter_inputs
                .compose
                .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
            if compose.project_name.is_none() {
                compose.project_name = Some(project_name.to_string());
            }
        }
        Self { adapter_inputs }
    }

    pub(crate) fn as_task_adapter_inputs(&self) -> &TaskAdapterInputsSpec {
        &self.adapter_inputs
    }

    pub(crate) fn into_task_adapter_inputs(self) -> TaskAdapterInputsSpec {
        self.adapter_inputs
    }

    pub(crate) fn workflow_value(&self, field: AdapterInputField) -> Option<String> {
        field.workflow_value(&self.adapter_inputs)
    }
}

trait WorkflowOverlaySpec: Clone {
    fn workflow_slot(adapter_inputs: &TaskAdapterInputsSpec) -> Option<&Self>;
    fn task_slot(task: &TaskSpec) -> Option<&Self>;
    fn task_slot_mut(task: &mut TaskSpec) -> &mut Option<Self>;
    fn branch_slot(branch: &TaskModeBranchSpec) -> Option<&Self>;
    fn branch_slot_mut(branch: &mut TaskModeBranchSpec) -> &mut Option<Self>;
    fn workflow_overlay_bound(&self) -> bool;
    fn set_workflow_overlay_bound(&mut self, bound: bool);
    fn merge_workflow_overlay(&mut self, overlay: &Self);
}

impl WorkflowOverlaySpec for TaskComposeAdapterInputsSpec {
    fn workflow_slot(adapter_inputs: &TaskAdapterInputsSpec) -> Option<&Self> {
        adapter_inputs.compose.as_ref()
    }

    fn task_slot(task: &TaskSpec) -> Option<&Self> {
        task.adapter_inputs.compose.as_ref()
    }

    fn task_slot_mut(task: &mut TaskSpec) -> &mut Option<Self> {
        &mut task.adapter_inputs.compose
    }

    fn branch_slot(branch: &TaskModeBranchSpec) -> Option<&Self> {
        branch.adapter_inputs.compose.as_ref()
    }

    fn branch_slot_mut(branch: &mut TaskModeBranchSpec) -> &mut Option<Self> {
        &mut branch.adapter_inputs.compose
    }

    fn workflow_overlay_bound(&self) -> bool {
        self.workflow_overlay_bound
    }

    fn set_workflow_overlay_bound(&mut self, bound: bool) {
        self.workflow_overlay_bound = bound;
    }

    fn merge_workflow_overlay(&mut self, overlay: &Self) {
        if self.cwd.is_none() {
            self.cwd = overlay.cwd.clone();
        }
        prepend_unique_strings(&mut self.env_files, &overlay.env_files);
        prepend_unique_strings(&mut self.files, &overlay.files);
        prepend_unique_strings(&mut self.profiles, &overlay.profiles);
        if self.project_name.is_none() {
            self.project_name = overlay.project_name.clone();
        }
    }
}

impl WorkflowOverlaySpec for TaskBakeAdapterInputsSpec {
    fn workflow_slot(adapter_inputs: &TaskAdapterInputsSpec) -> Option<&Self> {
        adapter_inputs.bake.as_ref()
    }

    fn task_slot(task: &TaskSpec) -> Option<&Self> {
        task.adapter_inputs.bake.as_ref()
    }

    fn task_slot_mut(task: &mut TaskSpec) -> &mut Option<Self> {
        &mut task.adapter_inputs.bake
    }

    fn branch_slot(branch: &TaskModeBranchSpec) -> Option<&Self> {
        branch.adapter_inputs.bake.as_ref()
    }

    fn branch_slot_mut(branch: &mut TaskModeBranchSpec) -> &mut Option<Self> {
        &mut branch.adapter_inputs.bake
    }

    fn workflow_overlay_bound(&self) -> bool {
        self.workflow_overlay_bound
    }

    fn set_workflow_overlay_bound(&mut self, bound: bool) {
        self.workflow_overlay_bound = bound;
    }

    fn merge_workflow_overlay(&mut self, overlay: &Self) {
        if self.cwd.is_none() {
            self.cwd = overlay.cwd.clone();
        }
        prepend_unique_strings(&mut self.files, &overlay.files);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdapterInputFamily {
    Compose,
    Bake,
}

pub(crate) const ADAPTER_INPUT_FAMILIES: [AdapterInputFamily; 2] =
    [AdapterInputFamily::Compose, AdapterInputFamily::Bake];

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdapterInputField {
    ComposeCwd,
    ComposeEnvFiles,
    ComposeFiles,
    ComposeProfiles,
    ComposeProjectName,
    BakeCwd,
    BakeFiles,
}

pub(crate) const ADAPTER_INPUT_FIELDS: [AdapterInputField; 7] = [
    AdapterInputField::ComposeCwd,
    AdapterInputField::ComposeEnvFiles,
    AdapterInputField::ComposeFiles,
    AdapterInputField::ComposeProfiles,
    AdapterInputField::ComposeProjectName,
    AdapterInputField::BakeCwd,
    AdapterInputField::BakeFiles,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AdapterInputFieldDescriptor {
    field: AdapterInputField,
    code: &'static str,
    family_name: &'static str,
    field_name: &'static str,
    runtime_file_kind: Option<&'static str>,
    runtime_file_label: Option<&'static str>,
}

const ADAPTER_INPUT_FIELD_DESCRIPTORS: [AdapterInputFieldDescriptor; 7] = [
    AdapterInputFieldDescriptor {
        field: AdapterInputField::ComposeCwd,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_CWD_OWNERSHIP",
        family_name: "compose",
        field_name: "cwd",
        runtime_file_kind: None,
        runtime_file_label: None,
    },
    AdapterInputFieldDescriptor {
        field: AdapterInputField::ComposeEnvFiles,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_ENV_FILES_OWNERSHIP",
        family_name: "compose",
        field_name: "env_files",
        runtime_file_kind: Some("compose_adapter_env_file"),
        runtime_file_label: Some("compose adapter env file"),
    },
    AdapterInputFieldDescriptor {
        field: AdapterInputField::ComposeFiles,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP",
        family_name: "compose",
        field_name: "files",
        runtime_file_kind: Some("compose_adapter_file"),
        runtime_file_label: Some("compose file"),
    },
    AdapterInputFieldDescriptor {
        field: AdapterInputField::ComposeProfiles,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROFILES_OWNERSHIP",
        family_name: "compose",
        field_name: "profiles",
        runtime_file_kind: None,
        runtime_file_label: None,
    },
    AdapterInputFieldDescriptor {
        field: AdapterInputField::ComposeProjectName,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROJECT_NAME_OWNERSHIP",
        family_name: "compose",
        field_name: "project_name",
        runtime_file_kind: None,
        runtime_file_label: None,
    },
    AdapterInputFieldDescriptor {
        field: AdapterInputField::BakeCwd,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_CWD_OWNERSHIP",
        family_name: "bake",
        field_name: "cwd",
        runtime_file_kind: None,
        runtime_file_label: None,
    },
    AdapterInputFieldDescriptor {
        field: AdapterInputField::BakeFiles,
        code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_FILES_OWNERSHIP",
        family_name: "bake",
        field_name: "files",
        runtime_file_kind: Some("bake_adapter_file"),
        runtime_file_label: Some("bake file"),
    },
];

impl AdapterInputField {
    const fn descriptor(self) -> AdapterInputFieldDescriptor {
        ADAPTER_INPUT_FIELD_DESCRIPTORS[self as usize]
    }

    pub(crate) const fn code(self) -> &'static str {
        self.descriptor().code
    }

    pub(crate) const fn family_name(self) -> &'static str {
        self.descriptor().family_name
    }

    pub(crate) const fn field_name(self) -> &'static str {
        self.descriptor().field_name
    }

    pub(crate) fn from_family_and_field_names(
        adapter_family: &str,
        field_name: &str,
    ) -> Option<Self> {
        ADAPTER_INPUT_FIELD_DESCRIPTORS
            .iter()
            .find(|descriptor| {
                descriptor.family_name == adapter_family && descriptor.field_name == field_name
            })
            .map(|descriptor| descriptor.field)
    }

    pub(crate) fn workflow_location(self, workflow_name: &str) -> String {
        format!(
            "workflows.{workflow_name}.adapter_inputs.{}.{}",
            self.family_name(),
            self.field_name()
        )
    }

    pub(crate) fn task_location(self, task_name: &str) -> String {
        format!(
            "tasks.{task_name}.adapter_inputs.{}.{}",
            self.family_name(),
            self.field_name()
        )
    }

    pub(crate) fn branch_location(self, task_name: &str, backend: Backend) -> String {
        let backend = match backend {
            Backend::Native => "native",
            Backend::Container => "container",
            Backend::Remote => "remote",
        };
        format!(
            "tasks.{task_name}.execution.modes.{backend}.adapter_inputs.{}.{}",
            self.family_name(),
            self.field_name()
        )
    }

    pub(crate) fn workflow_value(self, adapter_inputs: &TaskAdapterInputsSpec) -> Option<String> {
        match self {
            Self::ComposeCwd => adapter_inputs
                .compose
                .as_ref()
                .and_then(|compose| compose.cwd.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            Self::ComposeEnvFiles => adapter_inputs
                .compose
                .as_ref()
                .map(|compose| compose.env_files.as_slice())
                .filter(|values| !values.is_empty())
                .map(render_adapter_input_value_list),
            Self::ComposeFiles => adapter_inputs
                .compose
                .as_ref()
                .map(|compose| compose.files.as_slice())
                .filter(|values| !values.is_empty())
                .map(render_adapter_input_value_list),
            Self::ComposeProfiles => adapter_inputs
                .compose
                .as_ref()
                .map(|compose| compose.profiles.as_slice())
                .filter(|values| !values.is_empty())
                .map(render_adapter_input_value_list),
            Self::ComposeProjectName => adapter_inputs
                .compose
                .as_ref()
                .and_then(|compose| compose.project_name.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            Self::BakeCwd => adapter_inputs
                .bake
                .as_ref()
                .and_then(|bake| bake.cwd.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            Self::BakeFiles => adapter_inputs
                .bake
                .as_ref()
                .map(|bake| bake.files.as_slice())
                .filter(|values| !values.is_empty())
                .map(render_adapter_input_value_list),
        }
    }

    pub(crate) fn task_declared(self, task: &TaskSpec) -> bool {
        match self {
            Self::ComposeCwd => task
                .adapter_inputs
                .compose
                .as_ref()
                .filter(|compose| !compose.workflow_overlay_bound)
                .and_then(|compose| compose.cwd.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty()),
            Self::ComposeEnvFiles => task.adapter_inputs.compose.as_ref().is_some_and(|compose| {
                !compose.workflow_overlay_bound && !compose.env_files.is_empty()
            }),
            Self::ComposeFiles => task.adapter_inputs.compose.as_ref().is_some_and(|compose| {
                !compose.workflow_overlay_bound && !compose.files.is_empty()
            }),
            Self::ComposeProfiles => task.adapter_inputs.compose.as_ref().is_some_and(|compose| {
                !compose.workflow_overlay_bound && !compose.profiles.is_empty()
            }),
            Self::ComposeProjectName => task
                .adapter_inputs
                .compose
                .as_ref()
                .filter(|compose| !compose.workflow_overlay_bound)
                .and_then(|compose| compose.project_name.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty()),
            Self::BakeCwd => task
                .adapter_inputs
                .bake
                .as_ref()
                .filter(|bake| !bake.workflow_overlay_bound)
                .and_then(|bake| bake.cwd.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty()),
            Self::BakeFiles => task
                .adapter_inputs
                .bake
                .as_ref()
                .is_some_and(|bake| !bake.workflow_overlay_bound && !bake.files.is_empty()),
        }
    }

    pub(crate) fn branch_declared(self, branch: &TaskModeBranchSpec) -> bool {
        match self {
            Self::ComposeCwd => branch
                .adapter_inputs
                .compose
                .as_ref()
                .filter(|compose| !compose.workflow_overlay_bound)
                .and_then(|compose| compose.cwd.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty()),
            Self::ComposeEnvFiles => {
                branch
                    .adapter_inputs
                    .compose
                    .as_ref()
                    .is_some_and(|compose| {
                        !compose.workflow_overlay_bound && !compose.env_files.is_empty()
                    })
            }
            Self::ComposeFiles => branch
                .adapter_inputs
                .compose
                .as_ref()
                .is_some_and(|compose| {
                    !compose.workflow_overlay_bound && !compose.files.is_empty()
                }),
            Self::ComposeProfiles => {
                branch
                    .adapter_inputs
                    .compose
                    .as_ref()
                    .is_some_and(|compose| {
                        !compose.workflow_overlay_bound && !compose.profiles.is_empty()
                    })
            }
            Self::ComposeProjectName => branch
                .adapter_inputs
                .compose
                .as_ref()
                .filter(|compose| !compose.workflow_overlay_bound)
                .and_then(|compose| compose.project_name.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty()),
            Self::BakeCwd => branch
                .adapter_inputs
                .bake
                .as_ref()
                .filter(|bake| !bake.workflow_overlay_bound)
                .and_then(|bake| bake.cwd.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty()),
            Self::BakeFiles => branch
                .adapter_inputs
                .bake
                .as_ref()
                .is_some_and(|bake| !bake.workflow_overlay_bound && !bake.files.is_empty()),
        }
    }

    pub(crate) fn runtime_file_kind(self) -> Option<&'static str> {
        self.descriptor().runtime_file_kind
    }

    pub(crate) fn runtime_file_label(self) -> Option<&'static str> {
        self.descriptor().runtime_file_label
    }

    pub(crate) fn backend_paths(self, task: &TaskSpec, backend: Backend) -> Vec<String> {
        match self {
            Self::ComposeEnvFiles => task.compose_adapter_env_files_for_backend(backend),
            Self::ComposeFiles => task.compose_adapter_files_for_backend(backend),
            Self::BakeFiles => task.bake_adapter_files_for_backend(backend),
            Self::ComposeCwd | Self::ComposeProfiles | Self::ComposeProjectName | Self::BakeCwd => {
                Vec::new()
            }
        }
    }

    fn runtime_env_var(self) -> Option<&'static str> {
        match self {
            Self::ComposeEnvFiles => Some("COMPOSE_ENV_FILES"),
            Self::ComposeFiles => Some("COMPOSE_FILE"),
            Self::ComposeProfiles => Some("COMPOSE_PROFILES"),
            Self::ComposeProjectName => Some("COMPOSE_PROJECT_NAME"),
            Self::BakeFiles => Some("BUILDX_BAKE_FILE"),
            Self::ComposeCwd | Self::BakeCwd => None,
        }
    }

    fn runtime_env_value(self, task: &TaskSpec, backend: Backend) -> Option<String> {
        let adapter_cwd = task_effective_adapter_cwd(task, backend);
        match self {
            Self::ComposeEnvFiles => {
                let values = rebase_repo_relative_adapter_paths(
                    &self.backend_paths(task, backend),
                    adapter_cwd,
                );
                (!values.is_empty()).then(|| values.join(","))
            }
            Self::ComposeFiles | Self::BakeFiles => {
                let values = rebase_repo_relative_adapter_paths(
                    &self.backend_paths(task, backend),
                    adapter_cwd,
                );
                (!values.is_empty()).then(|| render_adapter_file_env_value(&values))
            }
            Self::ComposeProfiles => {
                let values = task.compose_adapter_profiles_for_backend(backend);
                (!values.is_empty()).then(|| values.join(","))
            }
            Self::ComposeProjectName => task
                .compose_adapter_project_name_for_backend(backend)
                .map(ToString::to_string),
            Self::ComposeCwd | Self::BakeCwd => None,
        }
    }
}

fn merge_legacy_workflow_adapter_inputs(
    target: &mut TaskAdapterInputsSpec,
    legacy: &TaskAdapterInputsSpec,
) {
    if let Some(legacy_compose) = legacy.compose.as_ref() {
        let compose = target
            .compose
            .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
        if compose.cwd.is_none() {
            compose.cwd = legacy_compose.cwd.clone();
        }
        if compose.env_files.is_empty() {
            compose.env_files = legacy_compose.env_files.clone();
        }
        if compose.files.is_empty() {
            compose.files = legacy_compose.files.clone();
        }
        if compose.profiles.is_empty() {
            compose.profiles = legacy_compose.profiles.clone();
        }
        if compose.project_name.is_none() {
            compose.project_name = legacy_compose.project_name.clone();
        }
    }
    if let Some(legacy_bake) = legacy.bake.as_ref() {
        let bake = target
            .bake
            .get_or_insert_with(TaskBakeAdapterInputsSpec::default);
        if bake.cwd.is_none() {
            bake.cwd = legacy_bake.cwd.clone();
        }
        if bake.files.is_empty() {
            bake.files = legacy_bake.files.clone();
        }
    }
}

pub(crate) fn workflow_declares_compose_file_alias(env: &WorkflowEnvSpec) -> bool {
    !env.compose_files.is_empty()
}

pub(crate) fn workflow_declares_compose_project_name_alias(env: &WorkflowEnvSpec) -> bool {
    env.compose_project_name
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

pub(crate) fn workflow_duplicates_canonical_compose_file_alias(workflow: &WorkflowSpec) -> bool {
    workflow.env.as_ref().is_some_and(|env| {
        workflow_declares_compose_file_alias(env)
            && (workflow
                .adapter_inputs
                .compose
                .as_ref()
                .is_some_and(|compose| !compose.files.is_empty())
                || env
                    .adapter_inputs
                    .compose
                    .as_ref()
                    .is_some_and(|compose| !compose.files.is_empty()))
    })
}

pub(crate) fn workflow_duplicates_canonical_compose_project_name_alias(
    workflow: &WorkflowSpec,
) -> bool {
    workflow.env.as_ref().is_some_and(|env| {
        workflow_declares_compose_project_name_alias(env)
            && (workflow
                .adapter_inputs
                .compose
                .as_ref()
                .and_then(|compose| compose.project_name.as_deref())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
                || env
                    .adapter_inputs
                    .compose
                    .as_ref()
                    .and_then(|compose| compose.project_name.as_deref())
                    .map(str::trim)
                    .is_some_and(|value| !value.is_empty()))
    })
}

impl AdapterInputFamily {
    pub(crate) const fn family_name(self) -> &'static str {
        match self {
            Self::Compose => "compose",
            Self::Bake => "bake",
        }
    }

    pub(crate) const fn replaceable_ownership_code(self) -> &'static str {
        match self {
            Self::Compose => "OTA_CONTRACT_ADVISORY_REPLACEABLE_COMPOSE_ENV_FILE_OWNERSHIP",
            Self::Bake => "OTA_CONTRACT_ADVISORY_REPLACEABLE_BAKE_FILE_OWNERSHIP",
        }
    }

    pub(crate) const fn empty_marker_code(self) -> &'static str {
        match self {
            Self::Compose => "OTA_CONTRACT_ADVISORY_EMPTY_COMPOSE_ADAPTER_INPUT_MARKER",
            Self::Bake => "OTA_CONTRACT_ADVISORY_EMPTY_BAKE_ADAPTER_INPUT_MARKER",
        }
    }

    pub(crate) fn uses_shell(self, command: &str) -> bool {
        let lower = command.to_ascii_lowercase();
        match self {
            Self::Compose => lower.contains("docker compose") || lower.contains("podman compose"),
            Self::Bake => lower.contains("docker buildx bake"),
        }
    }

    pub(crate) fn uses_command(self, command: &TaskCommandSpec) -> bool {
        match self {
            Self::Compose => {
                (command.exe.trim().eq_ignore_ascii_case("docker")
                    || command.exe.trim().eq_ignore_ascii_case("podman"))
                    && command
                        .args
                        .first()
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("compose"))
            }
            Self::Bake => {
                command.exe.trim().eq_ignore_ascii_case("docker")
                    && command
                        .args
                        .first()
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("buildx"))
                    && command
                        .args
                        .get(1)
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("bake"))
            }
        }
    }

    pub(crate) fn uses_launch(self, launch: Option<&TaskLaunchSpec>) -> bool {
        match launch {
            Some(TaskLaunchSpec::Command(command)) => self.uses_command(command),
            Some(TaskLaunchSpec::Container(_)) | None => false,
        }
    }

    pub(crate) fn obvious_replaceable_shell(self, command: &str) -> bool {
        let lower = command.to_ascii_lowercase();
        match self {
            Self::Compose => {
                ((lower.contains("docker compose") || lower.contains("podman compose"))
                    && (lower.contains("--env-file")
                        || lower.contains(" -f ")
                        || lower.contains(" --file ")
                        || lower.contains(" --project-directory ")
                        || lower.contains(" --profile ")
                        || lower.contains(" -p ")
                        || lower.contains(" --project-name ")))
                    || (lower.trim_start().starts_with("cd ")
                        && (lower.contains("&& docker compose")
                            || lower.contains("; docker compose")
                            || lower.contains("&& podman compose")
                            || lower.contains("; podman compose")))
            }
            Self::Bake => {
                (lower.contains("docker buildx bake")
                    && (lower.contains(" -f ") || lower.contains(" --file ")))
                    || (lower.trim_start().starts_with("cd ")
                        && (lower.contains("&& docker buildx bake")
                            || lower.contains("; docker buildx bake")))
            }
        }
    }

    pub(crate) fn obvious_replaceable_command(self, command: &TaskCommandSpec) -> bool {
        match self {
            Self::Compose => {
                (command.exe.trim().eq_ignore_ascii_case("docker")
                    || command.exe.trim().eq_ignore_ascii_case("podman"))
                    && command
                        .args
                        .first()
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("compose"))
                    && command.args.iter().any(|arg| {
                        matches!(
                            arg.trim(),
                            "--env-file"
                                | "-f"
                                | "--file"
                                | "--project-directory"
                                | "--profile"
                                | "-p"
                                | "--project-name"
                        )
                    })
            }
            Self::Bake => {
                command.exe.trim().eq_ignore_ascii_case("docker")
                    && command
                        .args
                        .first()
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("buildx"))
                    && command
                        .args
                        .get(1)
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("bake"))
                    && command
                        .args
                        .iter()
                        .any(|arg| matches!(arg.trim(), "-f" | "--file"))
            }
        }
    }

    pub(crate) fn effective_cwd(self, task: &TaskSpec, backend: Backend) -> Option<&str> {
        match self {
            Self::Compose => task.compose_adapter_cwd_for_backend(backend),
            Self::Bake => task.bake_adapter_cwd_for_backend(backend),
        }
    }

    pub(crate) fn task_declares_inputs(self, task: &TaskSpec) -> bool {
        match self {
            Self::Compose => task_declares_family_inputs::<TaskComposeAdapterInputsSpec>(task),
            Self::Bake => task_declares_family_inputs::<TaskBakeAdapterInputsSpec>(task),
        }
    }

    pub(crate) fn branch_declares_inputs(self, branch: &TaskModeBranchSpec) -> bool {
        match self {
            Self::Compose => branch_declares_family_inputs::<TaskComposeAdapterInputsSpec>(branch),
            Self::Bake => branch_declares_family_inputs::<TaskBakeAdapterInputsSpec>(branch),
        }
    }

    pub(crate) fn workflow_requires_support(self, overlay: &WorkflowAdapterOverlay) -> bool {
        match self {
            Self::Compose => overlay
                .as_task_adapter_inputs()
                .compose
                .as_ref()
                .is_some_and(|compose| !compose.is_empty()),
            Self::Bake => overlay
                .as_task_adapter_inputs()
                .bake
                .as_ref()
                .is_some_and(|bake| !bake.is_empty()),
        }
    }

    pub(crate) fn task_supports(self, task: &TaskSpec) -> bool {
        self.task_declares_inputs(task)
            || self.task_uses_adapter(task)
            || task.variants.iter().any(|variant| {
                self.shell_uses_adapter(variant.run.as_deref())
                    || self.shell_uses_adapter(variant.script.as_deref())
                    || self.command_uses_adapter(variant.command.as_ref())
            })
            || task.execution.as_ref().is_some_and(|execution| {
                execution
                    .modes
                    .iter()
                    .any(|(_, branch)| self.branch_supports(branch))
            })
    }

    pub(crate) fn bind_workflow_overlays(
        self,
        task: &mut TaskSpec,
        workflow_adapter_inputs: &TaskAdapterInputsSpec,
    ) -> bool {
        match self {
            Self::Compose => self.bind_workflow_overlay_spec::<TaskComposeAdapterInputsSpec>(
                task,
                workflow_adapter_inputs,
            ),
            Self::Bake => self.bind_workflow_overlay_spec::<TaskBakeAdapterInputsSpec>(
                task,
                workflow_adapter_inputs,
            ),
        }
    }

    fn bind_workflow_overlay_spec<S: WorkflowOverlaySpec>(
        self,
        task: &mut TaskSpec,
        workflow_adapter_inputs: &TaskAdapterInputsSpec,
    ) -> bool {
        let Some(workflow_overlay) = S::workflow_slot(workflow_adapter_inputs).cloned() else {
            return false;
        };

        let mut bound = false;
        if self.task_supports_direct_binding(task) {
            bind_task_workflow_overlay::<S>(task, &workflow_overlay);
            bound = true;
        }
        if let Some(execution) = task.execution.as_mut() {
            for branch in [
                execution.modes.native.as_mut(),
                execution.modes.container.as_mut(),
                execution.modes.remote.as_mut(),
            ]
            .into_iter()
            .flatten()
            {
                if !self.branch_supports(branch) {
                    continue;
                }
                bind_branch_workflow_overlay::<S>(branch, &workflow_overlay);
                bound = true;
            }
        }
        bound
    }

    fn task_supports_direct_binding(self, task: &TaskSpec) -> bool {
        self.task_declares_inputs(task)
            || self.task_uses_adapter(task)
            || task.variants.iter().any(|variant| {
                self.shell_uses_adapter(variant.run.as_deref())
                    || self.shell_uses_adapter(variant.script.as_deref())
                    || self.command_uses_adapter(variant.command.as_ref())
            })
    }

    fn branch_supports(self, branch: &TaskModeBranchSpec) -> bool {
        self.branch_declares_inputs(branch)
            || self.shell_uses_adapter(branch.run.as_deref())
            || self.shell_uses_adapter(branch.script.as_deref())
            || self.command_uses_adapter(branch.command.as_ref())
            || self.launch_uses_adapter(branch.launch.as_ref())
    }

    fn task_uses_adapter(self, task: &TaskSpec) -> bool {
        self.shell_uses_adapter(task.run.as_deref())
            || self.shell_uses_adapter(task.script.as_deref())
            || self.command_uses_adapter(task.command.as_ref())
            || self.launch_uses_adapter(task.launch.as_ref())
    }

    fn shell_uses_adapter(self, command: Option<&str>) -> bool {
        let Some(value) = command.map(str::trim).filter(|value| !value.is_empty()) else {
            return false;
        };
        self.uses_shell(value)
    }

    fn command_uses_adapter(self, command: Option<&TaskCommandSpec>) -> bool {
        command.is_some_and(|command| self.uses_command(command))
    }

    fn launch_uses_adapter(self, launch: Option<&TaskLaunchSpec>) -> bool {
        self.uses_launch(launch)
    }
}

fn task_declares_family_inputs<S: WorkflowOverlaySpec>(task: &TaskSpec) -> bool {
    S::task_slot(task).is_some_and(|spec| !spec.workflow_overlay_bound())
}

fn branch_declares_family_inputs<S: WorkflowOverlaySpec>(branch: &TaskModeBranchSpec) -> bool {
    S::branch_slot(branch).is_some_and(|spec| !spec.workflow_overlay_bound())
}

fn bind_task_workflow_overlay<S: WorkflowOverlaySpec>(task: &mut TaskSpec, overlay: &S) {
    let inserted = S::task_slot(task).is_none();
    let spec = S::task_slot_mut(task).get_or_insert_with(|| overlay.clone());
    if inserted {
        spec.set_workflow_overlay_bound(true);
    }
    spec.merge_workflow_overlay(overlay);
}

fn bind_branch_workflow_overlay<S: WorkflowOverlaySpec>(
    branch: &mut TaskModeBranchSpec,
    overlay: &S,
) {
    let inserted = S::branch_slot(branch).is_none();
    let spec = S::branch_slot_mut(branch).get_or_insert_with(|| overlay.clone());
    if inserted {
        spec.set_workflow_overlay_bound(true);
    }
    spec.merge_workflow_overlay(overlay);
}

pub(crate) fn bind_workflow_adapter_overlays(
    task: &mut TaskSpec,
    workflow_adapter_inputs: &TaskAdapterInputsSpec,
) -> bool {
    let mut bound = false;
    for family in ADAPTER_INPUT_FAMILIES {
        bound |= family.bind_workflow_overlays(task, workflow_adapter_inputs);
    }
    bound
}

pub(crate) fn task_effective_adapter_cwd(task: &TaskSpec, backend: Backend) -> Option<&str> {
    for family in ADAPTER_INPUT_FAMILIES {
        if family.task_supports(task)
            && let Some(cwd) = family.effective_cwd(task, backend)
        {
            return Some(cwd);
        }
    }
    None
}

pub(crate) fn rebase_repo_relative_adapter_paths(
    paths: &[String],
    adapter_cwd: Option<&str>,
) -> Vec<String> {
    paths
        .iter()
        .map(|path| rebase_repo_relative_adapter_path(path, adapter_cwd))
        .collect()
}

fn rebase_repo_relative_adapter_path(path: &str, adapter_cwd: Option<&str>) -> String {
    let path = path.trim();
    let Some(adapter_cwd) = adapter_cwd.map(str::trim).filter(|cwd| !cwd.is_empty()) else {
        return path.to_string();
    };
    let target = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let base = adapter_cwd
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let shared = target
        .iter()
        .zip(base.iter())
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = Vec::new();
    for _ in shared..base.len() {
        relative.push(String::from(".."));
    }
    for segment in target.iter().skip(shared) {
        relative.push((*segment).to_string());
    }
    if relative.is_empty() {
        String::from(".")
    } else {
        relative.join("/")
    }
}

fn render_adapter_file_env_value(paths: &[String]) -> String {
    if cfg!(windows) {
        paths.join(";")
    } else {
        paths.join(":")
    }
}

pub(crate) fn task_adapter_env_bindings(
    task: &TaskSpec,
    backend: Backend,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for field in ADAPTER_INPUT_FIELDS {
        let Some(name) = field.runtime_env_var() else {
            continue;
        };
        if let Some(value) = field.runtime_env_value(task, backend) {
            env.insert(name.to_string(), value);
        }
    }
    env
}

fn prepend_unique_strings(target: &mut Vec<String>, additions: &[String]) {
    if additions.is_empty() {
        return;
    }
    let mut merged = Vec::with_capacity(additions.len() + target.len());
    for value in additions {
        if !merged.iter().any(|existing| existing == value) {
            merged.push(value.clone());
        }
    }
    for value in target.iter() {
        if !merged.iter().any(|existing| existing == value) {
            merged.push(value.clone());
        }
    }
    *target = merged;
}

fn render_adapter_input_value_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{
        ADAPTER_INPUT_FIELDS, AdapterInputField, WorkflowAdapterOverlay, task_adapter_env_bindings,
        task_effective_adapter_cwd,
    };
    use crate::schema::{
        Backend, TaskAdapterInputsSpec, TaskComposeAdapterInputsSpec, TaskExecutionWhenSpec,
        TaskRequirementsSpec, TaskSpec, WorkflowEnvSpec, WorkflowPrepareSpec,
        WorkflowReadinessSpec, WorkflowServicesSpec, WorkflowSpec,
    };
    use std::collections::BTreeMap;

    #[test]
    fn adapter_input_field_registry_round_trips_family_and_field_names() {
        for field in ADAPTER_INPUT_FIELDS {
            assert_eq!(
                AdapterInputField::from_family_and_field_names(
                    field.family_name(),
                    field.field_name()
                ),
                Some(field)
            );
            assert!(
                field
                    .code()
                    .starts_with("OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_")
            );
        }
    }

    #[test]
    fn adapter_input_file_fields_keep_runtime_metadata() {
        for field in ADAPTER_INPUT_FIELDS {
            match field {
                AdapterInputField::ComposeCwd | AdapterInputField::BakeCwd => {
                    assert!(field.runtime_file_kind().is_none());
                    assert!(field.runtime_file_label().is_none());
                }
                AdapterInputField::ComposeEnvFiles
                | AdapterInputField::ComposeFiles
                | AdapterInputField::BakeFiles => {
                    assert!(field.runtime_file_kind().is_some());
                    assert!(field.runtime_file_label().is_some());
                }
                AdapterInputField::ComposeProfiles | AdapterInputField::ComposeProjectName => {
                    assert!(field.runtime_file_kind().is_none());
                    assert!(field.runtime_file_label().is_none());
                }
            }
        }
    }

    #[test]
    fn task_adapter_env_bindings_project_registry_owned_env_vars() {
        let task = TaskSpec {
            description: None,
            notes: None,
            category: None,
            context: None,
            env: BTreeMap::new(),
            env_files: Vec::new(),
            env_bindings: BTreeMap::new(),
            adapter_inputs: crate::schema::TaskAdapterInputsSpec {
                compose: Some(TaskComposeAdapterInputsSpec {
                    cwd: Some(String::from("ops/compose")),
                    env_files: vec![String::from("ops/compose/.env.compose")],
                    files: vec![String::from("ops/compose/docker-compose.yml")],
                    profiles: vec![String::from("web")],
                    project_name: Some(String::from("app")),
                    workflow_overlay_bound: false,
                }),
                bake: None,
            },
            inputs: BTreeMap::new(),
            targets: BTreeMap::new(),
            run: Some(String::from("docker compose up")),
            script: None,
            command: None,
            prepare: None,
            launch: None,
            action: None,
            aggregate: None,
            effects: crate::schema::TaskEffectsSpec::default(),
            requirements: TaskRequirementsSpec::default(),
            depends_on: Vec::new(),
            requires_services: Vec::new(),
            runtime: None,
            after_success: Vec::new(),
            after_failure: Vec::new(),
            after_always: Vec::new(),
            safe_for_agent: false,
            internal: false,
            variants: Vec::new(),
            execution: None,
            when: TaskExecutionWhenSpec::default(),
            projected_env_materialization_paths: Vec::new(),
        };

        let env = task_adapter_env_bindings(&task, Backend::Native);
        assert_eq!(
            env.get("COMPOSE_ENV_FILES").map(String::as_str),
            Some(".env.compose")
        );
        assert_eq!(
            env.get("COMPOSE_FILE").map(String::as_str),
            Some("docker-compose.yml")
        );
        assert_eq!(env.get("COMPOSE_PROFILES").map(String::as_str), Some("web"));
        assert_eq!(
            env.get("COMPOSE_PROJECT_NAME").map(String::as_str),
            Some("app")
        );
    }

    #[test]
    fn task_effective_adapter_cwd_uses_family_registry_order() {
        let task = TaskSpec {
            description: None,
            notes: None,
            category: None,
            context: None,
            env: BTreeMap::new(),
            env_files: Vec::new(),
            env_bindings: BTreeMap::new(),
            adapter_inputs: crate::schema::TaskAdapterInputsSpec {
                compose: Some(TaskComposeAdapterInputsSpec {
                    cwd: Some(String::from("ops/compose")),
                    env_files: Vec::new(),
                    files: Vec::new(),
                    profiles: Vec::new(),
                    project_name: None,
                    workflow_overlay_bound: false,
                }),
                bake: Some(crate::schema::TaskBakeAdapterInputsSpec {
                    cwd: Some(String::from("ops/bake")),
                    files: Vec::new(),
                    workflow_overlay_bound: false,
                }),
            },
            inputs: BTreeMap::new(),
            targets: BTreeMap::new(),
            run: Some(String::from("docker compose up")),
            script: None,
            command: None,
            prepare: None,
            launch: None,
            action: None,
            aggregate: None,
            effects: crate::schema::TaskEffectsSpec::default(),
            requirements: TaskRequirementsSpec::default(),
            depends_on: Vec::new(),
            requires_services: Vec::new(),
            runtime: None,
            after_success: Vec::new(),
            after_failure: Vec::new(),
            after_always: Vec::new(),
            safe_for_agent: false,
            internal: false,
            variants: Vec::new(),
            execution: None,
            when: TaskExecutionWhenSpec::default(),
            projected_env_materialization_paths: Vec::new(),
        };

        assert_eq!(
            task_effective_adapter_cwd(&task, Backend::Native),
            Some("ops/compose")
        );
    }

    #[test]
    fn workflow_adapter_overlay_merges_canonical_and_compat_compose_truth() {
        let workflow = WorkflowSpec {
            intent: None,
            description: None,
            notes: None,
            adapter_inputs: TaskAdapterInputsSpec {
                compose: Some(TaskComposeAdapterInputsSpec {
                    cwd: Some(String::from("ops/compose")),
                    env_files: vec![String::from(".env.workflow")],
                    files: Vec::new(),
                    profiles: vec![String::from("web")],
                    project_name: None,
                    workflow_overlay_bound: false,
                }),
                bake: None,
            },
            env: Some(WorkflowEnvSpec {
                profile: None,
                compose_env_file_services: Vec::new(),
                adapter_inputs: TaskAdapterInputsSpec::default(),
                compose_files: vec![String::from("compose.base.yaml")],
                compose_project_name: Some(String::from("workflow-app")),
            }),
            prepare: None::<WorkflowPrepareSpec>,
            setup: None,
            run: None,
            services: WorkflowServicesSpec::default(),
            readiness: WorkflowReadinessSpec::default(),
            exposes: Vec::new(),
        };

        let overlay = WorkflowAdapterOverlay::from_workflow(&workflow);
        let compose = overlay
            .as_task_adapter_inputs()
            .compose
            .as_ref()
            .expect("compose overlay should exist");
        assert_eq!(compose.cwd.as_deref(), Some("ops/compose"));
        assert_eq!(compose.env_files, vec![String::from(".env.workflow")]);
        assert_eq!(compose.files, vec![String::from("compose.base.yaml")]);
        assert_eq!(compose.profiles, vec![String::from("web")]);
        assert_eq!(compose.project_name.as_deref(), Some("workflow-app"));
    }
}
