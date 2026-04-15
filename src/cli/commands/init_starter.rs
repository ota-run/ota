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

use crate::detector::{
    DetectCheck, DetectCheckKind, DetectCheckSeverity, DetectContract, DetectProject, DetectReport,
    DetectTask,
};
use crate::schema::{AgentBootstrapConfig, AgentBootstrapTargetConfig, AgentConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StarterPack {
    Node,
    Python,
}

impl StarterPack {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Python => "python",
        }
    }

    pub(crate) fn provenance_source(self) -> String {
        format!("ota.init#starter_pack.{}", self.as_str())
    }
}

pub(super) fn bootstrap_init_contract(report: &DetectReport) -> DetectContract {
    let mut contract = report.contract.clone();
    apply_starter_contract_defaults(&mut contract, &report.root);
    contract
}

pub(super) fn apply_starter_contract_defaults(contract: &mut DetectContract, root: &Path) {
    if contract.project.is_none()
        && let Some(name) = directory_name_for_root(root)
    {
        contract.project = Some(DetectProject { name });
    }
    if let Some(agent) = contract.agent.as_mut() {
        if agent.bootstrap.is_none() {
            agent.bootstrap = Some(starter_agent_bootstrap());
        }
    } else {
        contract.agent = starter_agent_from_detected_contract(contract, root);
    }
}

fn starter_agent_bootstrap() -> AgentBootstrapConfig {
    AgentBootstrapConfig {
        ota: Some(AgentBootstrapTargetConfig {
            note: Some(String::from(
                "Only install ota if it is missing and installation is approved.",
            )),
            sh: Some(String::from(
                "curl -fsSL https://dist.ota.run/install.sh | sh",
            )),
            powershell: Some(String::from("irm https://dist.ota.run/install.ps1 | iex")),
        }),
    }
}

fn starter_agent_from_detected_contract(
    contract: &DetectContract,
    root: &Path,
) -> Option<AgentConfig> {
    let mut safe_tasks = Vec::new();
    for task_name in ["setup", "test"] {
        if contract.tasks.contains_key(task_name) {
            safe_tasks.push(task_name.to_string());
        }
    }
    for (task_name, task) in &contract.tasks {
        if task.safe_for_agent && !safe_tasks.iter().any(|safe| safe == task_name) {
            safe_tasks.push(task_name.clone());
        }
    }
    if safe_tasks.is_empty() {
        return None;
    }

    let writable_paths = starter_agent_writable_paths(root);
    if writable_paths.is_empty() {
        return None;
    }
    let entrypoint = contract
        .tasks
        .contains_key("setup")
        .then(|| String::from("setup"));
    let default_task = if contract.tasks.contains_key("test") {
        Some(String::from("test"))
    } else {
        safe_tasks.first().cloned()
    };
    let verify_after_changes = if contract.tasks.contains_key("test") {
        vec![String::from("test")]
    } else {
        Vec::new()
    };

    let mut notes =
        String::from("Use `ota validate` before changes and `ota doctor` after edits.\n");
    if let Some(task_name) = default_task
        .as_deref()
        .or(entrypoint.as_deref())
        .or_else(|| safe_tasks.first().map(String::as_str))
    {
        notes.push_str(&format!("Use `ota run {task_name}` to verify changes.\n"));
    }

    Some(AgentConfig {
        entrypoint,
        default_task,
        safe_tasks,
        verify_after_changes,
        writable_paths,
        protected_paths: vec![String::from("ota.yaml")],
        bootstrap: Some(starter_agent_bootstrap()),
        notes: Some(notes),
    })
}

fn starter_agent_writable_paths(root: &Path) -> Vec<String> {
    let mut writable_paths = Vec::new();
    for candidate in ["src", "tests", "docs"] {
        if root.join(candidate).is_dir() {
            writable_paths.push(candidate.to_string());
        }
    }
    writable_paths
}

fn directory_name_for_root(root: &Path) -> Option<String> {
    if let Some(name) = root.file_name().and_then(|name| name.to_str())
        && !name.is_empty()
        && name != "."
    {
        return Some(name.to_string());
    }

    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            cwd.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
        })
        .filter(|name| !name.is_empty())
}

pub(crate) fn starter_pack_contract(pack: StarterPack, root: &Path) -> DetectContract {
    let project = directory_name_for_root(root).map(|name| DetectProject { name });
    let mut contract = DetectContract {
        version: 1,
        project,
        ..DetectContract::default()
    };

    match pack {
        StarterPack::Node => {
            contract
                .runtimes
                .insert(String::from("node"), String::from("22"));
            contract
                .tools
                .insert(String::from("pnpm"), String::from("10"));
            contract.checks.push(DetectCheck {
                name: String::from("node-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("node --version"),
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "pnpm install",
                    Some(String::from("Install repo dependencies.")),
                ),
            );
            contract.tasks.insert(
                String::from("dev"),
                pack_task(
                    "dev",
                    "pnpm dev",
                    Some(String::from("Start the local development loop.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    "pnpm test",
                    Some(String::from("Run the default automated test command.")),
                ),
            );
        }
        StarterPack::Python => {
            contract
                .runtimes
                .insert(String::from("python"), String::from("3.12"));
            contract.checks.push(DetectCheck {
                name: String::from("python-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("python --version"),
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "python -m pip install -r requirements.txt",
                    Some(String::from(
                        "Install Python dependencies from requirements.txt.",
                    )),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    "pytest",
                    Some(String::from("Run the default Python test command.")),
                ),
            );
        }
    }

    apply_starter_contract_defaults(&mut contract, root);
    contract
}

fn pack_task(task_name: &str, run: &str, note: Option<String>) -> DetectTask {
    let mut notes = String::from("Run `ota run ");
    notes.push_str(task_name);
    notes.push_str("` to execute this task.\n");
    if let Some(note) = note {
        notes.push_str(&note);
    }

    DetectTask {
        run: String::from(run),
        notes: Some(notes),
        safe_for_agent: false,
    }
}
