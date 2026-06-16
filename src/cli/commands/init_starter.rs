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
    FileCheckExpectation, TaskActionSpec, TaskBundlerHydrationSourceSpec,
    TaskCargoHydrationSourceSpec, TaskCommandSpec, TaskCopyIfMissingActionSpec,
    TaskDependencyHydrationMedium, TaskDependencyHydrationPrepareSpec,
    TaskDependencyHydrationSourceSpec, TaskDotnetRestoreHydrationSourceSpec, TaskEffectsSpec,
    TaskGoModulesHydrationSourceSpec, TaskGradleHydrationSourceSpec, TaskMavenHydrationMode,
    TaskMavenHydrationSourceSpec, TaskNetworkEffectKind, TaskNodePackageManagerHydrationMode,
    TaskNodePackageManagerHydrationSourceSpec, TaskNodePackageManagerKind, TaskPrepareSpec,
    TaskRequirementsSpec, TaskUvHydrationSourceSpec, ToolRequirement,
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
    Ruby,
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
            StarterPack::Ruby,
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
            Self::Ruby => "ruby",
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
                summary: "Conventional Node starter with toolchain-owned Node plus package-manager-driven setup and script-aware dev/test tasks.",
                when: "Use this for repo-level Node apps or services that need an explicit JavaScript starter instead of detector-led init. The default path keeps Node ownership under `toolchains.node`, seeds first-class package-manager hydration for `setup`, and you can override the package manager with `--package-manager` when the repo is intentionally npm-, yarn-, or bun-based. `dev` and `test` are seeded only when the root `package.json` declares those scripts.",
                toolchains: &["node"],
                runtimes: &[],
                tools: &[],
                checks: &[],
                tasks: &["setup", "dev", "test"],
                options: NODE_PACK_OPTIONS,
                does_not_infer: &[
                    "the repo's package manager unless `--package-manager` says so",
                    "repo-specific script names or extra task variants beyond seeded `setup` plus optional root `dev`/`test` script tasks",
                    "dotenv env sources from repo files such as `.env.local` or `.env`",
                ],
            },
            Self::Python => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Python starter with uv-managed toolchain ownership and uv-native setup/test tasks.",
                when: "Use this for Python repos that should start from toolchain-owned Python (`toolchains.python`) and uv-managed dependency hydration plus task execution. The default test path uses first-class `command` task execution for `uv run pytest`, and you can switch to `uv run python -m unittest` with `--test-runner unittest` when that is the repo's conventional test entrypoint.",
                toolchains: &["python"],
                runtimes: &[],
                tools: &[],
                checks: &[],
                tasks: &["setup", "test"],
                options: PYTHON_PACK_OPTIONS,
                does_not_infer: &[
                    "repo-specific pyproject dependency groups, lock strategy, or uv workspace layout beyond the seeded uv hydration lane plus test loop",
                    "repo-specific test layout beyond the selected `pytest` or `unittest` entrypoint",
                ],
            },
            Self::Ruby => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Ruby starter with toolchain-owned Ruby and Bundler-driven setup/test tasks.",
                when: "Use this for Ruby repos that should start from `toolchains.ruby` ownership and the standard Bundler loop without relying on detector-led init.",
                toolchains: &["ruby"],
                runtimes: &[],
                tools: &[],
                checks: &[],
                tasks: &["setup", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "framework-specific commands (for example Rails, Sinatra, or Hanami server entrypoints) beyond the seeded Bundler setup/test surface",
                    "repo-specific test wrappers or flags beyond the seeded `bundle exec rake test` command",
                ],
            },
            Self::Go => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional Go starter with toolchain-owned Go plus module download, build, and test tasks.",
                when: "Use this for Go module repos that should start from `toolchains.go` ownership and the standard `go mod download`, `go build`, and `go test` flow without relying on detector-led init.",
                toolchains: &["go"],
                runtimes: &[],
                tools: &[],
                checks: &[],
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
                checks: &[],
                tasks: &["setup", "build", "test"],
                options: NO_PACK_OPTIONS,
                does_not_infer: &[
                    "workspace members, feature flags, or custom cargo aliases beyond the standard fetch/build/test loop",
                ],
            },
            Self::Dotnet => StarterPackCatalogEntry {
                pack: self,
                summary: "Conventional .NET starter with toolchain-owned .NET plus restore, build, and test tasks.",
                when: "Use this for .NET repos that should start from `toolchains.dotnet` ownership and the standard `dotnet restore`, `dotnet build`, and `dotnet test` loop without relying on detector-led init.",
                toolchains: &["dotnet"],
                runtimes: &[],
                tools: &[],
                checks: &[],
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

    fn script_command_spec(self, script_name: &str) -> TaskCommandSpec {
        match self {
            Self::Npm => TaskCommandSpec {
                exe: String::from("npm"),
                args: vec![String::from("run"), script_name.to_string()],
            },
            Self::Pnpm => TaskCommandSpec {
                exe: String::from("pnpm"),
                args: vec![script_name.to_string()],
            },
            Self::Yarn => TaskCommandSpec {
                exe: String::from("yarn"),
                args: vec![script_name.to_string()],
            },
            Self::Bun => TaskCommandSpec {
                exe: String::from("bun"),
                args: vec![String::from("run"), script_name.to_string()],
            },
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

    fn test_command_spec(self) -> TaskCommandSpec {
        match self {
            Self::Pytest => TaskCommandSpec {
                exe: String::from("uv"),
                args: vec![String::from("run"), String::from("pytest")],
            },
            Self::Unittest => TaskCommandSpec {
                exe: String::from("uv"),
                args: vec![
                    String::from("run"),
                    String::from("python"),
                    String::from("-m"),
                    String::from("unittest"),
                ],
            },
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
            | StarterPack::Ruby
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
    normalize_detected_starter_surfaces(contract, &report.root);
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

fn normalize_detected_starter_surfaces(contract: &mut DetectContract, root: &Path) {
    normalize_detected_node_starter(contract, root);
    normalize_detected_ruby_starter(contract, root);
    normalize_detected_java_starter(contract, root);
    normalize_detected_dotnet_starter(contract, root);
    normalize_detected_simple_hydration_tasks(contract);
    normalize_detected_simple_command_tasks(contract);
}

fn normalize_detected_node_starter(contract: &mut DetectContract, root: &Path) {
    if !root.join("package.json").exists() {
        return;
    }
    let package_manager = detected_node_package_manager(contract, root);
    let Some(package_manager) = package_manager else {
        return;
    };

    let version = contract
        .runtimes
        .remove("node")
        .unwrap_or_else(|| String::from("*"));
    let mut package_managers = BTreeMap::new();
    match package_manager {
        NodePackageManager::Pnpm => {
            package_managers.insert(
                String::from("pnpm"),
                contract
                    .tools
                    .remove("pnpm")
                    .unwrap_or_else(|| String::from("*")),
            );
        }
        NodePackageManager::Yarn => {
            package_managers.insert(
                String::from("yarn"),
                contract
                    .tools
                    .remove("yarn")
                    .unwrap_or_else(|| String::from("*")),
            );
        }
        NodePackageManager::Npm | NodePackageManager::Bun => {}
    }

    contract.toolchains.insert(
        String::from("node"),
        DetectToolchainSpec {
            provider: crate::schema::ToolchainProvider::Corepack,
            version,
            package_managers,
            fulfillment: None,
        },
    );

    if !contract.tasks.contains_key("setup") {
        contract.tasks.insert(
            String::from("setup"),
            pack_dependency_hydration_task(
                "setup",
                &format!(
                    "Hydrate Node dependencies through {}.",
                    package_manager.as_str()
                ),
                TaskDependencyHydrationSourceSpec::NodePackageManager(
                    TaskNodePackageManagerHydrationSourceSpec {
                        cwd: String::from("."),
                        manager: match package_manager {
                            NodePackageManager::Npm => TaskNodePackageManagerKind::Npm,
                            NodePackageManager::Pnpm => TaskNodePackageManagerKind::Pnpm,
                            NodePackageManager::Yarn => TaskNodePackageManagerKind::Yarn,
                            NodePackageManager::Bun => TaskNodePackageManagerKind::Bun,
                        },
                        mode: TaskNodePackageManagerHydrationMode::Install,
                        frozen_lockfile: false,
                    },
                ),
                "node",
                vec![String::from("node_modules")],
                match package_manager {
                    NodePackageManager::Bun => BTreeMap::from([(
                        String::from("bun"),
                        ToolRequirement::Simple(String::from("*")),
                    )]),
                    _ => BTreeMap::new(),
                },
            ),
        );
    }

    for (task_name, task) in &mut contract.tasks {
        if task.command.is_some() || task.prepare.is_some() || task.action.is_some() {
            continue;
        }
        if task.run == package_manager.dev_command() || task.run == package_manager.test_command() {
            task.command = Some(package_manager.script_command_spec(task_name));
            task.run.clear();
        }
    }
}

fn normalize_detected_ruby_starter(contract: &mut DetectContract, root: &Path) {
    if !root.join("Gemfile").exists() {
        return;
    }
    let bundler_version = contract.tools.remove("bundler");
    let Some(bundler_version) = bundler_version else {
        return;
    };

    contract.toolchains.insert(
        String::from("ruby"),
        DetectToolchainSpec {
            provider: crate::schema::ToolchainProvider::Ruby,
            version: contract
                .runtimes
                .remove("ruby")
                .unwrap_or_else(|| String::from("*")),
            package_managers: BTreeMap::from([(String::from("bundler"), bundler_version)]),
            fulfillment: None,
        },
    );

    if !contract.tasks.contains_key("setup") {
        contract.tasks.insert(
            String::from("setup"),
            pack_dependency_hydration_task(
                "setup",
                "Hydrate Ruby gem dependencies through Bundler.",
                TaskDependencyHydrationSourceSpec::Bundler(TaskBundlerHydrationSourceSpec {
                    cwd: String::from("."),
                    path: String::from("vendor/bundle"),
                }),
                "ruby",
                vec![String::from("vendor/bundle")],
                BTreeMap::new(),
            ),
        );
    }
}

fn normalize_detected_java_starter(contract: &mut DetectContract, root: &Path) {
    let uses_maven = root.join("pom.xml").exists();
    let uses_gradle = [
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
    ]
    .iter()
    .any(|path| root.join(path).exists());
    if !uses_maven && !uses_gradle {
        return;
    }

    contract.toolchains.insert(
        String::from("java"),
        DetectToolchainSpec {
            provider: crate::schema::ToolchainProvider::Sdkman,
            version: contract
                .runtimes
                .remove("java")
                .unwrap_or_else(|| String::from("*")),
            package_managers: BTreeMap::new(),
            fulfillment: None,
        },
    );

    if uses_gradle && !contract.tasks.contains_key("setup") {
        let wrapper = root.join("gradlew").exists();
        contract.tasks.insert(
            String::from("setup"),
            pack_dependency_hydration_task(
                "setup",
                if wrapper {
                    "Hydrate Gradle dependencies through the repo wrapper."
                } else {
                    "Hydrate Gradle dependencies for the repo."
                },
                TaskDependencyHydrationSourceSpec::Gradle(TaskGradleHydrationSourceSpec {
                    cwd: String::from("."),
                    wrapper,
                }),
                "java",
                vec![String::from(".gradle")],
                if wrapper {
                    BTreeMap::new()
                } else {
                    BTreeMap::from([(
                        String::from("gradle"),
                        ToolRequirement::Simple(String::from("*")),
                    )])
                },
            ),
        );
    }
}

fn normalize_detected_dotnet_starter(contract: &mut DetectContract, root: &Path) {
    let has_dotnet_project = root
        .read_dir()
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| matches!(ext, "sln" | "csproj"))
        });
    if !has_dotnet_project {
        return;
    }
    if contract.tools.remove("dotnet").is_none() && !contract.runtimes.contains_key("dotnet") {
        return;
    }

    contract.toolchains.insert(
        String::from("dotnet"),
        DetectToolchainSpec {
            provider: crate::schema::ToolchainProvider::Dotnet,
            version: contract
                .runtimes
                .remove("dotnet")
                .unwrap_or_else(|| String::from("*")),
            package_managers: BTreeMap::new(),
            fulfillment: None,
        },
    );
}

fn normalize_detected_simple_hydration_tasks(contract: &mut DetectContract) {
    for task in contract.tasks.values_mut() {
        if task.command.is_some() || task.prepare.is_some() || task.action.is_some() {
            continue;
        }
        let Some((prepare, requirements, effects)) = detect_prepare_from_run(task.run.as_str())
        else {
            continue;
        };
        task.run.clear();
        task.prepare = Some(prepare);
        task.requirements = requirements;
        task.effects = effects;
    }
}

fn normalize_detected_simple_command_tasks(contract: &mut DetectContract) {
    for task in contract.tasks.values_mut() {
        if task.command.is_some() || task.prepare.is_some() || task.action.is_some() {
            continue;
        }
        let Some(command) = simple_command_spec(task.run.as_str()) else {
            continue;
        };
        task.run.clear();
        task.command = Some(command);
    }
}

fn detect_prepare_from_run(
    run: &str,
) -> Option<(TaskPrepareSpec, TaskRequirementsSpec, TaskEffectsSpec)> {
    match run {
        "uv sync" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::Uv(TaskUvHydrationSourceSpec {
                    cwd: String::from("."),
                }),
                targets: Vec::new(),
            }),
            "python",
            vec![String::from(".venv")],
            BTreeMap::new(),
        )),
        "go mod download" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::GoModules(
                    TaskGoModulesHydrationSourceSpec {
                        cwd: String::from("."),
                    },
                ),
                targets: Vec::new(),
            }),
            "go",
            Vec::new(),
            BTreeMap::new(),
        )),
        "cargo fetch" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::Cargo(TaskCargoHydrationSourceSpec {
                    cwd: String::from("."),
                }),
                targets: Vec::new(),
            }),
            "rust",
            Vec::new(),
            BTreeMap::new(),
        )),
        "dotnet restore" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::DotnetRestore(
                    TaskDotnetRestoreHydrationSourceSpec {
                        cwd: String::from("."),
                    },
                ),
                targets: Vec::new(),
            }),
            "dotnet",
            vec![String::from("obj")],
            BTreeMap::new(),
        )),
        "./mvnw -q -DskipTests dependency:go-offline" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::Maven(TaskMavenHydrationSourceSpec {
                    cwd: String::from("."),
                    wrapper: true,
                    mode: TaskMavenHydrationMode::GoOffline,
                    skip_tests: true,
                }),
                targets: Vec::new(),
            }),
            "java",
            Vec::new(),
            BTreeMap::new(),
        )),
        "mvn -q -DskipTests dependency:go-offline" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::Maven(TaskMavenHydrationSourceSpec {
                    cwd: String::from("."),
                    wrapper: false,
                    mode: TaskMavenHydrationMode::GoOffline,
                    skip_tests: true,
                }),
                targets: Vec::new(),
            }),
            "java",
            Vec::new(),
            BTreeMap::from([(
                String::from("maven"),
                ToolRequirement::Simple(String::from("*")),
            )]),
        )),
        "gradle dependencies" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::Gradle(TaskGradleHydrationSourceSpec {
                    cwd: String::from("."),
                    wrapper: false,
                }),
                targets: Vec::new(),
            }),
            "java",
            vec![String::from(".gradle")],
            BTreeMap::from([(
                String::from("gradle"),
                ToolRequirement::Simple(String::from("*")),
            )]),
        )),
        "./gradlew dependencies" => Some(hydration_shape(
            TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source: TaskDependencyHydrationSourceSpec::Gradle(TaskGradleHydrationSourceSpec {
                    cwd: String::from("."),
                    wrapper: true,
                }),
                targets: Vec::new(),
            }),
            "java",
            vec![String::from(".gradle")],
            BTreeMap::new(),
        )),
        "npm install" => Some(node_hydration_shape(TaskNodePackageManagerKind::Npm, false)),
        "npm ci" => Some(node_hydration_shape(TaskNodePackageManagerKind::Npm, true)),
        "pnpm install" => Some(node_hydration_shape(
            TaskNodePackageManagerKind::Pnpm,
            false,
        )),
        "yarn install" => Some(node_hydration_shape(
            TaskNodePackageManagerKind::Yarn,
            false,
        )),
        "bun install" => Some(node_hydration_shape(TaskNodePackageManagerKind::Bun, false)),
        _ => return None,
    }
}

fn node_hydration_shape(
    manager: TaskNodePackageManagerKind,
    ci: bool,
) -> (TaskPrepareSpec, TaskRequirementsSpec, TaskEffectsSpec) {
    hydration_shape(
        TaskPrepareSpec::DependencyHydration(TaskDependencyHydrationPrepareSpec {
            medium: TaskDependencyHydrationMedium::PackageDependencies,
            source: TaskDependencyHydrationSourceSpec::NodePackageManager(
                TaskNodePackageManagerHydrationSourceSpec {
                    cwd: String::from("."),
                    manager,
                    mode: if ci {
                        TaskNodePackageManagerHydrationMode::Ci
                    } else {
                        TaskNodePackageManagerHydrationMode::Install
                    },
                    frozen_lockfile: false,
                },
            ),
            targets: Vec::new(),
        }),
        "node",
        vec![String::from("node_modules")],
        if matches!(manager, TaskNodePackageManagerKind::Bun) {
            BTreeMap::from([(
                String::from("bun"),
                ToolRequirement::Simple(String::from("*")),
            )])
        } else {
            BTreeMap::new()
        },
    )
}

fn hydration_shape(
    prepare: TaskPrepareSpec,
    toolchain: &str,
    writes: Vec<String>,
    tools: BTreeMap<String, ToolRequirement>,
) -> (TaskPrepareSpec, TaskRequirementsSpec, TaskEffectsSpec) {
    (
        prepare,
        TaskRequirementsSpec {
            toolchains: vec![String::from(toolchain)],
            tools,
            ..TaskRequirementsSpec::default()
        },
        TaskEffectsSpec {
            writes,
            network: true,
            network_kind: Some(TaskNetworkEffectKind::DependencyHydration),
            ..TaskEffectsSpec::default()
        },
    )
}

fn simple_command_spec(run: &str) -> Option<TaskCommandSpec> {
    let trimmed = run.trim();
    if trimmed.is_empty()
        || trimmed.contains(['|', '&', ';', '<', '>', '$', '`', '\n', '\r', '"', '\''])
    {
        return None;
    }
    let parts = trimmed
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<_>>();
    let (exe, args) = parts.split_first()?;
    Some(TaskCommandSpec {
        exe: exe.clone(),
        args: args.to_vec(),
    })
}

fn detected_node_package_manager(
    contract: &DetectContract,
    root: &Path,
) -> Option<NodePackageManager> {
    if contract.tools.contains_key("pnpm") {
        Some(NodePackageManager::Pnpm)
    } else if contract.tools.contains_key("yarn") {
        Some(NodePackageManager::Yarn)
    } else if contract.tools.contains_key("bun") {
        Some(NodePackageManager::Bun)
    } else if contract.tools.contains_key("npm") {
        Some(NodePackageManager::Npm)
    } else if contract
        .tasks
        .values()
        .any(|task| task.run.starts_with("npm run ") || task.run == "npm test")
    {
        Some(NodePackageManager::Npm)
    } else if root.join("pnpm-workspace.yaml").exists() || root.join("pnpm-lock.yaml").exists() {
        Some(NodePackageManager::Pnpm)
    } else if root.join("yarn.lock").exists() {
        Some(NodePackageManager::Yarn)
    } else if root.join("bun.lock").exists() || root.join("bun.lockb").exists() {
        Some(NodePackageManager::Bun)
    } else if root.join("package-lock.json").exists() || root.join("npm-shrinkwrap.json").exists() {
        Some(NodePackageManager::Npm)
    } else if root.join("package.json").exists() {
        Some(NodePackageManager::Npm)
    } else {
        None
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
            command: None,
            action: Some(TaskActionSpec::CopyIfMissing(TaskCopyIfMissingActionSpec {
                from: from.to_string(),
                to: to.to_string(),
            })),
            prepare: None,
            requirements: TaskRequirementsSpec::default(),
            effects: TaskEffectsSpec::default(),
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

    let mut added_ci_topology = false;
    if root.join(".github").join("workflows").is_dir() {
        protected_paths.insert(String::from(".github/workflows"));
        added_ci_topology = true;
    }
    if added_ci_topology {
        provenance.insert(format!("{provenance_prefix}:ci_topology_default"));
    }

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

    if starter_agent_has_node_stack(contract) {
        paths.extend([
            "package-lock.json",
            "pnpm-lock.yaml",
            "pnpm-workspace.yaml",
            "yarn.lock",
            "bun.lock",
            "bun.lockb",
        ]);
    }
    if starter_agent_has_python_stack(contract) {
        paths.extend(["uv.lock", "Pipfile", "requirements.txt", ".python-version"]);
    }
    if starter_agent_has_stack(contract, "go") {
        paths.push("go.sum");
    }
    if starter_agent_has_stack(contract, "rust") || contract.tools.contains_key("cargo") {
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

fn starter_agent_has_stack(contract: &DetectContract, name: &str) -> bool {
    contract.runtimes.contains_key(name) || contract.toolchains.contains_key(name)
}

fn starter_agent_has_node_stack(contract: &DetectContract) -> bool {
    starter_agent_has_stack(contract, "node")
        || ["npm", "pnpm", "yarn", "bun"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
        || contract
            .toolchains
            .get("node")
            .is_some_and(|toolchain| !toolchain.package_managers.is_empty())
}

fn starter_agent_has_python_stack(contract: &DetectContract) -> bool {
    starter_agent_has_stack(contract, "python")
        || ["pip", "pipenv", "uv"]
            .iter()
            .any(|tool| contract.tools.contains_key(*tool))
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

    if starter_agent_has_node_stack(contract) {
        extensions.extend([
            "css", "html", "js", "jsx", "mjs", "mts", "sass", "scss", "ts", "tsx", "vue",
        ]);
    }

    if starter_agent_has_python_stack(contract) {
        extensions.extend(["py", "pyi"]);
    }

    if starter_agent_has_stack(contract, "go") {
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

    if starter_agent_has_stack(contract, "rust") || contract.tools.contains_key("cargo") {
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

    if contract.toolchains.contains_key("dotnet")
        || contract.runtimes.contains_key("dotnet")
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
            | ".github"
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
        return allowed_extensions.contains(&extension);
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

    if source.starts_with("Gemfile")
        || source.starts_with(".ruby-version")
        || source.starts_with("Rakefile")
    {
        return Some((StarterPack::Ruby, 4, normalize_pack_signal(source)));
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
        value if value.starts_with("Gemfile.lock") => String::from("Gemfile.lock"),
        value if value.starts_with("Gemfile") => String::from("Gemfile"),
        value if value.starts_with(".ruby-version") => String::from(".ruby-version"),
        value if value.starts_with("Rakefile") => String::from("Rakefile"),
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
            let setup_task = match package_manager {
                NodePackageManager::Npm => pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Node dependencies through npm.",
                    TaskDependencyHydrationSourceSpec::NodePackageManager(
                        TaskNodePackageManagerHydrationSourceSpec {
                            cwd: String::from("."),
                            manager: TaskNodePackageManagerKind::Npm,
                            mode: TaskNodePackageManagerHydrationMode::Install,
                            frozen_lockfile: false,
                        },
                    ),
                    "node",
                    vec![String::from("node_modules")],
                    BTreeMap::new(),
                ),
                NodePackageManager::Pnpm => pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Node dependencies through pnpm.",
                    TaskDependencyHydrationSourceSpec::NodePackageManager(
                        TaskNodePackageManagerHydrationSourceSpec {
                            cwd: String::from("."),
                            manager: TaskNodePackageManagerKind::Pnpm,
                            mode: TaskNodePackageManagerHydrationMode::Install,
                            frozen_lockfile: false,
                        },
                    ),
                    "node",
                    vec![String::from("node_modules")],
                    BTreeMap::new(),
                ),
                NodePackageManager::Yarn => pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Node dependencies through Yarn.",
                    TaskDependencyHydrationSourceSpec::NodePackageManager(
                        TaskNodePackageManagerHydrationSourceSpec {
                            cwd: String::from("."),
                            manager: TaskNodePackageManagerKind::Yarn,
                            mode: TaskNodePackageManagerHydrationMode::Install,
                            frozen_lockfile: false,
                        },
                    ),
                    "node",
                    vec![String::from("node_modules")],
                    BTreeMap::new(),
                ),
                NodePackageManager::Bun => pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Node dependencies through Bun.",
                    TaskDependencyHydrationSourceSpec::NodePackageManager(
                        TaskNodePackageManagerHydrationSourceSpec {
                            cwd: String::from("."),
                            manager: TaskNodePackageManagerKind::Bun,
                            mode: TaskNodePackageManagerHydrationMode::Install,
                            frozen_lockfile: false,
                        },
                    ),
                    "node",
                    vec![String::from("node_modules")],
                    BTreeMap::from([(
                        String::from("bun"),
                        ToolRequirement::Simple(String::from("*")),
                    )]),
                ),
            };
            contract.tasks.insert(String::from("setup"), setup_task);
            if node_root_package_json_has_script(root, "dev") {
                contract.tasks.insert(
                    String::from("dev"),
                    pack_task(
                        "dev",
                        package_manager.dev_command(),
                        Some(String::from("Start the local development loop.")),
                    ),
                );
            }
            if node_root_package_json_has_script(root, "test") {
                contract.tasks.insert(
                    String::from("test"),
                    pack_task(
                        "test",
                        package_manager.test_command(),
                        Some(String::from("Run the default automated test command.")),
                    ),
                );
            }
        }
        StarterPack::Python => {
            let test_runner = config
                .selected_python_test_runner()
                .expect("python pack should always resolve a test runner");
            let mut package_managers = BTreeMap::new();
            package_managers.insert(String::from("uv"), String::from("*"));
            contract.toolchains.insert(
                String::from("python"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Uv,
                    version: String::from("3.12"),
                    package_managers,
                    fulfillment: None,
                },
            );
            contract.tasks.insert(
                String::from("setup"),
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Python dependencies through uv.",
                    TaskDependencyHydrationSourceSpec::Uv(TaskUvHydrationSourceSpec {
                        cwd: String::from("."),
                    }),
                    "python",
                    vec![String::from(".venv")],
                    BTreeMap::new(),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task_command(
                    "test",
                    test_runner.test_command_spec(),
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
        StarterPack::Ruby => {
            let mut package_managers = BTreeMap::new();
            package_managers.insert(String::from("bundler"), String::from("2.5"));
            contract.toolchains.insert(
                String::from("ruby"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Ruby,
                    version: String::from("3.3.11"),
                    package_managers,
                    fulfillment: None,
                },
            );
            contract.tasks.insert(
                String::from("setup"),
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Ruby gem dependencies through Bundler.",
                    TaskDependencyHydrationSourceSpec::Bundler(TaskBundlerHydrationSourceSpec {
                        cwd: String::from("."),
                        path: String::from("vendor/bundle"),
                    }),
                    "ruby",
                    vec![String::from("vendor/bundle")],
                    BTreeMap::new(),
                ),
            );
            contract.tasks.insert(
                String::from("test"),
                pack_task(
                    "test",
                    "bundle exec rake test",
                    Some(String::from("Run the default Ruby test suite.")),
                ),
            );
        }
        StarterPack::Go => {
            contract.toolchains.insert(
                String::from("go"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Go,
                    version: String::from("1.24"),
                    package_managers: BTreeMap::new(),
                    fulfillment: None,
                },
            );
            contract.tasks.insert(
                String::from("setup"),
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Go module dependencies.",
                    TaskDependencyHydrationSourceSpec::GoModules(
                        TaskGoModulesHydrationSourceSpec {
                            cwd: String::from("."),
                        },
                    ),
                    "go",
                    Vec::new(),
                    BTreeMap::new(),
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
            contract.tasks.insert(
                String::from("setup"),
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Cargo dependencies for the repo.",
                    TaskDependencyHydrationSourceSpec::Cargo(TaskCargoHydrationSourceSpec {
                        cwd: String::from("."),
                    }),
                    "rust",
                    Vec::new(),
                    BTreeMap::new(),
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
            contract.toolchains.insert(
                String::from("dotnet"),
                DetectToolchainSpec {
                    provider: crate::schema::ToolchainProvider::Dotnet,
                    version: String::from("9.0"),
                    package_managers: BTreeMap::new(),
                    fulfillment: None,
                },
            );
            contract.tasks.insert(
                String::from("setup"),
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate the default .NET dependencies through dotnet restore.",
                    TaskDependencyHydrationSourceSpec::DotnetRestore(
                        TaskDotnetRestoreHydrationSourceSpec {
                            cwd: String::from("."),
                        },
                    ),
                    "dotnet",
                    vec![String::from("obj")],
                    BTreeMap::new(),
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
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Maven dependencies for the repo.",
                    TaskDependencyHydrationSourceSpec::Maven(TaskMavenHydrationSourceSpec {
                        cwd: String::from("."),
                        wrapper: uses_wrapper,
                        mode: Default::default(),
                        skip_tests: false,
                    }),
                    "java",
                    Vec::new(),
                    if uses_wrapper {
                        BTreeMap::new()
                    } else {
                        BTreeMap::from([(
                            String::from("maven"),
                            ToolRequirement::Simple(String::from("*")),
                        )])
                    },
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
                pack_dependency_hydration_task(
                    "setup",
                    "Hydrate Gradle dependencies for the repo.",
                    TaskDependencyHydrationSourceSpec::Gradle(TaskGradleHydrationSourceSpec {
                        cwd: String::from("."),
                        wrapper: uses_wrapper,
                    }),
                    "java",
                    vec![String::from(".gradle")],
                    if uses_wrapper {
                        BTreeMap::new()
                    } else {
                        BTreeMap::from([(
                            String::from("gradle"),
                            ToolRequirement::Simple(String::from("*")),
                        )])
                    },
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
        notes.push_str(note);
    }

    DetectTask {
        description,
        run: String::new(),
        command: simple_command_spec(run),
        action: None,
        prepare: None,
        requirements: TaskRequirementsSpec::default(),
        effects: TaskEffectsSpec::default(),
        depends_on: Vec::new(),
        notes: Some(notes),
        internal: task_name == "setup",
        safe_for_agent: false,
    }
}

fn pack_task_command(
    task_name: &str,
    command: TaskCommandSpec,
    description: Option<String>,
) -> DetectTask {
    let mut notes = String::from("Run `ota run ");
    notes.push_str(task_name);
    notes.push_str("` to execute this task.\n");
    if let Some(note) = description.as_deref() {
        notes.push_str(note);
    }

    DetectTask {
        description,
        run: String::new(),
        command: Some(command),
        action: None,
        prepare: None,
        requirements: TaskRequirementsSpec::default(),
        effects: TaskEffectsSpec::default(),
        depends_on: Vec::new(),
        notes: Some(notes),
        internal: task_name == "setup",
        safe_for_agent: false,
    }
}

fn pack_dependency_hydration_task(
    task_name: &str,
    description: &str,
    source: TaskDependencyHydrationSourceSpec,
    toolchain: &str,
    writes: Vec<String>,
    tools: BTreeMap<String, ToolRequirement>,
) -> DetectTask {
    let mut notes = String::from("Run `ota run ");
    notes.push_str(task_name);
    notes.push_str("` to execute this task.\n");
    notes.push_str(description);

    DetectTask {
        description: Some(String::from(description)),
        run: String::new(),
        command: None,
        action: None,
        prepare: Some(TaskPrepareSpec::DependencyHydration(
            TaskDependencyHydrationPrepareSpec {
                medium: TaskDependencyHydrationMedium::PackageDependencies,
                source,
                targets: Vec::new(),
            },
        )),
        requirements: TaskRequirementsSpec {
            toolchains: vec![String::from(toolchain)],
            tools,
            ..TaskRequirementsSpec::default()
        },
        effects: TaskEffectsSpec {
            writes,
            network: true,
            network_kind: Some(TaskNetworkEffectKind::DependencyHydration),
            ..TaskEffectsSpec::default()
        },
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

fn node_root_package_json_has_script(root: &Path, script: &str) -> bool {
    let path = root.join("package.json");
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(package) = serde_json::from_str::<JsonValue>(&contents) else {
        return false;
    };
    package
        .get("scripts")
        .and_then(JsonValue::as_object)
        .is_some_and(|scripts| scripts.contains_key(script))
}

#[cfg(test)]
mod tests {
    use super::{
        StarterPack, StarterPackConfig, StarterPackOptions, bootstrap_init_contract,
        starter_agent_exceptions_for_boundary, starter_pack_contract,
    };
    use crate::detector::{DetectContract, DetectReport, DetectTask};
    use crate::schema::{
        AgentPosture, EnvSource, EnvSourceKind, TaskDependencyHydrationSourceSpec,
        TaskMavenHydrationMode, TaskNodePackageManagerKind, TaskPrepareSpec,
    };
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
                command: None,
                action: None,
                prepare: None,
                requirements: crate::schema::TaskRequirementsSpec::default(),
                effects: crate::schema::TaskEffectsSpec::default(),
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
    fn bootstrap_contract_normalizes_detected_node_starter_surfaces() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("package.json"),
            r#"{
  "name": "demo-node",
  "packageManager": "pnpm@10.1.0",
  "scripts": {
    "dev": "vite",
    "test": "vitest"
  }
}"#,
        )
        .expect("write package.json");

        let report = crate::detector::detect_repo(fixture.path()).expect("detect report");
        let contract = bootstrap_init_contract(&report);

        let node = contract.toolchains.get("node").expect("node toolchain");
        assert_eq!(node.version, "*");
        assert_eq!(
            node.package_managers.get("pnpm").map(String::as_str),
            Some("10.1.0")
        );
        assert!(
            !contract.tools.contains_key("pnpm"),
            "detected init should not keep pnpm as a standalone tool when node toolchain ownership exists"
        );

        let setup = contract.tasks.get("setup").expect("setup task");
        match setup.prepare.as_ref() {
            Some(TaskPrepareSpec::DependencyHydration(prepare)) => match &prepare.source {
                TaskDependencyHydrationSourceSpec::NodePackageManager(source) => {
                    assert_eq!(source.manager, TaskNodePackageManagerKind::Pnpm);
                }
                other => panic!("expected node package-manager hydration, got {other:?}"),
            },
            other => panic!("expected dependency hydration setup, got {other:?}"),
        }
        assert!(setup.command.is_none());
        assert!(setup.run.is_empty());

        let dev = contract.tasks.get("dev").expect("dev task");
        assert_eq!(dev.run, "");
        assert_eq!(
            dev.command
                .as_ref()
                .map(|command| command.preview())
                .as_deref(),
            Some("pnpm dev")
        );
        let test = contract.tasks.get("test").expect("test task");
        assert_eq!(
            test.command
                .as_ref()
                .map(|command| command.preview())
                .as_deref(),
            Some("pnpm test")
        );
    }

    #[test]
    fn bootstrap_contract_normalizes_detected_npm_script_repo_to_toolchain_and_setup() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("package.json"),
            r#"{
  "name": "demo-node",
  "scripts": {
    "dev": "vite",
    "check": "tsc --noEmit"
  }
}"#,
        )
        .expect("write package.json");

        let report = crate::detector::detect_repo(fixture.path()).expect("detect report");
        let contract = bootstrap_init_contract(&report);

        let node = contract.toolchains.get("node").expect("node toolchain");
        assert_eq!(node.version, "*");

        let setup = contract.tasks.get("setup").expect("setup task");
        match setup.prepare.as_ref() {
            Some(TaskPrepareSpec::DependencyHydration(prepare)) => match &prepare.source {
                TaskDependencyHydrationSourceSpec::NodePackageManager(source) => {
                    assert_eq!(source.manager, TaskNodePackageManagerKind::Npm);
                }
                other => panic!("expected node package-manager hydration, got {other:?}"),
            },
            other => panic!("expected dependency hydration setup, got {other:?}"),
        }

        let dev = contract.tasks.get("dev").expect("dev task");
        assert_eq!(
            dev.command
                .as_ref()
                .map(|command| command.preview())
                .as_deref(),
            Some("npm run dev")
        );
        let check = contract.tasks.get("check").expect("check task");
        assert_eq!(
            check
                .command
                .as_ref()
                .map(|command| command.preview())
                .as_deref(),
            Some("npm run check")
        );
    }

    #[test]
    fn bootstrap_contract_normalizes_detected_maven_setup_to_dependency_hydration() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("pom.xml"),
            r#"<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion><groupId>com.example</groupId><artifactId>demo-java</artifactId><version>1.0.0</version></project>"#,
        )
        .expect("write pom.xml");

        let report = crate::detector::detect_repo(fixture.path()).expect("detect report");
        let contract = bootstrap_init_contract(&report);

        let java = contract.toolchains.get("java").expect("java toolchain");
        assert_eq!(java.version, "*");

        let setup = contract.tasks.get("setup").expect("setup task");
        match setup.prepare.as_ref() {
            Some(TaskPrepareSpec::DependencyHydration(prepare)) => match &prepare.source {
                TaskDependencyHydrationSourceSpec::Maven(source) => {
                    assert!(!source.wrapper);
                    assert_eq!(source.mode, TaskMavenHydrationMode::GoOffline);
                    assert!(source.skip_tests);
                }
                other => panic!("expected maven hydration, got {other:?}"),
            },
            other => panic!("expected dependency hydration setup, got {other:?}"),
        }
        assert_eq!(
            contract
                .tasks
                .get("build")
                .and_then(|task| task.command.as_ref())
                .map(|command| command.preview())
                .as_deref(),
            Some("mvn package")
        );
        assert_eq!(
            contract
                .tasks
                .get("test")
                .and_then(|task| task.command.as_ref())
                .map(|command| command.preview())
                .as_deref(),
            Some("mvn test")
        );
    }

    #[test]
    fn bootstrap_contract_normalizes_detected_gradle_setup_to_dependency_hydration() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("build.gradle.kts"),
            r#"plugins { java }

java {
  toolchain {
    languageVersion.set(JavaLanguageVersion.of(21))
  }
}
"#,
        )
        .expect("write build.gradle.kts");
        std::fs::write(fixture.path().join("gradlew"), "#!/bin/sh\n").expect("write gradlew");

        let report = crate::detector::detect_repo(fixture.path()).expect("detect report");
        let contract = bootstrap_init_contract(&report);

        let setup = contract.tasks.get("setup").expect("setup task");
        match setup.prepare.as_ref() {
            Some(TaskPrepareSpec::DependencyHydration(prepare)) => match &prepare.source {
                TaskDependencyHydrationSourceSpec::Gradle(source) => {
                    assert!(source.wrapper);
                }
                other => panic!("expected gradle hydration, got {other:?}"),
            },
            other => panic!("expected dependency hydration setup, got {other:?}"),
        }
        assert_eq!(
            contract
                .tasks
                .get("build")
                .and_then(|task| task.command.as_ref())
                .map(|command| command.preview())
                .as_deref(),
            Some("gradle build")
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
    fn starter_pack_node_omits_dev_and_test_without_root_scripts() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("package.json"),
            r#"{
  "name": "no-root-scripts"
}"#,
        )
        .expect("write package.json");

        let contract = starter_pack_contract(
            StarterPackConfig {
                pack: StarterPack::Node,
                options: StarterPackOptions::default(),
            },
            fixture.path(),
        );

        assert!(contract.tasks.contains_key("setup"));
        assert!(!contract.tasks.contains_key("dev"));
        assert!(!contract.tasks.contains_key("test"));
    }

    #[test]
    fn starter_pack_node_seeds_dev_and_test_when_root_scripts_exist() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("package.json"),
            r#"{
  "name": "with-root-scripts",
  "scripts": {
    "dev": "vite",
    "test": "vitest"
  }
}"#,
        )
        .expect("write package.json");

        let contract = starter_pack_contract(
            StarterPackConfig {
                pack: StarterPack::Node,
                options: StarterPackOptions::default(),
            },
            fixture.path(),
        );

        assert!(contract.tasks.contains_key("setup"));
        assert!(contract.tasks.contains_key("dev"));
        assert!(contract.tasks.contains_key("test"));
        assert_eq!(
            contract
                .tasks
                .get("dev")
                .and_then(|task| task.command.as_ref())
                .map(|command| command.preview())
                .as_deref(),
            Some("pnpm dev")
        );
        assert_eq!(
            contract
                .tasks
                .get("test")
                .and_then(|task| task.command.as_ref())
                .map(|command| command.preview())
                .as_deref(),
            Some("pnpm test")
        );
    }

    #[test]
    fn starter_pack_protects_ci_workflows_and_avoids_github_writable_root() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::create_dir_all(fixture.path().join(".github/workflows"))
            .expect("create workflows dir");
        std::fs::write(
            fixture.path().join(".github/workflows/ci.yml"),
            "name: ci\non: [push]\n",
        )
        .expect("write workflow file");
        std::fs::write(
            fixture.path().join("package.json"),
            r#"{
  "name": "ci-protected-demo",
  "scripts": {
    "test": "vitest"
  }
}"#,
        )
        .expect("write package.json");

        let contract = starter_pack_contract(
            StarterPackConfig {
                pack: StarterPack::Node,
                options: StarterPackOptions::default(),
            },
            fixture.path(),
        );
        let agent = contract.agent.expect("starter pack agent");
        assert!(
            agent
                .protected_paths
                .contains(&String::from(".github/workflows")),
            "starter contracts should protect CI workflows by default"
        );
        assert!(
            !agent.writable_paths.iter().any(|path| path == ".github"),
            "starter contracts should not infer .github as writable in readiness_strict posture"
        );
        let inferred = agent
            .inferred_boundary
            .expect("starter pack inferred boundary");
        assert!(
            inferred
                .provenance
                .protected_paths
                .iter()
                .any(|entry| entry == "init:ci_topology_default"),
            "starter contracts should record CI topology protection provenance"
        );
    }

    #[test]
    fn starter_pack_contract_uses_toolchain_stacks_for_agent_boundaries() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::create_dir_all(fixture.path().join("cmd")).unwrap();
        std::fs::write(fixture.path().join("cmd").join("main.go"), "package main\n").unwrap();
        std::fs::write(fixture.path().join("go.sum"), "").unwrap();

        let contract = starter_pack_contract(
            StarterPackConfig {
                pack: StarterPack::Go,
                options: StarterPackOptions::default(),
            },
            fixture.path(),
        );

        assert!(
            contract.toolchains.contains_key("go"),
            "go starter should own Go through toolchains"
        );
        assert!(
            !contract.runtimes.contains_key("go"),
            "go starter should not duplicate Go under runtimes"
        );
        let agent = contract.agent.expect("starter pack agent");
        assert!(
            agent.protected_paths.contains(&String::from("go.sum")),
            "toolchain-owned Go repos should protect the module lockfile"
        );
        assert!(
            agent.writable_paths.contains(&String::from("cmd")),
            "toolchain-owned Go source detection should still infer source edit scope"
        );
    }

    #[test]
    fn starter_pack_dotnet_uses_toolchain_owner() {
        let fixture = TempDir::new().expect("fixture");
        std::fs::write(
            fixture.path().join("demo.csproj"),
            "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>\n",
        )
        .unwrap();

        let contract = starter_pack_contract(
            StarterPackConfig {
                pack: StarterPack::Dotnet,
                options: StarterPackOptions::default(),
            },
            fixture.path(),
        );

        assert!(
            contract.toolchains.contains_key("dotnet"),
            "dotnet starter should own .NET through toolchains"
        );
        assert!(
            !contract.runtimes.contains_key("dotnet"),
            "dotnet starter should not duplicate .NET under runtimes"
        );
        assert!(
            !contract.tools.contains_key("dotnet"),
            "dotnet starter should not duplicate .NET under tools"
        );
        assert!(
            contract.checks.is_empty(),
            "dotnet starter should rely on toolchain-owned probe semantics instead of duplicate installed checks"
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
                command: None,
                action: None,
                prepare: None,
                requirements: crate::schema::TaskRequirementsSpec::default(),
                effects: crate::schema::TaskEffectsSpec::default(),
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
