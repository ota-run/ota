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
use std::fs;
use std::path::Path;

use serde_json::Value as JsonValue;

use crate::detector::{
    DetectCheck, DetectCheckKind, DetectCheckSeverity, DetectContract, DetectProject, DetectReport,
    DetectTask, DetectToolchainSpec, Inference,
};
use crate::schema::{
    AgentBootstrapConfig, AgentBootstrapTargetConfig, AgentBoundaryProvenanceConfig, AgentConfig,
    AgentExceptionsConfig, AgentInferredBoundaryConfig, AgentPosture, EnvSource, EnvSourceKind,
    FileCheckExpectation, TaskActionSpec, TaskCopyIfMissingActionSpec,
};

const INIT_ENV_SOURCE_CANDIDATES: &[(EnvSourceKind, &str)] = &[
    (EnvSourceKind::Dotenv, ".env.local"),
    (EnvSourceKind::Dotenv, ".env"),
    (
        EnvSourceKind::Properties,
        "src/main/resources/application.properties",
    ),
    (EnvSourceKind::Json, "appsettings.json"),
    (EnvSourceKind::Json, "appsettings.Development.json"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum StarterPack {
    Node,
    Python,
    Go,
    Rust,
    Dotnet,
    PhpComposer,
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
    pub(crate) toolchains: &'static [&'static str],
    pub(crate) runtimes: &'static [&'static str],
    pub(crate) tools: &'static [&'static str],
    pub(crate) checks: &'static [&'static str],
    pub(crate) tasks: &'static [&'static str],
    pub(crate) options: &'static [StarterPackCatalogOption],
    pub(crate) does_not_infer: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StarterPackSignal {
    pub(crate) marker: String,
    pub(crate) weight: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StarterPackAdvisory {
    pub(crate) suggested_pack: StarterPack,
    pub(crate) selected_pack_score: usize,
    pub(crate) suggested_pack_score: usize,
    pub(crate) score_gap: usize,
    pub(crate) signals: Vec<String>,
    pub(crate) signal_details: Vec<StarterPackSignal>,
    pub(crate) selected_signal_details: Vec<StarterPackSignal>,
}

impl StarterPack {
    pub(crate) fn all() -> &'static [StarterPack] {
        &[
            StarterPack::Node,
            StarterPack::Python,
            StarterPack::Go,
            StarterPack::Rust,
            StarterPack::Dotnet,
            StarterPack::PhpComposer,
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
            Self::Dotnet => "dotnet",
            Self::PhpComposer => "php-composer",
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
                summary: "Conventional Node starter with toolchain-owned Node and package-manager-driven setup, dev, and test tasks.",
                when: "Use this for repo-level Node apps or services that need an explicit JavaScript starter instead of detector-led init. The default path keeps Node ownership under `toolchains.node` and uses pnpm via Corepack, and you can override the package manager with `--package-manager` when the repo is intentionally npm-, yarn-, or bun-based.",
                toolchains: &["node"],
                runtimes: &[],
                tools: &[],
                checks: &["node-installed"],
                tasks: &["setup", "dev", "test"],
                options: NODE_PACK_OPTIONS,
                does_not_infer: &[
                    "the repo's package manager unless `--package-manager` says so",
                    "repo-specific script names or extra task variants beyond the seeded `setup`, `dev`, and `test` loop",
                    "dotenv env sources from repo files such as `.env.local` or `.env`",
                ],
            },
            Self::Python => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Python starter with uv-managed toolchain ownership and uv-native setup/test tasks.",
                when: "Use this for Python repos that should start from toolchain-owned Python (`toolchains.python`) and uv-managed task execution. The default test path uses `uv run pytest`, and you can switch to `uv run python -m unittest` with `--test-runner unittest` when that is the repo's conventional test entrypoint.",
                toolchains: &["python"],
                runtimes: &[],
                tools: &["uv"],
                checks: &["python-installed"],
                tasks: &["setup", "test"],
                options: PYTHON_PACK_OPTIONS,
                does_not_infer: &[
                    "repo-specific pyproject dependency groups, lock strategy, or uv workspace layout beyond the seeded `uv sync` + test loop",
                    "repo-specific test layout beyond the selected `pytest` or `unittest` entrypoint",
                ],
            },
            Self::Go => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Go starter with module download, build, and test tasks.",
                when: "Use this for Go module repos that should start from the standard `go mod download`, `go build`, and `go test` flow without relying on detector-led init.",
                toolchains: &[],
                runtimes: &["go"],
                tools: &[],
                checks: &["go-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "workspace layout, code generation, or custom build flags beyond the standard module download/build/test loop",
                ],
            },
            Self::Rust => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Rust starter with Cargo fetch, build, and test tasks.",
                when: "Use this for Cargo-managed Rust repos that should start from the standard fetch/build/test flow without relying on detector-led init.",
                toolchains: &["rust"],
                runtimes: &[],
                tools: &[],
                checks: &["rust-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "workspace members, feature flags, or custom cargo aliases beyond the standard fetch/build/test loop",
                ],
            },
            Self::Dotnet => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional .NET starter with restore, build, and test tasks.",
                when: "Use this for .NET repos that should start from the standard `dotnet restore`, `dotnet build`, and `dotnet test` loop without relying on detector-led init.",
                toolchains: &[],
                runtimes: &["dotnet"],
                tools: &["dotnet"],
                checks: &["dotnet-installed"],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "solution-specific target selection, test filtering, or custom dotnet CLI flags beyond the standard restore/build/test loop",
                ],
            },
            Self::PhpComposer => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional PHP starter for Composer-managed repos with Composer install and optional existing test-script reuse.",
                when: "Use this for Composer-managed PHP repos that should start from `composer install` and, when the repo already declares `scripts.test`, the existing Composer test script without relying on detector-led init.",
                toolchains: &[],
                runtimes: &["php"],
                tools: &["composer"],
                checks: &["php-installed", "composer-installed"],
                tasks: &["setup"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "framework-specific entrypoints, web server commands, or whether the repo uses phpunit, pest, artisan, or another test wrapper unless the repo already declares a Composer `scripts.test` entry",
                ],
            },
            Self::JavaMaven => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Java starter for Maven-driven repos with build and test lifecycles, preferring `mvnw` when the repo already ships it.",
                when: "Use this when the repo is intentionally Maven-based and you want an explicit Java starter without relying on repo detection. If `mvnw` already exists, ota uses the wrapper instead of requiring a global Maven install.",
                toolchains: &["java"],
                runtimes: &[],
                tools: &[],
                checks: &[],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "multi-module reactor details, plugin goals, or org-specific wrapper/bootstrap scripts beyond the standard Maven build/test loop",
                ],
            },
            Self::JavaGradle => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Java starter for Gradle-driven repos with build and test lifecycles, preferring `gradlew` when the repo already ships it.",
                when: "Use this when the repo is intentionally Gradle-based and you want an explicit Java starter without relying on repo detection. If `gradlew` already exists, ota uses the wrapper instead of requiring a global Gradle install.",
                toolchains: &["java"],
                runtimes: &[],
                tools: &[],
                checks: &[],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "multi-project build logic, custom Gradle tasks, or org-specific wrapper/bootstrap scripts beyond the standard Gradle build/test loop",
                ],
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

    fn standalone_tool(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Npm => Some(("npm", "*")),
            Self::Pnpm | Self::Yarn => None,
            Self::Bun => Some(("bun", "1.2")),
        }
    }

    fn toolchain_package_manager(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Npm | Self::Bun => None,
            Self::Pnpm => Some(("pnpm", "11")),
            Self::Yarn => Some(("yarn", "4")),
        }
    }

    fn setup_command(self) -> &'static str {
        match self {
            Self::Npm => "npm install",
            Self::Pnpm => "corepack pnpm install",
            Self::Yarn => "corepack yarn install",
            Self::Bun => "bun install",
        }
    }

    fn dev_command(self) -> &'static str {
        match self {
            Self::Npm => "npm run dev",
            Self::Pnpm => "corepack pnpm dev",
            Self::Yarn => "corepack yarn dev",
            Self::Bun => "bun run dev",
        }
    }

    fn test_command(self) -> &'static str {
        match self {
            Self::Npm => "npm test",
            Self::Pnpm => "corepack pnpm test",
            Self::Yarn => "corepack yarn test",
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
            Self::Pytest => "uv run pytest",
            Self::Unittest => "uv run python -m unittest",
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
            | StarterPack::Dotnet
            | StarterPack::PhpComposer
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

    pub(crate) fn explicit_node_package_manager(self) -> Option<NodePackageManager> {
        (self.pack == StarterPack::Node)
            .then_some(self.options.node_package_manager)
            .flatten()
    }

    pub(crate) fn selected_node_package_manager(self) -> Option<NodePackageManager> {
        (self.pack == StarterPack::Node).then(|| {
            self.options
                .node_package_manager
                .unwrap_or(NodePackageManager::default_for_pack())
        })
    }

    pub(crate) fn explicit_python_test_runner(self) -> Option<PythonTestRunner> {
        (self.pack == StarterPack::Python)
            .then_some(self.options.python_test_runner)
            .flatten()
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

    pub(crate) fn explicit_option_pairs(self) -> Vec<(&'static str, &'static str)> {
        let mut pairs = Vec::new();
        if let Some(package_manager) = self.explicit_node_package_manager() {
            pairs.push(("package-manager", package_manager.as_str()));
        }
        if let Some(test_runner) = self.explicit_python_test_runner() {
            pairs.push(("test-runner", test_runner.as_str()));
        }
        pairs
    }

    pub(crate) fn command(self) -> String {
        let mut command = format!("ota init --pack {}", self.pack.as_str());
        for (name, value) in self.explicit_option_pairs() {
            command.push_str(&format!(" --{name} {value}"));
        }
        command
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
    let mut signal_weights = BTreeMap::<StarterPack, BTreeMap<String, usize>>::new();
    let mut seen_signals = BTreeSet::<(StarterPack, String)>::new();

    for inference in &detected.inferences {
        let Some((pack, weight, signal)) = pack_signal_for_inference(inference) else {
            continue;
        };
        if seen_signals.insert((pack, signal.clone())) {
            *scores.entry(pack).or_default() += weight;
            signal_weights
                .entry(pack)
                .or_default()
                .insert(signal, weight);
        }
    }

    let mut ranked = scores
        .iter()
        .map(|(pack, score)| (*pack, *score))
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

    let selected_score = scores.get(&selected_pack).copied().unwrap_or_default();
    if best_score <= selected_score {
        return None;
    }

    let signal_details = top_pack_signal_details(signal_weights.get(&suggested_pack));
    let selected_signal_details = top_pack_signal_details(signal_weights.get(&selected_pack));

    Some(StarterPackAdvisory {
        suggested_pack,
        selected_pack_score: selected_score,
        suggested_pack_score: best_score,
        score_gap: best_score.saturating_sub(selected_score),
        signals: signal_details
            .iter()
            .map(|signal| signal.marker.clone())
            .collect(),
        signal_details,
        selected_signal_details,
    })
}

fn top_pack_signal_details(weights: Option<&BTreeMap<String, usize>>) -> Vec<StarterPackSignal> {
    let Some(weights) = weights else {
        return Vec::new();
    };

    let mut details = weights
        .iter()
        .map(|(signal, weight)| StarterPackSignal {
            marker: signal.clone(),
            weight: *weight,
        })
        .collect::<Vec<_>>();
    details.sort_by(|left, right| {
        right
            .weight
            .cmp(&left.weight)
            .then_with(|| left.marker.cmp(&right.marker))
    });
    details.truncate(3);
    details
}

pub(super) fn bootstrap_init_contract(report: &DetectReport) -> DetectContract {
    let mut contract = report.contract.clone();
    apply_inferred_init_env_sources(&mut contract, &report.root);
    apply_detected_starter_contract_defaults(&mut contract, report);
    contract
}

pub(super) fn apply_inferred_init_env_sources(contract: &mut DetectContract, root: &Path) {
    for source in inferred_init_env_sources(root) {
        if contract
            .env
            .sources
            .iter()
            .any(|existing| existing.kind == source.kind && existing.path == source.path)
        {
            continue;
        }
        contract.env.sources.push(source);
    }
}

fn inferred_init_env_sources(root: &Path) -> Vec<EnvSource> {
    INIT_ENV_SOURCE_CANDIDATES
        .iter()
        .filter(|(_, path)| root.join(path).is_file())
        .map(|(kind, path)| EnvSource {
            kind: *kind,
            path: (*path).to_string(),
            must_exist: false,
        })
        .collect()
}

pub(super) fn apply_starter_contract_defaults(contract: &mut DetectContract, root: &Path) {
    add_detected_env_copy_setup(contract, root);
    mark_setup_task_internal(contract);
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

pub(super) fn apply_detected_agent_boundary(contract: &mut DetectContract, report: &DetectReport) {
    let Some(agent) = contract.agent.as_mut() else {
        return;
    };

    let boundary = starter_agent_boundary_inference_for_detect_report(report);
    merge_agent_paths(&mut agent.writable_paths, &boundary.writable_paths);
    merge_agent_paths(&mut agent.protected_paths, &boundary.protected_paths);

    let inferred = agent
        .inferred_boundary
        .get_or_insert_with(|| AgentInferredBoundaryConfig {
            reviewed: false,
            provenance: AgentBoundaryProvenanceConfig::default(),
        });

    merge_agent_paths(
        &mut inferred.provenance.writable_paths,
        &boundary.writable_provenance,
    );
    merge_agent_paths(
        &mut inferred.provenance.protected_paths,
        &boundary.protected_provenance,
    );
}

fn merge_agent_paths(existing: &mut Vec<String>, incoming: &[String]) {
    let mut merged: BTreeSet<String> = existing.iter().cloned().collect();
    merged.extend(incoming.iter().cloned());
    let mut merged = merged.into_iter().collect::<Vec<_>>();
    merged.sort_unstable();
    *existing = merged;
}

pub(super) fn apply_detected_starter_contract_defaults(
    contract: &mut DetectContract,
    report: &DetectReport,
) {
    add_detected_env_copy_setup(contract, &report.root);
    mark_setup_task_internal(contract);
    if contract.project.is_none()
        && let Some(name) = directory_name_for_root(&report.root)
    {
        contract.project = Some(DetectProject { name });
    }
    if let Some(agent) = contract.agent.as_mut() {
        if agent.bootstrap.is_none() {
            agent.bootstrap = Some(starter_agent_bootstrap());
        }
    } else {
        contract.agent = starter_agent_from_detected_candidate(contract, report);
    }
}

pub(super) fn mark_setup_task_internal(contract: &mut DetectContract) {
    if let Some(task) = contract.tasks.get_mut("setup") {
        task.internal = true;
    }
}

fn add_detected_env_copy_setup(contract: &mut DetectContract, root: &Path) {
    let Some((task_name, from, to)) = env_copy_candidate(root) else {
        return;
    };
    let copy_task_name = if contract.tasks.contains_key("setup") {
        if contract.tasks.contains_key(task_name) {
            return;
        }
        let Some(setup) = contract.tasks.get_mut("setup") else {
            return;
        };
        if setup.depends_on.iter().any(|dep| dep == task_name) {
            return;
        }
        setup.depends_on.push(task_name.to_string());
        task_name
    } else {
        "setup"
    };
    if !contract
        .env
        .sources
        .iter()
        .any(|source| source.kind == EnvSourceKind::Dotenv && source.path == to)
    {
        contract.env.sources.push(EnvSource {
            kind: EnvSourceKind::Dotenv,
            path: to.to_string(),
            must_exist: false,
        });
    }
    contract.tasks.insert(
        copy_task_name.to_string(),
        DetectTask {
            description: Some(format!("Create `{to}` from `{from}` when it is missing.")),
            run: String::new(),
            action: Some(TaskActionSpec::CopyIfMissing(TaskCopyIfMissingActionSpec {
                from: from.to_string(),
                to: to.to_string(),
            })),
            depends_on: Vec::new(),
            notes: Some(format!(
                "Run `ota run {copy_task_name}` to materialize the local environment file without overwriting existing values."
            )),
            internal: true,
            safe_for_agent: true,
        },
    );
    if !contract
        .checks
        .iter()
        .any(|check| check.name == "env-template-present")
    {
        contract.checks.push(DetectCheck {
            name: String::from("env-template-present"),
            kind: DetectCheckKind::File,
            severity: DetectCheckSeverity::Info,
            run: String::new(),
            path: Some(from.to_string()),
            expect: Some(FileCheckExpectation::File),
        });
    }
}

fn env_copy_candidate(root: &Path) -> Option<(&'static str, &'static str, &'static str)> {
    [
        ("setup:env-local", ".env.local.example", ".env.local"),
        ("setup:env", ".env.example", ".env"),
    ]
    .into_iter()
    .find(|(_, from, to)| root.join(from).is_file() && !root.join(to).exists())
}

fn starter_agent_bootstrap() -> AgentBootstrapConfig {
    let ota_version = format!("v{}", env!("CARGO_PKG_VERSION"));
    AgentBootstrapConfig {
        ota: Some(AgentBootstrapTargetConfig {
            note: Some(String::from(
                "Only install ota if it is missing and installation is approved.",
            )),
            sh: Some(format!(
                "curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION={} sh",
                ota_version
            )),
            powershell: Some(format!(
                "$env:OTA_VERSION='{}'; irm https://dist.ota.run/install.ps1 | iex",
                ota_version
            )),
        }),
    }
}

fn starter_agent_from_detected_contract(
    contract: &DetectContract,
    root: &Path,
) -> Option<AgentConfig> {
    let safe_tasks = starter_agent_safe_tasks(contract);
    if safe_tasks.is_empty() {
        return None;
    }

    let boundary = starter_agent_boundary_inference(contract, root);
    starter_agent_config_from_parts(contract, root, safe_tasks, boundary)
}

fn starter_agent_from_detected_candidate(
    contract: &DetectContract,
    report: &DetectReport,
) -> Option<AgentConfig> {
    let safe_tasks = starter_agent_safe_tasks(contract);
    if safe_tasks.is_empty() {
        return None;
    }

    let boundary = starter_agent_boundary_inference_for_detect_report(report);
    starter_agent_config_from_parts(contract, &report.root, safe_tasks, boundary)
}

fn starter_agent_safe_tasks(contract: &DetectContract) -> Vec<String> {
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
    safe_tasks
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StarterAgentBoundaryOutcomeKind {
    Inferred,
    PartiallyInferred,
    Omitted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StarterAgentBoundaryOutcome {
    pub(super) kind: StarterAgentBoundaryOutcomeKind,
    pub(super) safe_tasks: Vec<String>,
    pub(super) writable_paths: Vec<String>,
    pub(super) protected_paths: Vec<String>,
}

pub(super) fn starter_agent_boundary_outcome_from_detect_report(
    report: &DetectReport,
) -> StarterAgentBoundaryOutcome {
    let safe_tasks = starter_agent_safe_tasks(&report.contract);
    let boundary = starter_agent_boundary_inference_for_detect_report(report);
    starter_agent_boundary_outcome_from_parts(safe_tasks, boundary)
}

fn starter_agent_boundary_outcome_from_parts(
    safe_tasks: Vec<String>,
    boundary: StarterAgentBoundaryInference,
) -> StarterAgentBoundaryOutcome {
    let kind = if !safe_tasks.is_empty() {
        StarterAgentBoundaryOutcomeKind::Inferred
    } else if !boundary.writable_paths.is_empty() {
        StarterAgentBoundaryOutcomeKind::PartiallyInferred
    } else {
        StarterAgentBoundaryOutcomeKind::Omitted
    };

    StarterAgentBoundaryOutcome {
        kind,
        safe_tasks,
        writable_paths: boundary.writable_paths,
        protected_paths: boundary.protected_paths,
    }
}

#[derive(Debug, Default)]
struct StarterAgentBoundaryInference {
    writable_paths: Vec<String>,
    protected_paths: Vec<String>,
    writable_provenance: Vec<String>,
    protected_provenance: Vec<String>,
}

fn starter_agent_config_from_parts(
    contract: &DetectContract,
    _root: &Path,
    safe_tasks: Vec<String>,
    boundary: StarterAgentBoundaryInference,
) -> Option<AgentConfig> {
    let entrypoint = contract
        .tasks
        .contains_key("setup")
        .then(|| String::from("setup"));
    let default_task = preferred_agent_task(&safe_tasks);
    let verify_after_changes = preferred_agent_verify_tasks(&safe_tasks);

    let mut notes = String::from(
        "Review `agent.writable_paths` and `agent.protected_paths`, then set `agent.inferred_boundary.reviewed: true` before letting automation edit this repo.\nUse `ota validate` before changes and `ota doctor` after edits.\n",
    );
    if let Some(task_name) = default_task
        .as_deref()
        .or(entrypoint.as_deref())
        .or_else(|| safe_tasks.first().map(String::as_str))
    {
        notes.push_str(&format!("Use `ota run {task_name}` to verify changes.\n"));
    }

    let posture = AgentPosture::ReadinessStrict;
    let exceptions = starter_agent_exceptions_for_boundary(posture, &boundary.writable_paths);

    Some(AgentConfig {
        posture,
        entrypoint,
        default_task,
        safe_tasks,
        verify_after_changes,
        writable_paths: boundary.writable_paths,
        exceptions,
        protected_paths: boundary.protected_paths,
        inferred_boundary: Some(AgentInferredBoundaryConfig {
            reviewed: false,
            provenance: AgentBoundaryProvenanceConfig {
                writable_paths: boundary.writable_provenance,
                protected_paths: boundary.protected_provenance,
            },
        }),
        bootstrap: Some(starter_agent_bootstrap()),
        notes: Some(notes),
    })
}

fn starter_agent_exceptions_for_boundary(
    posture: AgentPosture,
    writable_paths: &[String],
) -> AgentExceptionsConfig {
    let mut exceptions = AgentExceptionsConfig::default();
    if posture == AgentPosture::ContractAuthoring
        && writable_paths
            .iter()
            .any(|path| path == "." || path == "ota.yaml")
    {
        exceptions.sensitive_writes.push(String::from("ota.yaml"));
    }
    exceptions
}

fn starter_agent_boundary_inference(
    contract: &DetectContract,
    root: &Path,
) -> StarterAgentBoundaryInference {
    let mut writable_provenance = BTreeSet::new();
    let mut protected_provenance = BTreeSet::new();
    let writable_paths = starter_agent_writable_paths_with_semantic_roots(
        contract,
        root,
        &[],
        "init",
        &mut writable_provenance,
    );
    let protected_paths =
        starter_agent_protected_paths(contract, root, "init", &mut protected_provenance);
    StarterAgentBoundaryInference {
        writable_paths,
        protected_paths,
        writable_provenance: writable_provenance.into_iter().collect(),
        protected_provenance: protected_provenance.into_iter().collect(),
    }
}

fn starter_agent_boundary_inference_for_detect_report(
    report: &DetectReport,
) -> StarterAgentBoundaryInference {
    let semantic_roots = starter_agent_semantic_roots_from_detect_report(report);
    let mut writable_provenance = BTreeSet::new();
    let mut protected_provenance = BTreeSet::new();
    let writable_paths = starter_agent_writable_paths_with_semantic_roots(
        &report.contract,
        &report.root,
        semantic_roots.as_slice(),
        "detect",
        &mut writable_provenance,
    );
    let protected_paths = starter_agent_protected_paths_for_detect_report(
        report,
        "detect",
        &mut protected_provenance,
    );
    StarterAgentBoundaryInference {
        writable_paths,
        protected_paths,
        writable_provenance: writable_provenance.into_iter().collect(),
        protected_provenance: protected_provenance.into_iter().collect(),
    }
}

fn starter_agent_writable_paths_with_semantic_roots(
    contract: &DetectContract,
    root: &Path,
    semantic_roots: &[String],
    provenance_prefix: &str,
    provenance: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut writable_paths = BTreeSet::new();
    let allowed_extensions = starter_agent_stack_source_extensions(contract);
    let mut added_common_roots = false;
    let mut added_stack_roots = false;
    let mut added_nested_roots = false;
    let mut added_semantic_roots = false;
    let mut added_scanned_roots = false;

    for candidate in [
        "tests",
        "test",
        "docs",
        "doc",
        "app",
        "components",
        "lib",
        "public",
        "scripts",
        "cmd",
        "internal",
        "pkg",
        "pages",
        "server",
        "client",
        "frontend",
        "backend",
        "shared",
        "ui",
        "api",
        "hooks",
        "utils",
        "types",
        "routes",
        "resources",
        "prisma",
    ] {
        if root.join(candidate).is_dir() {
            writable_paths.insert(candidate.to_string());
            added_common_roots = true;
        }
    }

    for candidate in ["src", "apps", "packages", "crates", "services", "examples"] {
        let path = root.join(candidate);
        if !path.is_dir() {
            continue;
        }
        if starter_agent_dir_has_direct_source_files(&path, allowed_extensions.as_deref()) {
            writable_paths.insert(candidate.to_string());
            added_stack_roots = true;
            continue;
        }
        for nested in starter_agent_collect_nested_source_roots(
            root,
            candidate,
            &path,
            allowed_extensions.as_deref(),
        ) {
            writable_paths.insert(nested);
            added_nested_roots = true;
        }
    }

    for candidate in semantic_roots {
        if starter_agent_valid_writable_path(root, candidate) {
            writable_paths.insert(candidate.clone());
            added_semantic_roots = true;
        }
    }

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten().take(256) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if starter_agent_ignored_scan_dir(name) {
                continue;
            }
            if matches!(
                name,
                "src" | "apps" | "packages" | "crates" | "services" | "examples"
            ) {
                continue;
            }
            if starter_agent_dir_contains_source_files(&path, 3, allowed_extensions.as_deref()) {
                writable_paths.insert(name.to_string());
                added_scanned_roots = true;
            }
        }
    }

    if added_common_roots {
        provenance.insert(format!("{provenance_prefix}:common_source_roots"));
    }
    if added_stack_roots {
        provenance.insert(format!("{provenance_prefix}:stack_source_roots"));
    }
    if added_nested_roots {
        provenance.insert(format!("{provenance_prefix}:nested_project_root"));
    }
    if added_semantic_roots {
        provenance.insert(format!("{provenance_prefix}:semantic_root_inference"));
    }
    if added_scanned_roots {
        provenance.insert(format!("{provenance_prefix}:stack_source_scan"));
    }

    writable_paths.into_iter().collect()
}

fn starter_agent_semantic_roots_from_detect_report(report: &DetectReport) -> Vec<String> {
    let allowed_extensions = starter_agent_stack_source_extensions(&report.contract);
    let mut roots = BTreeSet::new();
    for inference in &report.inferences {
        let source = inference
            .source
            .split('#')
            .next()
            .unwrap_or(inference.source.as_str());
        if let Some(root_path) = starter_agent_semantic_root_from_source(
            &report.root,
            source,
            allowed_extensions.as_deref(),
        ) {
            roots.insert(root_path);
        }
    }
    roots.into_iter().collect()
}

fn starter_agent_protected_paths(
    contract: &DetectContract,
    root: &Path,
    provenance_prefix: &str,
    provenance: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut protected_paths = BTreeSet::from([String::from("ota.yaml")]);
    provenance.insert(format!("{provenance_prefix}:contract_file_default"));

    let mut added_stack_companions = false;
    for candidate in starter_agent_stack_companion_protected_paths(contract) {
        if root.join(candidate).is_file() {
            protected_paths.insert(candidate.to_string());
            added_stack_companions = true;
        }
    }
    if added_stack_companions {
        provenance.insert(format!("{provenance_prefix}:stack_companion_control_files"));
    }

    protected_paths.into_iter().collect()
}

fn starter_agent_protected_paths_for_detect_report(
    report: &DetectReport,
    provenance_prefix: &str,
    provenance: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut protected_paths = starter_agent_protected_paths(
        &report.contract,
        &report.root,
        provenance_prefix,
        provenance,
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut added_detected_control_files = false;

    for inference in &report.inferences {
        let source = inference
            .source
            .split('#')
            .next()
            .unwrap_or(inference.source.as_str());
        let candidate = Path::new(source);
        if candidate.as_os_str().is_empty() {
            continue;
        }
        let absolute = report.root.join(candidate);
        if !absolute.is_file() {
            continue;
        }
        if starter_agent_is_protected_control_file(candidate) {
            protected_paths.insert(candidate.to_string_lossy().to_string());
            added_detected_control_files = true;
        }
    }
    if added_detected_control_files {
        provenance.insert(format!("{provenance_prefix}:detected_control_files"));
    }

    protected_paths.into_iter().collect()
}

fn starter_agent_semantic_root_from_source(
    root: &Path,
    source: &str,
    allowed_extensions: Option<&[&'static str]>,
) -> Option<String> {
    if source.is_empty() || !source.contains('/') {
        return None;
    }
    let relative = Path::new(source);
    let absolute = root.join(relative);
    if !absolute.is_file() || !starter_agent_is_semantic_anchor_file(&absolute, allowed_extensions)
    {
        return None;
    }

    let segments = relative
        .iter()
        .filter_map(|segment| segment.to_str())
        .collect::<Vec<_>>();
    if segments.len() < 2 || starter_agent_ignored_scan_dir(segments[0]) {
        return None;
    }

    for (index, segment) in segments
        .iter()
        .enumerate()
        .take(segments.len().saturating_sub(1))
    {
        if starter_agent_source_anchor_dir(segment) && index > 0 {
            let candidate = segments[..index].join("/");
            if starter_agent_valid_writable_path(root, &candidate) {
                return Some(candidate);
            }
        }
    }

    let candidate = relative.parent()?.to_string_lossy().to_string();
    starter_agent_valid_writable_path(root, &candidate).then_some(candidate)
}

fn starter_agent_is_semantic_anchor_file(
    path: &Path,
    allowed_extensions: Option<&[&'static str]>,
) -> bool {
    starter_agent_is_source_like_file(path, allowed_extensions)
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(starter_agent_is_semantic_manifest_name)
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(starter_agent_is_semantic_manifest_extension)
}

fn starter_agent_is_semantic_manifest_name(name: &str) -> bool {
    matches!(
        name,
        "package.json"
            | "go.mod"
            | "Cargo.toml"
            | "composer.json"
            | "Gemfile"
            | "mix.exs"
            | "build.sbt"
            | "Package.swift"
            | "pubspec.yaml"
            | "CMakeLists.txt"
            | "project.clj"
            | "deps.edn"
            | "stack.yaml"
            | "dune-project"
            | ".ocaml-version"
            | "Project.toml"
            | "DESCRIPTION"
            | "rebar.config"
            | "build.zig"
            | "dub.json"
            | "dub.sdl"
            | "fpm.toml"
            | "shard.yml"
            | "elm.json"
            | "cpanfile"
            | "Makefile.PL"
            | "gleam.toml"
            | "v.mod"
            | "alire.toml"
            | "foundry.toml"
            | "tclapp.tcl"
            | "pkgIndex.tcl"
            | "main.rkt"
            | "info.rkt"
            | "main.sh"
            | "run.sh"
            | "deno.json"
            | "deno.jsonc"
    )
}

fn starter_agent_is_semantic_manifest_extension(extension: &str) -> bool {
    matches!(
        extension,
        "csproj" | "fsproj" | "cabal" | "rockspec" | "opam" | "nimble" | "hxml" | "ps1"
    )
}

fn starter_agent_is_protected_control_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if starter_agent_is_protected_control_name(name) {
        return true;
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(starter_agent_is_protected_control_extension)
}

fn starter_agent_is_protected_control_name(name: &str) -> bool {
    matches!(
        name,
        "package.json"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "pnpm-workspace.yaml"
            | "yarn.lock"
            | "bun.lock"
            | "bun.lockb"
            | "pyproject.toml"
            | "Pipfile"
            | "uv.lock"
            | "requirements.txt"
            | "setup.cfg"
            | ".python-version"
            | "go.mod"
            | "go.sum"
            | "Cargo.toml"
            | "Cargo.lock"
            | "composer.json"
            | "composer.lock"
            | "Gemfile"
            | "Gemfile.lock"
            | ".ruby-version"
            | "mix.exs"
            | "mix.lock"
            | "build.sbt"
            | "Package.swift"
            | "pubspec.yaml"
            | "pubspec.lock"
            | "CMakeLists.txt"
            | "project.clj"
            | "deps.edn"
            | "stack.yaml"
            | "Project.toml"
            | "DESCRIPTION"
            | "dune-project"
            | ".ocaml-version"
            | "rebar.config"
            | "build.zig"
            | "fpm.toml"
            | "shard.yml"
            | "elm.json"
            | "cpanfile"
            | "Makefile.PL"
            | "gleam.toml"
            | "v.mod"
            | "alire.toml"
            | "foundry.toml"
            | "deno.json"
            | "deno.jsonc"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "settings.gradle"
            | "settings.gradle.kts"
            | ".java-version"
            | ".sdkmanrc"
            | ".nvmrc"
            | ".node-version"
            | "global.json"
    )
}

fn starter_agent_is_protected_control_extension(extension: &str) -> bool {
    matches!(
        extension,
        "csproj" | "fsproj" | "cabal" | "rockspec" | "opam" | "nimble" | "hxml"
    )
}

fn starter_agent_stack_companion_protected_paths(contract: &DetectContract) -> Vec<&'static str> {
    let mut paths = Vec::new();

    if ["npm", "pnpm", "yarn", "bun"]
        .iter()
        .any(|tool| contract.tools.contains_key(*tool))
    {
        paths.extend([
            "package-lock.json",
            "pnpm-lock.yaml",
            "pnpm-workspace.yaml",
            "yarn.lock",
            "bun.lock",
            "bun.lockb",
        ]);
    }
    if contract.runtimes.contains_key("python")
        || ["uv", "pipenv", "pip"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
    {
        paths.extend(["uv.lock", "Pipfile", "requirements.txt", ".python-version"]);
    }
    if contract.runtimes.contains_key("go") {
        paths.push("go.sum");
    }
    if contract.runtimes.contains_key("rust") || contract.tools.contains_key("cargo") {
        paths.push("Cargo.lock");
    }
    if contract.runtimes.contains_key("php") || contract.tools.contains_key("composer") {
        paths.push("composer.lock");
    }
    if contract.runtimes.contains_key("ruby") || contract.tools.contains_key("bundler") {
        paths.extend(["Gemfile.lock", ".ruby-version"]);
    }
    if contract.tools.contains_key("mix") {
        paths.push("mix.lock");
    }
    if contract.runtimes.contains_key("dart")
        || contract.tools.contains_key("dart")
        || contract.tools.contains_key("flutter")
    {
        paths.push("pubspec.lock");
    }

    paths
}

fn starter_agent_source_anchor_dir(name: &str) -> bool {
    matches!(
        name,
        "src"
            | "app"
            | "lib"
            | "components"
            | "pages"
            | "public"
            | "server"
            | "client"
            | "frontend"
            | "backend"
            | "shared"
            | "ui"
            | "api"
            | "hooks"
            | "utils"
            | "types"
            | "routes"
            | "resources"
            | "tests"
            | "test"
            | "pkg"
            | "cmd"
            | "internal"
            | "services"
            | "examples"
            | "apps"
            | "packages"
            | "crates"
    )
}

fn starter_agent_collect_nested_source_roots(
    root: &Path,
    prefix: &str,
    dir: &Path,
    allowed_extensions: Option<&[&'static str]>,
) -> Vec<String> {
    let mut roots = BTreeSet::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    for entry in entries.flatten().take(256) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if starter_agent_ignored_scan_dir(name) {
            continue;
        }
        let candidate = format!("{prefix}/{name}");
        if starter_agent_dir_contains_source_files(&path, 3, allowed_extensions)
            && starter_agent_valid_writable_path(root, &candidate)
        {
            roots.insert(candidate);
        }
    }

    roots.into_iter().collect()
}

fn starter_agent_dir_has_direct_source_files(
    dir: &Path,
    allowed_extensions: Option<&[&'static str]>,
) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };

    for entry in entries.flatten().take(256) {
        let path = entry.path();
        if path.is_file() && starter_agent_is_source_like_file(&path, allowed_extensions) {
            return true;
        }
    }

    false
}

fn starter_agent_valid_writable_path(root: &Path, candidate: &str) -> bool {
    if candidate.is_empty() || candidate == "." {
        return false;
    }
    let path = root.join(candidate);
    if !path.is_dir() {
        return false;
    }
    let first_segment = Path::new(candidate)
        .iter()
        .next()
        .and_then(|segment| segment.to_str());
    !first_segment.is_some_and(starter_agent_ignored_scan_dir)
}

fn preferred_agent_task(safe_tasks: &[String]) -> Option<String> {
    for candidate in [
        "test",
        "typecheck",
        "check",
        "verify",
        "lint",
        "build",
        "ci",
    ] {
        if safe_tasks.iter().any(|task| task == candidate) {
            return Some(candidate.to_string());
        }
    }
    safe_tasks.first().cloned()
}

fn preferred_agent_verify_tasks(safe_tasks: &[String]) -> Vec<String> {
    preferred_agent_task(safe_tasks).into_iter().collect()
}

fn starter_agent_stack_source_extensions(contract: &DetectContract) -> Option<Vec<&'static str>> {
    let mut extensions = BTreeSet::new();

    if contract.runtimes.contains_key("node")
        || ["npm", "pnpm", "yarn", "bun"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
    {
        extensions.extend([
            "css", "html", "js", "jsx", "mjs", "mts", "sass", "scss", "ts", "tsx", "vue",
        ]);
    }

    if contract.runtimes.contains_key("python")
        || ["pip", "pipenv", "uv"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
    {
        extensions.extend(["py", "pyi"]);
    }

    if contract.runtimes.contains_key("go") {
        extensions.extend(["go"]);
    }

    if contract.runtimes.contains_key("dart")
        || contract.tools.contains_key("dart")
        || contract.tools.contains_key("flutter")
    {
        extensions.extend(["dart"]);
    }

    if contract.runtimes.contains_key("julia") || contract.tools.contains_key("julia") {
        extensions.extend(["jl"]);
    }

    if contract.runtimes.contains_key("r") || contract.tools.contains_key("r") {
        extensions.extend(["R", "r"]);
    }

    if contract.runtimes.contains_key("nim") || contract.tools.contains_key("nimble") {
        extensions.extend(["nim"]);
    }

    if contract.tools.contains_key("rebar3") {
        extensions.extend(["erl", "hrl"]);
    }

    if contract.runtimes.contains_key("zig") || contract.tools.contains_key("zig") {
        extensions.extend(["zig"]);
    }

    if contract.tools.contains_key("dub") {
        extensions.extend(["d"]);
    }

    if contract.tools.contains_key("fpm") {
        extensions.extend(["f", "f03", "f08", "f90", "f95", "for"]);
    }

    if contract.runtimes.contains_key("rust") || contract.tools.contains_key("cargo") {
        extensions.extend(["rs"]);
    }

    if contract.runtimes.contains_key("crystal") || contract.tools.contains_key("crystal") {
        extensions.extend(["cr"]);
    }

    if contract.tools.contains_key("elm") {
        extensions.extend(["elm"]);
    }

    if contract.runtimes.contains_key("perl")
        || contract.tools.contains_key("perl")
        || contract.tools.contains_key("cpanm")
    {
        extensions.extend(["pl", "pm"]);
    }

    if contract.tools.contains_key("haxe") {
        extensions.extend(["hx"]);
    }

    if contract.tools.contains_key("gleam") {
        extensions.extend(["gleam"]);
    }

    if contract.tools.contains_key("v") {
        extensions.extend(["v"]);
    }

    if contract.tools.contains_key("alr") {
        extensions.extend(["adb", "ads"]);
    }

    if contract.runtimes.contains_key("php") || contract.tools.contains_key("composer") {
        extensions.extend(["php"]);
    }

    if contract.runtimes.contains_key("ruby") || contract.tools.contains_key("bundler") {
        extensions.extend(["rb", "rake"]);
    }

    if contract.toolchains.contains_key("java")
        || contract.runtimes.contains_key("java")
        || contract.runtimes.contains_key("kotlin")
        || ["maven", "gradle", "kotlin"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
    {
        extensions.extend(["java", "kt", "kts"]);
    }

    if contract.runtimes.contains_key("dotnet")
        || contract.runtimes.contains_key("fsharp")
        || contract.tools.contains_key("dotnet")
    {
        extensions.extend(["cs", "fs", "fsx", "vb"]);
    }

    if contract.runtimes.contains_key("c") || contract.tools.contains_key("cmake") {
        extensions.extend(["c", "h"]);
    }

    if contract.runtimes.contains_key("cpp") || contract.tools.contains_key("cmake") {
        extensions.extend(["cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"]);
    }

    if contract.runtimes.contains_key("ocaml")
        || ["dune", "opam"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
    {
        extensions.extend(["ml", "mli", "re", "rei"]);
    }

    if contract.tools.contains_key("leiningen") || contract.tools.contains_key("clojure") {
        extensions.extend(["clj", "cljc", "cljs", "edn"]);
    }

    if contract.tools.contains_key("cabal") || contract.tools.contains_key("stack") {
        extensions.extend(["hs", "lhs"]);
    }

    if contract.tools.contains_key("luarocks") {
        extensions.extend(["lua"]);
    }

    if contract.tools.contains_key("mix") {
        extensions.extend(["ex", "exs", "heex"]);
    }

    if contract.tools.contains_key("sbt") || contract.runtimes.contains_key("scala") {
        extensions.extend(["scala", "sc"]);
    }

    if contract.tools.contains_key("swift") {
        extensions.extend(["swift"]);
    }

    if contract.runtimes.contains_key("solidity") || contract.tools.contains_key("forge") {
        extensions.extend(["sol"]);
    }

    if contract.tools.contains_key("tclsh") {
        extensions.extend(["tcl"]);
    }

    if contract.tools.contains_key("racket") {
        extensions.extend(["rkt"]);
    }

    if contract.runtimes.contains_key("shell") || contract.tools.contains_key("bash") {
        extensions.extend(["bash", "sh", "zsh"]);
    }

    if contract.runtimes.contains_key("powershell")
        || contract.runtimes.contains_key("pwsh")
        || contract.tools.contains_key("pwsh")
    {
        extensions.extend(["ps1", "psd1", "psm1"]);
    }

    if contract.runtimes.contains_key("deno") || contract.tools.contains_key("deno") {
        extensions.extend(["cjs", "cts", "js", "jsx", "mjs", "mts", "ts", "tsx"]);
    }

    if extensions.is_empty() {
        None
    } else {
        Some(extensions.into_iter().collect())
    }
}

fn starter_agent_ignored_scan_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | ".next"
            | ".nuxt"
            | ".turbo"
            | ".cache"
            | ".venv"
            | "venv"
            | "__pycache__"
            | "node_modules"
            | "vendor"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | "out"
            | "config"
            | "database"
            | "migrations"
            | "manifests"
            | "deploy"
            | "infra"
            | "bin"
            | "obj"
    )
}

fn starter_agent_dir_contains_source_files(
    dir: &Path,
    depth: usize,
    allowed_extensions: Option<&[&'static str]>,
) -> bool {
    if depth == 0 {
        return false;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };

    for entry in entries.flatten().take(256) {
        let path = entry.path();
        if path.is_file() && starter_agent_is_source_like_file(&path, allowed_extensions) {
            return true;
        }
        if path.is_dir() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            if starter_agent_ignored_scan_dir(name) {
                continue;
            }
            if starter_agent_dir_contains_source_files(
                &path,
                depth.saturating_sub(1),
                allowed_extensions,
            ) {
                return true;
            }
        }
    }

    false
}

fn starter_agent_is_source_like_file(
    path: &Path,
    allowed_extensions: Option<&[&'static str]>,
) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    if let Some(allowed_extensions) = allowed_extensions {
        return allowed_extensions
            .iter()
            .any(|allowed| *allowed == extension);
    }

    matches!(
        extension,
        "c" | "cc"
            | "cpp"
            | "cr"
            | "cs"
            | "css"
            | "dart"
            | "d"
            | "f"
            | "f03"
            | "f08"
            | "f90"
            | "f95"
            | "for"
            | "gleam"
            | "elm"
            | "erl"
            | "ex"
            | "exs"
            | "fs"
            | "go"
            | "h"
            | "hrl"
            | "hx"
            | "hpp"
            | "html"
            | "java"
            | "js"
            | "jl"
            | "jsx"
            | "kt"
            | "kts"
            | "lua"
            | "nim"
            | "pm"
            | "pl"
            | "php"
            | "ps1"
            | "psd1"
            | "psm1"
            | "py"
            | "r"
            | "R"
            | "rb"
            | "rkt"
            | "rs"
            | "sass"
            | "scala"
            | "sc"
            | "scss"
            | "sh"
            | "sol"
            | "swift"
            | "tcl"
            | "ts"
            | "tsx"
            | "v"
            | "vue"
            | "zig"
            | "zsh"
    )
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

    if source.starts_with("global.json") || source.ends_with(".sln") || source.ends_with(".csproj")
    {
        return Some((StarterPack::Dotnet, 4, normalize_pack_signal(source)));
    }

    if source.starts_with("composer.json") {
        return Some((StarterPack::PhpComposer, 4, normalize_pack_signal(source)));
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
        value if value.starts_with("global.json") => String::from("global.json"),
        value if value.starts_with("composer.json#config.platform.php") => {
            String::from("composer platform php")
        }
        value if value.starts_with("composer.json#require.php") => {
            String::from("composer php requirement")
        }
        value if value.starts_with("composer.json") => String::from("composer.json"),
        value if value.ends_with(".sln") => String::from("solution file"),
        value if value.ends_with(".csproj") => String::from("project file"),
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
            let mut package_managers = BTreeMap::new();
            if let Some((name, version)) = package_manager.toolchain_package_manager() {
                package_managers.insert(String::from(name), String::from(version));
            }
            contract.toolchains.insert(
                String::from("node"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Corepack,
                    version: String::from("22"),
                    package_managers,
                    fulfillment: None,
                },
            );
            if let Some((tool, version)) = package_manager.standalone_tool() {
                contract
                    .tools
                    .insert(String::from(tool), String::from(version));
            }
            contract.checks.push(DetectCheck {
                name: String::from("node-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("node --version"),
                path: None,
                expect: None,
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
            contract.toolchains.insert(
                String::from("python"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Uv,
                    version: String::from("3.12"),
                    package_managers: BTreeMap::new(),
                    fulfillment: None,
                },
            );
            contract.tools.insert(String::from("uv"), String::from("*"));
            contract.checks.push(DetectCheck {
                name: String::from("python-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("python --version"),
                path: None,
                expect: None,
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "uv sync",
                    Some(String::from("Install and sync dependencies with uv.")),
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
                path: None,
                expect: None,
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
            contract.toolchains.insert(
                String::from("rust"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Rustup,
                    version: String::from("1.85"),
                    package_managers: BTreeMap::new(),
                    fulfillment: None,
                },
            );
            contract.checks.push(DetectCheck {
                name: String::from("rust-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("rustc --version"),
                path: None,
                expect: None,
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
        StarterPack::Dotnet => {
            contract
                .runtimes
                .insert(String::from("dotnet"), String::from("9.0"));
            contract
                .tools
                .insert(String::from("dotnet"), String::from("*"));
            contract.checks.push(DetectCheck {
                name: String::from("dotnet-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("dotnet --version"),
                path: None,
                expect: None,
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "dotnet restore",
                    Some(String::from("Restore the default .NET dependencies.")),
                ),
            );
            contract.tasks.insert(
                String::from("build"),
                pack_task(
                    "build",
                    "dotnet build",
                    Some(String::from("Build the default .NET solution or project.")),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    "dotnet test",
                    Some(String::from("Run the default .NET test suite.")),
                ),
            );
        }
        StarterPack::PhpComposer => {
            contract
                .runtimes
                .insert(String::from("php"), String::from("8.3"));
            contract
                .tools
                .insert(String::from("composer"), String::from("*"));
            contract.checks.push(DetectCheck {
                name: String::from("php-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("php --version"),
                path: None,
                expect: None,
            });
            contract.checks.push(DetectCheck {
                name: String::from("composer-installed"),
                kind: DetectCheckKind::Precondition,
                severity: DetectCheckSeverity::Error,
                run: String::from("composer --version"),
                path: None,
                expect: None,
            });
            contract.tasks.insert(
                String::from("setup"),
                pack_task(
                    "setup",
                    "composer install",
                    Some(String::from("Install Composer dependencies for the repo.")),
                ),
            );
            if composer_has_test_script(root) {
                contract.tasks.insert(
                    String::from("test"),
                    pack_task(
                        "test",
                        "composer run test",
                        Some(String::from("Run the existing Composer test script.")),
                    ),
                );
            }
        }
        StarterPack::JavaMaven => {
            let uses_wrapper = root.join("mvnw").exists();
            contract.toolchains.insert(
                String::from("java"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Sdkman,
                    version: String::from("22"),
                    package_managers: BTreeMap::new(),
                    fulfillment: None,
                },
            );
            if !uses_wrapper {
                contract
                    .tools
                    .insert(String::from("maven"), String::from("3.9"));
                contract.checks.push(DetectCheck {
                    name: String::from("maven-installed"),
                    kind: DetectCheckKind::Precondition,
                    severity: DetectCheckSeverity::Error,
                    run: String::from("mvn --version"),
                    path: None,
                    expect: None,
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
            contract.toolchains.insert(
                String::from("java"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Sdkman,
                    version: String::from("22"),
                    package_managers: BTreeMap::new(),
                    fulfillment: None,
                },
            );
            if !uses_wrapper {
                contract
                    .tools
                    .insert(String::from("gradle"), String::from("8"));
                contract.checks.push(DetectCheck {
                    name: String::from("gradle-installed"),
                    kind: DetectCheckKind::Precondition,
                    severity: DetectCheckSeverity::Error,
                    run: String::from("gradle --version"),
                    path: None,
                    expect: None,
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
        action: None,
        depends_on: Vec::new(),
        notes: Some(notes),
        internal: task_name == "setup",
        safe_for_agent: false,
    }
}

fn composer_has_test_script(root: &Path) -> bool {
    let path = root.join("composer.json");
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(composer) = serde_json::from_str::<JsonValue>(&contents) else {
        return false;
    };

    composer
        .get("scripts")
        .and_then(JsonValue::as_object)
        .is_some_and(|scripts| scripts.contains_key("test"))
}

#[cfg(test)]
mod tests {
    use super::{
        StarterPack, StarterPackConfig, StarterPackOptions, bootstrap_init_contract,
        starter_agent_exceptions_for_boundary, starter_pack_contract,
    };
    use crate::detector::{DetectContract, DetectReport, DetectTask};
    use crate::schema::{AgentPosture, EnvSource, EnvSourceKind};
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    #[test]
    fn bootstrap_contract_marks_setup_internal_by_default() {
        let fixture = TempDir::new().expect("fixture");
        let mut tasks = BTreeMap::new();
        tasks.insert(
            String::from("setup"),
            DetectTask {
                description: None,
                run: String::from("echo setup"),
                action: None,
                depends_on: Vec::new(),
                notes: None,
                internal: false,
                safe_for_agent: false,
            },
        );
        let report = DetectReport {
            root: fixture.path().to_path_buf(),
            contract: DetectContract {
                version: 1,
                tasks,
                ..DetectContract::default()
            },
            inferences: Vec::new(),
        };

        let contract = bootstrap_init_contract(&report);

        assert_eq!(
            contract.tasks.get("setup").map(|task| task.internal),
            Some(true)
        );
    }

    #[test]
    fn starter_pack_contract_marks_setup_internal_by_default() {
        let fixture = TempDir::new().expect("fixture");
        let contract = starter_pack_contract(
            StarterPackConfig {
                pack: StarterPack::Node,
                options: StarterPackOptions::default(),
            },
            fixture.path(),
        );

        assert_eq!(
            contract.tasks.get("setup").map(|task| task.internal),
            Some(true)
        );
        let agent = contract.agent.expect("starter pack agent");
        assert!(
            agent.protected_paths.contains(&String::from("ota.yaml")),
            "starter init contracts should protect ota.yaml by default"
        );
        assert!(
            agent.exceptions.sensitive_writes.is_empty(),
            "readiness-strict starter contracts should not emit sensitive write exceptions without broader authority"
        );
        let inferred_boundary = agent
            .inferred_boundary
            .expect("starter pack inferred boundary");
        assert!(!inferred_boundary.reviewed);
        assert_eq!(
            inferred_boundary.provenance.protected_paths,
            vec![String::from("init:contract_file_default")]
        );
    }

    #[test]
    fn starter_contract_adds_ota_sensitive_write_exception_for_contract_authoring() {
        let exceptions = starter_agent_exceptions_for_boundary(
            AgentPosture::ContractAuthoring,
            &[String::from("src"), String::from("ota.yaml")],
        );

        assert_eq!(exceptions.sensitive_writes, vec![String::from("ota.yaml")]);
    }

    #[test]
    fn starter_contract_adds_ota_sensitive_write_exception_for_root_writable_boundary() {
        let exceptions = starter_agent_exceptions_for_boundary(
            AgentPosture::ContractAuthoring,
            &[String::from(".")],
        );

        assert_eq!(exceptions.sensitive_writes, vec![String::from("ota.yaml")]);
    }

    #[test]
    fn bootstrap_contract_carries_curated_detected_env_sources() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(fixture.path().join(".env.local"), "APP_PORT=3000\n").unwrap();
        std::fs::create_dir_all(fixture.path().join("src/main/resources")).unwrap();
        std::fs::write(
            fixture
                .path()
                .join("src/main/resources/application.properties"),
            "app.port=8080\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("appsettings.json"),
            "{ \"App\": { \"Port\": 8081 } }",
        )
        .unwrap();

        let report = crate::detector::detect_repo(fixture.path()).unwrap();
        let contract = bootstrap_init_contract(&report);

        assert_eq!(
            contract.env.sources,
            vec![
                EnvSource {
                    kind: EnvSourceKind::Dotenv,
                    path: String::from(".env.local"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Properties,
                    path: String::from("src/main/resources/application.properties"),
                    must_exist: false,
                },
                EnvSource {
                    kind: EnvSourceKind::Json,
                    path: String::from("appsettings.json"),
                    must_exist: false,
                },
            ]
        );
    }

    #[test]
    fn bootstrap_contract_infers_agent_writable_paths_for_long_tail_stack_roots() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("ota-nimble.nimble"),
            "version = \"0.1.0\"\nrequires \"nim >= 2.0.0\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(fixture.path().join("tooling")).unwrap();
        std::fs::create_dir_all(fixture.path().join("queries")).unwrap();
        std::fs::write(
            fixture.path().join("tooling").join("main.nim"),
            "echo \"hello\"\n",
        )
        .unwrap();
        std::fs::write(
            fixture.path().join("queries").join("schema.sql"),
            "select 1;\n",
        )
        .unwrap();

        let mut tasks = BTreeMap::new();
        tasks.insert(
            String::from("test"),
            DetectTask {
                description: None,
                run: String::from("nimble test"),
                action: None,
                depends_on: Vec::new(),
                notes: None,
                internal: false,
                safe_for_agent: false,
            },
        );
        let mut tools = BTreeMap::new();
        tools.insert(String::from("nimble"), String::from("*"));
        let report = DetectReport {
            root: fixture.path().to_path_buf(),
            contract: DetectContract {
                version: 1,
                tools,
                tasks,
                ..DetectContract::default()
            },
            inferences: Vec::new(),
        };
        let contract = bootstrap_init_contract(&report);
        let agent = contract.agent.expect("starter agent");

        assert!(agent.writable_paths.contains(&String::from("tooling")));
        assert!(!agent.writable_paths.contains(&String::from("queries")));
    }
}
