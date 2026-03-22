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
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inference {
    pub field: String,
    pub value: String,
    pub source: String,
    pub confidence: Confidence,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct DetectContract {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<DetectProject>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub runtimes: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tools: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub services: BTreeMap<String, DetectService>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tasks: BTreeMap<String, DetectTask>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectProject {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectTask {
    pub run: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct DetectService {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectReport {
    pub root: PathBuf,
    pub contract: DetectContract,
    pub inferences: Vec<Inference>,
}

impl DetectReport {
    pub fn high_confidence_contract(&self) -> DetectContract {
        let mut contract = DetectContract {
            version: 1,
            ..DetectContract::default()
        };

        for inference in &self.inferences {
            if inference.confidence != Confidence::High {
                continue;
            }

            if inference.field == "project.name" {
                contract.project = Some(DetectProject {
                    name: inference.value.clone(),
                });
                continue;
            }

            if let Some(runtime) = inference.field.strip_prefix("runtimes.") {
                contract
                    .runtimes
                    .insert(runtime.to_string(), inference.value.clone());
                continue;
            }

            if let Some(tool) = inference.field.strip_prefix("tools.") {
                contract
                    .tools
                    .insert(tool.to_string(), inference.value.clone());
                continue;
            }

            if let Some(service_field) = inference.field.strip_prefix("services.")
                && let Some((service_name, field_name)) = service_field.split_once('.')
            {
                let service = contract
                    .services
                    .entry(service_name.to_string())
                    .or_default();
                match field_name {
                    "provider" => service.provider = Some(inference.value.clone()),
                    "start" => service.start = Some(inference.value.clone()),
                    "stop" => service.stop = Some(inference.value.clone()),
                    "healthcheck" => service.healthcheck = Some(inference.value.clone()),
                    _ => {}
                }
                continue;
            }

            if let Some(task_field) = inference.field.strip_prefix("tasks.")
                && let Some(task_name) = task_field.strip_suffix(".run")
            {
                contract.tasks.insert(
                    task_name.to_string(),
                    DetectTask {
                        run: inference.value.clone(),
                    },
                );
            }
        }

        contract
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DetectError {
    #[error("failed to read `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse `{path}`: {message}")]
    Parse { path: String, message: String },
}

pub fn detect_repo(root: &Path) -> Result<DetectReport, DetectError> {
    let root = canonicalize_root(root);
    let mut builder = DetectBuilder::new(root.clone());

    detect_package_json(&root, &mut builder)?;
    detect_nvmrc(&root, &mut builder)?;
    detect_node_version_file(&root, &mut builder)?;
    detect_python_version_file(&root, &mut builder)?;
    detect_go_mod(&root, &mut builder)?;
    detect_tool_versions(&root, &mut builder)?;
    detect_pyproject(&root, &mut builder)?;
    detect_gradle(&root, &mut builder)?;
    detect_pom_xml(&root, &mut builder)?;
    detect_compose_services(&root, &mut builder)?;
    detect_directory_name(&root, &mut builder);

    Ok(builder.finish())
}

fn detect_package_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("package.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let package: JsonValue =
        serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(name) = package.get("name").and_then(JsonValue::as_str) {
        builder.set_project_name(
            name.to_string(),
            "package.json#name".to_string(),
            Confidence::High,
        );
    }

    let mut package_manager_name = None;

    if let Some(package_manager) = package.get("packageManager").and_then(JsonValue::as_str) {
        if let Some((name, version)) = package_manager.split_once('@') {
            if !name.is_empty() && !version.is_empty() {
                builder.set_tool(
                    name.to_string(),
                    version.to_string(),
                    "package.json#packageManager".to_string(),
                    Confidence::High,
                );
                package_manager_name = Some(name.to_string());
            }
        }
    }

    if let Some(node) = package
        .get("engines")
        .and_then(|engines| engines.get("node"))
        .and_then(JsonValue::as_str)
    {
        builder.set_runtime(
            "node".to_string(),
            node.to_string(),
            "package.json#engines.node".to_string(),
            Confidence::Medium,
        );
    }

    if let Some(scripts) = package.get("scripts").and_then(JsonValue::as_object) {
        let task_confidence = if package_manager_name.is_some() {
            Confidence::High
        } else {
            Confidence::Medium
        };
        let package_manager = package_manager_name
            .clone()
            .unwrap_or_else(|| "npm".to_string());

        for (name, _) in scripts {
            if let Some(run) = task_command(&package_manager, name) {
                builder.set_task(
                    name.to_string(),
                    run,
                    format!("package.json#scripts.{name}"),
                    task_confidence,
                );
            }
        }
    }

    Ok(())
}

fn detect_nvmrc(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".nvmrc");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "node".to_string(),
            version,
            ".nvmrc".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_node_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".node-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "node".to_string(),
            version,
            ".node-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_tool_versions(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".tool-versions");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(tool) = parts.next() else {
            continue;
        };
        let Some(version) = parts.next() else {
            continue;
        };

        match tool {
            "nodejs" | "node" => builder.set_runtime(
                "node".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "python" => builder.set_runtime(
                "python".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "go" | "golang" => builder.set_runtime(
                "go".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "pnpm" | "npm" | "yarn" | "bun" => builder.set_tool(
                tool.to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            _ => {}
        }
    }

    Ok(())
}

fn detect_python_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".python-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "python".to_string(),
            version,
            ".python-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_pyproject(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pyproject.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(name) = document
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
    {
        builder.set_project_name(
            name.to_string(),
            "pyproject.toml#project.name".to_string(),
            Confidence::High,
        );
    } else if let Some(name) = document
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("name"))
        .and_then(TomlValue::as_str)
    {
        builder.set_project_name(
            name.to_string(),
            "pyproject.toml#tool.poetry.name".to_string(),
            Confidence::High,
        );
    }

    if let Some(python) = document
        .get("project")
        .and_then(|project| project.get("requires-python"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "pyproject.toml#project.requires-python".to_string(),
            Confidence::Medium,
        );
    } else if let Some(python) = document
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dependencies"))
        .and_then(|dependencies| dependencies.get("python"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "pyproject.toml#tool.poetry.dependencies.python".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_go_mod(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("go.mod");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(module) = trimmed.strip_prefix("module ") {
            if let Some(name) = module.split('/').next_back() {
                builder.set_project_name(
                    name.to_string(),
                    "go.mod#module".to_string(),
                    Confidence::Medium,
                );
            }
        } else if let Some(version) = trimmed.strip_prefix("go ") {
            builder.set_runtime(
                "go".to_string(),
                version.trim().to_string(),
                "go.mod#go".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_gradle(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let settings_path = ["settings.gradle.kts", "settings.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    if let Some(path) = settings_path {
        let contents = read_file(&path)?;
        if let Some(name) = extract_quoted_assignment(&contents, "rootProject.name") {
            builder.set_project_name(
                name,
                format!(
                    "{}#rootProject.name",
                    path.file_name().unwrap().to_string_lossy()
                ),
                Confidence::High,
            );
        }
    }

    let build_path = ["build.gradle.kts", "build.gradle"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists());
    if let Some(path) = build_path {
        let contents = read_file(&path)?;
        if let Some(version) = extract_gradle_java_version(&contents) {
            builder.set_runtime(
                "java".to_string(),
                version,
                format!(
                    "{}#java.toolchain",
                    path.file_name().unwrap().to_string_lossy()
                ),
                Confidence::High,
            );
        }
    }

    let wrapper_path = root
        .join("gradle")
        .join("wrapper")
        .join("gradle-wrapper.properties");
    if wrapper_path.exists() {
        let contents = read_file(&wrapper_path)?;
        if let Some(version) = extract_gradle_wrapper_version(&contents) {
            builder.set_tool(
                "gradle".to_string(),
                version,
                "gradle/wrapper/gradle-wrapper.properties#distributionUrl".to_string(),
                Confidence::High,
            );
            builder.set_task(
                "build".to_string(),
                "./gradlew build".to_string(),
                "gradle/wrapper/gradle-wrapper.properties#distributionUrl".to_string(),
                Confidence::High,
            );
            builder.set_task(
                "test".to_string(),
                "./gradlew test".to_string(),
                "gradle/wrapper/gradle-wrapper.properties#distributionUrl".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_pom_xml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pom.xml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;

    if let Some(name) = extract_xml_tag(&contents, "artifactId") {
        builder.set_project_name(name, "pom.xml#artifactId".to_string(), Confidence::High);
    }

    for tag in [
        "maven.compiler.release",
        "maven.compiler.target",
        "maven.compiler.source",
        "java.version",
    ] {
        if let Some(version) = extract_xml_tag(&contents, tag) {
            builder.set_runtime(
                "java".to_string(),
                version,
                format!("pom.xml#{tag}"),
                Confidence::High,
            );
            break;
        }
    }

    let maven_wrapper = detect_maven_wrapper(root)?;
    if let Some((version, source)) = &maven_wrapper {
        builder.set_tool(
            "maven".to_string(),
            version.clone(),
            source.clone(),
            Confidence::High,
        );
    } else {
        builder.set_tool(
            "maven".to_string(),
            "*".to_string(),
            "pom.xml".to_string(),
            Confidence::Medium,
        );
    }

    let build_command = if root.join("mvnw").exists() {
        "./mvnw package"
    } else {
        "mvn package"
    };
    let test_command = if root.join("mvnw").exists() {
        "./mvnw test"
    } else {
        "mvn test"
    };
    let task_source = maven_wrapper
        .as_ref()
        .map(|(_, source)| source.as_str())
        .unwrap_or("pom.xml");
    let task_confidence = if maven_wrapper.is_some() {
        Confidence::High
    } else {
        Confidence::Medium
    };

    builder.set_task(
        "build".to_string(),
        build_command.to_string(),
        task_source.to_string(),
        task_confidence,
    );
    builder.set_task(
        "test".to_string(),
        test_command.to_string(),
        task_source.to_string(),
        task_confidence,
    );

    Ok(())
}

fn detect_maven_wrapper(root: &Path) -> Result<Option<(String, String)>, DetectError> {
    let wrapper_script = root.join("mvnw");
    let wrapper_properties = root
        .join(".mvn")
        .join("wrapper")
        .join("maven-wrapper.properties");

    if !wrapper_script.exists() || !wrapper_properties.exists() {
        return Ok(None);
    }

    let contents = read_file(&wrapper_properties)?;
    Ok(extract_maven_wrapper_version(&contents).map(|version| {
        (
            version,
            String::from(".mvn/wrapper/maven-wrapper.properties#distributionUrl"),
        )
    }))
}

fn detect_compose_services(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ]
    .iter()
    .map(|name| root.join(name))
    .find(|path| path.exists());
    let Some(path) = path else {
        return Ok(());
    };

    let contents = read_file(&path)?;
    let document: YamlValue =
        serde_yaml::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    let Some(services) = document.get("services").and_then(YamlValue::as_mapping) else {
        return Ok(());
    };

    let file_name = path.file_name().unwrap().to_string_lossy();
    for service_name in services.keys().filter_map(YamlValue::as_str) {
        builder.set_service_provider(
            service_name.to_string(),
            "docker-compose".to_string(),
            format!("{file_name}#services.{service_name}"),
            Confidence::High,
        );
        builder.set_service_start(
            service_name.to_string(),
            format!("docker compose up -d {service_name}"),
            format!("{file_name}#services.{service_name}"),
            Confidence::Medium,
        );
        builder.set_service_stop(
            service_name.to_string(),
            format!("docker compose stop {service_name}"),
            format!("{file_name}#services.{service_name}"),
            Confidence::Medium,
        );

        if let Some(command) = services
            .get(YamlValue::String(service_name.to_string()))
            .and_then(extract_compose_healthcheck_command)
        {
            builder.set_service_healthcheck(
                service_name.to_string(),
                command,
                format!("{file_name}#services.{service_name}.healthcheck.test"),
                Confidence::Medium,
            );
        }
    }

    Ok(())
}

fn extract_compose_healthcheck_command(service: &YamlValue) -> Option<String> {
    let test = service
        .as_mapping()?
        .get(YamlValue::String(String::from("healthcheck")))?
        .as_mapping()?
        .get(YamlValue::String(String::from("test")))?;

    match test {
        YamlValue::String(command) => {
            let command = command.trim();
            if command.is_empty() {
                None
            } else {
                Some(command.to_string())
            }
        }
        YamlValue::Sequence(parts) => {
            let values = parts
                .iter()
                .map(YamlValue::as_str)
                .collect::<Option<Vec<_>>>()?;
            let first = *values.first()?;
            match first {
                "NONE" => None,
                "CMD-SHELL" => {
                    let command = values.get(1)?.trim();
                    if command.is_empty() {
                        None
                    } else {
                        Some(command.to_string())
                    }
                }
                "CMD" => {
                    let command = values
                        .iter()
                        .skip(1)
                        .map(|part| part.trim())
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if command.is_empty() {
                        None
                    } else {
                        Some(command)
                    }
                }
                _ => {
                    let command = values
                        .iter()
                        .map(|part| part.trim())
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if command.is_empty() {
                        None
                    } else {
                        Some(command)
                    }
                }
            }
        }
        _ => None,
    }
}

fn detect_directory_name(root: &Path, builder: &mut DetectBuilder) {
    if builder.contract.project.is_none()
        && let Some(name) = root.file_name().and_then(|name| name.to_str())
        && !name.is_empty()
    {
        builder.set_project_name(
            name.to_string(),
            "directory-name".to_string(),
            Confidence::Low,
        );
    }
}

fn extract_maven_wrapper_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        let url = line.strip_prefix("distributionUrl=")?;
        let version = url
            .split("apache-maven-")
            .nth(1)?
            .split('-')
            .next()?
            .trim();
        if version.is_empty() {
            None
        } else {
            Some(version.to_string())
        }
    })
}

fn extract_quoted_assignment(contents: &str, prefix: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        let value = trimmed.strip_prefix(prefix)?.trim_start();
        let value = value.strip_prefix('=')?.trim_start();
        extract_quoted_string(value)
    })
}

fn extract_quoted_string(input: &str) -> Option<String> {
    let quote = input.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let rest = &input[quote.len_utf8()..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn extract_gradle_java_version(contents: &str) -> Option<String> {
    for marker in [
        "JavaLanguageVersion.of(",
        "languageVersion = JavaLanguageVersion.of(",
    ] {
        if let Some(start) = contents.find(marker) {
            let rest = &contents[start + marker.len()..];
            let end = rest.find(')')?;
            let digits = rest[..end].trim();
            if !digits.is_empty() {
                return Some(digits.to_string());
            }
        }
    }

    None
}

fn extract_gradle_wrapper_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        let url = trimmed.strip_prefix("distributionUrl=")?;
        let file = url.rsplit('/').next()?;
        let version = file.strip_prefix("gradle-")?.split('-').next()?.trim();
        if version.is_empty() {
            None
        } else {
            Some(version.to_string())
        }
    })
}

fn extract_xml_tag(contents: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = contents.find(&open)? + open.len();
    let end = contents[start..].find(&close)? + start;
    let value = contents[start..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn task_command(package_manager: &str, script_name: &str) -> Option<String> {
    match package_manager {
        "pnpm" => Some(format!("pnpm {script_name}")),
        "yarn" => Some(format!("yarn {script_name}")),
        "bun" => Some(format!("bun run {script_name}")),
        "npm" => Some(format!("npm run {script_name}")),
        _ => None,
    }
}

fn read_file(path: &Path) -> Result<String, DetectError> {
    fs::read_to_string(path).map_err(|source| DetectError::Read {
        path: path.display().to_string(),
        source,
    })
}

fn canonicalize_root(root: &Path) -> PathBuf {
    if root.is_dir() {
        root.to_path_buf()
    } else {
        root.parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    }
}

struct DetectBuilder {
    root: PathBuf,
    contract: DetectContract,
    inferences: BTreeMap<String, Inference>,
}

impl DetectBuilder {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            contract: DetectContract {
                version: 1,
                ..DetectContract::default()
            },
            inferences: BTreeMap::new(),
        }
    }

    fn finish(self) -> DetectReport {
        DetectReport {
            root: self.root,
            contract: self.contract,
            inferences: self.inferences.into_values().collect(),
        }
    }

    fn set_project_name(&mut self, value: String, source: String, confidence: Confidence) {
        let field = "project.name".to_string();
        if self.should_replace(&field, &source, confidence) {
            self.contract.project = Some(DetectProject {
                name: value.clone(),
            });
            self.record(field, value, source, confidence);
        }
    }

    fn set_runtime(&mut self, name: String, value: String, source: String, confidence: Confidence) {
        let field = format!("runtimes.{name}");
        if self.should_replace(&field, &source, confidence) {
            self.contract.runtimes.insert(name, value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_tool(&mut self, name: String, value: String, source: String, confidence: Confidence) {
        let field = format!("tools.{name}");
        if self.should_replace(&field, &source, confidence) {
            self.contract.tools.insert(name, value.clone());
            self.record(field, value, source, confidence);
        }
    }

    fn set_task(&mut self, name: String, run: String, source: String, confidence: Confidence) {
        let field = format!("tasks.{name}.run");
        if self.should_replace(&field, &source, confidence) {
            self.contract
                .tasks
                .insert(name, DetectTask { run: run.clone() });
            self.record(field, run, source, confidence);
        }
    }

    fn set_service_provider(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        self.set_service_field(name, "provider", value, source, confidence);
    }

    fn set_service_start(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        self.set_service_field(name, "start", value, source, confidence);
    }

    fn set_service_stop(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        self.set_service_field(name, "stop", value, source, confidence);
    }

    fn set_service_healthcheck(
        &mut self,
        name: String,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        self.set_service_field(name, "healthcheck", value, source, confidence);
    }

    fn set_service_field(
        &mut self,
        name: String,
        field_name: &str,
        value: String,
        source: String,
        confidence: Confidence,
    ) {
        let field = format!("services.{name}.{field_name}");
        if self.should_replace(&field, &source, confidence) {
            let service = self.contract.services.entry(name).or_default();
            match field_name {
                "provider" => service.provider = Some(value.clone()),
                "start" => service.start = Some(value.clone()),
                "stop" => service.stop = Some(value.clone()),
                "healthcheck" => service.healthcheck = Some(value.clone()),
                _ => {}
            }
            self.record(field, value, source, confidence);
        }
    }

    fn should_replace(&self, field: &str, source: &str, confidence: Confidence) -> bool {
        self.inferences.get(field).is_none_or(|existing| {
            confidence > existing.confidence
                || (confidence == existing.confidence
                    && source_priority(field, source) > source_priority(field, &existing.source))
        })
    }

    fn record(&mut self, field: String, value: String, source: String, confidence: Confidence) {
        self.inferences.insert(
            field.clone(),
            Inference {
                field,
                value,
                source,
                confidence,
            },
        );
    }
}

fn source_priority(field: &str, source: &str) -> u8 {
    match field {
        "project.name" => match source {
            "package.json#name" => 5,
            "settings.gradle.kts#rootProject.name" => 4,
            "settings.gradle#rootProject.name" => 4,
            "pyproject.toml#project.name" => 4,
            "pyproject.toml#tool.poetry.name" => 3,
            "pom.xml#artifactId" => 3,
            "go.mod#module" => 2,
            "directory-name" => 1,
            _ => 0,
        },
        "runtimes.node" => match source {
            ".nvmrc" => 4,
            ".node-version" => 3,
            ".tool-versions" => 2,
            "package.json#engines.node" => 1,
            _ => 0,
        },
        "runtimes.python" => match source {
            ".python-version" => 4,
            ".tool-versions" => 3,
            "pyproject.toml#project.requires-python" => 2,
            "pyproject.toml#tool.poetry.dependencies.python" => 1,
            _ => 0,
        },
        "runtimes.go" => match source {
            "go.mod#go" => 2,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.java" => match source {
            "build.gradle.kts#java.toolchain" => 3,
            "build.gradle#java.toolchain" => 3,
            "pom.xml#maven.compiler.release" => 2,
            "pom.xml#maven.compiler.target" => 2,
            "pom.xml#maven.compiler.source" => 2,
            "pom.xml#java.version" => 1,
            _ => 0,
        },
        _ if field.starts_with("tools.") => match source {
            "gradle/wrapper/gradle-wrapper.properties#distributionUrl" => 3,
            ".mvn/wrapper/maven-wrapper.properties#distributionUrl" => 3,
            "package.json#packageManager" => 2,
            "pom.xml" => 1,
            ".tool-versions" => 1,
            _ => 0,
        },
        _ if field.starts_with("services.") => match source {
            "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml" => 2,
            _ if source.starts_with("docker-compose.yml#services.")
                || source.starts_with("docker-compose.yaml#services.")
                || source.starts_with("compose.yml#services.")
                || source.starts_with("compose.yaml#services.") =>
            {
                2
            }
            _ => 0,
        },
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::{Confidence, detect_repo};

    #[test]
    fn prefers_nvmrc_over_package_json_engines() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "engines": { "node": "20" },
  "packageManager": "pnpm@10.2.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("node"),
            Some(&"22".to_string())
        );
        assert_eq!(
            report
                .inferences
                .iter()
                .find(|inference| inference.field == "runtimes.node")
                .unwrap()
                .confidence,
            Confidence::High
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("dev")
                .map(|task| task.run.as_str()),
            Some("pnpm dev")
        );
    }

    #[test]
    fn detects_python_and_go_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pyproject.toml",
            r#"[project]
name = "ota-py"
requires-python = ">=3.12"
"#,
        );
        fixture.write("go.mod", "module github.com/ota/run\n\ngo 1.24.0\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-py")
        );
        assert_eq!(
            report.contract.runtimes.get("python"),
            Some(&">=3.12".to_string())
        );
        assert_eq!(
            report.contract.runtimes.get("go"),
            Some(&"1.24.0".to_string())
        );
    }

    #[test]
    fn detects_gradle_java_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "settings.gradle.kts",
            r#"rootProject.name = "ota-java-service""#,
        );
        fixture.write(
            "build.gradle.kts",
            r#"java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}"#,
        );
        fixture.write(
            "gradle/wrapper/gradle-wrapper.properties",
            "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.10.2-bin.zip\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-java-service")
        );
        assert_eq!(
            report.contract.runtimes.get("java"),
            Some(&"21".to_string())
        );
        assert_eq!(
            report.contract.tools.get("gradle"),
            Some(&"8.10.2".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("./gradlew build")
        );
    }

    #[test]
    fn detects_maven_java_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pom.xml",
            r#"<project>
  <artifactId>ota-maven-service</artifactId>
  <properties>
    <maven.compiler.release>21</maven.compiler.release>
  </properties>
</project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-maven-service")
        );
        assert_eq!(
            report.contract.runtimes.get("java"),
            Some(&"21".to_string())
        );
        assert_eq!(report.contract.tools.get("maven"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("mvn test")
        );
    }

    #[test]
    fn detects_maven_wrapper_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pom.xml",
            r#"<project>
  <artifactId>ota-maven-service</artifactId>
  <properties>
    <maven.compiler.release>21</maven.compiler.release>
  </properties>
</project>"#,
        );
        fixture.write("mvnw", "#!/bin/sh\n");
        fixture.write(
            ".mvn/wrapper/maven-wrapper.properties",
            "distributionUrl=https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.9/apache-maven-3.9.9-bin.zip\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.tools.get("maven"),
            Some(&"3.9.9".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("./mvnw package")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("./mvnw test")
        );
    }

    #[test]
    fn detects_compose_services() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yml",
            r#"services:
  web:
    build: .
  db:
    image: postgres:16
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.provider.as_deref()),
            Some("docker-compose")
        );
        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.start.as_deref()),
            Some("docker compose up -d web")
        );
        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.stop.as_deref()),
            Some("docker compose stop db")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.web.provider"
                && inference.source == "docker-compose.yml#services.web"
                && inference.confidence == Confidence::High
        }));
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.web.start"
                && inference.source == "docker-compose.yml#services.web"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_compose_service_healthcheck() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yml",
            r#"services:
  db:
    image: postgres:16
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -h localhost -p 5432"]
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.healthcheck.as_deref()),
            Some("pg_isready -h localhost -p 5432")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.healthcheck"
                && inference.source == "docker-compose.yml#services.db.healthcheck.test"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_services_from_docker_compose_yaml() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yaml",
            r#"services:
  web:
    build: .
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("web")
                .and_then(|service| service.provider.as_deref()),
            Some("docker-compose")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.web.provider"
                && inference.source == "docker-compose.yaml#services.web"
                && inference.confidence == Confidence::High
        }));
    }

    #[test]
    fn detects_services_from_compose_yaml() {
        let fixture = Fixture::new();
        fixture.write(
            "compose.yaml",
            r#"services:
  db:
    image: postgres:16
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.start.as_deref()),
            Some("docker compose up -d db")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.start"
                && inference.source == "compose.yaml#services.db"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_services_from_compose_yml() {
        let fixture = Fixture::new();
        fixture.write(
            "compose.yml",
            r#"services:
  cache:
    image: redis:7
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("cache")
                .and_then(|service| service.stop.as_deref()),
            Some("docker compose stop cache")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.cache.stop"
                && inference.source == "compose.yml#services.cache"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_services_from_docker_compose_yaml_with_string_healthcheck() {
        let fixture = Fixture::new();
        fixture.write(
            "docker-compose.yaml",
            r#"services:
  db:
    image: postgres:16
    healthcheck:
      test: "pg_isready -h localhost -p 5432"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.healthcheck.as_deref()),
            Some("pg_isready -h localhost -p 5432")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.healthcheck"
                && inference.source == "docker-compose.yaml#services.db.healthcheck.test"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn detects_compose_service_healthcheck_from_cmd_array() {
        let fixture = Fixture::new();
        fixture.write(
            "compose.yaml",
            r#"services:
  db:
    image: postgres:16
    healthcheck:
      test: ["CMD", "pg_isready", "-h", "localhost", "-p", "5432"]
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .services
                .get("db")
                .and_then(|service| service.healthcheck.as_deref()),
            Some("pg_isready -h localhost -p 5432")
        );
        assert!(report.inferences.iter().any(|inference| {
            inference.field == "services.db.healthcheck"
                && inference.source == "compose.yaml#services.db.healthcheck.test"
                && inference.confidence == Confidence::Medium
        }));
    }

    #[test]
    fn prefers_nvmrc_over_node_version_file() {
        let fixture = Fixture::new();
        fixture.write(".nvmrc", "22\n");
        fixture.write(".node-version", "24\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("node"),
            Some(&"22".to_string())
        );
    }

    #[test]
    fn prefers_python_version_file_over_tool_versions() {
        let fixture = Fixture::new();
        fixture.write(".tool-versions", "python 3.12.4\n");
        fixture.write(".python-version", "3.13.2\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("python"),
            Some(&"3.13.2".to_string())
        );
    }

    #[test]
    fn prefers_go_mod_over_tool_versions() {
        let fixture = Fixture::new();
        fixture.write(".tool-versions", "go 1.23.0\n");
        fixture.write("go.mod", "module github.com/ota/run\n\ngo 1.24.1\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("go"),
            Some(&"1.24.1".to_string())
        );
    }

    #[test]
    fn prefers_package_json_project_name_over_pyproject() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web"
}"#,
        );
        fixture.write(
            "pyproject.toml",
            r#"[project]
name = "ota-api"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-web")
        );
    }

    #[test]
    fn prefers_package_json_package_manager_over_tool_versions() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.4.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".tool-versions", "pnpm 9.0.0\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.tools.get("pnpm"),
            Some(&"10.4.0".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("dev")
                .map(|task| task.run.as_str()),
            Some("pnpm dev")
        );
    }

    #[test]
    fn projects_high_confidence_fields_only_for_write_mode() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-app",
  "engines": { "node": "20" },
  "packageManager": "pnpm@10.2.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        let contract = report.high_confidence_contract();

        assert_eq!(
            contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-app")
        );
        assert_eq!(contract.tools.get("pnpm"), Some(&"10.2.0".to_string()));
        assert_eq!(
            contract.tasks.get("dev").map(|task| task.run.as_str()),
            Some("pnpm dev")
        );
        assert!(!contract.runtimes.contains_key("node"));
    }

    struct Fixture {
        dir: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                dir: TempDir::new().unwrap(),
            }
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.dir.path().join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
    }
}
