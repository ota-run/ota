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
use std::path::Path;

use crate::detector::{
    DetectCheck, DetectCheckKind, DetectCheckSeverity, DetectContract, DetectProject, DetectReport,
    DetectTask, Inference,
};
use crate::schema::{AgentBootstrapConfig, AgentBootstrapTargetConfig, AgentConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StarterPack {
    Node,
    Python,
    Go,
    Rust,
    JavaMaven,
    JavaGradle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NodePackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PythonTestRunner {
    Pytest,
    Unittest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct StarterPackOptions {
    pub(crate) node_package_manager: Option<NodePackageManager>,
    pub(crate) python_test_runner: Option<PythonTestRunner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StarterPackConfig {
    pub(crate) pack: StarterPack,
    pub(crate) options: StarterPackOptions,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StarterPackCatalogOption {
    pub(crate) flag: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) default: &'static str,
    pub(crate) values: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StarterPackCatalogEntry {
    pub(crate) pack: StarterPack,
    pub(crate) summary: &'static str,
    pub(crate) when: &'static str,
    pub(crate) runtimes: &'static [&'static str],
    pub(crate) tools: &'static [&'static str],
    pub(crate) checks: &'static [&'static str],
    pub(crate) tasks: &'static [&'static str],
    pub(crate) options: &'static [StarterPackCatalogOption],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StarterPackAdvisory {
    pub(crate) suggested_pack: StarterPack,
    pub(crate) signals: Vec<String>,
}

impl StarterPack {
    pub(crate) fn all() -> &'static [StarterPack] {
        &[
            StarterPack::Node,
            StarterPack::Python,
            StarterPack::Go,
            StarterPack::Rust,
            StarterPack::JavaMaven,
            StarterPack::JavaGradle,
        ]
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Python => "python",
            Self::Go => "go",
            Self::Rust => "rust",
            Self::JavaMaven => "java-maven",
            Self::JavaGradle => "java-gradle",
        }
    }

    pub(crate) fn command(self) -> String {
        format!("ota init --pack {}", self.as_str())
    }

    pub(crate) fn preview_command(self) -> String {
        format!("ota init --pack {} --dry-run .", self.as_str())
    }

    pub(crate) fn provenance_source(self) -> String {
        format!("ota.init#starter_pack.{}", self.as_str())
    }

    pub(crate) fn catalog_entry(self) -> StarterPackCatalogEntry {
        match self {
            Self::Node => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Node starter with pnpm-based setup, dev, and test tasks.",
                when: "Use this for repo-level Node apps or services that need an explicit JavaScript starter instead of detector-led init. The default path uses pnpm, and you can override it with `--package-manager` when the repo is intentionally npm-, yarn-, or bun-based.",
                runtimes: &["node"],
                tools: &["pnpm"],
                checks: &["node-installed"],
                tasks: &["setup", "dev", "test"],
                options: NODE_PACK_OPTIONS,
            },
            Self::Python => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Python starter with requirements-based setup and pytest.",
                when: "Use this for Python repos that install from requirements.txt. The default path uses pytest, and you can switch to `python -m unittest` with `--test-runner unittest` when that is the repo's conventional test entrypoint.",
                runtimes: &["python"],
                tools: &[],
                checks: &["python-installed"],
                tasks: &["setup", "test"],
                options: PYTHON_PACK_OPTIONS,
            },
            Self::Go => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Go starter with module download, build, and test tasks.",
                when: "Use this for Go module repos that should start from the standard `go mod download`, `go build`, and `go test` flow without relying on detector-led init.",
                runtimes: &["go"],
                tools: &[],
                checks: &["go-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
            },
            Self::Rust => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Rust starter with Cargo fetch, build, and test tasks.",
                when: "Use this for Cargo-managed Rust repos that should start from the standard fetch/build/test flow without relying on detector-led init.",
                runtimes: &["rust"],
                tools: &["cargo"],
                checks: &["rust-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
            },
            Self::JavaMaven => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Java starter for Maven-driven repos with build and test lifecycles, preferring `mvnw` when the repo already ships it.",
                when: "Use this when the repo is intentionally Maven-based and you want an explicit Java starter without relying on repo detection. If `mvnw` already exists, ota uses the wrapper instead of requiring a global Maven install.",
                runtimes: &["java"],
                tools: &[],
                checks: &["java-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
            },
            Self::JavaGradle => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Java starter for Gradle-driven repos with build and test lifecycles, preferring `gradlew` when the repo already ships it.",
                when: "Use this when the repo is intentionally Gradle-based and you want an explicit Java starter without relying on repo detection. If `gradlew` already exists, ota uses the wrapper instead of requiring a global Gradle install.",
                runtimes: &["java"],
                tools: &[],
                checks: &["java-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
            },
        }
    }
}

const NO_PACK_OPTIONS: &[StarterPackCatalogOption] = &[];

const NODE_PACK_OPTIONS: &[StarterPackCatalogOption] = &[StarterPackCatalogOption {
    flag: "--package-manager",
    summary: "Choose the package manager used for setup and script execution.",
    default: "pnpm",
    values: &["npm", "pnpm", "yarn", "bun"],
}];

const PYTHON_PACK_OPTIONS: &[StarterPackCatalogOption] = &[StarterPackCatalogOption {
    flag: "--test-runner",
    summary: "Choose the conventional Python test entrypoint for the starter.",
    default: "pytest",
    values: &["pytest", "unittest"],
}];

impl NodePackageManager {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
            Self::Bun => "bun",
        }
    }

    pub(crate) const fn default_for_pack() -> Self {
        Self::Pnpm
    }

    fn tool(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Npm => Some(("npm", "*")),
            Self::Pnpm => Some(("pnpm", "10")),
            Self::Yarn => Some(("yarn", "4")),
            Self::Bun => Some(("bun", "1.2")),
        }
    }

    fn setup_command(self) -> &'static str {
        match self {
            Self::Npm => "npm install",
            Self::Pnpm => "pnpm install",
            Self::Yarn => "yarn install",
            Self::Bun => "bun install",
        }
    }

    fn dev_command(self) -> &'static str {
        match self {
            Self::Npm => "npm run dev",
            Self::Pnpm => "pnpm dev",
            Self::Yarn => "yarn dev",
            Self::Bun => "bun run dev",
        }
    }

    fn test_command(self) -> &'static str {
        match self {
            Self::Npm => "npm test",
            Self::Pnpm => "pnpm test",
            Self::Yarn => "yarn test",
            Self::Bun => "bun run test",
        }
    }

    fn provenance_source(self) -> String {
        format!(
            "ota.init#starter_pack.node.package_manager.{}",
            self.as_str()
        )
    }
}

impl PythonTestRunner {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pytest => "pytest",
            Self::Unittest => "unittest",
        }
    }

    pub(crate) const fn default_for_pack() -> Self {
        Self::Pytest
    }

    fn test_command(self) -> &'static str {
        match self {
            Self::Pytest => "pytest",
            Self::Unittest => "python -m unittest",
        }
    }

    fn provenance_source(self) -> String {
        format!("ota.init#starter_pack.python.test_runner.{}", self.as_str())
    }
}

impl StarterPackConfig {
    pub(crate) fn new(pack: StarterPack, options: StarterPackOptions) -> Result<Self, String> {
        match pack {
            StarterPack::Node if options.python_test_runner.is_some() => Err(String::from(
                "`--test-runner` is only supported with `ota init --pack python`",
            )),
            StarterPack::Python if options.node_package_manager.is_some() => Err(String::from(
                "`--package-manager` is only supported with `ota init --pack node`",
            )),
            StarterPack::Go
            | StarterPack::Rust
            | StarterPack::JavaMaven
            | StarterPack::JavaGradle
                if options.node_package_manager.is_some()
                    || options.python_test_runner.is_some() =>
            {
                if options.node_package_manager.is_some() {
                    Err(String::from(
                        "`--package-manager` is only supported with `ota init --pack node`",
                    ))
                } else {
                    Err(String::from(
                        "`--test-runner` is only supported with `ota init --pack python`",
                    ))
                }
            }
            _ => Ok(Self { pack, options }),
        }
    }

    pub(crate) fn selected_node_package_manager(self) -> Option<NodePackageManager> {
        (self.pack == StarterPack::Node).then(|| {
            self.options
                .node_package_manager
                .unwrap_or(NodePackageManager::default_for_pack())
        })
    }

    pub(crate) fn selected_python_test_runner(self) -> Option<PythonTestRunner> {
        (self.pack == StarterPack::Python).then(|| {
            self.options
                .python_test_runner
                .unwrap_or(PythonTestRunner::default_for_pack())
        })
    }

    pub(crate) fn provenance_source(self) -> String {
        if let Some(package_manager) = self.selected_node_package_manager() {
            return package_manager.provenance_source();
        }
        if let Some(test_runner) = self.selected_python_test_runner() {
            return test_runner.provenance_source();
        }
        self.pack.provenance_source()
    }

    pub(crate) fn selected_option_pairs(self) -> Vec<(&'static str, &'static str)> {
        let mut pairs = Vec::new();
        if let Some(package_manager) = self.selected_node_package_manager() {
            pairs.push(("package-manager", package_manager.as_str()));
        }
        if let Some(test_runner) = self.selected_python_test_runner() {
            pairs.push(("test-runner", test_runner.as_str()));
        }
        pairs
    }

    pub(crate) fn command(self) -> String {
        let mut command = format!("ota init --pack {}", self.pack.as_str());
        for (name, value) in self.selected_option_pairs() {
            command.push_str(&format!(" --{name} {value}"));
        }
        command
    }

    pub(crate) fn preview_command(self) -> String {
        format!("{} --dry-run .", self.command())
    }
}

pub(crate) fn starter_pack_catalog() -> Vec<StarterPackCatalogEntry> {
    StarterPack::all()
        .iter()
        .copied()
        .map(StarterPack::catalog_entry)
        .collect()
}

pub(crate) fn starter_pack_advisory(
    selected_pack: StarterPack,
    detected: &DetectReport,
) -> Option<StarterPackAdvisory> {
    let mut scores = BTreeMap::<StarterPack, usize>::new();
    let mut signals = BTreeMap::<StarterPack, BTreeSet<String>>::new();
    let mut seen_signals = BTreeSet::<(StarterPack, String)>::new();

    for inference in &detected.inferences {
        let Some((pack, weight, signal)) = pack_signal_for_inference(inference) else {
            continue;
        };
        if seen_signals.insert((pack, signal.clone())) {
            *scores.entry(pack).or_default() += weight;
            signals.entry(pack).or_default().insert(signal);
        }
    }

    let selected_score = scores.get(&selected_pack).copied().unwrap_or_default();
    if selected_score > 0 {
        return None;
    }

    let mut ranked = scores
        .into_iter()
        .filter(|(pack, score)| *pack != selected_pack && *score >= 3)
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| left.0.as_str().cmp(right.0.as_str()))
    });

    let (suggested_pack, best_score) = ranked.first().copied()?;
    if ranked
        .get(1)
        .is_some_and(|(_, next_score)| *next_score == best_score)
    {
        return None;
    }

    Some(StarterPackAdvisory {
        suggested_pack,
        signals: signals
            .remove(&suggested_pack)
            .unwrap_or_default()
            .into_iter()
            .take(3)
            .collect(),
    })
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

fn pack_signal_for_inference(inference: &Inference) -> Option<(StarterPack, usize, String)> {
    let source = inference.source.as_str();

    if source.starts_with("package.json")
        || matches!(
            source,
            ".nvmrc"
                | ".node-version"
                | "pnpm-workspace.yaml"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "bun.lock"
                | "bun.lockb"
                | "package-lock.json"
                | "npm-shrinkwrap.json"
        )
    {
        return Some((StarterPack::Node, 3, normalize_pack_signal(source)));
    }

    if source.starts_with("pyproject.toml")
        || source.starts_with("Pipfile")
        || source.starts_with("setup.cfg")
        || matches!(source, "requirements.txt" | "uv.lock")
    {
        return Some((StarterPack::Python, 3, normalize_pack_signal(source)));
    }

    if source.starts_with("go.mod") {
        return Some((StarterPack::Go, 4, normalize_pack_signal(source)));
    }

    if source.starts_with("Cargo.toml") {
        return Some((StarterPack::Rust, 4, normalize_pack_signal(source)));
    }

    if source.starts_with("pom.xml") || source.starts_with(".mvn/wrapper/maven-wrapper.properties")
    {
        return Some((StarterPack::JavaMaven, 4, normalize_pack_signal(source)));
    }

    if source.starts_with("build.gradle")
        || source.starts_with("settings.gradle")
        || source.starts_with("gradle/wrapper/gradle-wrapper.properties")
    {
        return Some((StarterPack::JavaGradle, 4, normalize_pack_signal(source)));
    }

    None
}

fn normalize_pack_signal(source: &str) -> String {
    match source {
        value if value.starts_with("package.json") => String::from("package.json"),
        value if value.starts_with("pyproject.toml") => String::from("pyproject.toml"),
        value if value.starts_with("Pipfile") => String::from("Pipfile"),
        value if value.starts_with("setup.cfg") => String::from("setup.cfg"),
        value if value.starts_with("go.mod") => String::from("go.mod"),
        value if value.starts_with("Cargo.toml") => String::from("Cargo.toml"),
        value if value.starts_with("pom.xml") => String::from("pom.xml"),
        value if value.starts_with("build.gradle.kts") => String::from("build.gradle.kts"),
        value if value.starts_with("build.gradle") => String::from("build.gradle"),
        value if value.starts_with("settings.gradle.kts") => String::from("settings.gradle.kts"),
        value if value.starts_with("settings.gradle") => String::from("settings.gradle"),
        value if value.starts_with("gradle/wrapper/gradle-wrapper.properties") => {
            String::from("gradle wrapper")
        }
        value if value.starts_with(".mvn/wrapper/maven-wrapper.properties") => {
            String::from("maven wrapper")
        }
        value => value.to_string(),
    }
}

pub(crate) fn starter_pack_contract(config: StarterPackConfig, root: &Path) -> DetectContract {
    let project = directory_name_for_root(root).map(|name| DetectProject { name });
    let mut contract = DetectContract {
        version: 1,
        project,
        ..DetectContract::default()
    };

    match config.pack {
        StarterPack::Node => {
            let package_manager = config
                .selected_node_package_manager()
                .expect("node pack should always resolve a package manager");
            contract
                .runtimes
                .insert(String::from("node"), String::from("22"));
            if let Some((tool, version)) = package_manager.tool() {
                contract
                    .tools
                    .insert(String::from(tool), String::from(version));
            }
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
                    package_manager.setup_command(),
                    Some(String::from("Install repo dependencies.")),
                ),
            );
            contract.tasks.insert(
                String::from("dev"),
                pack_task(
                    "dev",
                    package_manager.dev_command(),
                    Some(String::from("Start the local development loop.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    package_manager.test_command(),
                    Some(String::from("Run the default automated test command.")),
                ),
            );
        }
        StarterPack::Python => {
            let test_runner = config
                .selected_python_test_runner()
                .expect("python pack should always resolve a test runner");
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
                    test_runner.test_command(),
                    Some(match test_runner {
                        PythonTestRunner::Pytest => {
                            String::from("Run the default Python test command.")
                        }
                        PythonTestRunner::Unittest => {
                            String::from("Run the default Python unittest suite.")
                        }
                    }),
                ),
            );
        }
        StarterPack::Go => {
            contract
                .runtimes
                .insert(String::from("go"), String::from("1.24"));
            contract.checks.push(DetectCheck {
                name: String::from("go-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("go version"),
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "go mod download",
                    Some(String::from("Download Go module dependencies.")),
                ),
            );
            contract.tasks.insert(
                String::from("build"),
                pack_task(
                    "build",
                    "go build ./...",
                    Some(String::from("Build the Go packages in this module.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    "go test ./...",
                    Some(String::from("Run the default Go test suite.")),
                ),
            );
        }
        StarterPack::Rust => {
            contract
                .runtimes
                .insert(String::from("rust"), String::from("1.85"));
            contract
                .tools
                .insert(String::from("cargo"), String::from("*"));
            contract.checks.push(DetectCheck {
                name: String::from("rust-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("rustc --version"),
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "cargo fetch",
                    Some(String::from("Fetch Cargo dependencies for the repo.")),
                ),
            );
            contract.tasks.insert(
                String::from("build"),
                pack_task(
                    "build",
                    "cargo build",
                    Some(String::from("Build the default Cargo outputs.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    "cargo test",
                    Some(String::from("Run the default Cargo test suite.")),
                ),
            );
        }
        StarterPack::JavaMaven => {
            let uses_wrapper = root.join("mvnw").exists();
            contract
                .runtimes
                .insert(String::from("java"), String::from("22"));
            contract.checks.push(DetectCheck {
                name: String::from("java-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("java --version"),
            });
            if !uses_wrapper {
                contract
                    .tools
                    .insert(String::from("maven"), String::from("3.9"));
                contract.checks.push(DetectCheck {
                    name: String::from("maven-installed"),
                    kind: DetectCheckKind::Precondition,
                    severity: DetectCheckSeverity::Error,
                    run: String::from("mvn --version"),
                });
            }
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    if uses_wrapper {
                        "./mvnw -q dependency:resolve"
                    } else {
                        "mvn -q dependency:resolve"
                    },
                    Some(String::from("Resolve Maven dependencies for the repo.")),
                ),
            );
            contract.tasks.insert(
                String::from("build"),
                pack_task(
                    "build",
                    if uses_wrapper {
                        "./mvnw package"
                    } else {
                        "mvn package"
                    },
                    Some(String::from("Build the default Maven package output.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    if uses_wrapper {
                        "./mvnw test"
                    } else {
                        "mvn test"
                    },
                    Some(String::from("Run the default Maven test lifecycle.")),
                ),
            );
        }
        StarterPack::JavaGradle => {
            let uses_wrapper = root.join("gradlew").exists();
            contract
                .runtimes
                .insert(String::from("java"), String::from("22"));
            contract.checks.push(DetectCheck {
                name: String::from("java-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("java --version"),
            });
            if !uses_wrapper {
                contract
                    .tools
                    .insert(String::from("gradle"), String::from("8"));
                contract.checks.push(DetectCheck {
                    name: String::from("gradle-installed"),
                    kind: DetectCheckKind::Precondition,
                    severity: DetectCheckSeverity::Error,
                    run: String::from("gradle --version"),
                });
            }
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    if uses_wrapper {
                        "./gradlew dependencies"
                    } else {
                        "gradle dependencies"
                    },
                    Some(String::from("Resolve Gradle dependencies for the repo.")),
                ),
            );
            contract.tasks.insert(
                String::from("build"),
                pack_task(
                    "build",
                    if uses_wrapper {
                        "./gradlew build"
                    } else {
                        "gradle build"
                    },
                    Some(String::from("Build the default Gradle outputs.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    if uses_wrapper {
                        "./gradlew test"
                    } else {
                        "gradle test"
                    },
                    Some(String::from("Run the default Gradle test lifecycle.")),
                ),
            );
        }
    }

    apply_starter_contract_defaults(&mut contract, root);
    contract
}

fn pack_task(task_name: &str, run: &str, description: Option<String>) -> DetectTask {
    let mut notes = String::from("Run `ota run ");
    notes.push_str(task_name);
    notes.push_str("` to execute this task.\n");
    if let Some(note) = description.as_deref() {
        notes.push_str(&note);
    }

    DetectTask {
        description,
        run: String::from(run),
        notes: Some(notes),
        safe_for_agent: false,
    }
}
