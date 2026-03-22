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

use serde::Serialize;

use crate::doctor::Finding;
use crate::schema::TaskSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: Option<String>,
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct DoctorSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub findings: &'a [Finding],
}

impl CommandOutput {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: None,
            exit_code: 0,
        }
    }

    pub fn status(exit_code: i32) -> Self {
        Self {
            stdout: String::new(),
            stderr: None,
            exit_code,
        }
    }

    pub fn failure(stderr: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: Some(stderr),
            exit_code: 1,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ValidateSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
}

#[derive(Debug, Serialize)]
pub struct ValidateFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TasksSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub tasks: Vec<TaskSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct TasksFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskSummary<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'a str>,
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<&'a str>,
    pub depends_on: &'a [String],
    pub safe_for_agent: bool,
}

impl<'a> TaskSummary<'a> {
    pub fn from_spec(name: &'a str, task: &'a TaskSpec) -> Self {
        Self {
            name,
            description: task.description.as_deref(),
            category: task.category.as_deref(),
            kind: task
                .execution_kind()
                .expect("validated task must declare exactly one execution form"),
            run: task.run.as_deref(),
            script: task.script.as_deref(),
            depends_on: &task.depends_on,
            safe_for_agent: task.safe_for_agent,
        }
    }
}
