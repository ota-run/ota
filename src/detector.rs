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
            "pyproject.toml#project.name" => 4,
            "pyproject.toml#tool.poetry.name" => 3,
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
        _ if field.starts_with("tools.") => match source {
            "package.json#packageManager" => 2,
            ".tool-versions" => 1,
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
            fs::write(self.dir.path().join(relative), contents).unwrap();
        }
    }
}
