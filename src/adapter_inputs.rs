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
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdapterInputFamily {
    Compose,
    Bake,
}

pub(crate) const ADAPTER_INPUT_FAMILIES: [AdapterInputFamily; 2] =
    [AdapterInputFamily::Compose, AdapterInputFamily::Bake];

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

impl AdapterInputField {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::ComposeCwd => "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_CWD_OWNERSHIP",
            Self::ComposeEnvFiles => {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_ENV_FILES_OWNERSHIP"
            }
            Self::ComposeFiles => {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP"
            }
            Self::ComposeProfiles => {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROFILES_OWNERSHIP"
            }
            Self::ComposeProjectName => {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROJECT_NAME_OWNERSHIP"
            }
            Self::BakeCwd => "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_CWD_OWNERSHIP",
            Self::BakeFiles => "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_FILES_OWNERSHIP",
        }
    }

    pub(crate) const fn family_name(self) -> &'static str {
        match self {
            Self::ComposeCwd
            | Self::ComposeEnvFiles
            | Self::ComposeFiles
            | Self::ComposeProfiles
            | Self::ComposeProjectName => "compose",
            Self::BakeCwd | Self::BakeFiles => "bake",
        }
    }

    pub(crate) const fn field_name(self) -> &'static str {
        match self {
            Self::ComposeCwd | Self::BakeCwd => "cwd",
            Self::ComposeEnvFiles => "env_files",
            Self::ComposeFiles | Self::BakeFiles => "files",
            Self::ComposeProfiles => "profiles",
            Self::ComposeProjectName => "project_name",
        }
    }

    pub(crate) fn from_family_and_field_names(
        adapter_family: &str,
        field_name: &str,
    ) -> Option<Self> {
        match (adapter_family, field_name) {
            ("compose", "cwd") => Some(Self::ComposeCwd),
            ("compose", "env_files") => Some(Self::ComposeEnvFiles),
            ("compose", "files") => Some(Self::ComposeFiles),
            ("compose", "profiles") => Some(Self::ComposeProfiles),
            ("compose", "project_name") => Some(Self::ComposeProjectName),
            ("bake", "cwd") => Some(Self::BakeCwd),
            ("bake", "files") => Some(Self::BakeFiles),
            _ => None,
        }
    }

    pub(crate) fn workflow_location(self, workflow_name: &str) -> String {
        match self {
            Self::ComposeCwd => format!("workflows.{workflow_name}.env.adapter_inputs.compose.cwd"),
            Self::ComposeEnvFiles => {
                format!("workflows.{workflow_name}.env.adapter_inputs.compose.env_files")
            }
            Self::ComposeFiles => {
                format!("workflows.{workflow_name}.env.adapter_inputs.compose.files")
            }
            Self::ComposeProfiles => {
                format!("workflows.{workflow_name}.env.adapter_inputs.compose.profiles")
            }
            Self::ComposeProjectName => {
                format!("workflows.{workflow_name}.env.adapter_inputs.compose.project_name")
            }
            Self::BakeCwd => format!("workflows.{workflow_name}.env.adapter_inputs.bake.cwd"),
            Self::BakeFiles => format!("workflows.{workflow_name}.env.adapter_inputs.bake.files"),
        }
    }

    pub(crate) fn task_location(self, task_name: &str) -> String {
        match self {
            Self::ComposeCwd => format!("tasks.{task_name}.adapter_inputs.compose.cwd"),
            Self::ComposeEnvFiles => format!("tasks.{task_name}.adapter_inputs.compose.env_files"),
            Self::ComposeFiles => format!("tasks.{task_name}.adapter_inputs.compose.files"),
            Self::ComposeProfiles => format!("tasks.{task_name}.adapter_inputs.compose.profiles"),
            Self::ComposeProjectName => {
                format!("tasks.{task_name}.adapter_inputs.compose.project_name")
            }
            Self::BakeCwd => format!("tasks.{task_name}.adapter_inputs.bake.cwd"),
            Self::BakeFiles => format!("tasks.{task_name}.adapter_inputs.bake.files"),
        }
    }

    pub(crate) fn branch_location(self, task_name: &str, backend: Backend) -> String {
        let backend = match backend {
            Backend::Native => "native",
            Backend::Container => "container",
            Backend::Remote => "remote",
        };
        match self {
            Self::ComposeCwd => {
                format!("tasks.{task_name}.execution.modes.{backend}.adapter_inputs.compose.cwd")
            }
            Self::ComposeEnvFiles => format!(
                "tasks.{task_name}.execution.modes.{backend}.adapter_inputs.compose.env_files"
            ),
            Self::ComposeFiles => {
                format!("tasks.{task_name}.execution.modes.{backend}.adapter_inputs.compose.files")
            }
            Self::ComposeProfiles => format!(
                "tasks.{task_name}.execution.modes.{backend}.adapter_inputs.compose.profiles"
            ),
            Self::ComposeProjectName => format!(
                "tasks.{task_name}.execution.modes.{backend}.adapter_inputs.compose.project_name"
            ),
            Self::BakeCwd => {
                format!("tasks.{task_name}.execution.modes.{backend}.adapter_inputs.bake.cwd")
            }
            Self::BakeFiles => {
                format!("tasks.{task_name}.execution.modes.{backend}.adapter_inputs.bake.files")
            }
        }
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
                .is_some_and(|compose| !compose.workflow_overlay_bound && !compose.files.is_empty()),
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
        match self {
            Self::ComposeEnvFiles => Some("compose_adapter_env_file"),
            Self::ComposeFiles => Some("compose_adapter_file"),
            Self::BakeFiles => Some("bake_adapter_file"),
            Self::ComposeCwd
            | Self::ComposeProfiles
            | Self::ComposeProjectName
            | Self::BakeCwd => None,
        }
    }

    pub(crate) fn runtime_file_label(self) -> Option<&'static str> {
        match self {
            Self::ComposeEnvFiles => Some("compose adapter env file"),
            Self::ComposeFiles => Some("compose file"),
            Self::BakeFiles => Some("bake file"),
            Self::ComposeCwd
            | Self::ComposeProfiles
            | Self::ComposeProjectName
            | Self::BakeCwd => None,
        }
    }

    pub(crate) fn backend_paths(self, task: &TaskSpec, backend: Backend) -> Vec<String> {
        match self {
            Self::ComposeEnvFiles => task.compose_adapter_env_files_for_backend(backend),
            Self::ComposeFiles => task.compose_adapter_files_for_backend(backend),
            Self::BakeFiles => task.bake_adapter_files_for_backend(backend),
            Self::ComposeCwd
            | Self::ComposeProfiles
            | Self::ComposeProjectName
            | Self::BakeCwd => Vec::new(),
        }
    }
}

pub(crate) fn effective_workflow_adapter_inputs(env: &WorkflowEnvSpec) -> TaskAdapterInputsSpec {
    let mut adapter_inputs = env.adapter_inputs.clone();
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
    adapter_inputs
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

pub(crate) fn workflow_duplicates_canonical_compose_file_alias(env: &WorkflowEnvSpec) -> bool {
    env.adapter_inputs
        .compose
        .as_ref()
        .is_some_and(|compose| !compose.files.is_empty())
        && workflow_declares_compose_file_alias(env)
}

pub(crate) fn workflow_duplicates_canonical_compose_project_name_alias(
    env: &WorkflowEnvSpec,
) -> bool {
    env.adapter_inputs
        .compose
        .as_ref()
        .and_then(|compose| compose.project_name.as_deref())
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && workflow_declares_compose_project_name_alias(env)
}

impl AdapterInputFamily {
    pub(crate) fn task_declares_inputs(self, task: &TaskSpec) -> bool {
        match self {
            Self::Compose => task
                .adapter_inputs
                .compose
                .as_ref()
                .is_some_and(|compose| !compose.workflow_overlay_bound),
            Self::Bake => task
                .adapter_inputs
                .bake
                .as_ref()
                .is_some_and(|bake| !bake.workflow_overlay_bound),
        }
    }

    pub(crate) fn branch_declares_inputs(self, branch: &TaskModeBranchSpec) -> bool {
        match self {
            Self::Compose => branch
                .adapter_inputs
                .compose
                .as_ref()
                .is_some_and(|compose| !compose.workflow_overlay_bound),
            Self::Bake => branch
                .adapter_inputs
                .bake
                .as_ref()
                .is_some_and(|bake| !bake.workflow_overlay_bound),
        }
    }

    pub(crate) fn workflow_requires_support(self, env: &WorkflowEnvSpec) -> bool {
        match self {
            Self::Compose => {
                workflow_declares_compose_file_alias(env)
                    || workflow_declares_compose_project_name_alias(env)
                    || env
                        .adapter_inputs
                        .compose
                        .as_ref()
                        .is_some_and(|compose| !compose.is_empty())
            }
            Self::Bake => env
                .adapter_inputs
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
            Self::Compose => {
                let Some(workflow_compose) = workflow_adapter_inputs.compose.as_ref() else {
                    return false;
                };
                let mut bound = false;
                if self.task_supports_direct_binding(task) {
                    let inserted = task.adapter_inputs.compose.is_none();
                    let compose = task
                        .adapter_inputs
                        .compose
                        .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
                    if inserted {
                        compose.workflow_overlay_bound = true;
                    }
                    if compose.cwd.is_none() {
                        compose.cwd = workflow_compose.cwd.clone();
                    }
                    prepend_unique_strings(&mut compose.env_files, &workflow_compose.env_files);
                    prepend_unique_strings(&mut compose.files, &workflow_compose.files);
                    prepend_unique_strings(&mut compose.profiles, &workflow_compose.profiles);
                    if compose.project_name.is_none() {
                        compose.project_name = workflow_compose.project_name.clone();
                    }
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
                        let inserted = branch.adapter_inputs.compose.is_none();
                        let compose = branch
                            .adapter_inputs
                            .compose
                            .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
                        if inserted {
                            compose.workflow_overlay_bound = true;
                        }
                        if compose.cwd.is_none() {
                            compose.cwd = workflow_compose.cwd.clone();
                        }
                        prepend_unique_strings(&mut compose.env_files, &workflow_compose.env_files);
                        prepend_unique_strings(&mut compose.files, &workflow_compose.files);
                        prepend_unique_strings(&mut compose.profiles, &workflow_compose.profiles);
                        if compose.project_name.is_none() {
                            compose.project_name = workflow_compose.project_name.clone();
                        }
                        bound = true;
                    }
                }
                bound
            }
            Self::Bake => {
                let Some(workflow_bake) = workflow_adapter_inputs.bake.as_ref() else {
                    return false;
                };
                let mut bound = false;
                if self.task_supports_direct_binding(task) {
                    let inserted = task.adapter_inputs.bake.is_none();
                    let bake = task
                        .adapter_inputs
                        .bake
                        .get_or_insert_with(TaskBakeAdapterInputsSpec::default);
                    if inserted {
                        bake.workflow_overlay_bound = true;
                    }
                    if bake.cwd.is_none() {
                        bake.cwd = workflow_bake.cwd.clone();
                    }
                    prepend_unique_strings(&mut bake.files, &workflow_bake.files);
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
                        let inserted = branch.adapter_inputs.bake.is_none();
                        let bake = branch
                            .adapter_inputs
                            .bake
                            .get_or_insert_with(TaskBakeAdapterInputsSpec::default);
                        if inserted {
                            bake.workflow_overlay_bound = true;
                        }
                        if bake.cwd.is_none() {
                            bake.cwd = workflow_bake.cwd.clone();
                        }
                        prepend_unique_strings(&mut bake.files, &workflow_bake.files);
                        bound = true;
                    }
                }
                bound
            }
        }
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
        let Some(value) = command
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_ascii_lowercase)
        else {
            return false;
        };
        match self {
            Self::Compose => value.contains("docker compose"),
            Self::Bake => value.contains("docker buildx bake"),
        }
    }

    fn command_uses_adapter(self, command: Option<&TaskCommandSpec>) -> bool {
        command.is_some_and(|command| {
            if !command.exe.trim().eq_ignore_ascii_case("docker") {
                return false;
            }
            match self {
                Self::Compose => command
                    .args
                    .first()
                    .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("compose")),
                Self::Bake => {
                    command
                        .args
                        .first()
                        .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("buildx"))
                        && command
                            .args
                            .get(1)
                            .is_some_and(|arg| arg.trim().eq_ignore_ascii_case("bake"))
                }
            }
        })
    }

    fn launch_uses_adapter(self, launch: Option<&TaskLaunchSpec>) -> bool {
        match launch {
            Some(TaskLaunchSpec::Command(command)) => self.command_uses_adapter(Some(command)),
            Some(TaskLaunchSpec::Container(_)) | None => false,
        }
    }
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
    if AdapterInputFamily::Compose.task_supports(task) {
        if let Some(cwd) = task.compose_adapter_cwd_for_backend(backend) {
            return Some(cwd);
        }
    }
    if AdapterInputFamily::Bake.task_supports(task) {
        if let Some(cwd) = task.bake_adapter_cwd_for_backend(backend) {
            return Some(cwd);
        }
    }
    None
}

pub(crate) fn rebase_repo_relative_adapter_paths(
    paths: &[String],
    adapter_cwd: Option<&str>,
) -> Vec<String> {
    paths.iter()
        .map(|path| rebase_repo_relative_adapter_path(path, adapter_cwd))
        .collect()
}

fn rebase_repo_relative_adapter_path(path: &str, adapter_cwd: Option<&str>) -> String {
    let path = path.trim();
    let Some(adapter_cwd) = adapter_cwd.map(str::trim).filter(|cwd| !cwd.is_empty()) else {
        return path.to_string();
    };
    let target = path.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
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
    use super::{ADAPTER_INPUT_FIELDS, AdapterInputField};

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
            assert!(field.code().starts_with("OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_"));
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
}
