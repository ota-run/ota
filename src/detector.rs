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
    #[serde(default, skip_serializing_if = "is_false")]
    pub safe_for_agent: bool,
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
                && let Some((task_name, field_name)) = task_field.split_once('.')
            {
                match field_name {
                    "run" => {
                        contract.tasks.insert(
                            task_name.to_string(),
                            DetectTask {
                                run: inference.value.clone(),
                                safe_for_agent: false,
                            },
                        );
                    }
                    "safe_for_agent" if inference.value == "true" => {
                        if let Some(task) = contract.tasks.get_mut(task_name) {
                            task.safe_for_agent = true;
                        }
                    }
                    _ => {}
                }
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
    detect_composer_json(&root, &mut builder)?;
    detect_nvmrc(&root, &mut builder)?;
    detect_node_version_file(&root, &mut builder)?;
    detect_ruby_version_file(&root, &mut builder)?;
    detect_python_version_file(&root, &mut builder)?;
    detect_java_version_file(&root, &mut builder)?;
    detect_global_json(&root, &mut builder)?;
    detect_sdkmanrc(&root, &mut builder)?;
    detect_go_mod(&root, &mut builder)?;
    detect_rust_toolchain_files(&root, &mut builder)?;
    detect_tool_versions(&root, &mut builder)?;
    detect_pyproject(&root, &mut builder)?;
    detect_pipfile(&root, &mut builder)?;
    detect_uv_lock(&root, &mut builder)?;
    detect_requirements_txt(&root, &mut builder)?;
    detect_setup_cfg(&root, &mut builder)?;
    detect_cargo_toml(&root, &mut builder)?;
    detect_gradle(&root, &mut builder)?;
    detect_pom_xml(&root, &mut builder)?;
    detect_ruby_markers(&root, &mut builder)?;
    detect_dotnet_markers(&root, &mut builder)?;
    detect_mix_exs(&root, &mut builder)?;
    detect_build_sbt(&root, &mut builder)?;
    detect_package_swift(&root, &mut builder)?;
    detect_pubspec_yaml(&root, &mut builder)?;
    detect_cmake(&root, &mut builder)?;
    detect_makefile(&root, &mut builder)?;
    detect_clojure_markers(&root, &mut builder)?;
    detect_haskell_markers(&root, &mut builder)?;
    detect_lua_markers(&root, &mut builder)?;
    detect_julia_markers(&root, &mut builder)?;
    detect_r_markers(&root, &mut builder)?;
    detect_ocaml_markers(&root, &mut builder)?;
    detect_nim_markers(&root, &mut builder)?;
    detect_erlang_markers(&root, &mut builder)?;
    detect_zig_markers(&root, &mut builder)?;
    detect_d_markers(&root, &mut builder)?;
    detect_fortran_markers(&root, &mut builder)?;
    detect_crystal_markers(&root, &mut builder)?;
    detect_elm_markers(&root, &mut builder)?;
    detect_perl_markers(&root, &mut builder)?;
    detect_haxe_markers(&root, &mut builder)?;
    detect_gleam_markers(&root, &mut builder)?;
    detect_v_markers(&root, &mut builder)?;
    detect_ada_markers(&root, &mut builder)?;
    detect_foundry_markers(&root, &mut builder)?;
    detect_kotlin_markers(&root, &mut builder)?;
    detect_fsharp_markers(&root, &mut builder)?;
    detect_tcl_markers(&root, &mut builder)?;
    detect_racket_markers(&root, &mut builder)?;
    detect_bash_markers(&root, &mut builder)?;
    detect_powershell_markers(&root, &mut builder)?;
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
    let mut task_confidence = Confidence::Medium;

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
                task_confidence = Confidence::High;
            }
        }
    } else if let Some((name, source, confidence)) = detect_node_package_manager_marker(root) {
        builder.set_tool(
            name.to_string(),
            "*".to_string(),
            source.to_string(),
            confidence,
        );
        package_manager_name = Some(name.to_string());
        task_confidence = confidence;
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

fn detect_composer_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("composer.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let composer: JsonValue =
        serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(name) = composer.get("name").and_then(JsonValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.to_string(),
            "composer.json#name".to_string(),
            Confidence::High,
        );
    }

    if let Some(runtime) = composer
        .get("config")
        .and_then(|config| config.get("platform"))
        .and_then(|platform| platform.get("php"))
        .and_then(JsonValue::as_str)
        && !runtime.trim().is_empty()
    {
        builder.set_runtime(
            "php".to_string(),
            runtime.trim().to_string(),
            "composer.json#config.platform.php".to_string(),
            Confidence::High,
        );
    } else if let Some(runtime) = composer
        .get("require")
        .and_then(|require| require.get("php"))
        .and_then(JsonValue::as_str)
        && !runtime.trim().is_empty()
    {
        builder.set_runtime(
            "php".to_string(),
            runtime.trim().to_string(),
            "composer.json#require.php".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_tool(
        "composer".to_string(),
        "*".to_string(),
        "composer.json".to_string(),
        Confidence::High,
    );

    if let Some(scripts) = composer.get("scripts").and_then(JsonValue::as_object) {
        for name in scripts.keys() {
            builder.set_task(
                name.to_string(),
                format!("composer run {name}"),
                format!("composer.json#scripts.{name}"),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_node_package_manager_marker(
    root: &Path,
) -> Option<(&'static str, &'static str, Confidence)> {
    [
        ("pnpm", "pnpm-workspace.yaml", Confidence::High),
        ("pnpm", "pnpm-lock.yaml", Confidence::High),
        ("yarn", "yarn.lock", Confidence::High),
        ("bun", "bun.lock", Confidence::High),
        ("bun", "bun.lockb", Confidence::High),
        ("npm", "package-lock.json", Confidence::High),
        ("npm", "npm-shrinkwrap.json", Confidence::High),
    ]
    .into_iter()
    .find(|(_, path, _)| root.join(path).exists())
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

fn detect_ruby_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".ruby-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "ruby".to_string(),
            version,
            ".ruby-version".to_string(),
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
            "rust" => builder.set_runtime(
                "rust".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "java" => builder.set_runtime(
                "java".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "php" => builder.set_runtime(
                "php".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "ruby" => builder.set_runtime(
                "ruby".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "dotnet" => builder.set_runtime(
                "dotnet".to_string(),
                version.to_string(),
                ".tool-versions".to_string(),
                Confidence::High,
            ),
            "elixir" => builder.set_runtime(
                "elixir".to_string(),
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

fn detect_java_version_file(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".java-version");
    if !path.exists() {
        return Ok(());
    }

    let version = read_file(&path)?.trim().trim_start_matches('v').to_string();
    if !version.is_empty() {
        builder.set_runtime(
            "java".to_string(),
            version,
            ".java-version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_global_json(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("global.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let global: JsonValue =
        serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
            path: path.display().to_string(),
            message: source.to_string(),
        })?;

    if let Some(version) = global
        .get("sdk")
        .and_then(|sdk| sdk.get("version"))
        .and_then(JsonValue::as_str)
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "dotnet".to_string(),
            version.trim().to_string(),
            "global.json#sdk.version".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_sdkmanrc(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join(".sdkmanrc");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "java" {
            continue;
        }

        let version = value.trim().trim_start_matches('v').to_string();
        if !version.is_empty() {
            builder.set_runtime(
                "java".to_string(),
                version,
                ".sdkmanrc#java".to_string(),
                Confidence::High,
            );
        }
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

fn detect_pipfile(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Pipfile");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(python) = document
        .get("requires")
        .and_then(|requires| requires.get("python_full_version"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "Pipfile#requires.python_full_version".to_string(),
            Confidence::Medium,
        );
    } else if let Some(python) = document
        .get("requires")
        .and_then(|requires| requires.get("python_version"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "python".to_string(),
            python.to_string(),
            "Pipfile#requires.python_version".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_tool(
        "pipenv".to_string(),
        "*".to_string(),
        "Pipfile".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_uv_lock(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("uv.lock");
    if !path.exists() {
        return Ok(());
    }

    let _ = read_file(&path)?;
    builder.set_tool(
        "uv".to_string(),
        "*".to_string(),
        "uv.lock".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_requirements_txt(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("requirements.txt");
    if !path.exists() {
        return Ok(());
    }

    let _ = read_file(&path)?;
    builder.set_tool(
        "pip".to_string(),
        "*".to_string(),
        "requirements.txt".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_setup_cfg(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("setup.cfg");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let mut section = "";

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = &trimmed[1..trimmed.len() - 1];
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        match (section, key) {
            ("metadata", "name") => builder.set_project_name(
                value.to_string(),
                "setup.cfg#metadata.name".to_string(),
                Confidence::High,
            ),
            ("options", "python_requires") => builder.set_runtime(
                "python".to_string(),
                value.to_string(),
                "setup.cfg#options.python_requires".to_string(),
                Confidence::Medium,
            ),
            _ => {}
        }
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

fn detect_rust_toolchain_files(
    root: &Path,
    builder: &mut DetectBuilder,
) -> Result<(), DetectError> {
    let toml_path = root.join("rust-toolchain.toml");
    if toml_path.exists() {
        let contents = read_file(&toml_path)?;
        let document: TomlValue =
            toml::from_str(&contents).map_err(|source| DetectError::Parse {
                path: toml_path.display().to_string(),
                message: source.to_string(),
            })?;
        if let Some(channel) = document
            .get("toolchain")
            .and_then(|toolchain| toolchain.get("channel"))
            .and_then(TomlValue::as_str)
        {
            builder.set_runtime(
                "rust".to_string(),
                channel.to_string(),
                "rust-toolchain.toml#toolchain.channel".to_string(),
                Confidence::High,
            );
        }
    }

    let path = root.join("rust-toolchain");
    if path.exists() {
        let contents = read_file(&path)?;
        let version = contents
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .unwrap_or_default()
            .trim_start_matches('v')
            .to_string();
        if !version.is_empty() {
            builder.set_runtime(
                "rust".to_string(),
                version,
                "rust-toolchain".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_cargo_toml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Cargo.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(name) = document
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(TomlValue::as_str)
    {
        builder.set_project_name(
            name.to_string(),
            "Cargo.toml#package.name".to_string(),
            Confidence::High,
        );
    }

    if let Some(version) = document
        .get("package")
        .and_then(|package| package.get("rust-version"))
        .and_then(TomlValue::as_str)
    {
        builder.set_runtime(
            "rust".to_string(),
            version.to_string(),
            "Cargo.toml#package.rust-version".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_tool(
        "cargo".to_string(),
        "*".to_string(),
        "Cargo.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "build".to_string(),
        "cargo build".to_string(),
        "Cargo.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "cargo test".to_string(),
        "Cargo.toml".to_string(),
        Confidence::High,
    );

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
            Confidence::High,
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
    let task_confidence = Confidence::High;

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

fn detect_ruby_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let gemfile = root.join("Gemfile");
    if !gemfile.exists() {
        return Ok(());
    }

    builder.set_tool(
        "bundler".to_string(),
        "*".to_string(),
        "Gemfile".to_string(),
        Confidence::High,
    );

    let contents = read_file(&gemfile)?;
    if let Some(version) = extract_ruby_gemfile_version(&contents) {
        builder.set_runtime(
            "ruby".to_string(),
            version,
            "Gemfile#ruby".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_dotnet_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let mut has_dotnet = false;
    let mut project_name = None;

    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if name.ends_with(".sln") || name.ends_with(".csproj") || name.ends_with(".fsproj") {
            has_dotnet = true;
            if project_name.is_none() {
                project_name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(ToString::to_string);
            }
        }
    }

    if !has_dotnet {
        return Ok(());
    }

    builder.set_tool(
        "dotnet".to_string(),
        "*".to_string(),
        "dotnet-project".to_string(),
        Confidence::High,
    );

    if let Some(name) = project_name
        && !name.trim().is_empty()
    {
        builder.set_project_name(name, "dotnet-project".to_string(), Confidence::Medium);
    }

    builder.set_task(
        "build".to_string(),
        "dotnet build".to_string(),
        "dotnet-project".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "dotnet test".to_string(),
        "dotnet-project".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_mix_exs(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("mix.exs");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "mix".to_string(),
        "*".to_string(),
        "mix.exs".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "mix test".to_string(),
        "mix.exs".to_string(),
        Confidence::High,
    );

    if let Some(app) = extract_mix_app_name(&contents) {
        builder.set_project_name(app, "mix.exs#project.app".to_string(), Confidence::High);
    }
    if let Some(version) = extract_mix_elixir_version(&contents) {
        builder.set_runtime(
            "elixir".to_string(),
            version,
            "mix.exs#project.elixir".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_build_sbt(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("build.sbt");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "sbt".to_string(),
        "*".to_string(),
        "build.sbt".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_sbt_quoted_assignment(&contents, "name") {
        builder.set_project_name(name, "build.sbt#name".to_string(), Confidence::High);
    }

    if let Some(version) = extract_sbt_quoted_assignment(&contents, "scalaVersion") {
        builder.set_runtime(
            "scala".to_string(),
            version,
            "build.sbt#scalaVersion".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "sbt compile".to_string(),
        "build.sbt#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "sbt test".to_string(),
        "build.sbt#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "run".to_string(),
        "sbt run".to_string(),
        "build.sbt#standard-tasks".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_package_swift(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Package.swift");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "swift".to_string(),
        "*".to_string(),
        "Package.swift".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_package_swift_name(&contents) {
        builder.set_project_name(name, "Package.swift#name".to_string(), Confidence::High);
    }

    builder.set_task(
        "build".to_string(),
        "swift build".to_string(),
        "Package.swift#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "swift test".to_string(),
        "Package.swift#standard-tasks".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "run".to_string(),
        "swift run".to_string(),
        "Package.swift#standard-tasks".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_pubspec_yaml(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pubspec.yaml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let pubspec: YamlValue = serde_yaml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    if let Some(name) = yaml_key_str(&pubspec, "name")
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "pubspec.yaml#name".to_string(),
            Confidence::High,
        );
    }

    builder.set_tool(
        "dart".to_string(),
        "*".to_string(),
        "pubspec.yaml".to_string(),
        Confidence::High,
    );

    if let Some(sdk) = yaml_nested_key_str(&pubspec, &["environment", "sdk"])
        && !sdk.trim().is_empty()
    {
        builder.set_runtime(
            "dart".to_string(),
            sdk.trim().to_string(),
            "pubspec.yaml#environment.sdk".to_string(),
            Confidence::High,
        );
    }

    if yaml_mapping_has_key(&pubspec, "flutter") {
        builder.set_tool(
            "flutter".to_string(),
            "*".to_string(),
            "pubspec.yaml#flutter".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "flutter build".to_string(),
            "pubspec.yaml#flutter-standard-tasks".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "flutter test".to_string(),
            "pubspec.yaml#flutter-standard-tasks".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "run".to_string(),
            "flutter run".to_string(),
            "pubspec.yaml#flutter-standard-tasks".to_string(),
            Confidence::High,
        );
    } else {
        builder.set_task(
            "test".to_string(),
            "dart test".to_string(),
            "pubspec.yaml#standard-tasks".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "run".to_string(),
            "dart run".to_string(),
            "pubspec.yaml#standard-tasks".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_cmake(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("CMakeLists.txt");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "cmake".to_string(),
        "*".to_string(),
        "CMakeLists.txt".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_cmake_project_name(&contents) {
        builder.set_project_name(
            name,
            "CMakeLists.txt#project".to_string(),
            Confidence::High,
        );
    }
    if let Some(version) = extract_cmake_standard(&contents, "CMAKE_C_STANDARD") {
        builder.set_runtime(
            "c".to_string(),
            version,
            "CMakeLists.txt#CMAKE_C_STANDARD".to_string(),
            Confidence::Medium,
        );
    }
    if let Some(version) = extract_cmake_standard(&contents, "CMAKE_CXX_STANDARD") {
        builder.set_runtime(
            "cpp".to_string(),
            version,
            "CMakeLists.txt#CMAKE_CXX_STANDARD".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "cmake -S . -B build && cmake --build build".to_string(),
        "CMakeLists.txt".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "ctest --test-dir build".to_string(),
        "CMakeLists.txt".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_makefile(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let source = ["Makefile", "GNUmakefile", "makefile"]
        .into_iter()
        .find(|name| root.join(name).exists());
    let Some(source) = source else {
        return Ok(());
    };

    builder.set_tool(
        "make".to_string(),
        "*".to_string(),
        source.to_string(),
        Confidence::High,
    );
    builder.set_task(
        "build".to_string(),
        "make".to_string(),
        source.to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "make test".to_string(),
        source.to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_clojure_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let project_path = root.join("project.clj");
    if project_path.exists() {
        let contents = read_file(&project_path)?;
        builder.set_tool(
            "leiningen".to_string(),
            "*".to_string(),
            "project.clj".to_string(),
            Confidence::High,
        );
        if let Some(name) = extract_clojure_defproject_name(&contents) {
            builder.set_project_name(name, "project.clj#defproject".to_string(), Confidence::High);
        }
        builder.set_task(
            "build".to_string(),
            "lein uberjar".to_string(),
            "project.clj".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "lein test".to_string(),
            "project.clj".to_string(),
            Confidence::High,
        );
    }

    let deps_path = root.join("deps.edn");
    if deps_path.exists() {
        builder.set_tool(
            "clojure".to_string(),
            "*".to_string(),
            "deps.edn".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "clojure -T:test".to_string(),
            "deps.edn".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_haskell_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    if root.join("stack.yaml").exists() {
        builder.set_tool(
            "stack".to_string(),
            "*".to_string(),
            "stack.yaml".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "stack build".to_string(),
            "stack.yaml".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "stack test".to_string(),
            "stack.yaml".to_string(),
            Confidence::High,
        );
    }

    let mut cabal_name = None;
    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("cabal"))
        {
            cabal_name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(ToString::to_string);
            break;
        }
    }

    if let Some(name) = cabal_name {
        builder.set_project_name(name, "cabal-file".to_string(), Confidence::High);
        builder.set_tool(
            "cabal".to_string(),
            "*".to_string(),
            "cabal-file".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "build".to_string(),
            "cabal build".to_string(),
            "cabal-file".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "cabal test".to_string(),
            "cabal-file".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_lua_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let mut rockspec_name = None;
    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("rockspec"))
        {
            rockspec_name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(ToString::to_string);
            break;
        }
    }

    let Some(name) = rockspec_name else {
        return Ok(());
    };

    builder.set_project_name(name, "rockspec".to_string(), Confidence::High);
    builder.set_tool(
        "luarocks".to_string(),
        "*".to_string(),
        "rockspec".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "build".to_string(),
        "luarocks make".to_string(),
        "rockspec".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "luarocks test".to_string(),
        "rockspec".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_julia_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("Project.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "julia".to_string(),
        "*".to_string(),
        "Project.toml".to_string(),
        Confidence::High,
    );
    if let Some(name) = document.get("name").and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(name.trim().to_string(), "Project.toml#name".to_string(), Confidence::High);
    }
    if let Some(version) = document
        .get("compat")
        .and_then(|compat| compat.get("julia"))
        .and_then(TomlValue::as_str)
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "julia".to_string(),
            version.trim().to_string(),
            "Project.toml#compat.julia".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "julia --project=. -e 'using Pkg; Pkg.build()'".to_string(),
        "Project.toml".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "julia --project=. -e 'using Pkg; Pkg.test()'".to_string(),
        "Project.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_r_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("DESCRIPTION");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "r".to_string(),
        "*".to_string(),
        "DESCRIPTION".to_string(),
        Confidence::High,
    );

    if let Some(name) = extract_dcf_value(&contents, "Package")
        && !name.trim().is_empty()
    {
        builder.set_project_name(name.trim().to_string(), "DESCRIPTION#Package".to_string(), Confidence::High);
    }
    if let Some(depends) = extract_dcf_value(&contents, "Depends")
        && let Some(version) = extract_r_depends_version(&depends)
    {
        builder.set_runtime(
            "r".to_string(),
            version,
            "DESCRIPTION#Depends.R".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "R CMD build .".to_string(),
        "DESCRIPTION".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "check".to_string(),
        "R CMD check .".to_string(),
        "DESCRIPTION".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_ocaml_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let dune_path = root.join("dune-project");
    let opam_file = find_extension_file(root, "opam")?;
    let ocaml_version_path = root.join(".ocaml-version");

    if !dune_path.exists() && opam_file.is_none() && !ocaml_version_path.exists() {
        return Ok(());
    }

    if dune_path.exists() {
        let contents = read_file(&dune_path)?;
        builder.set_tool(
            "dune".to_string(),
            "*".to_string(),
            "dune-project".to_string(),
            Confidence::High,
        );
        if let Some(name) = extract_dune_project_name(&contents)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "dune-project#name".to_string(),
                Confidence::High,
            );
        }
        builder.set_task(
            "build".to_string(),
            "dune build".to_string(),
            "dune-project".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "dune runtest".to_string(),
            "dune-project".to_string(),
            Confidence::High,
        );
    }

    if opam_file.is_some() {
        builder.set_tool(
            "opam".to_string(),
            "*".to_string(),
            "opam-file".to_string(),
            Confidence::Medium,
        );
    }

    if ocaml_version_path.exists() {
        let version = read_file(&ocaml_version_path)?.trim().to_string();
        if !version.is_empty() {
            builder.set_runtime(
                "ocaml".to_string(),
                version,
                ".ocaml-version".to_string(),
                Confidence::High,
            );
        }
    }

    Ok(())
}

fn detect_nim_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let Some(path) = find_extension_file(root, "nimble")? else {
        return Ok(());
    };

    builder.set_tool(
        "nimble".to_string(),
        "*".to_string(),
        "nimble-file".to_string(),
        Confidence::High,
    );

    if let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
        && !name.trim().is_empty()
    {
        builder.set_project_name(name.trim().to_string(), "nimble-file".to_string(), Confidence::High);
    }

    let contents = read_file(&path)?;
    if let Some(version) = extract_nimble_requires_version(&contents) {
        builder.set_runtime(
            "nim".to_string(),
            version,
            "nimble-file#requires.nim".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "nimble build".to_string(),
        "nimble-file".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "nimble test".to_string(),
        "nimble-file".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_erlang_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("rebar.config");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "rebar3".to_string(),
        "*".to_string(),
        "rebar.config".to_string(),
        Confidence::High,
    );
    if let Some(name) = extract_rebar_app_name(&contents)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "rebar.config#app".to_string(),
            Confidence::High,
        );
    }
    builder.set_task(
        "build".to_string(),
        "rebar3 compile".to_string(),
        "rebar.config".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "rebar3 eunit".to_string(),
        "rebar.config".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_zig_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("build.zig");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "zig".to_string(),
        "*".to_string(),
        "build.zig".to_string(),
        Confidence::High,
    );
    if let Some(version) = extract_zig_build_api_version(&contents) {
        builder.set_runtime(
            "zig".to_string(),
            version,
            "build.zig#std.Build".to_string(),
            Confidence::Medium,
        );
    }
    builder.set_task(
        "build".to_string(),
        "zig build".to_string(),
        "build.zig".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "zig build test".to_string(),
        "build.zig".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_d_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let dub_json = root.join("dub.json");
    let dub_sdl = root.join("dub.sdl");

    if dub_json.exists() {
        let contents = read_file(&dub_json)?;
        let document: JsonValue =
            serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
                path: dub_json.display().to_string(),
                message: source.to_string(),
            })?;

        builder.set_tool(
            "dub".to_string(),
            "*".to_string(),
            "dub.json".to_string(),
            Confidence::High,
        );
        if let Some(name) = document.get("name").and_then(JsonValue::as_str)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "dub.json#name".to_string(),
                Confidence::High,
            );
        }
        builder.set_task(
            "build".to_string(),
            "dub build".to_string(),
            "dub.json".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "dub test".to_string(),
            "dub.json".to_string(),
            Confidence::High,
        );
        return Ok(());
    }

    if dub_sdl.exists() {
        let contents = read_file(&dub_sdl)?;
        builder.set_tool(
            "dub".to_string(),
            "*".to_string(),
            "dub.sdl".to_string(),
            Confidence::High,
        );
        if let Some(name) = extract_dub_sdl_name(&contents)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "dub.sdl#name".to_string(),
                Confidence::High,
            );
        }
        builder.set_task(
            "build".to_string(),
            "dub build".to_string(),
            "dub.sdl".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "dub test".to_string(),
            "dub.sdl".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_fortran_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("fpm.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "fpm".to_string(),
        "*".to_string(),
        "fpm.toml".to_string(),
        Confidence::High,
    );

    if let Some(name) = document
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "fpm.toml#project.name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "fpm build".to_string(),
        "fpm.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "fpm test".to_string(),
        "fpm.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_crystal_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("shard.yml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let shard: YamlValue = serde_yaml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "crystal".to_string(),
        "*".to_string(),
        "shard.yml".to_string(),
        Confidence::High,
    );

    if let Some(name) = yaml_key_str(&shard, "name")
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "shard.yml#name".to_string(),
            Confidence::High,
        );
    }
    if let Some(version) = yaml_key_str(&shard, "crystal")
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "crystal".to_string(),
            version.trim().to_string(),
            "shard.yml#crystal".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "shards build".to_string(),
        "shard.yml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "crystal spec".to_string(),
        "shard.yml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_elm_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("elm.json");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let elm: JsonValue = serde_json::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "elm".to_string(),
        "*".to_string(),
        "elm.json".to_string(),
        Confidence::High,
    );
    if let Some(name) = elm.get("name").and_then(JsonValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "elm.json#name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "elm make src/Main.elm".to_string(),
        "elm.json".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "elm-test".to_string(),
        "elm.json".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_perl_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let cpanfile = root.join("cpanfile");
    if cpanfile.exists() {
        builder.set_tool(
            "cpanm".to_string(),
            "*".to_string(),
            "cpanfile".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "setup".to_string(),
            "cpanm --installdeps .".to_string(),
            "cpanfile".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "prove -lr t".to_string(),
            "cpanfile".to_string(),
            Confidence::Medium,
        );
    }

    let makefile_pl = root.join("Makefile.PL");
    if makefile_pl.exists() {
        builder.set_tool(
            "perl".to_string(),
            "*".to_string(),
            "Makefile.PL".to_string(),
            Confidence::High,
        );
        if let Ok(contents) = read_file(&makefile_pl)
            && let Some(name) = extract_makefile_pl_name(&contents)
            && !name.trim().is_empty()
        {
            builder.set_project_name(
                name.trim().to_string(),
                "Makefile.PL#name".to_string(),
                Confidence::Medium,
            );
        }
        builder.set_task(
            "build".to_string(),
            "perl Makefile.PL && make".to_string(),
            "Makefile.PL".to_string(),
            Confidence::High,
        );
        builder.set_task(
            "test".to_string(),
            "make test".to_string(),
            "Makefile.PL".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_haxe_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let Some(hxml) = find_extension_file(root, "hxml")? else {
        return Ok(());
    };

    builder.set_tool(
        "haxe".to_string(),
        "*".to_string(),
        "hxml".to_string(),
        Confidence::High,
    );
    if let Some(name) = hxml.file_stem().and_then(|stem| stem.to_str())
        && !name.trim().is_empty()
    {
        builder.set_project_name(name.trim().to_string(), "hxml".to_string(), Confidence::Medium);
        builder.set_task(
            "build".to_string(),
            format!("haxe {}.hxml", name.trim()),
            "hxml".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_gleam_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("gleam.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "gleam".to_string(),
        "*".to_string(),
        "gleam.toml".to_string(),
        Confidence::High,
    );
    if let Some(name) = document.get("name").and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "gleam.toml#name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "gleam build".to_string(),
        "gleam.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "gleam test".to_string(),
        "gleam.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_v_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("v.mod");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    builder.set_tool(
        "v".to_string(),
        "*".to_string(),
        "v.mod".to_string(),
        Confidence::High,
    );
    if let Some(name) = extract_v_mod_name(&contents)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "v.mod#name".to_string(),
            Confidence::High,
        );
    }
    builder.set_task(
        "build".to_string(),
        "v .".to_string(),
        "v.mod".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "v test .".to_string(),
        "v.mod".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_ada_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("alire.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "alr".to_string(),
        "*".to_string(),
        "alire.toml".to_string(),
        Confidence::High,
    );
    if let Some(name) = document
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
        && !name.trim().is_empty()
    {
        builder.set_project_name(
            name.trim().to_string(),
            "alire.toml#project.name".to_string(),
            Confidence::High,
        );
    }

    builder.set_task(
        "build".to_string(),
        "alr build".to_string(),
        "alire.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "alr test".to_string(),
        "alire.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_foundry_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("foundry.toml");
    if !path.exists() {
        return Ok(());
    }

    let contents = read_file(&path)?;
    let document: TomlValue = toml::from_str(&contents).map_err(|source| DetectError::Parse {
        path: path.display().to_string(),
        message: source.to_string(),
    })?;

    builder.set_tool(
        "forge".to_string(),
        "*".to_string(),
        "foundry.toml".to_string(),
        Confidence::High,
    );
    if let Some(version) = document
        .get("profile")
        .and_then(|profile| profile.get("default"))
        .and_then(|default| default.get("solc_version"))
        .and_then(TomlValue::as_str)
        && !version.trim().is_empty()
    {
        builder.set_runtime(
            "solidity".to_string(),
            version.trim().to_string(),
            "foundry.toml#profile.default.solc_version".to_string(),
            Confidence::Medium,
        );
    }

    builder.set_task(
        "build".to_string(),
        "forge build".to_string(),
        "foundry.toml".to_string(),
        Confidence::High,
    );
    builder.set_task(
        "test".to_string(),
        "forge test".to_string(),
        "foundry.toml".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_kotlin_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let path = root.join("pom.xml");
    if path.exists() {
        let contents = read_file(&path)?;
        if contents.contains("kotlin-maven-plugin")
            || contents.contains("<kotlin.version>")
            || contents.contains("org.jetbrains.kotlin")
        {
            builder.set_runtime(
                "kotlin".to_string(),
                extract_xml_tag(&contents, "kotlin.version").unwrap_or_else(|| "*".to_string()),
                "pom.xml#kotlin.version".to_string(),
                Confidence::Medium,
            );
        }
    }

    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("kts"))
        {
            builder.set_tool(
                "kotlin".to_string(),
                "*".to_string(),
                "kotlin-script".to_string(),
                Confidence::High,
            );
            if let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
                && !name.trim().is_empty()
            {
                builder.set_project_name(
                    name.trim().to_string(),
                    "kotlin-script".to_string(),
                    Confidence::Medium,
                );
            }
            let script = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("main.kts");
            builder.set_task(
                "run".to_string(),
                format!("kotlin {script}"),
                "kotlin-script".to_string(),
                Confidence::High,
            );
            break;
        }
    }

    Ok(())
}

fn detect_fsharp_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let mut has_fsharp = false;
    let mut project_name = None;

    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("fsproj"))
        {
            has_fsharp = true;
            if project_name.is_none() {
                project_name = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(ToString::to_string);
            }
        }
    }

    if !has_fsharp {
        return Ok(());
    }

    builder.set_tool(
        "dotnet".to_string(),
        "*".to_string(),
        "fsharp-project".to_string(),
        Confidence::High,
    );
    if let Some(name) = project_name
        && !name.trim().is_empty()
    {
        builder.set_project_name(name, "fsharp-project".to_string(), Confidence::Medium);
    }
    builder.set_runtime(
        "fsharp".to_string(),
        "*".to_string(),
        "fsharp-project".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "build".to_string(),
        "dotnet build".to_string(),
        "fsharp-project".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "test".to_string(),
        "dotnet test".to_string(),
        "fsharp-project".to_string(),
        Confidence::Medium,
    );

    Ok(())
}

fn detect_tcl_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let tclapp = root.join("tclapp.tcl");
    let pkg_index = root.join("pkgIndex.tcl");
    if !tclapp.exists() && !pkg_index.exists() {
        return Ok(());
    }

    builder.set_tool(
        "tclsh".to_string(),
        "*".to_string(),
        if tclapp.exists() {
            "tclapp.tcl".to_string()
        } else {
            "pkgIndex.tcl".to_string()
        },
        Confidence::High,
    );
    if tclapp.exists() {
        builder.set_project_name(
            "tclapp".to_string(),
            "tclapp.tcl".to_string(),
            Confidence::Low,
        );
        builder.set_task(
            "run".to_string(),
            "tclsh tclapp.tcl".to_string(),
            "tclapp.tcl".to_string(),
            Confidence::High,
        );
    }

    Ok(())
}

fn detect_racket_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let info = root.join("info.rkt");
    let main = root.join("main.rkt");
    if !info.exists() && !main.exists() {
        return Ok(());
    }

    builder.set_tool(
        "racket".to_string(),
        "*".to_string(),
        if info.exists() {
            "info.rkt".to_string()
        } else {
            "main.rkt".to_string()
        },
        Confidence::High,
    );

    if main.exists() {
        builder.set_task(
            "run".to_string(),
            "racket main.rkt".to_string(),
            "main.rkt".to_string(),
            Confidence::High,
        );
    }
    if info.exists() {
        builder.set_task(
            "test".to_string(),
            "raco test .".to_string(),
            "info.rkt".to_string(),
            Confidence::Medium,
        );
    }

    Ok(())
}

fn detect_bash_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let script = if root.join("main.sh").exists() {
        Some("main.sh".to_string())
    } else if root.join("run.sh").exists() {
        Some("run.sh".to_string())
    } else {
        find_extension_file(root, "sh")?
            .and_then(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
    };

    let Some(script) = script else {
        return Ok(());
    };

    builder.set_tool(
        "bash".to_string(),
        "*".to_string(),
        "bash-script".to_string(),
        Confidence::High,
    );
    builder.set_runtime(
        "shell".to_string(),
        "*".to_string(),
        "bash-script".to_string(),
        Confidence::Medium,
    );
    builder.set_project_name(
        Path::new(&script)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("shell-app")
            .to_string(),
        "bash-script".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "run".to_string(),
        format!("bash {script}"),
        "bash-script".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn detect_powershell_markers(root: &Path, builder: &mut DetectBuilder) -> Result<(), DetectError> {
    let Some(script) = find_extension_file(root, "ps1")?
        .and_then(|path| path.file_name().map(|name| name.to_string_lossy().to_string()))
    else {
        return Ok(());
    };

    builder.set_tool(
        "pwsh".to_string(),
        "*".to_string(),
        "powershell-script".to_string(),
        Confidence::High,
    );
    builder.set_runtime(
        "powershell".to_string(),
        "*".to_string(),
        "powershell-script".to_string(),
        Confidence::Medium,
    );
    builder.set_project_name(
        Path::new(&script)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("powershell-app")
            .to_string(),
        "powershell-script".to_string(),
        Confidence::Medium,
    );
    builder.set_task(
        "run".to_string(),
        format!("pwsh -File {script}"),
        "powershell-script".to_string(),
        Confidence::High,
    );

    Ok(())
}

fn extract_ruby_gemfile_version(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("ruby ") {
            continue;
        }
        let mut quote = None;
        let mut start = 0usize;
        for (idx, ch) in trimmed.char_indices() {
            if ch == '\'' || ch == '"' {
                quote = Some(ch);
                start = idx + 1;
                break;
            }
        }
        let quote = quote?;
        let rest = &trimmed[start..];
        let end = rest.find(quote)?;
        let version = rest[..end].trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
}

fn extract_mix_app_name(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("app:") {
            continue;
        }
        let marker = "app:";
        let start = trimmed.find(marker)? + marker.len();
        let rest = trimmed[start..].trim_start();
        let symbol = rest.strip_prefix(':')?;
        let end = symbol
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .unwrap_or(symbol.len());
        let name = &symbol[..end];
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

fn extract_mix_elixir_version(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("elixir:") {
            continue;
        }
        let marker = "elixir:";
        let start = trimmed.find(marker)? + marker.len();
        let rest = trimmed[start..].trim_start();
        let quote = rest.chars().next()?;
        if quote != '"' && quote != '\'' {
            continue;
        }
        let rest = &rest[1..];
        let end = rest.find(quote)?;
        let version = rest[..end].trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
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
        let version = url.split("apache-maven-").nth(1)?.split('-').next()?.trim();
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

fn extract_sbt_quoted_assignment(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix(key)?.trim_start();
        let rest = rest.strip_prefix(":=")?.trim_start();
        extract_quoted_string(rest)
    })
}

fn extract_package_swift_name(contents: &str) -> Option<String> {
    let mut in_package_decl = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if !in_package_decl && trimmed.contains("let package = Package(") {
            in_package_decl = true;
            continue;
        }
        if !in_package_decl {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name:") {
            return extract_quoted_string(rest.trim_start().trim_end_matches(','));
        }
    }

    None
}

fn yaml_key_str<'a>(value: &'a YamlValue, key: &str) -> Option<&'a str> {
    value
        .as_mapping()?
        .get(&YamlValue::String(key.to_string()))?
        .as_str()
}

fn yaml_nested_key_str<'a>(value: &'a YamlValue, keys: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in keys {
        current = current
            .as_mapping()?
            .get(&YamlValue::String((*key).to_string()))?;
    }
    current.as_str()
}

fn yaml_mapping_has_key(value: &YamlValue, key: &str) -> bool {
    value
        .as_mapping()
        .is_some_and(|mapping| mapping.contains_key(&YamlValue::String(key.to_string())))
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

fn extract_cmake_project_name(contents: &str) -> Option<String> {
    let start = contents.find("project(")? + "project(".len();
    let end = contents[start..].find(')')? + start;
    let inside = contents[start..end].trim();
    let first = inside.split_whitespace().next()?.trim_matches('"').trim();
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

fn extract_cmake_standard(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.contains(key) {
            return None;
        }
        line.split(|ch: char| !ch.is_ascii_digit())
            .find(|part| !part.is_empty())
            .map(ToString::to_string)
    })
}

fn extract_clojure_defproject_name(contents: &str) -> Option<String> {
    let start = contents.find("(defproject ")? + "(defproject ".len();
    let tail = contents[start..].trim_start();
    let token = tail
        .split(|ch: char| ch.is_whitespace() || ch == ')' || ch == '[' || ch == '(')
        .next()?
        .trim_matches('"')
        .trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_dcf_value(contents: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    contents.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&prefix).map(|value| value.trim().to_string())
    })
}

fn extract_r_depends_version(depends: &str) -> Option<String> {
    let start = depends.find("R (")?;
    let after = &depends[start + 3..];
    let end = after.find(')')?;
    let token = after[..end].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_dune_project_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        let inside = line.strip_prefix("(name ")?.strip_suffix(')')?.trim();
        if inside.is_empty() {
            None
        } else {
            Some(inside.to_string())
        }
    })
}

fn extract_nimble_requires_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("requires") || !line.contains("nim") {
            return None;
        }
        let quote = if line.contains('"') { '"' } else { '\'' };
        let start = line.find(quote)? + 1;
        let end = line[start..].find(quote)? + start;
        let value = line[start..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn extract_rebar_app_name(contents: &str) -> Option<String> {
    let start = contents.find("{app,")? + "{app,".len();
    let rest = contents[start..].trim_start();
    let end = rest.find('}')?;
    let token = rest[..end]
        .split(',')
        .next()
        .map(str::trim)?
        .trim_start_matches('\'')
        .trim_start_matches("<<")
        .trim_end_matches(">>")
        .trim_end_matches('\'')
        .trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn extract_zig_build_api_version(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.contains("std.Build") {
            return None;
        }
        let digits = line
            .split(|ch: char| !ch.is_ascii_digit() && ch != '.')
            .find(|part| part.contains('.') && part.chars().any(|ch| ch.is_ascii_digit()))?;
        let digits = digits.trim_start_matches('.');
        if digits.is_empty() {
            None
        } else {
            Some(digits.to_string())
        }
    })
}

fn extract_dub_sdl_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("name") {
            return None;
        }
        let quote = if line.contains('"') { '"' } else { '\'' };
        let start = line.find(quote)? + 1;
        let end = line[start..].find(quote)? + start;
        let value = line[start..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn extract_makefile_pl_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.to_ascii_lowercase().contains("name") {
            return None;
        }
        let quote = if line.contains('"') { '"' } else { '\'' };
        let start = line.find(quote)? + 1;
        let end = line[start..].find(quote)? + start;
        let value = line[start..end].trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn extract_v_mod_name(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("name:") {
            return None;
        }
        let value = line
            .trim_start_matches("name:")
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn find_extension_file(root: &Path, extension: &str) -> Result<Option<PathBuf>, DetectError> {
    for entry in fs::read_dir(root).map_err(|source| DetectError::Read {
        path: root.display().to_string(),
        source,
    })? {
        let entry = entry.map_err(|source| DetectError::Read {
            path: root.display().to_string(),
            source,
        })?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
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
                .insert(
                    name.clone(),
                    DetectTask {
                        run: run.clone(),
                        safe_for_agent: false,
                    },
                );
            self.record(field, run, source.clone(), confidence);
            if is_verifier_task_name(&name) {
                self.set_task_safe_for_agent(name, source, confidence);
            }
        }
    }

    fn set_task_safe_for_agent(&mut self, name: String, source: String, confidence: Confidence) {
        let field = format!("tasks.{name}.safe_for_agent");
        if !self.should_replace(&field, &source, confidence) {
            return;
        }
        if let Some(task) = self.contract.tasks.get_mut(&name) {
            task.safe_for_agent = true;
            self.record(field, String::from("true"), source, confidence);
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
            "Cargo.toml#package.name" => 4,
            "pyproject.toml#project.name" => 4,
            "setup.cfg#metadata.name" => 4,
            "pubspec.yaml#name" => 4,
            "build.sbt#name" => 4,
            "Package.swift#name" => 4,
            "CMakeLists.txt#project" => 4,
            "project.clj#defproject" => 4,
            "Project.toml#name" => 4,
            "DESCRIPTION#Package" => 4,
            "dune-project#name" => 4,
            "rebar.config#app" => 4,
            "pyproject.toml#tool.poetry.name" => 3,
            "pom.xml#artifactId" => 3,
            "composer.json#name" => 3,
            "mix.exs#project.app" => 3,
            "cabal-file" => 3,
            "rockspec" => 3,
            "nimble-file" => 3,
            "dub.json#name" => 3,
            "dub.sdl#name" => 3,
            "fpm.toml#project.name" => 3,
            "shard.yml#name" => 3,
            "elm.json#name" => 3,
            "Makefile.PL#name" => 3,
            "gleam.toml#name" => 3,
            "v.mod#name" => 3,
            "alire.toml#project.name" => 3,
            "fsharp-project" => 3,
            "powershell-script" => 3,
            "dotnet-project" => 2,
            "go.mod#module" => 2,
            "hxml" => 2,
            "kotlin-script" => 2,
            "bash-script" => 2,
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
            "setup.cfg#options.python_requires" => 2,
            "pyproject.toml#tool.poetry.dependencies.python" => 1,
            "Pipfile#requires.python_full_version" => 1,
            "Pipfile#requires.python_version" => 1,
            _ => 0,
        },
        "runtimes.go" => match source {
            "go.mod#go" => 2,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.java" => match source {
            ".java-version" => 5,
            ".sdkmanrc#java" => 4,
            "build.gradle.kts#java.toolchain" => 3,
            "build.gradle#java.toolchain" => 3,
            "pom.xml#maven.compiler.release" => 2,
            "pom.xml#maven.compiler.target" => 2,
            "pom.xml#maven.compiler.source" => 2,
            ".tool-versions" => 1,
            "pom.xml#java.version" => 1,
            _ => 0,
        },
        "runtimes.rust" => match source {
            "rust-toolchain.toml#toolchain.channel" => 3,
            "rust-toolchain" => 2,
            ".tool-versions" => 1,
            "Cargo.toml#package.rust-version" => 1,
            _ => 0,
        },
        "runtimes.php" => match source {
            "composer.json#config.platform.php" => 3,
            "composer.json#require.php" => 2,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.ruby" => match source {
            ".ruby-version" => 3,
            "Gemfile#ruby" => 2,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.dotnet" => match source {
            "global.json#sdk.version" => 3,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.elixir" => match source {
            "mix.exs#project.elixir" => 2,
            ".tool-versions" => 1,
            _ => 0,
        },
        "runtimes.scala" => match source {
            "build.sbt#scalaVersion" => 2,
            _ => 0,
        },
        "runtimes.dart" => match source {
            "pubspec.yaml#environment.sdk" => 2,
            _ => 0,
        },
        "runtimes.julia" => match source {
            "Project.toml#compat.julia" => 2,
            _ => 0,
        },
        "runtimes.r" => match source {
            "DESCRIPTION#Depends.R" => 2,
            _ => 0,
        },
        "runtimes.ocaml" => match source {
            ".ocaml-version" => 2,
            _ => 0,
        },
        "runtimes.nim" => match source {
            "nimble-file#requires.nim" => 2,
            _ => 0,
        },
        "runtimes.zig" => match source {
            "build.zig#std.Build" => 2,
            _ => 0,
        },
        "runtimes.crystal" => match source {
            "shard.yml#crystal" => 2,
            _ => 0,
        },
        "runtimes.solidity" => match source {
            "foundry.toml#profile.default.solc_version" => 2,
            _ => 0,
        },
        "runtimes.fsharp" => match source {
            "fsharp-project" => 2,
            _ => 0,
        },
        "runtimes.kotlin" => match source {
            "pom.xml#kotlin.version" => 2,
            _ => 0,
        },
        "runtimes.shell" => match source {
            "bash-script" => 2,
            _ => 0,
        },
        "runtimes.powershell" => match source {
            "powershell-script" => 2,
            _ => 0,
        },
        "runtimes.c" => match source {
            "CMakeLists.txt#CMAKE_C_STANDARD" => 2,
            _ => 0,
        },
        "runtimes.cpp" => match source {
            "CMakeLists.txt#CMAKE_CXX_STANDARD" => 2,
            _ => 0,
        },
        _ if field.starts_with("tools.") => match source {
            "gradle/wrapper/gradle-wrapper.properties#distributionUrl" => 3,
            ".mvn/wrapper/maven-wrapper.properties#distributionUrl" => 3,
            "stack.yaml" => 3,
            "package.json#packageManager" => 2,
            "composer.json" => 2,
            "Gemfile" => 2,
            "dotnet-project" => 2,
            "mix.exs" => 2,
            "build.sbt" => 2,
            "Package.swift" => 2,
            "pubspec.yaml" => 2,
            "pubspec.yaml#flutter" => 2,
            "CMakeLists.txt" => 2,
            "project.clj" => 2,
            "deps.edn" => 2,
            "cabal-file" => 2,
            "rockspec" => 2,
            "Project.toml" => 2,
            "DESCRIPTION" => 2,
            "dune-project" => 2,
            "opam-file" => 2,
            "nimble-file" => 2,
            "rebar.config" => 2,
            "build.zig" => 2,
            "dub.json" => 2,
            "dub.sdl" => 2,
            "fpm.toml" => 2,
            "shard.yml" => 2,
            "elm.json" => 2,
            "cpanfile" => 2,
            "Makefile.PL" => 2,
            "hxml" => 2,
            "gleam.toml" => 2,
            "v.mod" => 2,
            "alire.toml" => 2,
            "foundry.toml" => 2,
            "fsharp-project" => 2,
            "kotlin-script" => 2,
            "bash-script" => 2,
            "powershell-script" => 2,
            "tclapp.tcl" => 2,
            "pkgIndex.tcl" => 2,
            "info.rkt" => 2,
            "main.rkt" => 2,
            "pnpm-workspace.yaml" => 2,
            "pnpm-lock.yaml" => 2,
            "yarn.lock" => 2,
            "bun.lock" => 2,
            "bun.lockb" => 2,
            "package-lock.json" => 2,
            "npm-shrinkwrap.json" => 2,
            "uv.lock" => 2,
            "Cargo.toml" => 2,
            "requirements.txt" => 1,
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

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_verifier_task_name(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|token| {
            matches!(
                token,
                "test" | "tests" | "lint" | "typecheck" | "check" | "verify" | "fmt" | "format"
            )
        })
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
    fn detects_composer_php_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "composer.json",
            r#"{
  "name": "qredex/php-app",
  "require": {
    "php": "^8.2"
  },
  "scripts": {
    "test": "phpunit",
    "serve": "php -S localhost:8000 -t public"
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex/php-app")
        );
        assert_eq!(
            report.contract.runtimes.get("php"),
            Some(&"^8.2".to_string())
        );
        assert_eq!(report.contract.tools.get("composer"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("composer run test")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("serve")
                .map(|task| task.run.as_str()),
            Some("composer run serve")
        );
    }

    #[test]
    fn prefers_composer_platform_php_over_require_php() {
        let fixture = Fixture::new();
        fixture.write(
            "composer.json",
            r#"{
  "name": "qredex/php-app",
  "require": {
    "php": "^8.1"
  },
  "config": {
    "platform": {
      "php": "8.3.4"
    }
  }
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("php"),
            Some(&"8.3.4".to_string())
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "runtimes.php"
                    && inference.source == "composer.json#config.platform.php"
                    && inference.confidence == Confidence::High)
        );
    }

    #[test]
    fn detects_ruby_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Gemfile",
            r#"source "https://rubygems.org"
ruby "3.3.1"
gem "rails"
"#,
        );
        fixture.write(".ruby-version", "3.3.2\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("ruby"),
            Some(&"3.3.2".to_string())
        );
        assert_eq!(report.contract.tools.get("bundler"), Some(&"*".to_string()));
    }

    #[test]
    fn detects_dotnet_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "global.json",
            r#"{
  "sdk": {
    "version": "8.0.203"
  }
}"#,
        );
        fixture.write(
            "Qredex.App.csproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("dotnet"),
            Some(&"8.0.203".to_string())
        );
        assert_eq!(report.contract.tools.get("dotnet"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("dotnet build")
        );
    }

    #[test]
    fn detects_elixir_mix_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "mix.exs",
            r#"defmodule Qredex.MixProject do
  use Mix.Project

  def project do
    [
      app: :qredex,
      elixir: "~> 1.16"
    ]
  end
end
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex")
        );
        assert_eq!(
            report.contract.runtimes.get("elixir"),
            Some(&"~> 1.16".to_string())
        );
        assert_eq!(report.contract.tools.get("mix"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("mix test")
        );
    }

    #[test]
    fn detects_scala_build_sbt_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "build.sbt",
            r#"name := "qredex-scala"
scalaVersion := "2.13.16"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-scala")
        );
        assert_eq!(
            report.contract.runtimes.get("scala"),
            Some(&"2.13.16".to_string())
        );
        assert_eq!(report.contract.tools.get("sbt"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("sbt compile")
        );
    }

    #[test]
    fn detects_swift_package_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Package.swift",
            r#"// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "QredexSwift",
    targets: [
        .executableTarget(name: "QredexSwift")
    ]
)
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("QredexSwift")
        );
        assert_eq!(report.contract.tools.get("swift"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("swift test")
        );
    }

    #[test]
    fn detects_pubspec_flutter_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "pubspec.yaml",
            r#"name: qredex_flutter
environment:
  sdk: ">=3.3.0 <4.0.0"
flutter:
  uses-material-design: true
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_flutter")
        );
        assert_eq!(
            report.contract.runtimes.get("dart"),
            Some(&">=3.3.0 <4.0.0".to_string())
        );
        assert_eq!(report.contract.tools.get("dart"), Some(&"*".to_string()));
        assert_eq!(report.contract.tools.get("flutter"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("run")
                .map(|task| task.run.as_str()),
            Some("flutter run")
        );
    }

    #[test]
    fn detects_cmake_cpp_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "CMakeLists.txt",
            r#"cmake_minimum_required(VERSION 3.25)
project(qredex-cpp)
set(CMAKE_CXX_STANDARD 20)
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-cpp")
        );
        assert_eq!(report.contract.tools.get("cmake"), Some(&"*".to_string()));
        assert_eq!(report.contract.runtimes.get("cpp"), Some(&"20".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("cmake -S . -B build && cmake --build build")
        );
    }

    #[test]
    fn detects_clojure_project_clj_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "project.clj",
            r#"(defproject qredex-clj "0.1.0-SNAPSHOT"
  :dependencies [[org.clojure/clojure "1.12.0"]])
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-clj")
        );
        assert_eq!(
            report.contract.tools.get("leiningen"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("lein test")
        );
    }

    #[test]
    fn detects_haskell_stack_and_cabal_signals() {
        let fixture = Fixture::new();
        fixture.write("stack.yaml", "resolver: lts-22.11\n");
        fixture.write("qredex-hs.cabal", "name: qredex-hs\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-hs")
        );
        assert_eq!(report.contract.tools.get("stack"), Some(&"*".to_string()));
        assert_eq!(report.contract.tools.get("cabal"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("stack build")
        );
    }

    #[test]
    fn detects_lua_rockspec_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "qredex-lua-1.0.0-1.rockspec",
            r#"package = "qredex-lua"
version = "1.0.0-1"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex-lua-1.0.0-1")
        );
        assert_eq!(report.contract.tools.get("luarocks"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("luarocks make")
        );
    }

    #[test]
    fn detects_julia_project_toml_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Project.toml",
            r#"name = "QredexJulia"
[compat]
julia = "1.10"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("QredexJulia")
        );
        assert_eq!(
            report.contract.runtimes.get("julia"),
            Some(&"1.10".to_string())
        );
        assert_eq!(report.contract.tools.get("julia"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("julia --project=. -e 'using Pkg; Pkg.test()'")
        );
    }

    #[test]
    fn detects_r_description_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "DESCRIPTION",
            r#"Package: qredexr
Version: 0.1.0
Depends: R (>= 4.3.0)
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredexr")
        );
        assert_eq!(report.contract.runtimes.get("r"), Some(&">= 4.3.0".to_string()));
        assert_eq!(report.contract.tools.get("r"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("check")
                .map(|task| task.run.as_str()),
            Some("R CMD check .")
        );
    }

    #[test]
    fn detects_ocaml_dune_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "dune-project",
            r#"(lang dune 3.10)
(name qredex_ocaml)
"#,
        );
        fixture.write(".ocaml-version", "5.2.0\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_ocaml")
        );
        assert_eq!(
            report.contract.runtimes.get("ocaml"),
            Some(&"5.2.0".to_string())
        );
        assert_eq!(report.contract.tools.get("dune"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("dune runtest")
        );
    }

    #[test]
    fn detects_nim_nimble_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "qredexnim.nimble",
            r#"version       = "0.1.0"
requires "nim >= 2.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredexnim")
        );
        assert_eq!(
            report.contract.runtimes.get("nim"),
            Some(&"nim >= 2.0".to_string())
        );
        assert_eq!(report.contract.tools.get("nimble"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("nimble build")
        );
    }

    #[test]
    fn detects_erlang_rebar_signals() {
        let fixture = Fixture::new();
        fixture.write("rebar.config", "{erl_opts, [debug_info]}.\n{app, qredex_erlang}.\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_erlang")
        );
        assert_eq!(report.contract.tools.get("rebar3"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("rebar3 compile")
        );
    }

    #[test]
    fn detects_zig_build_zig_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "build.zig",
            "const std = @import(\"std\");\npub fn build(b: *std.Build.0.13.0) void {}\n",
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(report.contract.tools.get("zig"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("zig"),
            Some(&"0.13.0".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("zig build test")
        );
    }

    #[test]
    fn detects_d_dub_json_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "dub.json",
            r#"{
  "name": "qredex_d",
  "description": "qredex d app"
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_d")
        );
        assert_eq!(report.contract.tools.get("dub"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("dub build")
        );
    }

    #[test]
    fn detects_fortran_fpm_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "fpm.toml",
            r#"[project]
name = "qredex_fortran"
version = "0.1.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_fortran")
        );
        assert_eq!(report.contract.tools.get("fpm"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("fpm test")
        );
    }

    #[test]
    fn detects_crystal_shard_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "shard.yml",
            r#"name: qredex_crystal
version: 0.1.0
crystal: ">= 1.11.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_crystal")
        );
        assert_eq!(
            report.contract.runtimes.get("crystal"),
            Some(&">= 1.11.0".to_string())
        );
        assert_eq!(report.contract.tools.get("crystal"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("shards build")
        );
    }

    #[test]
    fn detects_elm_json_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "elm.json",
            r#"{
  "type": "application",
  "name": "qredex/elm-app",
  "source-directories": ["src"]
}"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex/elm-app")
        );
        assert_eq!(report.contract.tools.get("elm"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("elm make src/Main.elm")
        );
    }

    #[test]
    fn detects_perl_makefile_pl_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Makefile.PL",
            r#"use ExtUtils::MakeMaker;
WriteMakefile(
    NAME => 'Qredex::Perl',
);
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Qredex::Perl")
        );
        assert_eq!(report.contract.tools.get("perl"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("make test")
        );
    }

    #[test]
    fn detects_haxe_hxml_signals() {
        let fixture = Fixture::new();
        fixture.write("build.hxml", "-cp src\n-main Main\n-js out/main.js\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("haxe"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("build")
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("haxe build.hxml")
        );
    }

    #[test]
    fn detects_gleam_toml_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "gleam.toml",
            r#"name = "qredex_gleam"
version = "1.0.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_gleam")
        );
        assert_eq!(report.contract.tools.get("gleam"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("gleam test")
        );
    }

    #[test]
    fn detects_v_mod_signals() {
        let fixture = Fixture::new();
        fixture.write("v.mod", "Module {\nname: 'qredex_v'\n}\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_v")
        );
        assert_eq!(report.contract.tools.get("v"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("v .")
        );
    }

    #[test]
    fn detects_ada_alire_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "alire.toml",
            r#"[project]
name = "qredex_ada"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("qredex_ada")
        );
        assert_eq!(report.contract.tools.get("alr"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("test")
                .map(|task| task.run.as_str()),
            Some("alr test")
        );
    }

    #[test]
    fn detects_foundry_solidity_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "foundry.toml",
            r#"[profile.default]
solc_version = "0.8.25"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("forge"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("solidity"),
            Some(&"0.8.25".to_string())
        );
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("forge build")
        );
    }

    #[test]
    fn detects_kotlin_script_signals() {
        let fixture = Fixture::new();
        fixture.write("app.kts", "println(\"hello\")\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("kotlin"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("app")
        );
        assert_eq!(
            report.contract.tasks.get("run").map(|task| task.run.as_str()),
            Some("kotlin app.kts")
        );
    }

    #[test]
    fn detects_fsharp_project_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Qredex.App.fsproj",
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#,
        );

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("dotnet"), Some(&"*".to_string()));
        assert_eq!(report.contract.runtimes.get("fsharp"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("Qredex.App")
        );
    }

    #[test]
    fn detects_tcl_markers() {
        let fixture = Fixture::new();
        fixture.write("tclapp.tcl", "puts \"hello\"\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("tclsh"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.tasks.get("run").map(|task| task.run.as_str()),
            Some("tclsh tclapp.tcl")
        );
    }

    #[test]
    fn detects_racket_markers() {
        let fixture = Fixture::new();
        fixture.write("main.rkt", "#lang racket\n(displayln \"hello\")\n");
        fixture.write("info.rkt", "#lang info\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("racket"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.tasks.get("run").map(|task| task.run.as_str()),
            Some("racket main.rkt")
        );
        assert_eq!(
            report.contract.tasks.get("test").map(|task| task.run.as_str()),
            Some("raco test .")
        );
    }

    #[test]
    fn detects_bash_script_signals() {
        let fixture = Fixture::new();
        fixture.write("main.sh", "#!/usr/bin/env bash\necho \"hello\"\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("bash"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("shell"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("main")
        );
        assert_eq!(
            report.contract.tasks.get("run").map(|task| task.run.as_str()),
            Some("bash main.sh")
        );
    }

    #[test]
    fn detects_powershell_script_signals() {
        let fixture = Fixture::new();
        fixture.write("bootstrap.ps1", "Write-Host \"ready\"\n");

        let report = detect_repo(fixture.path()).unwrap();
        assert_eq!(report.contract.tools.get("pwsh"), Some(&"*".to_string()));
        assert_eq!(
            report.contract.runtimes.get("powershell"),
            Some(&"*".to_string())
        );
        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("bootstrap")
        );
        assert_eq!(
            report.contract.tasks.get("run").map(|task| task.run.as_str()),
            Some("pwsh -File bootstrap.ps1")
        );
    }

    #[test]
    fn detects_cargo_signals() {
        let fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota-rust"
rust-version = "1.84"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report
                .contract
                .project
                .as_ref()
                .map(|project| project.name.as_str()),
            Some("ota-rust")
        );
        assert_eq!(
            report.contract.runtimes.get("rust"),
            Some(&"1.84".to_string())
        );
        assert_eq!(report.contract.tools.get("cargo"), Some(&"*".to_string()));
        assert_eq!(
            report
                .contract
                .tasks
                .get("build")
                .map(|task| task.run.as_str()),
            Some("cargo build")
        );
    }

    #[test]
    fn detects_rust_toolchain_toml() {
        let fixture = Fixture::new();
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota-rust"
rust-version = "1.80"
"#,
        );
        fixture.write(
            "rust-toolchain.toml",
            r#"[toolchain]
channel = "1.85.0"
"#,
        );

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("rust"),
            Some(&"1.85.0".to_string())
        );
    }

    #[test]
    fn detects_rust_toolchain_file() {
        let fixture = Fixture::new();
        fixture.write("rust-toolchain", "stable\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("rust"),
            Some(&"stable".to_string())
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
    fn detects_java_version_file() {
        let fixture = Fixture::new();
        fixture.write(".java-version", "21\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("java"),
            Some(&"21".to_string())
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "runtimes.java"
                    && inference.source == ".java-version"
                    && inference.confidence == Confidence::High)
        );
    }

    #[test]
    fn detects_sdkmanrc_java_version() {
        let fixture = Fixture::new();
        fixture.write(".sdkmanrc", "java=21.0.2-tem\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("java"),
            Some(&"21.0.2-tem".to_string())
        );
        assert!(
            report
                .inferences
                .iter()
                .any(|inference| inference.field == "runtimes.java"
                    && inference.source == ".sdkmanrc#java"
                    && inference.confidence == Confidence::High)
        );
    }

    #[test]
    fn prefers_java_version_file_over_tool_versions_for_java() {
        let fixture = Fixture::new();
        fixture.write(".java-version", "21\n");
        fixture.write(".tool-versions", "java 17.0.10-tem\n");

        let report = detect_repo(fixture.path()).unwrap();

        assert_eq!(
            report.contract.runtimes.get("java"),
            Some(&"21".to_string())
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
    fn treats_lockfile_package_manager_and_scripts_as_high_confidence() {
        let fixture = Fixture::new();
        fixture.write(
            "package.json",
            r#"{
  "name": "@qredex/merchant/app",
  "scripts": {
    "build": "next build",
    "dev": "next dev",
    "start": "next start",
    "typecheck": "tsc --noEmit"
  }
}"#,
        );
        fixture.write("package-lock.json", "{\n  \"name\": \"merchant-app\"\n}\n");

        let report = detect_repo(fixture.path()).unwrap();
        let contract = report.high_confidence_contract();

        assert_eq!(contract.tools.get("npm"), Some(&"*".to_string()));
        assert_eq!(
            contract.tasks.get("build").map(|task| task.run.as_str()),
            Some("npm run build")
        );
        assert_eq!(
            contract.tasks.get("dev").map(|task| task.run.as_str()),
            Some("npm run dev")
        );
        assert_eq!(
            contract.tasks.get("start").map(|task| task.run.as_str()),
            Some("npm run start")
        );
        assert_eq!(
            contract.tasks.get("typecheck").map(|task| task.run.as_str()),
            Some("npm run typecheck")
        );
        assert_eq!(
            contract.tasks.get("typecheck").map(|task| task.safe_for_agent),
            Some(true)
        );
        assert_eq!(
            contract.tasks.get("build").map(|task| task.safe_for_agent),
            Some(false)
        );

        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tools.npm"
                    && inference.source == "package-lock.json"
                    && inference.confidence == Confidence::High
            }),
            "expected npm tool inference from package-lock.json with high confidence"
        );
        assert!(
            report.inferences.iter().any(|inference| {
                inference.field == "tasks.typecheck.safe_for_agent"
                    && inference.value == "true"
                    && inference.confidence == Confidence::High
            }),
            "expected typecheck verifier tasks to be marked safe_for_agent=true"
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
