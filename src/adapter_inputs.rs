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
    TaskAdapterInputsSpec, TaskBakeAdapterInputsSpec, TaskCommandSpec,
    TaskComposeAdapterInputsSpec, TaskModeBranchSpec, TaskSpec, WorkflowEnvSpec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdapterInputFamily {
    Compose,
    Bake,
}

pub(crate) const ADAPTER_INPUT_FAMILIES: [AdapterInputFamily; 2] =
    [AdapterInputFamily::Compose, AdapterInputFamily::Bake];

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
            Self::Compose => task.adapter_inputs.compose.is_some(),
            Self::Bake => task.adapter_inputs.bake.is_some(),
        }
    }

    pub(crate) fn branch_declares_inputs(self, branch: &TaskModeBranchSpec) -> bool {
        match self {
            Self::Compose => branch.adapter_inputs.compose.is_some(),
            Self::Bake => branch.adapter_inputs.bake.is_some(),
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
                    let compose = task
                        .adapter_inputs
                        .compose
                        .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
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
                        let compose = branch
                            .adapter_inputs
                            .compose
                            .get_or_insert_with(TaskComposeAdapterInputsSpec::default);
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
                    let bake = task
                        .adapter_inputs
                        .bake
                        .get_or_insert_with(TaskBakeAdapterInputsSpec::default);
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
                        let bake = branch
                            .adapter_inputs
                            .bake
                            .get_or_insert_with(TaskBakeAdapterInputsSpec::default);
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
    }

    fn task_uses_adapter(self, task: &TaskSpec) -> bool {
        self.shell_uses_adapter(task.run.as_deref())
            || self.shell_uses_adapter(task.script.as_deref())
            || self.command_uses_adapter(task.command.as_ref())
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
