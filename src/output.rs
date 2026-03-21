use serde::Serialize;

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

impl CommandOutput {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: None,
            exit_code: 0,
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
    pub run: &'a str,
    pub depends_on: &'a [String],
    pub safe_for_agent: bool,
}

impl<'a> TaskSummary<'a> {
    pub fn from_spec(name: &'a str, task: &'a TaskSpec) -> Self {
        Self {
            name,
            description: task.description.as_deref(),
            category: task.category.as_deref(),
            run: &task.run,
            depends_on: &task.depends_on,
            safe_for_agent: task.safe_for_agent,
        }
    }
}
