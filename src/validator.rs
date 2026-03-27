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

use std::collections::{BTreeMap, BTreeSet};

use crate::schema::{AgentConfig, Contract, RuntimeRequirement, ServiceSpec, TaskSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ValidationErrors {
    message: String,
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    fn from_vec(errors: Vec<ValidationError>) -> Self {
        let mut message = String::from("INVALID ota.yaml");
        for error in &errors {
            message.push_str("\n- ");
            message.push_str(&error.message);
        }

        Self { message, errors }
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

pub fn validate_contract(contract: &Contract) -> Result<(), ValidationErrors> {
    let mut errors = Vec::new();

    validate_version(contract, &mut errors);
    validate_project(contract, &mut errors);
    validate_repo_workspace(contract, &mut errors);
    validate_execution(contract, &mut errors);
    validate_extensions(contract, &mut errors);
    validate_named_versions("runtime", &contract.runtimes, &mut errors, |value| {
        value.version()
    });
    validate_runtime_details(&contract.runtimes, &mut errors);
    validate_named_versions("tool", &contract.tools, &mut errors, |value| {
        value.version()
    });
    validate_services(&contract.services, &mut errors);
    validate_tasks(&contract.tasks, &mut errors);
    validate_checks(contract, &mut errors);
    validate_agent(contract.agent.as_ref(), &contract.tasks, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors::from_vec(errors))
    }
}

fn validate_version(contract: &Contract, errors: &mut Vec<ValidationError>) {
    if contract.version != 1 {
        errors.push(ValidationError::new(format!(
            "unsupported contract version `{}`; expected `1`",
            contract.version
        )));
    }
}

fn validate_project(contract: &Contract, errors: &mut Vec<ValidationError>) {
    if contract.project.name.trim().is_empty() {
        errors.push(ValidationError::new("`project.name` must not be empty"));
    }
}

fn validate_repo_workspace(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(workspace) = &contract.workspace else {
        return;
    };

    if workspace.members.is_empty() {
        errors.push(ValidationError::new(
            "`workspace.members` must contain at least one member",
        ));
        return;
    }

    let mut seen = BTreeSet::new();
    for member in &workspace.members {
        if member.trim().is_empty() {
            errors.push(ValidationError::new(
                "`workspace.members` must not contain empty member paths",
            ));
            continue;
        }

        if !seen.insert(member) {
            errors.push(ValidationError::new(format!(
                "`workspace.members` must not declare duplicate member `{member}`"
            )));
        }
    }
}

fn validate_execution(contract: &Contract, errors: &mut Vec<ValidationError>) {
    let Some(execution) = &contract.execution else {
        return;
    };

    if let Some(preferred) = execution.preferred
        && !execution.supported.is_empty()
        && !execution.supported.contains(&preferred)
    {
        errors.push(ValidationError::new(format!(
            "`execution.preferred` is set to `{}` but it is missing from `execution.supported`",
            format_backend(preferred)
        )));
    }

    if let Some(container) = execution
        .backends
        .as_ref()
        .and_then(|backends| backends.container.as_ref())
        && container.image.trim().is_empty()
    {
        errors.push(ValidationError::new(
            "`execution.backends.container.image` must not be empty",
        ));
    }

    if let Some(remote) = execution
        .backends
        .as_ref()
        .and_then(|backends| backends.remote.as_ref())
        && remote.provider.trim().is_empty()
    {
        errors.push(ValidationError::new(
            "`execution.backends.remote.provider` must not be empty",
        ));
    }

    if let Some(remote) = execution
        .backends
        .as_ref()
        .and_then(|backends| backends.remote.as_ref())
        && remote
            .target
            .as_deref()
            .is_some_and(|target| target.trim().is_empty())
    {
        errors.push(ValidationError::new(
            "`execution.backends.remote.target` must not be empty",
        ));
    }

    if let Some(remote) = execution
        .backends
        .as_ref()
        .and_then(|backends| backends.remote.as_ref())
        && remote
            .cwd
            .as_deref()
            .is_some_and(|cwd| cwd.trim().is_empty())
    {
        errors.push(ValidationError::new(
            "`execution.backends.remote.cwd` must not be empty",
        ));
    }

    if execution.preferred == Some(crate::schema::Backend::Container)
        && execution
            .backends
            .as_ref()
            .and_then(|backends| backends.container.as_ref())
            .is_none()
    {
        errors.push(ValidationError::new(
            "`execution.preferred: container` requires `execution.backends.container.image`",
        ));
    }

    if execution.preferred == Some(crate::schema::Backend::Container)
        && execution.lifecycle.is_none()
    {
        errors.push(ValidationError::new(
            "`execution.preferred: container` requires an explicit `execution.lifecycle`",
        ));
    }

    if execution.preferred == Some(crate::schema::Backend::Remote)
        && execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
            .is_none()
    {
        errors.push(ValidationError::new(
            "`execution.preferred: remote` requires `execution.backends.remote.provider`",
        ));
    }

    if execution.preferred == Some(crate::schema::Backend::Remote)
        && execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
            .and_then(|remote| remote.target.as_deref())
            .is_none()
    {
        let provider = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.remote.as_ref())
            .map(|remote| remote.provider.trim())
            .unwrap_or_default();
        let example = remote_target_example(provider);
        if provider.is_empty() {
            errors.push(ValidationError::new(
                "`execution.preferred: remote` requires `execution.backends.remote.target`",
            ));
        } else {
            errors.push(ValidationError::new(format!(
                "`execution.preferred: remote` with provider `{provider}` requires `execution.backends.remote.target` (example: `{example}`)"
            )));
        }
    }
}

fn validate_extensions(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for (name, extension) in &contract.extensions {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("extension names must not be empty"));
        }

        if extension.command.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "extension `{}` must not declare an empty `command`",
                name
            )));
        }

        if extension.api_version == 0 {
            errors.push(ValidationError::new(format!(
                "extension `{}` must declare `api_version` greater than zero",
                name
            )));
        }
    }
}

fn remote_target_example(provider: &str) -> &'static str {
    match provider {
        "daytona" => "sandbox-dev",
        "ssh" | "tsh" => "user@host",
        "kubectl" => "pod/ota-dev",
        _ => "remote-target",
    }
}

fn validate_named_versions<T>(
    label: &str,
    values: &BTreeMap<String, T>,
    errors: &mut Vec<ValidationError>,
    version: impl Fn(&T) -> &str,
) {
    for (name, value) in values {
        if name.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "{label} name must not be empty"
            )));
        }

        if version(value).trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "{label} `{name}` must declare a non-empty version"
            )));
        }
    }
}

fn validate_runtime_details(
    runtimes: &BTreeMap<String, RuntimeRequirement>,
    errors: &mut Vec<ValidationError>,
) {
    for (name, runtime) in runtimes {
        let RuntimeRequirement::Detailed(detail) = runtime else {
            continue;
        };

        if detail
            .provider
            .as_deref()
            .is_some_and(|provider| provider.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "runtime `{name}` must not declare an empty `provider`"
            )));
        }

        if detail
            .distribution
            .as_deref()
            .is_some_and(|distribution| distribution.trim().is_empty())
        {
            errors.push(ValidationError::new(format!(
                "runtime `{name}` must not declare an empty `distribution`"
            )));
        }
    }
}

fn validate_tasks(tasks: &BTreeMap<String, TaskSpec>, errors: &mut Vec<ValidationError>) {
    for (name, task) in tasks {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("task name must not be empty"));
        }

        let has_base_fields = task.run.is_some() || task.script.is_some();
        match (task.run.as_deref(), task.script.as_deref()) {
            (Some(run), None) if run.trim().is_empty() => errors.push(ValidationError::new(
                format!("task `{name}` must declare a non-empty `run` command"),
            )),
            (None, Some(script)) if script.trim().is_empty() => errors.push(ValidationError::new(
                format!("task `{name}` must declare a non-empty `script` body"),
            )),
            (Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                "task `{name}` must declare exactly one of `run` or `script`"
            ))),
            (Some(_), None) | (None, Some(_)) => {}
            (None, None) => {}
        }

        if !has_base_fields && task.variants.is_empty() {
            errors.push(ValidationError::new(format!(
                "task `{name}` must declare exactly one of `run` or `script`"
            )));
        }

        let mut seen_variant_os = BTreeSet::new();
        for (index, variant) in task.variants.iter().enumerate() {
            let Some(os) = variant.when.os.as_deref() else {
                errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} must declare `when.os`"
                )));
                continue;
            };

            if !matches!(os, "linux" | "macos" | "windows") {
                errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} declares unsupported `when.os: {os}`"
                )));
            }

            if !seen_variant_os.insert(os.to_string()) {
                errors.push(ValidationError::new(format!(
                    "task `{name}` must not declare multiple variants for `when.os: {os}`"
                )));
            }

            match (variant.run.as_deref(), variant.script.as_deref()) {
                (Some(run), None) if run.trim().is_empty() => {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` variant #{index} must declare a non-empty `run` command"
                    )))
                }
                (None, Some(script)) if script.trim().is_empty() => {
                    errors.push(ValidationError::new(format!(
                        "task `{name}` variant #{index} must declare a non-empty `script` body"
                    )))
                }
                (Some(_), Some(_)) => errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} must not declare both `run` and `script`"
                ))),
                (None, None) => errors.push(ValidationError::new(format!(
                    "task `{name}` variant #{index} must declare exactly one of `run` or `script`"
                ))),
                _ => {}
            }
        }

        for dependency in &task.depends_on {
            if !tasks.contains_key(dependency) {
                errors.push(ValidationError::new(format!(
                    "task `{name}` depends on unknown task `{dependency}`"
                )));
            }
        }
    }

    detect_task_cycles(tasks, errors);
}

fn validate_services(services: &BTreeMap<String, ServiceSpec>, errors: &mut Vec<ValidationError>) {
    for (name, service) in services {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("service name must not be empty"));
        }

        for (field, value) in [
            ("provider", service.provider.as_deref()),
            ("start", service.start.as_deref()),
            ("stop", service.stop.as_deref()),
            ("healthcheck", service.healthcheck.as_deref()),
        ] {
            if matches!(value, Some(value) if value.trim().is_empty()) {
                errors.push(ValidationError::new(format!(
                    "service `{name}` field `{field}` must not be empty"
                )));
            }
        }

        if matches!(service.timeout, Some(0)) {
            errors.push(ValidationError::new(format!(
                "service `{name}` must declare a timeout greater than zero"
            )));
        }

        if service.provider.is_none()
            && service.start.is_none()
            && service.stop.is_none()
            && service.healthcheck.is_none()
        {
            errors.push(ValidationError::new(format!(
                "service `{name}` must declare at least one of `provider`, `start`, `stop`, or `healthcheck`"
            )));
        }

        for dependency in &service.depends_on {
            if !services.contains_key(dependency) {
                errors.push(ValidationError::new(format!(
                    "service `{name}` depends on unknown service `{dependency}`"
                )));
            }
        }
    }

    detect_service_cycles(services, errors);
}

fn detect_task_cycles(tasks: &BTreeMap<String, TaskSpec>, errors: &mut Vec<ValidationError>) {
    let mut visited = BTreeSet::new();
    let mut active = Vec::new();
    let mut cycle_roots = BTreeSet::new();

    for name in tasks.keys() {
        visit_task(
            name,
            tasks,
            &mut visited,
            &mut active,
            &mut cycle_roots,
            errors,
        );
    }
}

fn detect_service_cycles(
    services: &BTreeMap<String, ServiceSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let mut visited = BTreeSet::new();
    let mut active = Vec::new();
    let mut cycle_roots = BTreeSet::new();

    for name in services.keys() {
        visit_service(
            name,
            services,
            &mut visited,
            &mut active,
            &mut cycle_roots,
            errors,
        );
    }
}

fn visit_task(
    name: &str,
    tasks: &BTreeMap<String, TaskSpec>,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
    cycle_roots: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if visited.contains(name) {
        return;
    }

    if let Some(index) = active.iter().position(|task| task == name) {
        let cycle = active[index..].to_vec();
        if cycle_roots.insert(cycle[0].clone()) {
            errors.push(ValidationError::new(format!(
                "task dependency cycle detected: {} -> {}",
                cycle.join(" -> "),
                name
            )));
        }
        return;
    }

    let Some(task) = tasks.get(name) else {
        return;
    };

    active.push(name.to_string());

    for dependency in &task.depends_on {
        if tasks.contains_key(dependency) {
            visit_task(dependency, tasks, visited, active, cycle_roots, errors);
        }
    }

    active.pop();
    visited.insert(name.to_string());
}

fn visit_service(
    name: &str,
    services: &BTreeMap<String, ServiceSpec>,
    visited: &mut BTreeSet<String>,
    active: &mut Vec<String>,
    cycle_roots: &mut BTreeSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    if visited.contains(name) {
        return;
    }

    if let Some(index) = active.iter().position(|service| service == name) {
        let cycle = active[index..].to_vec();
        if cycle_roots.insert(cycle[0].clone()) {
            errors.push(ValidationError::new(format!(
                "service dependency cycle detected: {} -> {}",
                cycle.join(" -> "),
                name
            )));
        }
        return;
    }

    let Some(service) = services.get(name) else {
        return;
    };

    active.push(name.to_string());

    for dependency in &service.depends_on {
        if services.contains_key(dependency) {
            visit_service(dependency, services, visited, active, cycle_roots, errors);
        }
    }

    active.pop();
    visited.insert(name.to_string());
}

fn validate_checks(contract: &Contract, errors: &mut Vec<ValidationError>) {
    for check in &contract.checks {
        if check.name.trim().is_empty() {
            errors.push(ValidationError::new("check name must not be empty"));
        }

        if check.run.trim().is_empty() {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare a non-empty `run` command",
                check.name
            )));
        }

        if matches!(check.timeout, Some(0)) {
            errors.push(ValidationError::new(format!(
                "check `{}` must declare a timeout greater than zero",
                check.name
            )));
        }
    }
}

fn validate_agent(
    agent: Option<&AgentConfig>,
    tasks: &BTreeMap<String, TaskSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(agent) = agent else {
        return;
    };

    validate_task_reference(
        "agent.entrypoint",
        agent.entrypoint.as_deref(),
        tasks,
        errors,
    );
    validate_task_reference(
        "agent.default_task",
        agent.default_task.as_deref(),
        tasks,
        errors,
    );

    for task in &agent.safe_tasks {
        validate_task_reference("agent.safe_tasks", Some(task.as_str()), tasks, errors);
    }

    for task in &agent.verify_after_changes {
        validate_task_reference(
            "agent.verify_after_changes",
            Some(task.as_str()),
            tasks,
            errors,
        );
    }

    for path in &agent.writable_paths {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.writable_paths` entries must not be empty",
            ));
        }
    }

    for path in &agent.protected_paths {
        if path.trim().is_empty() {
            errors.push(ValidationError::new(
                "`agent.protected_paths` entries must not be empty",
            ));
        }
    }
}

fn validate_task_reference(
    field: &str,
    task_name: Option<&str>,
    tasks: &BTreeMap<String, TaskSpec>,
    errors: &mut Vec<ValidationError>,
) {
    let Some(task_name) = task_name else {
        return;
    };

    if !tasks.contains_key(task_name) {
        errors.push(ValidationError::new(format!(
            "`{field}` references unknown task `{task_name}`"
        )));
    }
}

fn format_backend(backend: crate::schema::Backend) -> &'static str {
    match backend {
        crate::schema::Backend::Native => "native",
        crate::schema::Backend::Container => "container",
        crate::schema::Backend::Remote => "remote",
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::parser::parse_contract_str;

    use super::validate_contract;

    #[test]
    fn validates_a_minimal_contract() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_services_and_execution_lifecycle() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: persistent
services:
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -h localhost -p 5432
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_extension_descriptors() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: checker
    command: ota-ext-demo
    api_version: 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn validates_runtime_distribution() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java:
    version: "21"
    distribution: temurin
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_empty_runtime_distribution() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java:
    version: "21"
    distribution: "   "
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "runtime `java` must not declare an empty `distribution`"
        );
    }

    #[test]
    fn rejects_invalid_extension_descriptor_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
        kind: checker
        command: " "
        api_version: 0
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 2);
        assert!(
            errors
                .errors()
                .iter()
                .any(|error| error.to_string().contains("empty `command`"))
        );
        assert!(
            errors
                .errors()
                .iter()
                .any(|error| error.to_string().contains("greater than zero"))
        );
    }

    #[test]
    fn validates_remote_backend_target_and_cwd() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: daytona
      target: sandbox-dev
      cwd: /workspace
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        assert!(validate_contract(&contract).is_ok());
    }

    #[test]
    fn rejects_remote_backend_without_target_when_preferred() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: daytona
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `daytona` requires `execution.backends.remote.target` (example: `sandbox-dev`)"
        );
    }

    #[test]
    fn rejects_ssh_remote_backend_without_target_with_provider_specific_example() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `ssh` requires `execution.backends.remote.target` (example: `user@host`)"
        );
    }

    #[test]
    fn rejects_tsh_remote_backend_without_target_with_provider_specific_example() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `tsh` requires `execution.backends.remote.target` (example: `user@host`)"
        );
    }

    #[test]
    fn rejects_kubectl_remote_backend_without_target_with_provider_specific_example() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: remote` with provider `kubectl` requires `execution.backends.remote.target` (example: `pod/ota-dev`)"
        );
    }

    #[test]
    fn rejects_empty_service_declarations() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `postgres` must declare at least one of `provider`, `start`, `stop`, or `healthcheck`"
        );
    }

    #[test]
    fn rejects_unknown_task_dependencies() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: cargo run
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` depends on unknown task `setup`"
        );
    }

    #[test]
    fn rejects_unknown_service_dependencies() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: docker compose up -d api
    depends_on:
      - postgres
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `api` depends on unknown service `postgres`"
        );
    }

    #[test]
    fn rejects_zero_service_timeout() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    healthcheck: pg_isready -h localhost -p 5432
    timeout: 0
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service `postgres` must declare a timeout greater than zero"
        );
    }

    #[test]
    fn rejects_tasks_with_both_run_and_script() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: cargo run
    script: |
      cargo run
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run` or `script`"
        );
    }

    #[test]
    fn rejects_tasks_with_neither_run_nor_script() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    description: missing execution
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare exactly one of `run` or `script`"
        );
    }

    #[test]
    fn rejects_empty_script_bodies() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    script: "   "
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `dev` must declare a non-empty `script` body"
        );
    }

    #[test]
    fn rejects_task_variants_without_when_os() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    variants:
      - when: {}
        run: ./scripts/setup.sh
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `setup` variant #0 must declare `when.os`"
        );
    }

    #[test]
    fn rejects_duplicate_task_variant_os_values() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    variants:
      - when:
          os: macos
        run: ./scripts/setup-macos.sh
      - when:
          os: macos
        run: ./scripts/setup-macos-2.sh
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task `setup` must not declare multiple variants for `when.os: macos`"
        );
    }

    #[test]
    fn rejects_zero_check_timeout() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: slow-check
    kind: health
    severity: warn
    run: sleep 1
    timeout: 0
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "check `slow-check` must declare a timeout greater than zero"
        );
    }

    #[test]
    fn rejects_task_dependency_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo fetch
    depends_on:
      - build
  build:
    run: cargo build
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "task dependency cycle detected: build -> setup -> build"
        );
    }

    #[test]
    fn rejects_service_dependency_cycles() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: docker compose up -d api
    depends_on:
      - postgres
  postgres:
    required: true
    start: docker compose up -d postgres
    depends_on:
      - api
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "service dependency cycle detected: api -> postgres -> api"
        );
    }

    #[test]
    fn rejects_container_preferred_without_container_image() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: container` requires `execution.backends.container.image`"
        );
    }

    #[test]
    fn rejects_container_preferred_without_explicit_lifecycle() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  backends:
    container:
      image: ghcr.io/ota/dev:latest
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let errors = validate_contract(&contract).unwrap_err();
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(
            errors.errors()[0].to_string(),
            "`execution.preferred: container` requires an explicit `execution.lifecycle`"
        );
    }
}
