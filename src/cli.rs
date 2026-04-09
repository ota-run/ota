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

use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use clap::{ArgAction, Parser, Subcommand, ValueEnum, error::ErrorKind};

use crate::output::{CommandOutput, OutputFormat};
use crate::runner::ExecutionOverrides;

mod commands;

#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
#[command(name = "ota")]
#[command(
    about = "Diagnose, prepare, and run repos from one explicit contract.\nDoctor first, contract second.",
    version = env!("CARGO_PKG_VERSION"),
    after_help = "\nChoose a flow:\n  existing repo with ota.yaml  ota doctor\n  turn findings into a plan    ota explain\n  repo without ota.yaml        ota detect --dry-run .\n  review a starter contract    ota init --dry-run\n  prepare the repo             ota up\n  inspect env requirements     ota env\n  review policy boundary       ota policy review\n  generate agent guidance      ota agents\n  list runnable tasks          ota tasks --use\n  run a declared task          ota run ci\n\nWorkspace:\n  inspect readiness            ota workspace doctor .\n  explain blockers             ota workspace explain .\n  prepare the workspace        ota workspace up",
    help_template = "🦦 {name} v{version}\n{about-with-newline}\nUsage:\n  {usage}\n\n{all-args}{after-help}"
)]
pub struct Cli {
    /// Emit command-phase debug tracing to stderr.
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    debug: bool,
    /// Emit plain text output (no icons, no ANSI styling, ASCII list markers).
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    plain: bool,
    /// Reduce non-essential spacing in text output.
    #[arg(
        long,
        global = true,
        action = ArgAction::SetTrue,
        conflicts_with = "verbose"
    )]
    concise: bool,
    /// Keep full text output detail (default behavior).
    #[arg(
        long,
        global = true,
        action = ArgAction::SetTrue,
        conflicts_with = "concise"
    )]
    verbose: bool,
    /// Use an explicit ota.yaml or ota.workspace.yaml file instead of path discovery.
    #[arg(long, global = true)]
    file: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    #[command(display_order = 7)]
    /// Validate an Ota contract.
    Validate {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Run the command against one monorepo member declared by the root contract.
        #[arg(long)]
        member: Option<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 8)]
    /// List validated tasks from an Ota contract.
    Tasks {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Print compact runnable usage lines for each task.
        #[arg(long = "use", action = ArgAction::SetTrue)]
        use_cmd: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 9)]
    /// List declared services from an Ota contract.
    Services {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 10)]
    /// Inspect resolved environment requirements from an Ota contract.
    Env {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Inspect one merged monorepo member contract declared by the root contract.
        #[arg(long)]
        member: Option<String>,
        /// Inspect one task's environment requirements in addition to contract env.
        #[arg(long)]
        task: Option<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 4)]
    /// Run a validated task from an Ota contract.
    Run {
        /// Task name to execute.
        #[arg(index = 1)]
        task: String,
        /// Override the execution mode for this invocation.
        #[arg(long = "mode", visible_alias = "backend", value_enum)]
        backend: Option<RunBackend>,
        /// Override the execution lifecycle for this invocation.
        #[arg(long, value_enum)]
        lifecycle: Option<RunLifecycle>,
        /// Shorthand for `--lifecycle ephemeral`.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "lifecycle")]
        ephemeral: bool,
        /// Include the execution receipt in text output.
        #[arg(long, action = ArgAction::SetTrue)]
        receipt: bool,
        /// Stream raw child process output live instead of buffering it into the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Optional repo path. Put it after the task name and before task inputs.
        #[arg(index = 2)]
        path: Option<PathBuf>,
        /// Task inputs such as `--base-url http://...`, placed after the path.
        #[arg(index = 3)]
        #[arg(allow_hyphen_values = true)]
        inputs: Vec<String>,
    },
    #[command(display_order = 1)]
    /// Diagnose repo readiness from an Ota contract.
    Doctor {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Diagnose readiness in a specific execution context.
        #[arg(long, value_enum, default_value_t = DoctorModeArg::Native)]
        mode: DoctorModeArg,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 12)]
    /// Render Ota JSON findings as CI annotations or log lines.
    Annotations {
        /// Source JSON mode to render.
        #[arg(long, value_enum)]
        mode: AnnotationMode,
        /// Output format for rendered findings.
        #[arg(long, value_enum, default_value = "plain")]
        format: AnnotationFormat,
        /// Optional custom heading prefix.
        #[arg(long)]
        title: Option<String>,
        /// Path to a JSON file, or `-` to read from stdin.
        #[arg(long, value_name = "FILE")]
        input: PathBuf,
    },
    #[command(display_order = 2)]
    /// Explain readiness findings as an ordered remediation plan.
    Explain {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 5)]
    /// Create a starter Ota contract for a repo that does not yet have one.
    Init {
        /// Compatibility flag; writing is now the default.
        #[arg(long, action = ArgAction::SetTrue)]
        write: bool,
        /// Bootstrap a fuller starter contract when the detector has enough confidence.
        #[arg(long, action = ArgAction::SetTrue)]
        bootstrap: bool,
        /// Preview inferred contract output without writing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "write")]
        dry_run: bool,
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to a repo root.
        path: Option<PathBuf>,
    },
    #[command(display_order = 13)]
    /// Generate or sync AGENTS.md from an Ota contract.
    Agents {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Write the generated AGENTS.md to disk.
        #[arg(long, action = ArgAction::SetTrue)]
        write: bool,
        /// Optional output path for the generated AGENTS.md.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 11)]
    /// Run configured checks from an Ota contract.
    Check {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 15)]
    /// List staged extension descriptors from an Ota contract.
    Extensions {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Execute a named extension descriptor via its configured command.
        #[arg(long, conflicts_with = "publish")]
        run: Option<String>,
        /// Publish a named extension descriptor via its configured command.
        #[arg(long, conflicts_with = "run")]
        publish: Option<String>,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 3)]
    /// Prepare the repo for use with minimal prior knowledge.
    Up {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Preview the selected up plan without mutating repo or execution state.
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,
        /// Stream raw live service-start and setup output in text mode.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Override the execution mode for this invocation.
        #[arg(long = "mode", visible_alias = "backend", value_enum)]
        backend: Option<RunBackend>,
        /// Override the execution lifecycle for this invocation.
        #[arg(long, value_enum)]
        lifecycle: Option<RunLifecycle>,
        /// Shorthand for `--lifecycle ephemeral`.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "lifecycle")]
        ephemeral: bool,
        /// Include the execution receipt in text output.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "dry_run")]
        receipt: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long)]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 14)]
    /// Clean persistent execution state for a repo.
    Clean {
        /// Remove exited ota-managed containers from any repo.
        #[arg(long, action = ArgAction::SetTrue)]
        stale: bool,
        /// Preview stale cleanup without removing containers.
        #[arg(long, action = ArgAction::SetTrue, requires = "stale")]
        dry_run: bool,
        /// Emit machine-readable JSON output for stale cleanup.
        #[arg(long, action = ArgAction::SetTrue, requires = "stale")]
        json: bool,
        /// Run the command against one or more monorepo members declared by the root contract.
        #[arg(long, conflicts_with = "stale")]
        member: Vec<String>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 18)]
    /// Show the active policy pack and where ota loaded it from.
    Policy {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        #[command(subcommand)]
        command: Option<PolicyCommands>,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    #[command(display_order = 19)]
    /// Remove ota from this laptop.
    Uninstall,
    #[command(display_order = 16)]
    /// Update the installed Ota binary.
    #[command(alias = "upgrade")]
    SelfUpdate {
        /// Pin the update to a specific release version.
        #[arg(long)]
        version: Option<String>,
        /// Select the update channel.
        #[arg(long, value_enum)]
        channel: Option<UpdateChannel>,
    },
    #[command(display_order = 6)]
    /// Infer a starting contract from repo state.
    Detect {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Write a high-confidence ota.yaml contract.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "dry_run")]
        write: bool,
        /// Print inferred fields without writing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,
        /// Preview the exact starter ota.yaml contract without annotations.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with_all = ["write", "dry_run", "merge", "rewrite", "json"])]
        contract: bool,
        /// Preview how detected fields would merge into an existing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        merge: bool,
        /// Select detected field paths to apply when merging into an existing ota.yaml.
        #[arg(long, action = ArgAction::Append, value_name = "FIELD", requires = "merge")]
        apply: Vec<String>,
        /// Apply all eligible detected changes when merging into an existing ota.yaml.
        #[arg(long, action = ArgAction::SetTrue, requires = "merge")]
        apply_all: bool,
        /// Replace an existing ota.yaml with a regenerated detected contract.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "merge")]
        rewrite: bool,
        /// Confirm destructive rewrite mode.
        #[arg(long, action = ArgAction::SetTrue, requires = "rewrite")]
        yes: bool,
        /// Path to a repo root.
        path: Option<PathBuf>,
    },
    #[command(display_order = 10)]
    /// Compare two Ota contracts semantically.
    Diff {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Base contract path to compare against.
        base: PathBuf,
        /// Target contract path to compare.
        target: PathBuf,
    },
    #[command(display_order = 17)]
    /// Work with Ota workspace contracts.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum PolicyCommands {
    /// Review the policy-vs-contract boundary and approved sources.
    Review {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RunBackend {
    Native,
    Container,
    Remote,
}

impl From<RunBackend> for crate::schema::Backend {
    fn from(value: RunBackend) -> Self {
        match value {
            RunBackend::Native => crate::schema::Backend::Native,
            RunBackend::Container => crate::schema::Backend::Container,
            RunBackend::Remote => crate::schema::Backend::Remote,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RunLifecycle {
    Persistent,
    Ephemeral,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum UpdateChannel {
    Stable,
    Latest,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AnnotationMode {
    Doctor,
    WorkspaceDoctor,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AnnotationFormat {
    Plain,
    Github,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DoctorModeArg {
    Native,
    Container,
}

impl UpdateChannel {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Latest => "latest",
        }
    }
}

impl From<RunLifecycle> for crate::schema::Lifecycle {
    fn from(value: RunLifecycle) -> Self {
        match value {
            RunLifecycle::Persistent => crate::schema::Lifecycle::Persistent,
            RunLifecycle::Ephemeral => crate::schema::Lifecycle::Ephemeral,
        }
    }
}

impl From<DoctorModeArg> for crate::doctor::DoctorMode {
    fn from(value: DoctorModeArg) -> Self {
        match value {
            DoctorModeArg::Native => crate::doctor::DoctorMode::Native,
            DoctorModeArg::Container => crate::doctor::DoctorMode::Container,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum WorkspaceDoctorStatusArg {
    All,
    Ready,
    NotReady,
}

impl From<WorkspaceDoctorStatusArg> for commands::WorkspaceDoctorStatusFilter {
    fn from(value: WorkspaceDoctorStatusArg) -> Self {
        match value {
            WorkspaceDoctorStatusArg::All => commands::WorkspaceDoctorStatusFilter::All,
            WorkspaceDoctorStatusArg::Ready => commands::WorkspaceDoctorStatusFilter::Ready,
            WorkspaceDoctorStatusArg::NotReady => commands::WorkspaceDoctorStatusFilter::NotReady,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum WorkspaceDoctorSeverityArg {
    All,
    Error,
    Warn,
    Info,
}

impl From<WorkspaceDoctorSeverityArg> for commands::WorkspaceDoctorSeverityFilter {
    fn from(value: WorkspaceDoctorSeverityArg) -> Self {
        match value {
            WorkspaceDoctorSeverityArg::All => commands::WorkspaceDoctorSeverityFilter::All,
            WorkspaceDoctorSeverityArg::Error => commands::WorkspaceDoctorSeverityFilter::Error,
            WorkspaceDoctorSeverityArg::Warn => commands::WorkspaceDoctorSeverityFilter::Warn,
            WorkspaceDoctorSeverityArg::Info => commands::WorkspaceDoctorSeverityFilter::Info,
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
enum WorkspaceCommands {
    /// Create a starter ota.workspace.yaml from local repo structure.
    /// Use --bootstrap to scaffold missing child repo ota.yaml files.
    /// Use `ota services` for repo-level service listing and `ota workspace doctor` for workspace readiness.
    Init {
        /// Compatibility flag; init writes by default.
        #[arg(long, action = ArgAction::SetTrue)]
        write: bool,
        /// Bootstrap missing child repo ota.yaml files before writing ota.workspace.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        bootstrap: bool,
        /// Compatibility alias for preview; equivalent outcome to workspace detect dry-run.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "write")]
        dry_run: bool,
        /// Compatibility alias for additive merge; equivalent outcome to workspace detect merge.
        #[arg(long, action = ArgAction::SetTrue)]
        merge: bool,
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to a workspace root directory or ota.workspace.yaml target path.
        path: Option<PathBuf>,
    },
    /// Infer workspace contract shape and merge-ready additions.
    /// Use `ota services` for repo-level service listing and `ota workspace doctor` for workspace readiness.
    Detect {
        /// Write inferred workspace contract.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "dry_run")]
        write: bool,
        /// Preview inferred workspace contract without writing ota.workspace.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,
        /// Merge missing discovered repos into an existing ota.workspace.yaml.
        #[arg(long, action = ArgAction::SetTrue)]
        merge: bool,
        /// Replace an existing ota.workspace.yaml with a regenerated detected contract.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "merge")]
        rewrite: bool,
        /// Confirm destructive rewrite mode.
        #[arg(long, action = ArgAction::SetTrue, requires = "rewrite")]
        yes: bool,
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to a workspace root directory or ota.workspace.yaml target path.
        path: Option<PathBuf>,
    },
    /// Validate an ota.workspace.yaml contract.
    Validate {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// List workspace repo tasks in dependency order.
    Tasks {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// List workspace repos declared in ota.workspace.yaml.
    List {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Filter output to repositories with a specific readiness status.
        #[arg(long, value_enum)]
        status: Option<WorkspaceDoctorStatusArg>,
        /// Filter output to one workspace repo by name.
        #[arg(long)]
        repo: Option<String>,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Diagnose workspace repo readiness from an ota.workspace.yaml contract.
    /// Use `ota services` for repo-level service listing.
    Doctor {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Stream repo completion updates while building the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Maximum number of independent repos to diagnose at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Filter repos by readiness status.
        #[arg(long, value_enum, default_value_t = WorkspaceDoctorStatusArg::All)]
        status: WorkspaceDoctorStatusArg,
        /// Filter findings by severity.
        #[arg(long, value_enum, default_value_t = WorkspaceDoctorSeverityArg::All)]
        severity: WorkspaceDoctorSeverityArg,
        /// Filter output to one workspace repo by name.
        #[arg(long)]
        repo: Option<String>,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Explain workspace readiness findings as an ordered remediation plan.
    Explain {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to diagnose at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Filter repos by readiness status.
        #[arg(long, value_enum, default_value_t = WorkspaceDoctorStatusArg::All)]
        status: WorkspaceDoctorStatusArg,
        /// Filter findings by severity.
        #[arg(long, value_enum, default_value_t = WorkspaceDoctorSeverityArg::All)]
        severity: WorkspaceDoctorSeverityArg,
        /// Filter output to one workspace repo by name.
        #[arg(long)]
        repo: Option<String>,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Run configured checks across workspace repos.
    Check {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to check at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Prepare every repo in an ota.workspace.yaml contract.
    /// Use `ota services` for repo-level service listing.
    Up {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to prepare at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Suppress live progress output and print only the final workspace report.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "stream")]
        quiet: bool,
        /// Stream raw child process output live instead of buffering it into the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Include the execution receipt in text output.
        #[arg(long, action = ArgAction::SetTrue)]
        receipt: bool,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Refresh existing repos in an ota.workspace.yaml contract without cloning missing ones.
    /// Use `ota workspace up` for initial acquisition and preparation, or `--dry-run` to preview the refresh commands.
    Refresh {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to refresh at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Preview refresh commands without making any changes.
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,
        /// Force each refresh to fetch and hard-reset to the declared source or `--ref` override.
        #[arg(long, action = ArgAction::SetTrue)]
        force: bool,
        /// Prune stale remote-tracking refs during refresh.
        #[arg(long, action = ArgAction::SetTrue)]
        prune: bool,
        /// Override the source ref used for refresh, such as a branch, tag, or commit.
        #[arg(long = "ref")]
        git_ref: Option<String>,
        /// Suppress live progress output and print only the final workspace report.
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "stream")]
        quiet: bool,
        /// Stream raw child process output live instead of buffering it into the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Include the execution receipt in text output.
        #[arg(long, action = ArgAction::SetTrue)]
        receipt: bool,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Compare workspace repos against their declared source state without mutating anything.
    Diff {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to compare at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Show a compact read-only workspace status summary.
    Status {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to inspect at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Capture a read-only workspace receipt for audit and automation.
    Receipt {
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to inspect at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Path to an ota.workspace.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
    /// Run a task across workspace repos.
    Run {
        /// Task name to execute.
        #[arg(index = 1)]
        task: String,
        /// Print machine-readable JSON output.
        #[arg(long, action = ArgAction::SetTrue)]
        json: bool,
        /// Maximum number of independent repos to run at once.
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Stream raw child process output live instead of buffering it into the final report.
        #[arg(long, action = ArgAction::SetTrue)]
        stream: bool,
        /// Include the execution receipt in text output.
        #[arg(long, action = ArgAction::SetTrue)]
        receipt: bool,
        /// Optional workspace path. Put it after the task name and before task inputs.
        #[arg(index = 2)]
        path: Option<PathBuf>,
        /// Task inputs such as `--base-url http://...`, placed after the path.
        #[arg(index = 3)]
        #[arg(allow_hyphen_values = true)]
        inputs: Vec<String>,
    },
}

pub fn run() -> i32 {
    let output = run_with(std::env::args_os());

    if !output.stdout.is_empty() {
        println!("{}", output.stdout);
    }

    if let Some(stderr) = output.stderr {
        eprintln!("{stderr}");
    }

    output.exit_code
}

fn run_with<I, T>(args: I) -> CommandOutput
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    struct RunStateGuard {
        original_plain_mode: Option<OsString>,
        original_json_mode: Option<OsString>,
    }

    impl Drop for RunStateGuard {
        fn drop(&mut self) {
            commands::set_plain_mode(false);
            commands::set_concise_mode(false);
            commands::set_json_mode(false);

            unsafe {
                match self.original_plain_mode.take() {
                    Some(value) => std::env::set_var("OTA_PLAIN_MODE", value),
                    None => std::env::remove_var("OTA_PLAIN_MODE"),
                }
                match self.original_json_mode.take() {
                    Some(value) => std::env::set_var("OTA_JSON_MODE", value),
                    None => std::env::remove_var("OTA_JSON_MODE"),
                }
            }
        }
    }

    let _guard = RunStateGuard {
        original_plain_mode: std::env::var_os("OTA_PLAIN_MODE"),
        original_json_mode: std::env::var_os("OTA_JSON_MODE"),
    };
    commands::take_failure_locus();

    let args = args.into_iter().map(Into::into).collect::<Vec<OsString>>();
    let version_update_notice_rx = io::stderr().is_terminal().then(spawn_update_notice);
    if is_version_request(&args) {
        return maybe_append_update_notice(
            CommandOutput::success(render_version_output(&args)),
            version_update_notice_rx,
        );
    }

    let args = rewrite_task_input_path_hint(args);

    match Cli::try_parse_from(args.clone()) {
        Ok(cli) => run_cli(cli),
        Err(error) => {
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                return CommandOutput {
                    stdout: String::new(),
                    stderr: Some(error.render().to_string().trim_end().to_string()),
                    exit_code: 0,
                };
            }

            let mut stderr = error.render().to_string().trim_end().to_string();
            if error.kind() == ErrorKind::InvalidSubcommand
                && args
                    .get(1)
                    .map(|value| value.to_string_lossy() == "workspace")
                    .unwrap_or(false)
                && args
                    .get(2)
                    .map(|value| value.to_string_lossy() == "services")
                    .unwrap_or(false)
            {
                stderr.push_str(
                    "\n\nNext:\n▸ use `ota services` to list repo services\n▸ use `ota workspace doctor` to review workspace readiness",
                );
            }

            CommandOutput::failure_with_code(stderr, 2)
        }
    }
}

fn rewrite_task_input_path_hint(args: Vec<OsString>) -> Vec<OsString> {
    let mut rewritten = args;
    let Some((_, mut index)) = locate_task_command(&rewritten) else {
        return rewritten;
    };

    let mut input_flag_index = None;

    while index < rewritten.len() {
        let token = rewritten[index].to_string_lossy().to_string();
        let token_str = token.as_str();

        if token_str == "--" {
            input_flag_index = Some(index + 1);
            break;
        }

        if let Some(skip) = run_command_value_span(token_str) {
            index += skip;
            continue;
        }

        if token_str.starts_with('-') {
            input_flag_index = Some(index);
            break;
        }

        // A non-flag token after the task-specific flags is the explicit path.
        return rewritten;
    }

    if let Some(index) = input_flag_index {
        if index < rewritten.len() {
            rewritten.insert(index, OsString::from("."));
        }
    }

    rewritten
}

fn locate_task_command(args: &[OsString]) -> Option<(usize, usize)> {
    let mut index = 1;
    while index < args.len() {
        let current = args[index].to_string_lossy();
        if current.as_ref() == "run" {
            return Some((index, index + 2));
        }
        if current.as_ref() == "workspace"
            && index + 1 < args.len()
            && args[index + 1].to_string_lossy().as_ref() == "run"
        {
            return Some((index + 1, index + 3));
        }
        index += 1;
    }
    None
}

fn run_command_value_span(flag: &str) -> Option<usize> {
    if let Some((name, _)) = flag.split_once('=') {
        return match name {
            "--backend" | "--mode" | "--lifecycle" | "--member" | "--jobs" => Some(1),
            "--receipt" | "--json" | "--stream" | "--ephemeral" | "--persistent" => Some(1),
            _ => None,
        };
    }

    match flag {
        "--backend" | "--mode" | "--lifecycle" | "--member" | "--jobs" => Some(2),
        "--receipt" | "--json" | "--stream" | "--ephemeral" | "--persistent" => Some(1),
        _ => None,
    }
}

fn run_cli(cli: Cli) -> CommandOutput {
    let update_notice_rx = should_show_update_notice(&cli).then(spawn_update_notice);

    if should_show_command_spinner(&cli) {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(dispatch(cli));
        });
        return maybe_append_update_notice(wait_with_spinner(rx), update_notice_rx);
    }

    maybe_append_update_notice(dispatch(cli), update_notice_rx)
}

fn spawn_update_notice() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(crate::update::maybe_update_notice(env!(
            "CARGO_PKG_VERSION"
        )));
    });
    rx
}

fn should_show_command_spinner(cli: &Cli) -> bool {
    let json_spinner_exception = matches!(
        &cli.command,
        Commands::Doctor { .. }
            | Commands::Check { .. }
            | Commands::Diff { .. }
            | Commands::Extensions { .. }
            | Commands::Workspace {
                command: WorkspaceCommands::Doctor { .. }
                    | WorkspaceCommands::Explain { .. }
                    | WorkspaceCommands::List { .. }
                    | WorkspaceCommands::Refresh { .. }
                    | WorkspaceCommands::Diff { .. }
                    | WorkspaceCommands::Status { .. }
                    | WorkspaceCommands::Receipt { .. }
            }
    );

    io::stderr().is_terminal()
        && !cli.plain
        && !cli.debug
        && command_supports_spinner(&cli.command)
        && (!command_requests_json(&cli.command) || json_spinner_exception)
}

fn command_supports_spinner(command: &Commands) -> bool {
    matches!(
        command,
        Commands::Validate { .. }
            | Commands::Tasks { .. }
            | Commands::Clean { .. }
            | Commands::Services { .. }
            | Commands::Up { stream: false, .. }
            | Commands::Doctor { .. }
            | Commands::Check { .. }
            | Commands::Diff { .. }
            | Commands::Extensions { .. }
            | Commands::Init { .. }
            | Commands::Agents { .. }
            | Commands::Detect { .. }
            | Commands::Workspace {
                command: WorkspaceCommands::Validate { .. }
                    | WorkspaceCommands::Tasks { .. }
                    | WorkspaceCommands::List { .. }
                    | WorkspaceCommands::Up { .. }
                    | WorkspaceCommands::Refresh { .. }
                    | WorkspaceCommands::Diff { .. }
                    | WorkspaceCommands::Status { .. }
                    | WorkspaceCommands::Receipt { .. }
                    | WorkspaceCommands::Doctor { stream: false, .. }
                    | WorkspaceCommands::Explain { .. }
                    | WorkspaceCommands::Detect { .. }
                    | WorkspaceCommands::Init { .. },
            }
    )
}

fn should_show_update_notice(cli: &Cli) -> bool {
    io::stderr().is_terminal()
        && !cli.debug
        && !command_requests_json(&cli.command)
        && !matches!(
            &cli.command,
            Commands::SelfUpdate { .. } | Commands::Annotations { .. }
        )
}

fn maybe_append_update_notice(
    output: CommandOutput,
    update_notice_rx: Option<mpsc::Receiver<Option<String>>>,
) -> CommandOutput {
    if output.exit_code != 0 {
        return output;
    }

    let Some(rx) = update_notice_rx else {
        return output;
    };

    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(Some(notice)) => output.with_stderr(Some(format!("\n\x1b[1m{notice}\x1b[0m"))),
        _ => output,
    }
}

fn wait_with_spinner(rx: mpsc::Receiver<CommandOutput>) -> CommandOutput {
    let delay = Duration::from_millis(120);
    let start = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(output) => return output,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return CommandOutput::failure(String::from("command execution failed"));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if start.elapsed() >= delay {
                    break;
                }
            }
        }
    }

    let spinner = CommandSpinner::start();
    let output = rx.recv().expect("command worker should send a report");
    spinner.stop();
    output
}

struct CommandSpinner {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    clear_on_stop: bool,
}

impl CommandSpinner {
    fn start() -> Self {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = std::sync::Arc::clone(&stop);
        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut index = 0usize;
            let mut stderr = io::stderr();
            while !thread_stop.load(std::sync::atomic::Ordering::Relaxed) {
                let frame = frames[index % frames.len()];
                let _ = write!(stderr, "\r🦦 {frame}");
                let _ = stderr.flush();
                index += 1;
                thread::sleep(Duration::from_millis(160));
            }
        });

        Self {
            stop,
            handle: Some(handle),
            clear_on_stop: true,
        }
    }

    fn stop(mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        if self.clear_on_stop && io::stderr().is_terminal() {
            let mut stderr = io::stderr();
            let _ = write!(stderr, "\r\x1b[2K\r\n");
            let _ = stderr.flush();
        }
    }
}

fn is_version_request(args: &[OsString]) -> bool {
    if args.len() < 2 {
        return false;
    }

    let mut has_version = false;
    for arg in &args[1..] {
        let value = arg.to_string_lossy();
        match value.as_ref() {
            "--version" | "-V" => has_version = true,
            "--plain" => {}
            _ => return false,
        }
    }

    has_version
}

fn render_version_output(args: &[OsString]) -> String {
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    if args
        .iter()
        .any(|arg| arg.to_string_lossy().as_ref() == "--plain")
        || !io::stdout().is_terminal()
        || std::env::var_os("NO_COLOR").is_some()
    {
        return format!("🦦 {version}");
    }

    format!("🦦 \x1b[1;38;5;136m{version}\x1b[0m")
}

fn dispatch(cli: Cli) -> CommandOutput {
    commands::set_plain_mode(cli.plain);
    commands::set_concise_mode(cli.concise);
    commands::set_json_mode(matches!(
        &cli.command,
        Commands::Validate { json: true, .. }
            | Commands::Tasks { json: true, .. }
            | Commands::Services { json: true, .. }
            | Commands::Doctor { json: true, .. }
            | Commands::Explain { json: true, .. }
            | Commands::Check { json: true, .. }
            | Commands::Diff { json: true, .. }
            | Commands::Extensions { json: true, .. }
            | Commands::Init { json: true, .. }
            | Commands::Agents { json: true, .. }
            | Commands::Detect { json: true, .. }
            | Commands::Policy { json: true, .. }
            | Commands::Policy {
                command: Some(PolicyCommands::Review { json: true, .. }),
                ..
            }
    ));
    let debug = cli.debug;
    let file = cli.file;
    let command_for_footer = cli.command.clone();
    let output = match cli.command {
        Commands::Validate { json, member, path } => commands::validate(
            path.as_deref(),
            file.as_deref(),
            member.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Tasks {
            json,
            use_cmd,
            member,
            path,
        } => commands::tasks(
            path.as_deref(),
            file.as_deref(),
            &member,
            use_cmd,
            format_from_json(json),
            debug,
        ),
        Commands::Services { json, member, path } => commands::services(
            path.as_deref(),
            file.as_deref(),
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Env {
            json,
            member,
            task,
            path,
        } => commands::env(
            path.as_deref(),
            file.as_deref(),
            member.as_deref(),
            task.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Run {
            task,
            backend,
            lifecycle,
            ephemeral,
            receipt,
            stream,
            member,
            path,
            inputs,
        } => commands::run_command(
            task.as_str(),
            path.as_deref(),
            file.as_deref(),
            ExecutionOverrides {
                backend: backend.map(Into::into),
                lifecycle: if ephemeral {
                    Some(crate::schema::Lifecycle::Ephemeral)
                } else {
                    lifecycle.map(Into::into)
                },
            },
            &member,
            &inputs,
            debug,
            receipt,
            stream,
        ),
        Commands::Doctor {
            json,
            mode,
            member,
            path,
        } => commands::doctor(
            path.as_deref(),
            file.as_deref(),
            &member,
            mode.into(),
            format_from_json(json),
            debug,
        ),
        Commands::Annotations {
            mode,
            format,
            title,
            input,
        } => commands::annotations(mode, format, title.as_deref(), input.as_path()),
        Commands::Explain { json, member, path } => commands::explain(
            path.as_deref(),
            file.as_deref(),
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Check { json, member, path } => commands::check(
            path.as_deref(),
            file.as_deref(),
            &member,
            format_from_json(json),
            debug,
        ),
        Commands::Diff { json, base, target } => commands::diff(
            base.as_path(),
            target.as_path(),
            format_from_json(json),
            debug,
        ),
        Commands::Extensions {
            json,
            run,
            publish,
            member,
            path,
        } => commands::extensions(
            path.as_deref(),
            file.as_deref(),
            &member,
            run.as_deref(),
            publish.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Up {
            json,
            dry_run,
            stream,
            backend,
            lifecycle,
            ephemeral,
            receipt,
            member,
            path,
        } => commands::up(
            path.as_deref(),
            file.as_deref(),
            ExecutionOverrides {
                backend: backend.map(Into::into),
                lifecycle: if ephemeral {
                    Some(crate::schema::Lifecycle::Ephemeral)
                } else {
                    lifecycle.map(Into::into)
                },
            },
            &member,
            format_from_json(json),
            debug,
            dry_run,
            stream,
            receipt,
        ),
        Commands::Clean {
            stale,
            dry_run,
            json,
            member,
            path,
        } => commands::clean(
            path.as_deref(),
            file.as_deref(),
            &member,
            stale,
            dry_run,
            format_from_json(json),
            debug,
        ),
        Commands::Policy {
            json,
            command: None,
            path,
        } => commands::policy(
            path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Policy {
            command:
                Some(PolicyCommands::Review {
                    json,
                    path: review_path,
                }),
            ..
        } => commands::policy_review(
            review_path.as_deref(),
            file.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Uninstall => commands::uninstall(debug),
        Commands::SelfUpdate { version, channel } => commands::self_update(
            version.as_deref(),
            channel.as_ref().map(UpdateChannel::as_str),
            debug,
        ),
        Commands::Init {
            write: _write,
            bootstrap,
            dry_run,
            json,
            path,
        } => {
            if file.is_some() {
                return CommandOutput::failure_with_code(
                    String::from(
                        "`--file` is only supported for commands that read an existing contract",
                    ),
                    2,
                );
            }
            commands::init(
                path.as_deref(),
                !dry_run,
                bootstrap,
                format_from_json(json),
                debug,
            )
        }
        Commands::Agents {
            json,
            write,
            output,
            path,
        } => commands::agents(
            path.as_deref(),
            file.as_deref(),
            write,
            output.as_deref(),
            format_from_json(json),
            debug,
        ),
        Commands::Detect {
            json,
            write,
            dry_run,
            contract,
            merge,
            apply,
            apply_all,
            rewrite,
            yes,
            path,
        } => {
            if file.is_some() {
                return CommandOutput::failure_with_code(
                    String::from(
                        "`--file` is only supported for commands that read an existing contract",
                    ),
                    2,
                );
            }
            commands::detect(
                path.as_deref(),
                write,
                dry_run,
                contract,
                merge,
                &apply,
                apply_all,
                rewrite,
                yes,
                format_from_json(json),
                debug,
            )
        }
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Init {
                write: _write,
                bootstrap,
                dry_run,
                merge,
                json,
                path,
            } => {
                if file.is_some() {
                    return CommandOutput::failure_with_code(
                        String::from(
                            "`--file` is only supported for commands that read an existing contract",
                        ),
                        2,
                    );
                }
                if merge {
                    commands::workspace_init(
                        path.as_deref(),
                        !dry_run,
                        bootstrap,
                        true,
                        false,
                        false,
                        commands::WorkspaceScaffoldSurface::Detect,
                        format_from_json(json),
                        debug,
                    )
                } else if dry_run {
                    commands::workspace_init(
                        path.as_deref(),
                        false,
                        false,
                        false,
                        false,
                        false,
                        commands::WorkspaceScaffoldSurface::Init,
                        format_from_json(json),
                        debug,
                    )
                } else {
                    commands::workspace_init(
                        path.as_deref(),
                        true,
                        bootstrap,
                        false,
                        false,
                        false,
                        commands::WorkspaceScaffoldSurface::Init,
                        format_from_json(json),
                        debug,
                    )
                }
            }
            WorkspaceCommands::Detect {
                write,
                dry_run,
                merge,
                rewrite,
                yes,
                json,
                path,
            } => {
                if file.is_some() {
                    return CommandOutput::failure_with_code(
                        String::from(
                            "`--file` is only supported for commands that read an existing contract",
                        ),
                        2,
                    );
                }
                let write = if merge || rewrite {
                    !dry_run
                } else {
                    write && !dry_run
                };
                commands::workspace_init(
                    path.as_deref(),
                    write,
                    false,
                    merge,
                    rewrite,
                    yes,
                    commands::WorkspaceScaffoldSurface::Detect,
                    format_from_json(json),
                    debug,
                )
            }
            WorkspaceCommands::Validate { json, path } => commands::workspace_validate(
                path.as_deref(),
                file.as_deref(),
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Tasks { json, path } => commands::workspace_tasks(
                path.as_deref(),
                file.as_deref(),
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::List {
                json,
                status,
                repo,
                path,
            } => commands::workspace_list(
                path.as_deref(),
                file.as_deref(),
                status.map(Into::into),
                repo.as_deref(),
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Doctor {
                json,
                stream,
                jobs,
                status,
                severity,
                repo,
                path,
            } => commands::workspace_doctor(
                path.as_deref(),
                file.as_deref(),
                jobs,
                stream,
                commands::WorkspaceDoctorFilters {
                    status: status.into(),
                    severity: severity.into(),
                    repo,
                },
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Explain {
                json,
                jobs,
                status,
                severity,
                repo,
                path,
            } => commands::workspace_explain(
                path.as_deref(),
                file.as_deref(),
                jobs,
                commands::WorkspaceDoctorFilters {
                    status: status.into(),
                    severity: severity.into(),
                    repo,
                },
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Check { json, jobs, path } => commands::workspace_check(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Up {
                json,
                jobs,
                quiet,
                stream,
                receipt,
                path,
            } => commands::workspace_up(
                path.as_deref(),
                file.as_deref(),
                jobs,
                quiet,
                stream,
                format_from_json(json),
                debug,
                receipt,
            ),
            WorkspaceCommands::Refresh {
                json,
                jobs,
                dry_run,
                force,
                prune,
                git_ref,
                quiet,
                stream,
                receipt,
                path,
            } => commands::workspace_refresh(
                path.as_deref(),
                file.as_deref(),
                jobs,
                dry_run,
                force,
                prune,
                git_ref.as_deref(),
                quiet,
                stream,
                format_from_json(json),
                debug,
                receipt,
            ),
            WorkspaceCommands::Diff { json, jobs, path } => commands::workspace_diff(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Status { json, jobs, path } => commands::workspace_status(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Receipt { json, jobs, path } => commands::workspace_receipt(
                path.as_deref(),
                file.as_deref(),
                jobs,
                format_from_json(json),
                debug,
            ),
            WorkspaceCommands::Run {
                task,
                json,
                jobs,
                stream,
                receipt,
                path,
                inputs,
            } => commands::workspace_run(
                task.as_str(),
                path.as_deref(),
                file.as_deref(),
                jobs,
                stream,
                format_from_json(json),
                debug,
                receipt,
                &inputs,
            ),
        },
    };

    finalize_cli_output(output, cli.concise, &command_for_footer)
}

fn format_from_json(json: bool) -> OutputFormat {
    if json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    }
}

fn finalize_cli_output(
    mut output: CommandOutput,
    concise: bool,
    command: &Commands,
) -> CommandOutput {
    let json_requested = command_requests_json(command);

    if output.exit_code != 0 && output.exit_code != 2 && !json_requested {
        let where_label = commands::take_failure_locus()
            .unwrap_or_else(|| command_where_label(command).to_string());
        if let Some(stderr) = output.stderr.take() {
            let structured = commands::stylize_text_failure(where_label.as_str(), stderr.as_str());
            output.stderr = Some(append_try_footer(structured, command));
        }
    }

    if concise && !json_requested {
        output.stdout = collapse_blank_lines(output.stdout);
        if let Some(stderr) = output.stderr.take() {
            output.stderr = Some(collapse_blank_lines(stderr));
        }
    }

    output
}

fn append_try_footer(stderr: String, command: &Commands) -> String {
    const REPO_SETUP_SUGGESTION: &str = "run `ota init` to create a starter contract";
    const WORKSPACE_SETUP_SUGGESTION: &str =
        "run `ota workspace init` to create a starter workspace";
    const WORKSPACE_TASKS_SUGGESTION: &str =
        "fix the failing repo contract with `ota validate`, then rerun `ota workspace tasks`";
    const WORKSPACE_CHECK_SUGGESTION: &str =
        "run `ota workspace check` to review workspace readiness";
    const WORKSPACE_UP_SUGGESTION: &str =
        "run `ota workspace doctor` to review readiness before `ota workspace up`";
    const WORKSPACE_DETECT_DRY_RUN_SUGGESTION: &str =
        "preview the current workspace draft with `ota workspace detect --dry-run`";
    const WORKSPACE_INIT_HELP_SUGGESTION: &str =
        "preview the workspace starter contract with `ota workspace init --dry-run`";
    const AGENTS_SUGGESTION: &str = "run `ota agents --write` to generate AGENTS.md";

    if stderr.contains("Try: ") || stderr.contains("Next:") {
        return tighten_guidance_spacing(stderr);
    }

    let suggestion = match command {
        Commands::Validate { .. } => "run `ota init` to create a starter contract",
        Commands::Tasks { .. } => "run `ota tasks` to inspect available task names",
        Commands::Services { .. } => "run `ota services` to inspect declared services",
        Commands::Env { .. } => "run `ota env --task <name>` to inspect task env requirements",
        Commands::Run { .. } => "run `ota tasks --use` to inspect runnable task usage",
        Commands::Doctor { .. } => "run `ota init` to create a starter contract",
        Commands::Explain { .. } => {
            "run `ota doctor` to inspect readiness findings before `ota explain`"
        }
        Commands::Annotations { .. } => "run `ota annotations --help` to inspect rendering options",
        Commands::Diff { .. } => "compare two contract states with `ota diff <base> <target>`",
        Commands::Extensions { run, .. } => {
            if run.is_some() {
                "inspect available extensions with `ota extensions`"
            } else {
                REPO_SETUP_SUGGESTION
            }
        }
        Commands::Init { .. } => "preview the starter contract with `ota init --dry-run`",
        Commands::Agents { .. } => AGENTS_SUGGESTION,
        Commands::Check { .. } => "run `ota check` to review readiness",
        Commands::Up { .. } => {
            "run `ota up --dry-run` to preview preparation, or `ota doctor` to review readiness"
        }
        Commands::Clean { .. } => "ota clean --help",
        Commands::Policy { .. } => "run `ota policy --help` to inspect policy options",
        Commands::Uninstall => "run `ota uninstall --help` to inspect uninstall options",
        Commands::SelfUpdate { .. } => "run `ota self-update --help` to inspect update options",
        Commands::Detect { .. } => {
            if stderr.contains("failed to parse contract")
                || stderr.contains("failed to parse existing contract")
                || stderr.contains("failed to load existing contract for comparison")
            {
                "run `ota validate` to repair the existing contract, then rerun `ota detect --merge --apply <field name>` to apply selected fields"
            } else {
                "preview the detected contract with `ota detect --dry-run`"
            }
        }
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Init { .. } => WORKSPACE_INIT_HELP_SUGGESTION,
            WorkspaceCommands::Detect { .. } => {
                if stderr.contains("failed to parse contract")
                    || stderr.contains("could not be loaded")
                {
                    "review the failing repo contract with `ota validate` or `ota doctor`, then rerun `ota workspace detect --merge`"
                } else {
                    WORKSPACE_DETECT_DRY_RUN_SUGGESTION
                }
            }
            WorkspaceCommands::Validate { .. } => WORKSPACE_SETUP_SUGGESTION,
            WorkspaceCommands::Tasks { .. } => {
                if stderr.contains("failed to parse contract")
                    || stderr.contains("could not be loaded")
                {
                    WORKSPACE_TASKS_SUGGESTION
                } else {
                    "inspect the workspace contract with `ota workspace validate`, then rerun `ota workspace tasks`"
                }
            }
            WorkspaceCommands::List { .. } => WORKSPACE_SETUP_SUGGESTION,
            WorkspaceCommands::Doctor { .. } => WORKSPACE_SETUP_SUGGESTION,
            WorkspaceCommands::Explain { .. } => {
                "run `ota workspace doctor` to inspect readiness findings before `ota workspace explain`"
            }
            WorkspaceCommands::Check { .. } => WORKSPACE_CHECK_SUGGESTION,
            WorkspaceCommands::Up { .. } => WORKSPACE_UP_SUGGESTION,
            WorkspaceCommands::Refresh { .. } => {
                "run `ota workspace refresh` to sync existing repos before `ota workspace up`, or `ota workspace refresh --dry-run` to preview the refresh commands"
            }
            WorkspaceCommands::Diff { .. } => {
                "run `ota workspace refresh --dry-run` to preview sync commands, or `ota workspace refresh` to reconcile drift after inspecting `ota workspace diff`"
            }
            WorkspaceCommands::Status { .. } => {
                "run `ota workspace doctor` for readiness, `ota workspace diff` for drift, or `ota workspace status` for a compact combined summary"
            }
            WorkspaceCommands::Receipt { .. } => {
                "run `ota workspace status` for a combined readiness-and-drift scan, or `ota workspace receipt` when you want the same state as a receipt artifact"
            }
            WorkspaceCommands::Run { .. } => {
                if stderr.contains("failed to parse contract")
                    || stderr.contains("could not be loaded")
                {
                    "inspect the failing repo contract with `ota validate`, then rerun `ota workspace run <task>`"
                } else {
                    "run `ota workspace tasks` to inspect available task names, then rerun `ota workspace run <task>`"
                }
            }
        },
    };

    let next_header = commands::paint_next_label();
    let next_value = if suggestion.contains('`') {
        suggestion.to_string()
    } else {
        format!("`{suggestion}`")
    };

    if let Some(summary_title) = trailing_summary_title(&stderr) {
        let trimmed = stderr.trim_end_matches('\n');
        if let Some(idx) = trimmed.rfind(summary_title) {
            let (before_summary, summary_block) = trimmed.split_at(idx);
            let before_summary = before_summary.trim_end();
            return tighten_guidance_spacing(commands::stylize_inline_text(&format!(
                "{before_summary}\n{next_header} {next_value}\n\n{summary_block}"
            )));
        }
    }

    tighten_guidance_spacing(commands::stylize_inline_text(&format!(
        "{stderr}\n\n{next_header} {next_value}"
    )))
}

fn trailing_summary_title(stderr: &str) -> Option<&'static str> {
    if stderr.rfind("🦦  RUN SUMMARY").is_some() {
        Some("🦦  RUN SUMMARY")
    } else if stderr.rfind("🦦  UP SUMMARY").is_some() {
        Some("🦦  UP SUMMARY")
    } else {
        None
    }
}

fn tighten_guidance_spacing(text: String) -> String {
    let ends_with_newline = text.ends_with('\n');
    let mut lines = Vec::new();

    for line in text.lines() {
        if line.contains("Next:") || line.contains("Try:") {
            while lines
                .last()
                .is_some_and(|previous: &String| previous.trim().is_empty())
            {
                lines.pop();
            }
        }
        lines.push(line.to_string());
    }

    let mut tightened = lines.join("\n");
    if ends_with_newline {
        tightened.push('\n');
    }
    tightened
}

fn collapse_blank_lines(text: String) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
    }
    out
}

fn command_requests_json(command: &Commands) -> bool {
    match command {
        Commands::Validate { json, .. }
        | Commands::Tasks { json, .. }
        | Commands::Services { json, .. }
        | Commands::Env { json, .. }
        | Commands::Doctor { json, .. }
        | Commands::Explain { json, .. }
        | Commands::Diff { json, .. }
        | Commands::Extensions { json, .. }
        | Commands::Init { json, .. }
        | Commands::Agents { json, .. }
        | Commands::Check { json, .. }
        | Commands::Up { json, .. }
        | Commands::Detect { json, .. }
        | Commands::Policy {
            json,
            command: None,
            ..
        } => *json,
        Commands::Policy {
            command: Some(PolicyCommands::Review { json, .. }),
            ..
        } => *json,
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Init { json, .. }
            | WorkspaceCommands::Detect { json, .. }
            | WorkspaceCommands::Validate { json, .. }
            | WorkspaceCommands::Tasks { json, .. }
            | WorkspaceCommands::List { json, .. }
            | WorkspaceCommands::Doctor { json, .. }
            | WorkspaceCommands::Explain { json, .. }
            | WorkspaceCommands::Check { json, .. }
            | WorkspaceCommands::Up { json, .. }
            | WorkspaceCommands::Refresh { json, .. }
            | WorkspaceCommands::Diff { json, .. }
            | WorkspaceCommands::Status { json, .. }
            | WorkspaceCommands::Receipt { json, .. }
            | WorkspaceCommands::Run { json, .. } => *json,
        },
        Commands::Run { .. }
        | Commands::Clean { .. }
        | Commands::Uninstall
        | Commands::SelfUpdate { .. }
        | Commands::Annotations { .. } => false,
    }
}

fn command_where_label(command: &Commands) -> &'static str {
    match command {
        Commands::Validate { .. } => "ota validate",
        Commands::Tasks { .. } => "ota tasks",
        Commands::Services { .. } => "ota services",
        Commands::Env { .. } => "ota env",
        Commands::Run { .. } => "./ota.yaml",
        Commands::Doctor { .. } => "ota doctor",
        Commands::Annotations { .. } => "ota annotations",
        Commands::Explain { .. } => "ota explain",
        Commands::Extensions { .. } => "ota extensions",
        Commands::Init { .. } => "ota init",
        Commands::Agents { .. } => "ota agents",
        Commands::Check { .. } => "ota check",
        Commands::Diff { .. } => "ota diff",
        Commands::Up { .. } => "ota up",
        Commands::Clean { .. } => "ota clean",
        Commands::Policy { command: None, .. } => "ota policy",
        Commands::Policy {
            command: Some(PolicyCommands::Review { .. }),
            ..
        } => "ota policy review",
        Commands::Uninstall => "ota uninstall",
        Commands::SelfUpdate { .. } => "ota self-update",
        Commands::Detect { .. } => "ota detect",
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Init { .. } => "ota workspace init",
            WorkspaceCommands::Detect { .. } => "ota workspace detect",
            WorkspaceCommands::Validate { .. } => "ota workspace validate",
            WorkspaceCommands::Tasks { .. } => "ota workspace tasks",
            WorkspaceCommands::List { .. } => "ota workspace list",
            WorkspaceCommands::Doctor { .. } => "ota workspace doctor",
            WorkspaceCommands::Explain { .. } => "ota workspace explain",
            WorkspaceCommands::Check { .. } => "ota workspace check",
            WorkspaceCommands::Up { .. } => "ota workspace up",
            WorkspaceCommands::Refresh { .. } => "ota workspace refresh",
            WorkspaceCommands::Diff { .. } => "ota workspace diff",
            WorkspaceCommands::Status { .. } => "ota workspace status",
            WorkspaceCommands::Receipt { .. } => "ota workspace receipt",
            WorkspaceCommands::Run { .. } => "ota workspace run",
        },
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    #[cfg(unix)]
    use std::hash::{Hash, Hasher};
    use std::path::{Path, PathBuf};
    #[cfg(unix)]
    use std::process::Command;
    use std::sync::mpsc;
    #[cfg(unix)]
    use std::time::{Duration, Instant};

    use clap::Parser;
    use serde_json::Value;
    use serde_yaml::Value as YamlValue;
    use tempfile::TempDir;

    use crate::test_support::{CWD_MUTEX, ENV_MUTEX};

    use super::{
        Cli, Commands, PolicyCommands, append_try_footer, collapse_blank_lines, commands,
        maybe_append_update_notice, run_with,
    };
    use crate::output::CommandOutput;

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(dir: &std::path::Path) -> Self {
            let previous = std::env::current_dir().unwrap();
            std::env::set_current_dir(dir).unwrap();
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.previous);
        }
    }

    struct EnvVarGuard {
        name: &'static str,
        original: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(name: &'static str, value: OsString) -> Self {
            let original = std::env::var_os(name);
            unsafe {
                std::env::set_var(name, value);
            }
            Self { name, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.original.take() {
                Some(value) => unsafe {
                    std::env::set_var(self.name, value);
                },
                None => unsafe {
                    std::env::remove_var(self.name);
                },
            }
        }
    }

    #[cfg(unix)]
    fn write_fake_command(bin_dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
        let path = bin_dir.join(name);
        fs::write(&path, body).expect("write fake command");
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("fake command metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make fake command executable");
        path
    }

    #[cfg(windows)]
    fn write_fake_command(bin_dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
        let path = bin_dir.join(name);
        fs::write(&path, body).expect("write fake command");
        path
    }

    #[cfg(unix)]
    fn install_fake_docker(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
state_dir="$(dirname "$0")/docker-state"
mkdir -p "$state_dir"

command="$1"
shift

case "$command" in
  --version|version)
    printf 'Docker version 29.3.1, build deadbeef\n'
    exit 0
    ;;
  info)
    exit 0
    ;;
  ps)
    label_filter=""
    want_stale=0
    format=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -a)
          shift
          ;;
        --filter)
          case "$2" in
            label=*)
              label_filter="${2#label=}"
              ;;
            status=exited|status=dead)
              want_stale=1
              ;;
          esac
          shift 2
          ;;
        --format)
          format="$2"
          shift 2
          ;;
        *)
          shift
          ;;
      esac
    done
    for path_file in "$state_dir"/*.path; do
      [ -e "$path_file" ] || continue
      name=$(basename "$path_file" .path)
      if [ "$want_stale" = "1" ] && [ -f "$state_dir/$name.running" ]; then
        continue
      fi
      if [ -n "$label_filter" ]; then
        [ -f "$state_dir/$name.labels" ] || continue
        grep -Fx "$label_filter" "$state_dir/$name.labels" >/dev/null || continue
      fi
      if [ "$format" = "{{.Names}}" ]; then
        printf "%s\n" "$name"
      else
        printf "%s\n" "$name"
      fi
    done
    exit 0
    ;;
  inspect)
    if [ "$1" = "-f" ]; then
      format="$2"
      name="$3"
      [ -f "$state_dir/$name.path" ] || exit 1
      if [ "$format" = "{{.State.Running}}" ]; then
        if [ -f "$state_dir/$name.running" ]; then
          printf "true\n"
        else
          printf "false\n"
        fi
        exit 0
      fi
      exit 1
    fi
    name="$1"
    [ -f "$state_dir/$name.path" ]
    exit $?
    ;;
  start)
    name="$1"
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    printf "start\n" >> "$host_dir/docker-log.txt"
    exit 0
    ;;
  exec)
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -i)
          shift
          ;;
        --env)
          export "$2"
          shift 2
          ;;
        *)
          name="$1"
          shift
          break
          ;;
      esac
    done
    host_dir=$(cat "$state_dir/$name.path")
    printf "exec\n" >> "$host_dir/docker-log.txt"
    cd "$host_dir" || exit 1
    exec /bin/sh -c "$3"
    ;;
  run)
    detached=0
    mount=""
    name=""
    labels=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -d)
          detached=1
          shift
          ;;
        --rm|-i)
          shift
          ;;
        --entrypoint)
          shift 2
          ;;
        --name)
          name="$2"
          shift 2
          ;;
        --label)
          labels="${labels}${2}
"
          shift 2
          ;;
        -v)
          mount="$2"
          shift 2
          ;;
        -w)
          shift 2
          ;;
        --env)
          export "$2"
          shift 2
          ;;
        *)
          image="$1"
          shift
          break
          ;;
      esac
    done
    host_dir="${mount%%:*}"
    printf "%s" "$image" > "$host_dir/docker-image.txt"
    if [ "$detached" = "1" ]; then
      printf "%s" "$host_dir" > "$state_dir/$name.path"
      : > "$state_dir/$name.running"
      if [ -n "$labels" ]; then
        printf "%s" "$labels" > "$state_dir/$name.labels"
      fi
      printf "run-persistent\n" >> "$host_dir/docker-log.txt"
      exit 0
    fi
    printf "run-ephemeral\n" >> "$host_dir/docker-log.txt"
    cd "$host_dir" || exit 1
    if [ "$1" = "-c" ]; then
      exec /bin/sh -c "$2"
    fi
    if [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
      exec /bin/sh -c "$3"
    fi
    exec /bin/sh -c "$1"
    ;;
  rm)
    shift
    [ "$1" = "-f" ] && shift
    name="$1"
    [ -f "$state_dir/$name.path" ] || exit 1
    host_dir=$(cat "$state_dir/$name.path")
    rm -f "$state_dir/$name.path"
    rm -f "$state_dir/$name.running"
    rm -f "$state_dir/$name.labels"
    printf "rm\n" >> "$host_dir/docker-log.txt"
    exit 0
    ;;
esac

exit 1
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_empty_container_engine(path: &std::path::Path, name: &str) {
        fs::write(
            path,
            format!(
                r#"#!/bin/sh
case "$1" in
  --version|version)
    printf '{name} version 1.0.0\n'
    exit 0
    ;;
  info|ps)
    exit 0
    ;;
esac
exit 1
"#
            ),
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_cargo(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "cargo 1.99.0"
  exit 0
fi
exit 1
"#,
        )
        .unwrap();

        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(unix)]
    fn install_fake_daytona(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
command="$1"
shift

case "$command" in
  exec)
    target="$1"
    shift
    cwd=""
    if [ "$1" = "--cwd" ]; then
      cwd="$2"
      shift 2
    fi
    [ "$1" = "--" ] || exit 1
    shift
    [ -n "$cwd" ] || exit 1
    printf "exec %s\n" "$target" >> "$cwd/daytona-log.txt"
    cd "$cwd" || exit 1
    exec /bin/sh -lc "$3"
    ;;
esac

exit 1
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_ssh(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
target="$1"
shift
[ "$1" = "sh" ] || exit 1
shift
[ "$1" = "-lc" ] || exit 1
shift
printf "exec %s\n" "$target" >> "$OTA_SSH_LOG"
exec /bin/sh -lc "$1"
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    struct FakeSshGuard {
        original_path: Option<std::ffi::OsString>,
        original_log: Option<std::ffi::OsString>,
    }

    #[cfg(unix)]
    impl Drop for FakeSshGuard {
        fn drop(&mut self) {
            unsafe {
                match self.original_path.take() {
                    Some(path) => std::env::set_var("PATH", path),
                    None => std::env::remove_var("PATH"),
                }
                match self.original_log.take() {
                    Some(log) => std::env::set_var("OTA_SSH_LOG", log),
                    None => std::env::remove_var("OTA_SSH_LOG"),
                }
            }
        }
    }

    #[cfg(unix)]
    fn setup_fake_ssh(root: &std::path::Path) -> FakeSshGuard {
        let bin_dir = root.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let ssh_path = bin_dir.join("ssh");
        install_fake_ssh(&ssh_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&ssh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&ssh_path, permissions).unwrap();
        }

        let log_path = root.join("ssh-log.txt");
        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_SSH_LOG");
        let mut path_entries = vec![bin_dir];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_SSH_LOG", &log_path);
        }

        FakeSshGuard {
            original_path,
            original_log,
        }
    }

    #[cfg(unix)]
    fn install_fake_tsh(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
[ "$1" = "ssh" ] || exit 1
shift
target="$1"
shift
[ "$1" = "--" ] || exit 1
shift
[ "$1" = "sh" ] || exit 1
shift
[ "$1" = "-lc" ] || exit 1
shift
printf "exec %s\n" "$target" >> "$OTA_TSH_LOG"
exec /bin/sh -lc "$1"
"#,
        )
        .unwrap();
    }

    #[cfg(unix)]
    fn install_fake_kubectl(path: &std::path::Path) {
        fs::write(
            path,
            r#"#!/bin/sh
[ "$1" = "exec" ] || exit 1
shift
target="$1"
shift
[ "$1" = "--" ] || exit 1
shift
[ "$1" = "sh" ] || exit 1
shift
[ "$1" = "-lc" ] || exit 1
shift
printf "exec %s\n" "$target" >> "$OTA_KUBECTL_LOG"
exec /bin/sh -lc "$1"
"#,
        )
        .unwrap();
    }

    fn compact_path(path: &std::path::Path, _fallback: &str) -> String {
        std::fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string()
    }

    fn compact_contract(path: &std::path::Path) -> String {
        std::fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string()
    }

    fn compact_workspace(path: &std::path::Path) -> String {
        std::fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string()
    }

    fn strip_ansi(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' && chars.peek() == Some(&'[') {
                let _ = chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
            out.push(ch);
        }
        out
    }

    fn normalize_inline_whitespace(value: &str) -> String {
        value.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn normalize_snapshot_text(value: &str) -> String {
        value.replace("\r\n", "\n").trim_end().to_string()
    }

    fn snapshot_file(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots")
            .join(name)
    }

    fn prepend_path(bin_dir: &Path) -> OsString {
        let mut entries = vec![bin_dir.to_path_buf()];
        if let Some(value) = std::env::var_os("PATH") {
            entries.extend(std::env::split_paths(&value));
        }
        std::env::join_paths(entries).expect("join PATH")
    }

    fn assert_text_snapshot(name: &str, actual: &str) {
        let normalized = normalize_snapshot_text(actual);
        if should_update_snapshots() {
            fs::write(snapshot_file(name), &normalized).expect("write snapshot");
            return;
        }
        let expected = fs::read_to_string(snapshot_file(name)).expect("read snapshot");
        assert_eq!(normalized, normalize_snapshot_text(&expected));
    }

    fn assert_text_snapshot_for_dir(name: &str, actual: &str, dir: &std::path::Path) {
        let raw_dir = dir.display().to_string();
        let canonical_dir = fs::canonicalize(dir)
            .map(|value| value.display().to_string())
            .unwrap_or_else(|_| raw_dir.clone());
        let temp_name = dir
            .file_name()
            .map(|value| value.to_string_lossy().to_string());
        let normalized = normalize_snapshot_text(actual)
            .replace(&canonical_dir, "<TMP>")
            .replace(&raw_dir, "<TMP>");
        let normalized = temp_name
            .as_deref()
            .map(|name| normalized.replace(name, "<TMP>"))
            .unwrap_or(normalized)
            .replace("../<TMP>", "<TMP>")
            .replace("./<TMP>", "<TMP>");
        if should_update_snapshots() {
            fs::write(snapshot_file(name), &normalized).expect("write snapshot");
            return;
        }
        let expected = fs::read_to_string(snapshot_file(name)).expect("read snapshot");
        assert_eq!(normalized, normalize_snapshot_text(&expected));
    }

    fn should_update_snapshots() -> bool {
        let Ok(value) = std::env::var("OTA_UPDATE_SNAPSHOTS") else {
            return false;
        };
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    }

    #[test]
    fn validate_json_reports_success() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(
            output.exit_code, 0,
            "stdout={:?} stderr={:?}",
            output.stdout, output.stderr
        );
        assert!(output.stderr.is_none());

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.file_path().display().to_string());
        assert_eq!(json["summary"]["error_count"], 0);
    }

    #[test]
    fn validate_text_external_contract_next_steps_include_explicit_target() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "validate", fixture.file_path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        let contract_path = fs::canonicalize(fixture.file_path())
            .unwrap()
            .display()
            .to_string();
        assert!(stdout.contains(&format!("ota doctor {contract_path}")));
        assert!(stdout.contains(&format!("ota tasks --use {contract_path}")));
    }

    #[test]
    fn validate_text_snapshot_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot_for_dir(
            "validate_premium.txt",
            &strip_ansi(&output.stdout),
            fixture.dir.path(),
        );
    }

    #[test]
    fn validate_narrow_text_snapshot_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let _columns_guard = EnvVarGuard::set("COLUMNS", OsString::from("48"));
        let output = run_with(["ota", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot_for_dir(
            "validate_narrow_premium.txt",
            &strip_ansi(&output.stdout),
            fixture.dir.path(),
        );
    }

    #[test]
    fn validate_json_reports_validation_errors() {
        let fixture = ContractFixture::new(
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
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stdout.is_empty());

        let stderr = output.stderr.unwrap();
        let json: Value = serde_json::from_str(&stderr).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["error_count"], 1);
        assert_eq!(
            json["errors"][0],
            "task `dev` depends on unknown task `setup`"
        );
    }

    #[test]
    fn validate_json_reports_monorepo_member_success() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with([
            "ota",
            "validate",
            "--member",
            "api",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(
            output.exit_code, 0,
            "stdout={:?} stderr={:?}",
            output.stdout, output.stderr
        );
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.file_path().display().to_string());
        assert_eq!(json["summary"]["error_count"], 0);
    }

    #[test]
    fn validate_member_rejects_non_monorepo_root_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "validate", "--member", "api", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            normalize_inline_whitespace(output.stderr.as_deref().unwrap())
                .contains("requires a monorepo root contract")
        );
    }

    #[test]
    fn validate_member_rejects_unknown_monorepo_member() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );

        let output = run_with(["ota", "validate", "--member", "web", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr =
            normalize_inline_whitespace(&strip_ansi(output.stderr.as_deref().unwrap_or_default()));
        assert!(stderr.contains("does not declare monorepo member `web`"));
    }

    #[test]
    fn validate_root_monorepo_rejects_missing_member_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stderr.unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert!(
            json["errors"][0]
                .as_str()
                .unwrap()
                .contains("monorepo member `api`")
        );
    }

    #[test]
    fn validate_root_monorepo_rejects_invalid_member_override() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    depends_on:
      - setup
"#,
        );

        let output = run_with(["ota", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stderr.unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert!(json["errors"].as_array().unwrap().iter().any(|error| {
            error
                .as_str()
                .unwrap()
                .contains("monorepo member `api`: task `test` depends on unknown task `setup`")
        }));
    }

    #[test]
    fn tasks_json_reports_monorepo_member_inherited_and_overridden_tasks() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: 'true'
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: 'true'
"#,
        );

        let output = run_with(["ota", "tasks", "--member", "api", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let tasks = json["tasks"].as_array().unwrap();
        assert!(tasks.iter().any(|task| task["name"] == "setup"));
        assert!(tasks.iter().any(|task| task["name"] == "test"));
    }

    #[test]
    fn diff_reports_structural_changes_and_summary_counts() {
        let base = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );
        let target = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-app
tasks:
  lint:
    run: cargo fmt --check
  test:
    run: cargo test --workspace
"#,
        );

        let output = run_with(["ota", "diff", base.path(), target.path()]);

        assert_ne!(output.exit_code, 0);
        assert!(output.stderr.is_none());
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("DIFF"));
        assert!(stdout.contains("SUMMARY"));
        assert!(stdout.contains("Readiness impact:"));
        assert!(stdout.contains("»"));
        assert!(stdout.contains("DIFFERENT"));
        assert!(stdout.contains("Added:"));
        assert!(stdout.contains("Missing in target:"));
        assert!(stdout.contains("Changed:"));
        assert!(stdout.contains("project.name"));
        assert!(stdout.contains("tasks.lint.run"));
        assert!(stdout.contains("tasks.test.run"));
    }

    #[test]
    fn diff_json_reports_change_paths_and_summary_counts() {
        let base = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );
        let target = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-app
tasks:
  lint:
    run: cargo fmt --check
  test:
    run: cargo test --workspace
"#,
        );

        let output = run_with(["ota", "diff", "--json", base.path(), target.path()]);

        assert_ne!(output.exit_code, 0);
        assert!(output.stderr.is_none());

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["added_count"], 1);
        assert_eq!(json["summary"]["removed_count"], 0);
        assert_eq!(json["summary"]["changed_count"], 2);
        assert_eq!(json["summary"]["readiness_impact"], "changed");
        let paths = json["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|change| change["path"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert!(paths.contains(&"project.name"));
        assert!(paths.contains(&"tasks.lint.run"));
        assert!(paths.contains(&"tasks.test.run"));
    }

    #[test]
    fn diff_json_reports_policy_provenance_on_policy_changes() {
        let base = ContractFixture::new(
            r#"
version: 1
policies:
  env:
    OTA_ENV:
      required: false
      default: local
      allowed:
        - local
        - ci
"#,
        );
        let target = ContractFixture::new(
            r#"
version: 1
policies:
  env:
    OTA_ENV:
      required: false
      default: ci
      allowed:
        - local
        - ci
"#,
        );

        let output = run_with(["ota", "diff", "--json", base.path(), target.path()]);

        assert_ne!(output.exit_code, 0);
        assert!(output.stderr.is_none());

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let changes = json["changes"].as_array().unwrap();
        assert_eq!(changes[0]["path"], "policies.env.OTA_ENV.default");
        assert_eq!(changes[0]["provenance"], "policy");

        let text = run_with(["ota", "diff", base.path(), target.path()]);
        let stdout = strip_ansi(&text.stdout);
        assert!(stdout.contains("Provenance:"));
        assert!(stdout.contains("policy"));
    }

    #[test]
    fn explain_reports_remediation_steps_and_summary_counts() {
        let contract = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "explain", contract.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.is_none());
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("EXPLAIN"));
        assert!(stdout.contains("Overview"));
        assert!(stdout.contains("Plan"));
        assert!(stdout.contains("Findings:"));
        assert!(stdout.contains("Actions:"));
        assert!(stdout.contains("No tasks defined in contract"));
        assert!(stdout.contains("Why:"));
        assert!(stdout.contains("Next:"));
        assert!(!stdout.contains("Code:"));
    }

    #[test]
    fn explain_json_reports_steps_and_summary_counts() {
        let contract = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "explain", "--json", contract.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.is_none());

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["error_count"], 1);
        assert_eq!(json["summary"]["warn_count"], 1);
        assert_eq!(json["summary"]["info_count"], 0);
        assert_eq!(json["summary"]["step_count"], 2);
        let steps = json["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0]["order"], 1);
        assert_eq!(steps[0]["code"], "OTA_TASKS_MISSING");
        assert_eq!(steps[0]["summary"], "No tasks defined in contract");
    }

    #[test]
    fn explain_json_reports_policy_provenance_for_policy_findings() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
"#,
        )
        .unwrap();

        let output = run_with(["ota", "explain", "--json", fixture.path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        assert!(output.stderr.is_none());

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let steps = json["steps"].as_array().unwrap();
        assert_eq!(steps[0]["provenance"], "org policy");

        let text = run_with(["ota", "explain", fixture.path().to_str().unwrap()]);
        let stdout = strip_ansi(&text.stdout);
        assert!(!stdout.contains("Provenance:"));
    }

    #[test]
    fn workspace_explain_reports_policy_provenance_for_policy_findings() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        let api_dir = fixture.path().join("api");
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  api:
    path: api
    required: true
"#,
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: api
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "explain",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(!stdout.contains("Provenance:"));

        let json = run_with([
            "ota",
            "workspace",
            "explain",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);
        let parsed: Value = serde_json::from_str(&json.stdout).unwrap();
        assert_eq!(parsed["repos"][0]["steps"][0]["provenance"], "org policy");
    }

    #[test]
    fn workspace_explain_reports_remediation_steps_and_summary_counts() {
        let fixture = TempDir::new().unwrap();
        let api_dir = fixture.path().join("api");
        let web_dir = fixture.path().join("web");
        fs::create_dir_all(&api_dir).unwrap();
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  api:
    path: api
    required: true
  web:
    path: web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: api
"#,
        )
        .unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "explain",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("WORKSPACE EXPLAIN"));
        assert!(stdout.contains("Overview"));
        assert!(stdout.contains("Plan"));
        assert!(stdout.contains("Actions:"));
        assert!(stdout.contains("api"));
        assert!(stdout.contains("web"));
        assert!(stdout.contains("No tasks defined in contract"));
        assert!(!stdout.contains("Code:"));
    }

    #[test]
    fn workspace_explain_json_reports_steps_and_summary_counts() {
        let fixture = TempDir::new().unwrap();
        let api_dir = fixture.path().join("api");
        let web_dir = fixture.path().join("web");
        fs::create_dir_all(&api_dir).unwrap();
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  api:
    path: api
    required: true
  web:
    path: web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: api
"#,
        )
        .unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "explain",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["repo_count"], 2);
        assert_eq!(json["summary"]["not_ready_count"], 2);
        assert_eq!(json["summary"]["step_count"], 2);
        let repos = json["repos"].as_array().unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0]["steps"].as_array().unwrap().len(), 1);
        assert_eq!(repos[0]["steps"][0]["code"], "OTA_TASKS_MISSING");
        assert_eq!(
            repos[0]["steps"][0]["summary"],
            "No tasks defined in contract"
        );
    }

    #[test]
    fn tasks_json_reports_multiple_monorepo_members_in_order() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
tasks:
  lint:
    run: 'true'
"#,
        );

        let output = run_with([
            "ota",
            "tasks",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[1]["member"], "web");
        assert!(
            members[0]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "test")
        );
        assert!(
            members[1]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "lint")
        );
        assert!(json["tasks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn tasks_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
tasks:
  lint:
    run: printf web
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert!(
            json["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "setup")
        );
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[1]["member"], "web");
        assert!(
            members[0]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "test")
        );
        assert!(
            members[1]["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|task| task["name"] == "lint")
        );
    }

    #[test]
    fn tasks_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("TASKS "));
        assert!(stdout.contains("/ota.yaml"));
        assert!(stdout.contains("[member api]"));
        assert!(stdout.contains("setup"));
        assert!(stdout.contains("test"));
        assert!(stdout.contains("\n\n"));
    }

    #[test]
    fn tasks_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with([
            "ota",
            "tasks",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`--member api` was provided more than once"));
    }

    #[test]
    fn run_executes_monorepo_member_task_in_member_directory() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api > member-output.txt
"#,
        );

        let output = run_with(["ota", "run", "test", "--member", "api", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(
            fixture
                .dir
                .path()
                .join("api")
                .join("member-output.txt")
                .is_file()
        );
        assert!(!fixture.dir.path().join("member-output.txt").is_file());
    }

    #[test]
    fn run_executes_task_for_multiple_monorepo_members_in_order() {
        let fixture = ContractFixture::new_dir();
        let marker = fixture.dir.path().join("member-order.txt");
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
"#,
        );
        fixture.write(
            "api/ota.yaml",
            &format!(
                r#"
project:
  name: api
tasks:
  test:
    run: printf "api\n" >> "{}"
"#,
                marker.display()
            ),
        );
        fixture.write(
            "web/ota.yaml",
            &format!(
                r#"
project:
  name: web
tasks:
  test:
    run: printf "web\n" >> "{}"
"#,
                marker.display()
            ),
        );

        let output = run_with([
            "ota",
            "run",
            "test",
            "--member",
            "api",
            "--member",
            "web",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(fs::read_to_string(marker).unwrap(), "api\nweb\n");
    }

    #[test]
    fn run_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let output = run_with([
            "ota",
            "run",
            "test",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`--member api` was provided more than once"));
    }

    #[test]
    fn doctor_json_reports_monorepo_member_findings() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
tasks:
  test:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "doctor", "--member", "api", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(
            json["findings"][0]["summary"],
            "Missing environment variable: OTA_MEMBER_REQUIRED"
        );
    }

    #[test]
    fn doctor_json_reports_multiple_monorepo_members_in_order() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  setup:
    run: printf ready
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
tasks:
  test:
    run: printf ready
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
tasks:
  test:
    run: printf ready
"#,
        );

        let output = run_with([
            "ota",
            "doctor",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[1]["member"], "web");
        assert_eq!(members[0]["ok"], false);
        assert_eq!(members[1]["ok"], true);
        assert_eq!(
            members[0]["findings"][0]["summary"],
            "Missing environment variable: OTA_MEMBER_REQUIRED"
        );
    }

    #[test]
    fn doctor_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
tasks:
  test:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["error_count"], 1);
        assert_eq!(json["summary"]["warn_count"], 1);
        assert_eq!(json["summary"]["info_count"], 0);
        assert_eq!(json["findings"].as_array().unwrap().len(), 1);
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["ok"], false);
        assert_eq!(
            members[0]["findings"][0]["summary"],
            "Missing environment variable: OTA_MEMBER_REQUIRED"
        );
    }

    #[test]
    fn doctor_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains(&format!(
            "DOCTOR {}",
            compact_contract(&fixture.file_path())
        )));
        assert!(stdout.contains(&format!(
            "DOCTOR {} [member api]",
            compact_contract(&fixture.file_path())
        )));
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Missing environment variable: OTA_MEMBER_REQUIRED"));
        assert!(stdout.contains("\n\n"));
    }

    #[test]
    fn doctor_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "doctor",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`--member api` was provided more than once"));
    }

    #[test]
    fn up_json_runs_inherited_setup_in_monorepo_member_directory() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready > ready.txt
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "up", "--member", "api", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["status"], "READY");
        assert!(fixture.dir.path().join("api").join("ready.txt").is_file());
        assert!(!fixture.dir.path().join("ready.txt").is_file());
    }

    #[test]
    fn up_json_reports_root_monorepo_summary_with_members() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready > ready.txt
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["status"], "READY");
        assert_eq!(json["receipt"]["scope"], "repo");
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["ok"], true);
        assert_eq!(members[0]["status"], "READY");
        assert!(fixture.dir.path().join("ready.txt").is_file());
        assert!(fixture.dir.path().join("api").join("ready.txt").is_file());
    }

    #[test]
    fn up_dry_run_json_reports_root_monorepo_preview_with_members() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  setup:
    run: printf ready > ready.txt
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "up", "--json", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["dry_run"], true);
        assert_eq!(json["status"], "READY");
        assert_eq!(json["phase"], "preview");
        assert!(json["execution"]["task"] == "setup");
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["dry_run"], true);
        assert_eq!(members[0]["phase"], "preview");
        assert!(!fixture.dir.path().join("ready.txt").is_file());
        assert!(!fixture.dir.path().join("api").join("ready.txt").is_file());
    }

    #[test]
    fn up_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains(&format!("UP {}", compact_contract(&fixture.file_path()))));
        assert!(stdout.contains(&format!(
            "UP {} [member api]",
            compact_contract(&fixture.file_path())
        )));
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Missing environment variable: OTA_MEMBER_REQUIRED"));
        assert!(stdout.contains("\n\n"));
    }

    #[test]
    fn up_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "up",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`--member api` was provided more than once"));
    }

    #[cfg(unix)]
    #[test]
    fn clean_reports_persistent_container_cleanup() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: exit 0
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
        }

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        fixture.dir.path().display().to_string().hash(&mut hasher);
        "ghcr.io/ota/test:latest".hash(&mut hasher);
        "docker".hash(&mut hasher);
        let container_name = format!("ota-{:x}", hasher.finish());
        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(
            state_dir.join(format!("{container_name}.path")),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        let original_required = std::env::var_os("OTA_CONTAINER_ONLY_REQUIRED");
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", "present");
        }

        let output = run_with(["ota", "clean", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_required {
            Some(value) => unsafe {
                std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_CONTAINER_ONLY_REQUIRED");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            strip_ansi(&output.stdout),
            format!("Cleaned {}", compact_contract(&fixture.file_path()))
        );
        assert!(fixture.dir.path().join("docker-log.txt").exists());
    }

    #[test]
    fn clean_reports_no_cleanup_needed_for_remote_backend() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "clean", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            strip_ansi(&output.stdout),
            format!(
                "No cleanup needed for {}",
                compact_contract(&fixture.file_path())
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn clean_stale_dry_run_lists_labelled_and_legacy_ota_containers() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let podman_path = bin_dir.join("podman");
        install_fake_empty_container_engine(&podman_path, "podman");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
            let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&podman_path, permissions).unwrap();
        }

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(
            state_dir.join("ota-labelled.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-labelled.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=persistent\n",
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-legacy.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();

        let original_path = std::env::var_os("PATH");
        let original_required = std::env::var_os("OTA_CONTAINER_ONLY_REQUIRED");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", "present");
        }

        let output = run_with(["ota", "clean", "--stale", "--dry-run"]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_required {
            Some(value) => unsafe {
                std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_CONTAINER_ONLY_REQUIRED");
            },
        }

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Dry run stale ota-managed containers (2)"));
        assert!(stdout.contains("ota-labelled"));
        assert!(stdout.contains("ota-legacy"));
        assert!(state_dir.join("ota-labelled.path").exists());
        assert!(state_dir.join("ota-legacy.path").exists());
    }

    #[cfg(unix)]
    #[test]
    fn clean_stale_json_reports_removed_containers() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let podman_path = bin_dir.join("podman");
        install_fake_empty_container_engine(&podman_path, "podman");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
            let mut permissions = fs::metadata(&podman_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&podman_path, permissions).unwrap();
        }

        let state_dir = bin_dir.join("docker-state");
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(
            state_dir.join("ota-labelled.path"),
            fixture.dir.path().display().to_string(),
        )
        .unwrap();
        fs::write(
            state_dir.join("ota-labelled.labels"),
            "dev.ota.managed=true\ndev.ota.lifecycle=persistent\n",
        )
        .unwrap();

        let original_path = std::env::var_os("PATH");
        let original_required = std::env::var_os("OTA_CONTAINER_ONLY_REQUIRED");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", "present");
        }

        let output = run_with(["ota", "clean", "--stale", "--json"]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_required {
            Some(value) => unsafe {
                std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_CONTAINER_ONLY_REQUIRED");
            },
        }

        assert_eq!(output.exit_code, 0);
        let json: serde_json::Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["scope"], "stale");
        assert_eq!(json["dry_run"], false);
        assert_eq!(json["summary"]["matched_count"], 1);
        assert_eq!(json["summary"]["removed_count"], 1);
        assert_eq!(json["containers"][0]["ownership"], "label");
        assert!(!state_dir.join("ota-labelled.path").exists());
    }

    #[test]
    fn clean_stale_rejects_contract_scoped_arguments() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "clean", "--stale", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`ota clean --stale` is global"));
    }

    #[cfg(unix)]
    #[test]
    fn clean_stale_fails_when_container_engine_query_fails() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "docker",
            r#"#!/bin/sh
if [ "$1" = "--version" ] || [ "$1" = "version" ]; then
  printf "Docker version 29.3.1, build deadbeef\n"
  exit 0
fi
if [ "$1" = "ps" ]; then
  printf "Docker daemon is not running\n" >&2
  exit 1
fi
if [ "$1" = "info" ]; then
  exit 0
fi
exit 1
"#,
        );
        write_fake_command(
            &bin_dir,
            "podman",
            r#"#!/bin/sh
if [ "$1" = "--version" ] || [ "$1" = "version" ]; then
  printf "podman version 5.0.0\n"
  exit 0
fi
if [ "$1" = "ps" ]; then
  printf "Podman daemon is not running\n" >&2
  exit 1
fi
if [ "$1" = "info" ]; then
  exit 0
fi
exit 1
"#,
        );

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "clean", "--stale"]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 1);
        let stderr =
            normalize_inline_whitespace(&strip_ansi(output.stderr.as_deref().unwrap_or_default()));
        assert!(stderr.contains("could not list stale ota containers"));
        assert!(stderr.contains("Docker daemon is not running"));
    }

    #[cfg(unix)]
    #[test]
    fn clean_stale_continues_when_one_engine_queries_successfully() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "docker",
            r#"#!/bin/sh
if [ "$1" = "--version" ] || [ "$1" = "version" ]; then
  printf "Docker version 29.3.1, build deadbeef\n"
  exit 0
fi
if [ "$1" = "ps" ]; then
  printf "ota-test-stale\n"
  exit 0
fi
if [ "$1" = "rm" ]; then
  exit 0
fi
if [ "$1" = "info" ]; then
  exit 0
fi
exit 1
"#,
        );
        write_fake_command(
            &bin_dir,
            "podman",
            r#"#!/bin/sh
if [ "$1" = "--version" ] || [ "$1" = "version" ]; then
  printf "podman version 5.0.0\n"
  exit 0
fi
if [ "$1" = "ps" ]; then
  printf "Cannot connect to Podman.\n" >&2
  exit 1
fi
if [ "$1" = "info" ]; then
  exit 0
fi
exit 1
"#,
        );

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "clean", "--stale"]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        let stdout = normalize_inline_whitespace(&strip_ansi(&output.stdout));
        assert!(stdout.contains("Cleaned stale ota-managed containers (1)"));
        assert!(stdout.contains("ota-test-stale"));
    }

    #[test]
    fn clean_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with(["ota", "clean", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains(&format!(
            "No cleanup needed for {}",
            compact_contract(&fixture.file_path())
        )));
        assert!(stdout.contains(&format!(
            "No cleanup needed for {} [member api]",
            compact_contract(&fixture.file_path())
        )));
        assert!(stdout.contains("\n\n"));
    }

    #[test]
    fn clean_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "clean",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`--member api` was provided more than once"));
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_ephemeral_container_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Note:       using a fresh container image for this run"));
        assert!(output.stderr.as_deref().unwrap_or_default().is_empty());
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "ready"
        );
        assert!(
            fs::read_to_string(fixture.dir.path().join("docker-log.txt"))
                .unwrap()
                .contains("run-ephemeral")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_ssh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let ssh_path = bin_dir.join("ssh");
        install_fake_ssh(&ssh_path);
        let log_path = fixture.dir.path().join("ssh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&ssh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&ssh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_SSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_SSH_LOG", &log_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_SSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_SSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_tsh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let tsh_path = bin_dir.join("tsh");
        install_fake_tsh(&tsh_path);
        let log_path = fixture.dir.path().join("tsh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&tsh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&tsh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_TSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_TSH_LOG", &log_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_TSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_TSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_runs_setup_in_kubectl_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: pod/ota-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let kubectl_path = bin_dir.join("kubectl");
        install_fake_kubectl(&kubectl_path);
        let log_path = fixture.dir.path().join("kubectl-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&kubectl_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&kubectl_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_KUBECTL_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_KUBECTL_LOG", &log_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_KUBECTL_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_KUBECTL_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec pod/ota-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn up_overrides_native_contract_to_use_ephemeral_container_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  backends:
    container:
      image: ghcr.io/ota/test:latest
env:
  OTA_CONTAINER_ONLY_REQUIRED:
    required: true
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_required = std::env::var_os("OTA_CONTAINER_ONLY_REQUIRED");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", "present");
        }

        let output = run_with([
            "ota",
            "up",
            "--mode",
            "container",
            "--ephemeral",
            fixture.path(),
        ]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_required {
            Some(value) => unsafe {
                std::env::set_var("OTA_CONTAINER_ONLY_REQUIRED", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_CONTAINER_ONLY_REQUIRED");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Note:       using a fresh container image for this run"));
        assert!(output.stderr.as_deref().unwrap_or_default().is_empty());
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "ready"
        );
        assert!(
            fs::read_to_string(fixture.dir.path().join("docker-log.txt"))
                .unwrap()
                .contains("run-ephemeral")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_daytona_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
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
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let daytona_path = bin_dir.join("daytona");
        install_fake_daytona(&daytona_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&daytona_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&daytona_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "run", "setup", "--receipt", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(fixture.dir.path().join("daytona-log.txt"))
                .unwrap()
                .contains("exec sandbox-dev")
        );
        let rendered = strip_ansi(&format!(
            "{}\n{}",
            output.stdout,
            output.stderr.as_deref().unwrap_or_default()
        ));
        assert!(rendered.contains("Steps:"));
        assert!(rendered.contains("Summary"));
        assert!(rendered.contains("Target:"));
        assert!(rendered.contains("sandbox-dev"));
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_ssh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let ssh_path = bin_dir.join("ssh");
        install_fake_ssh(&ssh_path);
        let log_path = fixture.dir.path().join("ssh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&ssh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&ssh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_SSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_SSH_LOG", &log_path);
        }

        let output = run_with(["ota", "run", "setup", "--receipt", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_SSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_SSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_tsh_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let tsh_path = bin_dir.join("tsh");
        install_fake_tsh(&tsh_path);
        let log_path = fixture.dir.path().join("tsh-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&tsh_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&tsh_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_TSH_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_TSH_LOG", &log_path);
        }

        let output = run_with(["ota", "run", "setup", "--receipt", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_TSH_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_TSH_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec sandbox-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_uses_kubectl_remote_backend() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: pod/ota-dev
      cwd: {}
env:
  OTA_REMOTE_ENV:
    default: remote
tasks:
  setup:
    run: printf "$OTA_REMOTE_ENV" > prepared.txt
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let kubectl_path = bin_dir.join("kubectl");
        install_fake_kubectl(&kubectl_path);
        let log_path = fixture.dir.path().join("kubectl-log.txt");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&kubectl_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&kubectl_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let original_log = std::env::var_os("OTA_KUBECTL_LOG");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
            std::env::set_var("OTA_KUBECTL_LOG", &log_path);
        }

        let output = run_with(["ota", "run", "setup", "--ephemeral", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
        match original_log {
            Some(value) => unsafe {
                std::env::set_var("OTA_KUBECTL_LOG", value);
            },
            None => unsafe {
                std::env::remove_var("OTA_KUBECTL_LOG");
            },
        }

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "remote"
        );
        assert!(
            fs::read_to_string(&log_path)
                .unwrap()
                .contains("exec pod/ota-dev")
        );
    }

    #[cfg(unix)]
    #[test]
    fn run_remote_failure_preserves_child_exit_code() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            &format!(
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
      cwd: {}
tasks:
  fail:
    run: exit 7
"#,
                fixture.path()
            ),
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let daytona_path = bin_dir.join("daytona");
        install_fake_daytona(&daytona_path);
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&daytona_path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&daytona_path, permissions).unwrap();
        }

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "run", "fail", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 7);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        let why = stderr.find("Why:").unwrap();
        let next = stderr.find("Next:").unwrap();
        let summary = stderr.find("RUN SUMMARY").unwrap();
        assert!(why < summary);
        assert!(why < next);
        assert!(next < summary);
        assert!(stderr.contains("Mode:"));
        assert!(stderr.contains("remote"));
        assert!(!stderr.contains("Why: 🦦  RUN SUMMARY"));
    }

    #[test]
    fn run_with_unsupported_remote_provider_fails_cleanly() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: unknown
      target: sandbox-dev
tasks:
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "run", "setup", "--ephemeral", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr =
            normalize_inline_whitespace(&strip_ansi(output.stderr.as_deref().unwrap_or_default()));
        assert!(stderr.contains(
            "INVALID ota.yaml | - `execution.backends.remote.provider` `unknown` is not supported"
        ));
        assert!(stderr.contains(
            "declare a matching `backend_provider` extension or use a built-in provider"
        ));
    }

    #[test]
    fn run_with_kubectl_remote_provider_missing_target_fails_with_guidance() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
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
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr =
            normalize_inline_whitespace(&strip_ansi(output.stderr.as_deref().unwrap_or_default()));
        assert!(stderr.contains(
            "provider `kubectl` requires `execution.backends.remote.target` (example: `pod/ota-dev`)"
        ));
    }

    #[test]
    fn run_with_tsh_remote_provider_missing_target_fails_with_guidance() {
        let fixture = ContractFixture::new(
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
  setup:
    run: printf ready
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(
            normalize_inline_whitespace(&strip_ansi(output.stderr.as_deref().unwrap_or_default()))
                .contains(
                    "provider `tsh` requires `execution.backends.remote.target` (example: `user@host`)"
                )
        );
    }

    #[test]
    fn validate_discovers_member_contract_from_member_directory_without_member_flag() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );

        let nested = fixture.dir.path().join("api");
        let output = run_with(["ota", "validate", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        let expected = compact_contract(&fixture.dir.path().join("api").join("ota.yaml"));
        assert!(strip_ansi(&output.stdout).contains(&expected));
    }

    #[test]
    fn run_executes_member_task_from_member_directory_without_member_flag() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api > member-output.txt
"#,
        );

        let nested = fixture.dir.path().join("api");
        let output = run_with(["ota", "run", "test", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        assert!(nested.join("member-output.txt").is_file());
    }

    #[test]
    fn exit_code_usage_errors_are_two() {
        let output = run_with(["ota", "validate", "--unknown-flag"]);

        assert_eq!(output.exit_code, 2);
        assert!(strip_ansi(output.stderr.as_deref().unwrap_or_default()).contains("unexpected"));
    }

    #[test]
    fn root_help_lists_concise_and_verbose_flags() {
        let output = run_with(["ota", "--help"]);

        assert_eq!(output.exit_code, 0);
        let help = output
            .stderr
            .as_deref()
            .expect("help text should be present in stderr");
        assert!(help.contains("Diagnose, prepare, and run repos from one explicit contract."));
        assert!(help.contains("Doctor first, contract second."));
        assert!(help.contains("--concise"));
        assert!(help.contains("--verbose"));
        assert!(help.contains("ota doctor"));
        assert!(help.contains("ota explain"));
        assert!(help.contains("ota init --dry-run"));
        assert!(help.contains("ota detect --dry-run ."));
        assert!(help.contains("ota up"));
        let doctor = help.find("\n  doctor").unwrap();
        let explain = help.find("\n  explain").unwrap();
        let up = help.find("\n  up").unwrap();
        let run = help.find("\n  run").unwrap();
        let init = help.find("\n  init").unwrap();
        let detect = help.find("\n  detect").unwrap();
        assert!(doctor < explain && explain < up && up < run && run < init && init < detect);
    }

    #[test]
    fn detect_help_lists_rewrite_flags() {
        let output = run_with(["ota", "detect", "--help"]);

        assert_eq!(output.exit_code, 0);
        let help = output
            .stderr
            .as_deref()
            .expect("help text should be present in stderr");
        assert!(help.contains("--rewrite"));
        assert!(help.contains("--yes"));
    }

    #[test]
    fn workspace_detect_help_lists_rewrite_flags() {
        let output = run_with(["ota", "workspace", "detect", "--help"]);

        assert_eq!(output.exit_code, 0);
        let help = output
            .stderr
            .as_deref()
            .expect("help text should be present in stderr");
        assert!(help.contains("--rewrite"));
        assert!(help.contains("--yes"));
    }

    #[test]
    fn workspace_services_typo_points_to_repo_services_and_workspace_doctor() {
        let output = run_with(["ota", "workspace", "services"]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("unrecognized subcommand 'services'"));
        assert!(stderr.contains("ota services"));
        assert!(stderr.contains("ota workspace doctor"));
    }

    #[test]
    fn failure_adds_try_footer_when_next_is_not_present() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  build:
    run: cargo build
"#,
        );

        let output = run_with(["ota", "run", "missing", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Next: run `ota tasks --use` to inspect runnable task usage"));
    }

    #[test]
    fn run_failure_try_footer_stays_tight_before_run_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  fail:
    run: exit 7
"#,
        );

        let output = run_with(["ota", "run", "fail", fixture.path()]);

        assert_eq!(output.exit_code, 7);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Why: task `fail` failed with exit code 7"));
        assert!(stderr.contains("Next: run `ota tasks --use` to inspect runnable task usage"));
        assert!(stderr.contains(
            "Why: task `fail` failed with exit code 7\nNext: run `ota tasks --use` to inspect runnable task usage"
        ));
        assert!(!stderr.contains("Why: task `fail` failed with exit code 7\n\nNext:"));
        assert!(!stderr.contains(
            "Next: run `ota tasks --use` to inspect runnable task usage\n\n\nRUN SUMMARY"
        ));
    }

    #[test]
    fn append_try_footer_collapses_existing_next_gap_before_run_summary() {
        let stderr = "◉ ERROR  Operation failed\nWhere: ./ota.yaml\nWhy: task `install-from-source` failed with exit code 101\n\nNext: run `ota tasks --use` to inspect runnable task usage\n\n🦦  RUN SUMMARY\n\nScope:      repo";
        let rendered = strip_ansi(&append_try_footer(
            stderr.to_string(),
            &Commands::Run {
                task: String::from("install-from-source"),
                backend: None,
                lifecycle: None,
                ephemeral: false,
                receipt: false,
                stream: false,
                member: Vec::new(),
                path: None,
                inputs: Vec::new(),
            },
        ));

        assert!(rendered.contains(
            "Why: task `install-from-source` failed with exit code 101\nNext: run `ota tasks --use` to inspect runnable task usage"
        ));
        assert!(
            !rendered
                .contains("Why: task `install-from-source` failed with exit code 101\n\nNext:")
        );
    }

    #[test]
    fn run_non_interactive_failure_shows_output_excerpt_and_stream_hint() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  fail:
    script: |
      i=1
      while [ "$i" -le 24 ]; do
        printf 'line-%02d\n' "$i"
        i=$((i + 1))
      done
      exit 7
"#,
        );

        let output = run_with(["ota", "run", "fail", fixture.path()]);

        assert_eq!(output.exit_code, 7);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("run `ota tasks --use` to inspect runnable task usage"));
        assert!(stderr.contains("RUN SUMMARY"));
        assert!(!stderr.contains("Task output:"));
    }

    #[test]
    fn run_non_interactive_success_shows_captured_output_excerpt() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    script: |
      i=1
      while [ "$i" -le 15 ]; do
        printf 'line-%02d\n' "$i"
        i=$((i + 1))
      done
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("RUN SUMMARY"));
        assert!(!stderr.contains("Task output:"));
    }

    #[test]
    fn collapse_blank_lines_reduces_consecutive_empty_lines() {
        let input = "a\n\n\nb\n\n\n\nc\n";
        let output = collapse_blank_lines(input.to_string());
        assert_eq!(output, "a\nb\nc");
    }

    #[test]
    fn appends_update_notice_to_successful_output() {
        let (tx, rx) = mpsc::channel();
        tx.send(Some(String::from(
            "A newer `\u{1b}[38;5;130mota\u{1b}[39m` release is available: \u{1b}[92mv9.9.9\u{1b}[39m\nRun `\u{1b}[38;5;130mota self-update\u{1b}[39m` or `\u{1b}[38;5;130mota upgrade\u{1b}[39m` to update.",
        )))
        .unwrap();

        let output =
            maybe_append_update_notice(CommandOutput::success(String::from("ok")), Some(rx));

        assert_eq!(output.stdout, "ok");
        assert_eq!(
            output.stderr,
            Some(String::from(
                "\n\x1b[1mA newer `\x1b[38;5;130mota\x1b[39m` release is available: \x1b[92mv9.9.9\x1b[39m\nRun `\x1b[38;5;130mota self-update\x1b[39m` or `\x1b[38;5;130mota upgrade\x1b[39m` to update.\x1b[0m"
            ))
        );
    }

    #[test]
    fn self_update_does_not_use_command_spinner() {
        assert!(!super::command_supports_spinner(
            &super::Commands::SelfUpdate {
                version: None,
                channel: None,
            }
        ));
    }

    #[test]
    fn up_uses_command_spinner() {
        assert!(super::command_supports_spinner(&super::Commands::Up {
            path: None,
            json: false,
            dry_run: false,
            stream: false,
            backend: None,
            lifecycle: None,
            ephemeral: false,
            receipt: false,
            member: Vec::new(),
        }));
    }

    #[test]
    fn up_stream_disables_command_spinner() {
        assert!(!super::command_supports_spinner(&super::Commands::Up {
            path: None,
            json: false,
            dry_run: false,
            stream: true,
            backend: None,
            lifecycle: None,
            ephemeral: false,
            receipt: false,
            member: Vec::new(),
        }));
    }

    #[test]
    fn clean_uses_command_spinner() {
        assert!(super::command_supports_spinner(&super::Commands::Clean {
            stale: false,
            dry_run: false,
            json: false,
            member: Vec::new(),
            path: None,
        }));
    }

    #[test]
    fn version_output_respects_plain_flag() {
        let output = super::render_version_output(&[
            std::ffi::OsString::from("ota"),
            std::ffi::OsString::from("--version"),
            std::ffi::OsString::from("--plain"),
        ]);

        assert_eq!(output, format!("🦦 v{}", env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn validate_rejects_unknown_top_level_keys() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
unexpected: true
"#,
        );

        let output = run_with(["ota", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        assert!(strip_ansi(output.stderr.as_deref().unwrap_or_default()).contains("unexpected"));
    }

    #[test]
    fn validate_accepts_top_level_extensions_as_inert_contract_data() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: /workspace
extensions:
  demo:
    kind: backend_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  build:
    run: cargo build
"#,
        );

        let output = run_with(["ota", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(output.stderr.is_none());
    }

    #[test]
    fn top_level_extensions_are_accepted_consistently_across_repo_commands() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: /workspace
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  build:
    run: cargo build
"#,
        );

        for args in [
            vec!["ota", "validate", fixture.path()],
            vec!["ota", "tasks", fixture.path()],
            vec!["ota", "doctor", fixture.path()],
            vec!["ota", "check", fixture.path()],
            vec!["ota", "up", fixture.path()],
            vec!["ota", "run", "setup", fixture.path()],
        ] {
            let output = run_with(args);
            assert_ne!(output.exit_code, 2);
            let stderr = output.stderr.as_deref().unwrap_or("");
            assert!(!stderr.contains("unknown field"));
        }
    }

    #[test]
    fn validate_discovers_contract_from_nested_directory() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );
        let nested = fixture.dir.path().join("apps").join("web");
        fs::create_dir_all(&nested).unwrap();

        let _cwd = CurrentDirGuard::enter(&nested);
        let output = run_with(["ota", "validate"]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("VALIDATE"));
        assert!(stdout.contains("VALID"));
        assert!(stdout.contains("VALIDATE ./ota.yaml"));
        assert!(stdout.contains("run `ota doctor` to inspect readiness"));
        assert!(stdout.contains("run `ota tasks --use` to inspect runnable task usage"));
    }

    #[test]
    fn validate_missing_explicit_path_includes_contract_creation_next_steps() {
        let fixture = TempDir::new().unwrap();
        let missing = fixture.path().join("ota.yaml");

        let output = run_with(["ota", "validate", missing.to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("contract path does not exist"));
        assert!(stderr.contains("run `ota init` to create a starter contract"));
        assert!(stderr.contains("run `ota detect --dry-run` to preview inferred fields"));
        assert!(stderr.contains("run `ota detect --write` to write a detected contract"));
    }

    #[test]
    fn validate_supports_explicit_file_override() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with([
            "ota",
            "--file",
            fixture.file_path().to_str().unwrap(),
            "validate",
        ]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("VALIDATE"));
        assert!(stdout.contains("VALID"));
        assert!(stdout.contains(&compact_contract(&fixture.file_path())));
    }

    #[test]
    fn tasks_json_is_sorted_and_machine_readable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    description: Run tests
    notes: |
      Use this to verify the code before merging.
    run: cargo test
    safe_for_agent: true
  build:
    category: build
    run: cargo build
    depends_on:
      - test
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["tasks"][0]["name"], "build");
        assert_eq!(json["tasks"][0]["kind"], "run");
        assert_eq!(json["tasks"][0]["category"], "build");
        assert_eq!(json["tasks"][0]["depends_on"][0], "test");
        assert_eq!(json["tasks"][1]["name"], "test");
        assert_eq!(json["tasks"][1]["safe_for_agent"], true);
        assert_eq!(
            json["tasks"][1]["notes"],
            "Use this to verify the code before merging.\n"
        );
    }

    #[test]
    fn tasks_json_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
  test:
    run: cargo test
agent:
  entrypoint: setup
  default_task: test
  safe_tasks:
    - test
  verify_after_changes:
    - test
  writable_paths:
    - src
  bootstrap:
    ota:
      note: Only install ota if it is missing and installation is approved.
      sh: curl -fsSL https://dist.ota.run/install.sh | sh
      powershell: irm https://dist.ota.run/install.ps1 | iex
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["agent"]["entrypoint"], "setup");
        assert_eq!(json["agent"]["default_task"], "test");
        assert_eq!(json["agent"]["safe_tasks"][0], "test");
        assert_eq!(json["agent"]["verify_after_changes"][0], "test");
        assert_eq!(json["agent"]["writable_paths"][0], "src");
        assert_eq!(
            json["agent"]["bootstrap"]["ota"]["sh"],
            "curl -fsSL https://dist.ota.run/install.sh | sh"
        );
        assert_eq!(
            json["agent"]["bootstrap"]["ota"]["powershell"],
            "irm https://dist.ota.run/install.ps1 | iex"
        );
    }

    #[test]
    fn tasks_json_reports_script_tasks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    description: Prepare the repo
    notes: |
      Use this after cloning the repo.
    script: |
      printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["tasks"][0]["name"], "setup");
        assert_eq!(json["tasks"][0]["kind"], "script");
        assert_eq!(json["tasks"][0]["script"], "printf ready > prepared.txt\n");
        assert!(json["tasks"][0].get("run").is_none());
    }

    #[test]
    fn tasks_json_reports_selected_os_variant() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: ./scripts/setup.sh
    variants:
      - when:
          os: windows
        run: .\scripts\setup.ps1
      - when:
          os: macos
        run: ./scripts/setup-macos.sh
"#,
        );

        let output = run_with(["ota", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let task = &json["tasks"][0];

        match std::env::consts::OS {
            "macos" => {
                assert_eq!(task["run"], "./scripts/setup-macos.sh");
                assert_eq!(task["selected_variant_os"], "macos");
            }
            _ => {
                assert_eq!(task["run"], "./scripts/setup.sh");
                assert!(task.get("selected_variant_os").is_none());
            }
        }

        assert_eq!(task["variants"][0]["os"], "windows");
        assert_eq!(task["variants"][1]["os"], "macos");
    }

    #[test]
    fn tasks_text_reports_script_kind() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    notes: |
      Use this to initialize the repo before running other tasks.
    script: |
      printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("setup"));
        assert!(stdout.contains("Kind: script"));
        assert!(stdout.contains("Notes:"));
        assert!(stdout.contains("Use: `ota run setup`"));
    }

    #[test]
    fn tasks_text_indents_multiline_notes() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  validate:
    notes: |
      Example: `ota run validate`
      Use this before opening a pull request.
      It validates all example contracts, workspace contracts, and local markdown links.
    run: python3 -c "print('ok')"
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("Notes: Example: `ota run validate`"));
        assert!(stdout.contains("\n  Use this before opening a pull request."));
        assert!(stdout.contains(
            "\n  It validates all example contracts, workspace contracts, and local markdown links."
        ));
    }

    #[test]
    fn tasks_use_separates_blocks_with_blank_line() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  build:
    description: Build the site for production
    run: python3 -c "print('build')"
  ci:
    description: Canonical local verification
    run: python3 -c "print('ci')"
"#,
        );

        let output = run_with(["ota", "tasks", "--use", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        let build_idx = lines
            .iter()
            .position(|line| line.contains("build `ota run build`"))
            .expect("build task present");
        let ci_idx = lines
            .iter()
            .position(|line| line.contains("ci `ota run ci`"))
            .expect("ci task present");

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("build `ota run build`"));
        assert_eq!(ci_idx, build_idx + 3);
        assert!(lines[build_idx + 1].starts_with("  Description: Build the site for production"));
        assert!(lines[build_idx + 2].is_empty());
        assert!(lines[ci_idx + 1].starts_with("  Description: Canonical local verification"));
    }

    #[test]
    fn tasks_text_style_snapshot_contains_rich_header_and_bullets() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  build:
    run: cargo build
  dev:
    run: cargo run
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);
        let normalized = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(normalized.contains("TASKS "));
        assert!(normalized.contains("/ota.yaml"));
        assert!(normalized.contains("build"));
        assert!(normalized.contains("dev"));
        assert!(!normalized.contains("- Task:"));
    }

    #[test]
    fn tasks_use_prints_compact_usage_lines() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    description: Start the dev server
    notes: |
      Use this for local development and manual verification.
    run: cargo run
  start:
    description: Start the release server
    run: cargo run --release
  typecheck:
    description: Type-check the crate
    run: cargo check
"#,
        );

        let output = run_with(["ota", "tasks", "--use", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("TASKS "));
        assert!(stdout.contains("dev `ota run dev`"));
        assert!(stdout.contains("Description: Start the dev server"));
        assert!(stdout.contains("Notes:"));
        assert!(stdout.contains("start `ota run start`"));
        assert!(stdout.contains("typecheck `ota run typecheck`"));
        assert!(!stdout.contains("Command Preview:"));
    }

    #[test]
    fn annotations_renders_doctor_findings_into_github_annotations() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = dir.path().join("doctor.json");
        fs::write(
            &input,
            r#"
{
  "ok": false,
  "path": "/tmp/ota.yaml",
  "summary": {
    "error_count": 1,
    "warn_count": 1,
    "info_count": 0,
    "primary_blocker": {
      "severity": "error",
      "summary": "Missing container execution backend CLI: docker, podman",
      "why": "Required because execution.preferred=container",
      "next": "install one of the supported container engines"
    }
  },
  "findings": [
    {
      "severity": "error",
      "summary": "Missing container execution backend CLI: docker, podman",
      "why": "Required because execution.preferred=container",
      "next": "install one of the supported container engines"
    },
    {
      "severity": "warn",
      "summary": "Lifecycle is advisory only",
      "why": "This repo still runs tasks natively when lifecycle is ephemeral",
      "next": "review execution.lifecycle"
    }
  ]
}
"#,
        )
        .expect("write doctor json");

        let output = run_with([
            "ota",
            "annotations",
            "--mode",
            "doctor",
            "--format",
            "github",
            "--input",
            input.to_str().expect("utf8 path"),
        ]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains(
            "::notice title=ota doctor primary blocker::Missing container execution backend CLI: docker, podman | install one of the supported container engines"
        ));
        assert!(stdout.contains(
            "::error title=ota doctor finding::Missing container execution backend CLI: docker, podman | install one of the supported container engines"
        ));
        assert!(stdout.contains(
            "::warning title=ota doctor finding::Lifecycle is advisory only | review execution.lifecycle"
        ));
    }

    #[test]
    fn annotations_renders_workspace_doctor_findings_into_plain_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let input = dir.path().join("workspace-doctor.json");
        fs::write(
            &input,
            r#"
{
  "ok": false,
  "path": "/tmp/ota.workspace.yaml",
  "summary": {
    "repo_count": 1,
    "ready_count": 0,
    "not_ready_count": 1,
    "error_count": 1,
    "warn_count": 1,
    "info_count": 0,
    "primary_blocker": {
      "repo": "api",
      "severity": "error",
      "summary": "Missing container execution backend CLI: docker, podman",
      "why": "Required because execution.preferred=container",
      "next": "install one of the supported container engines"
    }
  },
  "repos": [
    {
      "name": "api",
      "path": "./api",
      "contract_path": "./api/ota.yaml",
      "required": true,
      "ok": false,
      "summary": {
        "error_count": 1,
        "warn_count": 1,
        "info_count": 0
      },
      "findings": [
        {
          "severity": "error",
          "summary": "Missing container execution backend CLI: docker, podman",
          "why": "Required because execution.preferred=container",
          "next": "install one of the supported container engines"
        },
        {
          "severity": "warn",
          "summary": "Lifecycle is advisory only",
          "why": "This repo still runs tasks natively when lifecycle is ephemeral",
          "next": "review execution.lifecycle"
        }
      ]
    }
  ]
}
"#,
        )
        .expect("write workspace doctor json");

        let output = run_with([
            "ota",
            "annotations",
            "--mode",
            "workspace-doctor",
            "--format",
            "plain",
            "--input",
            input.to_str().expect("utf8 path"),
        ]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains(
            "NOTICE: ota workspace doctor primary blocker [api]: Missing container execution backend CLI: docker, podman | install one of the supported container engines"
        ));
        assert!(stdout.contains(
            "ERROR: ota workspace doctor finding [api]: ./api: Missing container execution backend CLI: docker, podman | install one of the supported container engines"
        ));
        assert!(stdout.contains(
            "WARNING: ota workspace doctor finding [api]: ./api: Lifecycle is advisory only | review execution.lifecycle"
        ));
    }

    #[test]
    fn doctor_reports_contract_drift_as_warning_findings() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: existing
tasks:
  ci:
    run: cargo test
"#,
        )
        .expect("write ota.yaml");
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[package]
name = "detected"
version = "0.1.0"
"#,
        )
        .expect("write Cargo.toml");

        let _guard = CurrentDirGuard::enter(dir.path());
        let output = run_with(["ota", "doctor", "--json"]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert!(json["summary"]["warn_count"].as_u64().unwrap_or(0) >= 1);
        assert!(
            json["findings"]
                .as_array()
                .expect("findings array")
                .iter()
                .any(|finding| {
                    finding["summary"]
                        .as_str()
                        .unwrap_or_default()
                        .starts_with("Contract drift:")
                        && finding["ownership"] == "repo_contract"
                        && finding["provenance"]
                            .as_str()
                            .unwrap_or_default()
                            .contains("ota detect")
                }),
            "expected at least one contract-drift warning"
        );
    }

    #[test]
    fn doctor_does_not_report_semantically_equivalent_runtime_drift() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let _java_path = write_fake_command(
            &bin_dir,
            "java",
            if cfg!(windows) {
                "@echo off\r\necho java version \"25.0.2\"\r\n"
            } else {
                "#!/bin/sh\nprintf 'java version \"25.0.2\"\\n'\n"
            },
        );
        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).expect("join PATH");
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }
        fs::write(
            dir.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: existing
runtimes:
  java: <=21
tasks:
  ci:
    run: cargo test
"#,
        )
        .expect("write ota.yaml");
        fs::write(dir.path().join(".java-version"), "21\n").expect("write .java-version");

        let _guard = CurrentDirGuard::enter(dir.path());
        let output = run_with(["ota", "doctor", "--json"]);

        match original_path {
            Some(ref path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let findings = json["findings"].as_array().expect("findings array");
        assert!(
            findings.iter().any(|finding| {
                finding["summary"].as_str().unwrap_or_default()
                    == "Version mismatch for runtime: java"
            }),
            "expected the real runtime mismatch finding"
        );
        assert!(
            findings.iter().any(|finding| {
                finding["summary"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("Contract drift: `runtimes.java`")
            }),
            "expected the runtime drift warning"
        );
    }

    #[cfg(unix)]
    #[test]
    fn doctor_reports_tool_drift_warning_without_duplicate_blocker_line() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker_path)
                .expect("fake docker metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker_path, permissions).expect("make fake docker executable");
        }
        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).expect("join PATH");
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }
        fs::write(
            dir.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: existing
tools:
  docker: "20"
tasks:
  ci:
    run: cargo test
"#,
        )
        .expect("write ota.yaml");

        let _guard = CurrentDirGuard::enter(dir.path());
        let output = run_with(["ota", "doctor"]);

        match original_path {
            Some(ref path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(
            stdout.contains("Version mismatch for tool: docker"),
            "expected the docker version mismatch to be present"
        );
        assert!(
            stdout.contains("Review contract drift (2)"),
            "expected the grouped contract drift warning"
        );
        assert!(
            stdout.contains("tools.docker"),
            "expected the docker drift detail to be present"
        );
    }

    #[test]
    fn services_text_lists_service_details() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -U qredex -d qredex
    timeout: 30
"#,
        );

        let output = run_with(["ota", "services", fixture.path()]);
        let body = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(body.contains("SERVICES "));
        assert!(body.contains("postgres [required]"));
        assert!(body.contains("Provider: docker-compose"));
        assert!(body.contains("Start: docker compose up -d postgres"));
        assert!(body.contains("Stop: docker compose stop postgres"));
        assert!(body.contains("Healthcheck: pg_isready -U qredex -d qredex"));
        assert!(body.contains("Timeout: 30s"));
        assert!(body.contains("Managed By:"));
    }

    #[test]
    fn services_json_reports_service_summaries() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    healthcheck: pg_isready -U qredex -d qredex
"#,
        );

        let output = run_with(["ota", "services", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.file_path().display().to_string());
        assert_eq!(json["services"][0]["name"], "postgres");
        assert_eq!(json["services"][0]["provider"], "docker-compose");
        assert_eq!(json["services"][0]["required"], true);
    }

    #[test]
    fn plain_mode_disables_icons_and_uses_ascii_bullets() {
        let detect_fixture = ContractFixture::new_dir();
        detect_fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        let doctor_fixture = ContractFixture::new_dir();
        doctor_fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-web
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );
        let validate_fixture = ContractFixture::new_dir();
        validate_fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-web
"#,
        );
        let tasks_fixture = ContractFixture::new_dir();
        tasks_fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-web
tasks:
  build:
    run: cargo build
"#,
        );
        let env_fixture = ContractFixture::new_dir();
        env_fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-web
env:
  DATABASE_URL:
    required: true
    default: postgres://contract
tasks:
  test:
    env:
      DATABASE_URL: postgres://task
    run: cargo test
"#,
        );

        let detect = run_with([
            "ota",
            "--plain",
            "detect",
            "--dry-run",
            detect_fixture.path(),
        ]);
        let doctor = run_with(["ota", "--plain", "doctor", doctor_fixture.path()]);
        let explain = run_with(["ota", "--plain", "explain", doctor_fixture.path()]);
        let up = run_with(["ota", "--plain", "up", doctor_fixture.path()]);
        let validate = run_with(["ota", "--plain", "validate", validate_fixture.path()]);
        let tasks = run_with(["ota", "--plain", "tasks", tasks_fixture.path()]);
        let env = run_with([
            "ota",
            "--plain",
            "env",
            "--task",
            "test",
            env_fixture.path(),
        ]);

        assert_eq!(detect.exit_code, 0);
        assert!(detect.stdout.contains("DETECT PREVIEW "));
        assert!(
            detect
                .stdout
                .contains("\nNext:\n-  run `ota detect --write")
        );
        assert!(!detect.stdout.contains("🦦 "));
        assert!(!detect.stdout.contains("▸"));
        for output in [doctor, explain, up] {
            assert_eq!(output.exit_code, 0);
            let body = format!(
                "{}\n{}",
                output.stdout,
                output.stderr.as_deref().unwrap_or_default()
            );
            assert!(!body.contains("🦦"));
            assert!(!body.contains("➤"));
            assert!(!body.contains("»"));
            assert!(!body.contains("●"));
            assert!(!body.contains("→"));
            assert!(!body.contains("✦"));
            assert!(!body.contains("▸"));
        }
        for output in [&validate, &tasks, &env] {
            assert_eq!(output.exit_code, 0);
            let body = format!(
                "{}\n{}",
                output.stdout,
                output.stderr.as_deref().unwrap_or_default()
            );
            assert!(!body.contains("🦦"));
            assert!(!body.contains("➤"));
            assert!(!body.contains("»"));
            assert!(!body.contains("●"));
            assert!(!body.contains("→"));
            assert!(!body.contains("✦"));
            assert!(!body.contains("▸"));
        }
        assert!(strip_ansi(&validate.stdout).contains("VALIDATE"));
        assert!(strip_ansi(&tasks.stdout).contains("TASKS"));
        assert!(strip_ansi(&env.stdout).contains("ENV"));
    }

    #[test]
    fn tasks_text_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
agent:
  entrypoint: setup
  safe_tasks:
    - setup
  writable_paths:
    - src
"#,
        );

        let output = run_with(["ota", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Agent"));
        assert!(stdout.contains("Entrypoint: `setup`"));
        assert!(stdout.contains("Safe tasks: `setup`"));
        assert!(stdout.contains("Writable paths: `src`"));
        assert!(stdout.contains("Overview"));
        assert!(stdout.contains("Tasks: 1"));
        assert!(stdout.contains("Agent-safe: 0"));
    }

    #[test]
    fn doctor_json_reports_findings_once() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_JSON_MISSING:
    required: true
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let object = json.as_object().unwrap();
        assert_eq!(object.get("ok").unwrap(), &Value::Bool(false));
        assert!(object.get("findings").unwrap().is_array());
        assert_eq!(object.len(), 6);
    }

    #[test]
    fn doctor_json_includes_policy_provenance() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
"#,
        )
        .unwrap();

        let output = run_with(["ota", "doctor", "--json", fixture.path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = &json["findings"][0];
        assert_eq!(finding["code"], "OTA_POLICY_PACK_VIOLATION");
        assert_eq!(finding["category"], "policy");
        assert_eq!(finding["owner"], "org_policy");
        assert_eq!(finding["summary"], "Repo does not satisfy org policy pack");
        assert_eq!(finding["policy_outcome"], "blocked_by_policy");
        assert_eq!(finding["policy_reason"], "missing_required_sections");
        assert_eq!(finding["policy_source"], "org");
        assert_eq!(finding["install_scope"], "repo_local");
        assert_eq!(finding["mutation_allowed"], false);
        assert_eq!(finding["evidence"]["source"], "org_policy");
        assert!(finding["evidence"]["observed"].is_string());
        assert!(finding["evidence"]["expected"].is_string());
        assert_eq!(json["summary"]["verdict"], "policy_blocked");
        assert_eq!(json["summary"]["agent_verdict"], "not_ready");
    }

    #[test]
    fn policy_review_parses_review_subcommand() {
        let cli = Cli::parse_from(["ota", "policy", "review", "--json", "./ota.yaml"]);
        let command = cli.command;

        match &command {
            Commands::Policy {
                json: false,
                command:
                    Some(PolicyCommands::Review {
                        json: true,
                        path: Some(path),
                    }),
                path: None,
            } => {
                assert_eq!(path.as_path(), Path::new("./ota.yaml"));
            }
            other => panic!("unexpected command parsed: {other:?}"),
        }

        assert_eq!(super::command_where_label(&command), "ota policy review");
    }

    #[test]
    fn doctor_json_includes_policy_backed_provisioning_sources() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java: "22"
tools:
  maven: "3.9"
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      source_config:
        feed: internal-jdk
      approved_versions:
        - "22"
    maven:
      source: approved-manager
      approved_versions:
        - "3.9"
  adapter_bootstrap:
    mise:
      source: brew
      approved_versions:
        - "4.4"
"#,
        )
        .unwrap();

        let output = run_with(["ota", "doctor", "--json", fixture.path().to_str().unwrap()]);

        assert_ne!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let provisioning = json["provisioning"]
            .as_object()
            .expect("provisioning plan should be present");
        assert_eq!(provisioning["allowed"].as_array().unwrap().len(), 2);
        let provisioning_request = json["provisioning_request"]
            .as_object()
            .expect("provisioning request should be present");
        assert_eq!(provisioning_request["actions"].as_array().unwrap().len(), 2);
        assert_eq!(provisioning_request["actions"][0]["kind"], "select_source");
        assert_eq!(
            provisioning_request["actions"][0]["source_config"]["feed"],
            "internal-jdk"
        );
        let adapter_bootstrap = json["adapter_bootstrap"]
            .as_object()
            .expect("adapter bootstrap payload should be present");
        assert_eq!(
            adapter_bootstrap["plan"]["allowed"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            adapter_bootstrap["request"]["actions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(adapter_bootstrap["request"]["actions"][0]["source"], "brew");
        let findings = json["findings"].as_array().unwrap();
        let provisioning_finding = findings
            .iter()
            .find(|finding| finding["summary"] == "Policy-backed provisioning sources are declared")
            .expect("provisioning finding should be present");
        assert_eq!(provisioning_finding["severity"], "info");
        assert_eq!(
            provisioning_finding["policy_reason"],
            "policy_backed_provisioning_declared"
        );
        assert_eq!(provisioning_finding["policy_source"], "org");
        assert_eq!(provisioning_finding["install_scope"], "repo_local");
        assert_eq!(provisioning_finding["mutation_allowed"], false);
        assert!(
            provisioning_finding["why"]
                .as_str()
                .unwrap()
                .contains("source_config: feed=internal-jdk")
        );
        let finding = findings
            .iter()
            .find(|finding| finding["summary"] == "Adapter bootstrap sources are declared")
            .expect("adapter bootstrap finding should be present");
        assert_eq!(finding["summary"], "Adapter bootstrap sources are declared");
        assert_eq!(finding["severity"], "info");
        assert_eq!(finding["policy_outcome"], "policy_surface_available");
        assert_eq!(
            finding["policy_reason"],
            "policy_backed_adapter_bootstrap_declared"
        );
        assert_eq!(finding["policy_source"], "org");
        assert_eq!(finding["install_scope"], "repo_local");
        assert_eq!(finding["mutation_allowed"], false);
        assert!(
            finding["why"]
                .as_str()
                .unwrap()
                .contains("bootstrap missing adapter binaries")
        );
        assert_eq!(provisioning["allowed"][0]["name"], "java");
        assert_eq!(provisioning["allowed"][1]["name"], "maven");
        assert_eq!(provisioning["actions"].as_array().unwrap().len(), 2);
        assert_eq!(provisioning["actions"][0]["kind"], "select_source");
        assert_eq!(provisioning["actions"][0]["target_kind"], "runtime");
    }

    #[test]
    fn doctor_json_includes_execution_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
execution:
  preferred: remote
  supported:
    - remote
  lifecycle: ephemeral
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: /workspace
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["summary"]["error_count"], 0);
        assert_eq!(json["summary"]["warn_count"], 2);
        assert_eq!(json["summary"]["info_count"], 0);
        assert_eq!(json["summary"]["primary_blocker"]["severity"], "warn");
        assert_eq!(
            json["summary"]["primary_blocker"]["summary"],
            "Ephemeral lifecycle is advisory only in V1"
        );
        assert_eq!(json["mode"], "native");
        assert_eq!(json["execution"]["preferred"], "remote");
        assert_eq!(json["execution"]["supported"][0], "remote");
        assert_eq!(json["execution"]["lifecycle"], "ephemeral");
        assert_eq!(json["execution"]["backends"]["remote"]["provider"], "ssh");
        assert_eq!(
            json["execution"]["backends"]["remote"]["target"],
            "user@host"
        );
        assert_eq!(json["execution"]["backends"]["remote"]["cwd"], "/workspace");
        assert_eq!(json["extensions"]["demo"]["kind"], "check_provider");
        assert_eq!(json["extensions"]["demo"]["command"], "ota-ext-demo");
        assert_eq!(json["extensions"]["demo"]["api_version"], 1);
    }

    #[test]
    fn doctor_json_reports_selected_mode() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: persistent
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  ci:
    run: echo ci
runtimes:
  node: "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo v22.0.0\r\n  exit /b 0\r\n)\r\necho unsupported\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  shift\n  while [ \"$#\" -gt 1 ]; do\n    if [ \"$1\" = \"-lc\" ]; then\n      echo unsupported >&2\n      exit 1\n    fi\n    if [ \"$1\" = \"-c\" ] && [ \"$2\" = \"node --version\" ]; then\n      echo 'v22.0.0'\n      exit 0\n    fi\n    shift\n  done\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with([
            "ota",
            "doctor",
            "--mode",
            "container",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["mode"], "container");
        assert_eq!(json["summary"]["error_count"], 0);
    }

    #[test]
    fn doctor_json_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
  test:
    run: cargo test
agent:
  entrypoint: setup
  verify_after_changes:
    - test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["agent"]["entrypoint"], "setup");
        assert_eq!(json["agent"]["verify_after_changes"][0], "test");
    }

    #[test]
    fn doctor_json_reports_suspicious_ssh_remote_target_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert!(json["findings"].as_array().unwrap().iter().any(|finding| {
            finding["severity"] == "warn"
                && finding["summary"] == "Suspicious remote target for ssh: sandbox-dev"
        }));
    }

    #[test]
    fn doctor_json_reports_suspicious_kubectl_remote_target_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert!(json["findings"].as_array().unwrap().iter().any(|finding| {
            finding["severity"] == "warn"
                && finding["summary"] == "Suspicious remote target for kubectl: ota-dev"
        }));
    }

    #[test]
    fn doctor_json_suspicious_kubectl_warning_object_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Suspicious remote target for kubectl: ota-dev")
            .expect("expected suspicious kubectl target warning");

        assert_eq!(finding["severity"], "warn");
        assert_eq!(
            finding["summary"],
            "Suspicious remote target for kubectl: ota-dev"
        );
        assert_eq!(
            finding["why"],
            "remote provider `kubectl` is currently validated for `pod/<name>` style targets, but current target `ota-dev` does not start with `pod/`"
        );
        assert_eq!(
            finding["next"],
            "set `execution.backends.remote.target` to a pod target such as `pod/ota-dev`"
        );
    }

    #[test]
    fn doctor_json_suspicious_ssh_warning_object_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Suspicious remote target for ssh: sandbox-dev")
            .expect("expected suspicious ssh target warning");

        assert_eq!(finding["severity"], "warn");
        assert_eq!(
            finding["summary"],
            "Suspicious remote target for ssh: sandbox-dev"
        );
        assert_eq!(
            finding["why"],
            "remote provider `ssh` usually expects a `user@host` style target, but current target `sandbox-dev` has no `@` separator"
        );
        assert_eq!(
            finding["next"],
            "set `execution.backends.remote.target` to a host target such as `user@host` for provider `ssh`"
        );
    }

    #[test]
    fn doctor_json_suspicious_tsh_warning_object_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = json["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Suspicious remote target for tsh: sandbox-dev")
            .expect("expected suspicious tsh target warning");

        assert_eq!(finding["severity"], "warn");
        assert_eq!(
            finding["summary"],
            "Suspicious remote target for tsh: sandbox-dev"
        );
        assert_eq!(
            finding["why"],
            "remote provider `tsh` usually expects a `user@host` style target, but current target `sandbox-dev` has no `@` separator"
        );
        assert_eq!(
            finding["next"],
            "set `execution.backends.remote.target` to a host target such as `user@host` for provider `tsh`"
        );
    }

    #[test]
    fn doctor_json_reports_all_suspicious_remote_target_warnings_together() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - ssh-member
    - tsh-member
    - kubectl-member
"#,
        );
        fixture.write(
            "ssh-member/ota.yaml",
            r#"
project:
  name: ssh-member
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );
        fixture.write(
            "tsh-member/ota.yaml",
            r#"
project:
  name: tsh-member
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        );
        fixture.write(
            "kubectl-member/ota.yaml",
            r#"
project:
  name: kubectl-member
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let members = json["members"].as_array().unwrap();
        assert_eq!(members.len(), 3);
        let summaries = members
            .iter()
            .flat_map(|member| member["findings"].as_array().into_iter().flatten())
            .map(|finding| finding["summary"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        assert!(
            summaries
                .iter()
                .any(|summary| summary == "Suspicious remote target for ssh: sandbox-dev")
        );
        assert!(
            summaries
                .iter()
                .any(|summary| summary == "Suspicious remote target for tsh: sandbox-dev")
        );
        assert!(
            summaries
                .iter()
                .any(|summary| summary == "Suspicious remote target for kubectl: ota-dev")
        );
    }

    #[test]
    fn doctor_text_includes_agent_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: cargo build
agent:
  entrypoint: setup
  safe_tasks:
    - setup
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Agent"));
        assert!(stdout.contains("Entrypoint: `setup`"));
        assert!(stdout.contains("Safe tasks: `setup`"));
    }

    #[test]
    fn doctor_text_lists_extensions() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: /workspace
extensions:
  demo:
    kind: backend_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  setup:
    run: cargo build
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("Execution"));
        assert!(stdout.contains("Preferred: `remote`"));
        assert!(stdout.contains("Remote:"));
        assert!(stdout.contains("provider `ssh`"));
        assert!(stdout.contains("Extensions:"));
        assert!(stdout.contains("demo"));
        assert!(stdout.contains("Kind: backend_provider"));
        assert!(stdout.contains("Command: ota-ext-demo"));
    }

    #[test]
    fn doctor_text_reports_env_precedence_and_policy() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
env:
  OTA_TEST_BASE_URL:
    required: true
    default: http://localhost:8080
policies:
  env:
    OTA_TEST_BASE_URL: http://policy.example.com
tasks:
  setup:
    run: cargo build
"#,
        );

        let original = std::env::var_os("OTA_TEST_BASE_URL");
        unsafe {
            std::env::set_var("OTA_TEST_BASE_URL", "http://example.com");
        }

        let output = run_with(["ota", "doctor", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        match original {
            Some(value) => unsafe { std::env::set_var("OTA_TEST_BASE_URL", value) },
            None => unsafe { std::env::remove_var("OTA_TEST_BASE_URL") },
        }

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("Env precedence:"));
        assert!(stdout.contains(
            "Env: `OTA_TEST_BASE_URL` (policy, required, default=http://localhost:8080)"
        ));
        assert!(stdout.contains("policy > process > contract default > required missing"));
        assert!(stdout.contains("OTA_TEST_BASE_URL"));
        assert!(stdout.contains("required, default=http://localhost:8080"));
    }

    #[test]
    fn doctor_json_reports_policy_env_provenance() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
env:
  OTA_TEST_BASE_URL:
    required: true
    default: http://localhost:8080
policies:
  env:
    OTA_TEST_BASE_URL: http://policy.example.com
tasks:
  setup:
    run: cargo build
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();

        assert_eq!(output.exit_code, 0);
        assert_eq!(json["execution"]["env"][0]["name"], "OTA_TEST_BASE_URL");
        assert_eq!(
            json["execution"]["env"][0]["policy"],
            "http://policy.example.com"
        );
    }

    #[test]
    fn env_text_reports_contract_and_task_sources() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  DATABASE_URL:
    required: true
    default: postgres://contract
  OTA_TEST_ENV_HOME:
    default: /opt/jdk-21
tasks:
  test:
    env:
      DATABASE_URL: postgres://task
      CI: "true"
    run: cargo test
"#,
        );

        let output = run_with(["ota", "env", "--task", "test", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("ENV"));
        assert!(stdout.contains("Task: test"));
        assert!(stdout.contains("Contract env"));
        assert!(stdout.contains("Task env"));
        assert!(stdout.contains("DATABASE_URL"));
        assert!(stdout.contains("Source: task"));
        assert!(stdout.contains("OTA_TEST_ENV_HOME"));
        assert!(stdout.contains("Source: default"));
        assert!(stdout.contains("CI"));
        assert!(stdout.contains("Status: task"));
    }

    #[test]
    fn env_json_reports_missing_required_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  DATABASE_URL:
    required: true
tasks:
  test:
    run: cargo test
"#,
        );

        let original = std::env::var_os("DATABASE_URL");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }

        let output = run_with(["ota", "env", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();

        match original {
            Some(value) => unsafe { std::env::set_var("DATABASE_URL", value) },
            None => unsafe { std::env::remove_var("DATABASE_URL") },
        }

        assert_eq!(output.exit_code, 1);
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["missing_count"], 1);
        assert_eq!(json["env"][0]["status"], "missing");
        assert!(json["env"][0]["next"].as_str().unwrap().contains("ota env"));
    }

    #[test]
    fn extensions_text_lists_descriptors() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  setup:
    run: cargo build
"#,
        );

        let output = run_with(["ota", "extensions", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("EXTENSIONS"));
        assert!(stdout.contains("demo"));
        assert!(stdout.contains("Kind: check_provider"));
        assert!(stdout.contains("Command: ota-ext-demo"));
    }

    #[test]
    fn extensions_json_reports_descriptors() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  setup:
    run: cargo build
"#,
        );

        let output = run_with(["ota", "extensions", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["extensions"]["demo"]["kind"], "check_provider");
        assert_eq!(json["extensions"]["demo"]["command"], "ota-ext-demo");
        assert_eq!(json["extensions"]["demo"]["api_version"], 1);
    }

    #[test]
    fn extensions_run_executes_allowed_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: echo extension-run
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with(["ota", "extensions", "--run", "demo", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("EXTENSION RUN"));
        assert!(stdout.contains("demo"));
        assert!(stdout.contains("Kind: check_provider"));
        assert!(stdout.contains("Command: echo extension-run"));
        assert!(stdout.contains("Exit Code: 0"));
        assert!(stdout.contains("Stdout:"));
        assert!(stdout.contains("extension-run"));
    }

    #[test]
    fn extensions_run_rejects_export_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  release-upload:
    kind: export_provider
    command: echo release-upload
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with([
            "ota",
            "extensions",
            "--run",
            "release-upload",
            fixture.path(),
        ]);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap());

        assert_eq!(output.exit_code, 2);
        assert!(
            stderr.contains("extension `release-upload` kind `export_provider` is not executable")
        );
        assert!(stderr.contains("ota extensions --run"));
        assert!(stderr.contains("expected kind: `check_provider`"));
    }

    #[test]
    fn extensions_run_json_reports_execution_result() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: echo extension-run
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with([
            "ota",
            "extensions",
            "--run",
            "demo",
            "--json",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["extension"]["name"], "demo");
        assert_eq!(json["extension"]["kind"], "check_provider");
        assert_eq!(json["extension"]["command"], "echo extension-run");
        assert_eq!(json["extension"]["api_version"], 1);
        assert_eq!(json["exit_code"], 0);
        assert!(json["stdout"].as_str().unwrap().contains("extension-run"));
    }

    #[test]
    fn extensions_text_lists_export_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  release-upload:
    kind: export_provider
    command: ota-ext-upload
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with(["ota", "extensions", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("release-upload"));
        assert!(stdout.contains("Kind: export_provider"));
        assert!(stdout.contains("Command: ota-ext-upload"));
    }

    #[test]
    fn extensions_text_lists_backend_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: ota-ext-backend
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with(["ota", "extensions", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("backend-demo"));
        assert!(stdout.contains("Kind: backend_provider"));
        assert!(stdout.contains("Command: ota-ext-backend"));
    }

    #[test]
    fn extensions_json_reports_backend_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: ota-ext-backend
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with(["ota", "extensions", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(
            json["extensions"]["backend-demo"]["kind"],
            "backend_provider"
        );
        assert_eq!(
            json["extensions"]["backend-demo"]["command"],
            "ota-ext-backend"
        );
        assert_eq!(json["extensions"]["backend-demo"]["api_version"], 1);
    }

    #[test]
    fn extensions_publish_executes_export_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  release-upload:
    kind: export_provider
    command: echo release-upload
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with([
            "ota",
            "extensions",
            "--publish",
            "release-upload",
            fixture.path(),
        ]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("EXTENSION RUN"));
        assert!(stdout.contains("release-upload"));
        assert!(stdout.contains("Kind: export_provider"));
        assert!(stdout.contains("Command: echo release-upload"));
    }

    #[test]
    fn extensions_run_rejects_backend_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: echo backend-demo
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with(["ota", "extensions", "--run", "backend-demo", fixture.path()]);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap());

        assert_eq!(output.exit_code, 2);
        assert!(
            stderr.contains("extension `backend-demo` kind `backend_provider` is not executable")
        );
        assert!(stderr.contains("ota extensions --run"));
        assert!(stderr.contains("expected kind: `check_provider`"));
    }

    #[test]
    fn extensions_publish_rejects_backend_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: echo backend-demo
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with([
            "ota",
            "extensions",
            "--publish",
            "backend-demo",
            fixture.path(),
        ]);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap());

        assert_eq!(output.exit_code, 2);
        assert!(
            stderr.contains("extension `backend-demo` kind `backend_provider` is not executable")
        );
        assert!(stderr.contains("ota extensions --publish"));
        assert!(stderr.contains("expected kind: `export_provider`"));
    }

    #[test]
    fn extensions_publish_rejects_check_provider_descriptor() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
extensions:
  demo:
    kind: check_provider
    command: echo extension-run
    api_version: 1
tasks:
  setup:
    run: echo setup
"#,
        );

        let output = run_with(["ota", "extensions", "--publish", "demo", fixture.path()]);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap());

        assert_eq!(output.exit_code, 2);
        assert!(stderr.contains("extension `demo` kind `check_provider` is not executable"));
        assert!(stderr.contains("ota extensions --publish"));
        assert!(stderr.contains("expected kind: `export_provider`"));
    }

    #[test]
    fn init_dry_run_renders_starter_contract_and_annotations() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "init", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("INIT"));
        assert!(stdout.contains("Mode: detected"));
        assert!(stdout.contains(
            "Next: review this starter contract, edit it if needed, then run `ota init "
        ));
        assert!(stdout.contains("name: ota-web"));
        assert!(stdout.contains("tools.pnpm"));
        assert!(!fixture.file_path().exists());
    }

    #[test]
    fn agents_preview_renders_scaffold_when_agent_block_is_missing() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "agents", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("AGENTS"));
        assert!(stdout.contains("Target:"));
        assert!(stdout.contains("Managed block:"));
        assert!(stdout.contains("Next:"));
        assert!(stdout.contains("ota agents --write"));
        assert!(stdout.contains("ota doctor"));
        assert!(stdout.contains("No explicit `agent` block is declared in `ota.yaml` yet."));
        assert!(stdout.contains("- `ota tasks`"));
        assert!(stdout.contains("- `ota doctor`"));
        assert!(stdout.contains("- `ota detect --dry-run`"));
    }

    #[test]
    fn agents_preview_external_contract_uses_explicit_paths() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "agents", fixture.file_path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        let contract_path = fs::canonicalize(fixture.file_path())
            .unwrap()
            .display()
            .to_string();
        let agents_path = fs::canonicalize(fixture.dir.path())
            .unwrap()
            .join("AGENTS.md")
            .display()
            .to_string();
        assert!(stdout.contains(&format!("AGENTS {contract_path}")));
        assert!(stdout.contains(&agents_path));
        assert!(stdout.contains(&format!("ota agents --write {contract_path}")));
        assert!(stdout.contains(&format!("ota doctor {contract_path}")));
    }

    #[test]
    fn agents_json_reports_output_and_scaffold_content() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "agents", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["written"], false);
        assert_eq!(json["mode"], "preview");
        assert!(json["output"].is_string());
        assert!(
            json["content"]
                .as_str()
                .unwrap()
                .contains("No explicit `agent` block is declared in `ota.yaml` yet.")
        );
    }

    #[test]
    fn agents_write_creates_agents_md_from_agent_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
agent:
  entrypoint: setup
  default_task: ci
  safe_tasks:
    - setup
    - build
  verify_after_changes:
    - fmt
    - check
  writable_paths:
    - src
    - docs
  protected_paths:
    - Cargo.lock
  bootstrap:
    ota:
      note: Only install ota if it is missing and installation is approved.
      sh: curl -fsSL https://dist.ota.run/install.sh | sh
      powershell: irm https://dist.ota.run/install.ps1 | iex
  notes: |
    Use ota doctor first.
"#,
        );

        let output = run_with(["ota", "agents", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("AGENTS"));
        assert!(stdout.contains("Managed block:"));
        assert!(stdout.contains("already in sync") || stdout.contains("wrote"));
        assert!(stdout.contains("Next:"));
        assert!(stdout.contains("ota doctor"));
        let agents_md = fs::read_to_string(fixture.dir.path().join("AGENTS.md")).unwrap();
        assert!(agents_md.contains("# AGENTS.md"));
        assert!(agents_md.contains("Generated from `./ota.yaml`."));
        assert!(agents_md.contains("`entrypoint`: `setup` (`ota run setup`)"));
        assert!(
            agents_md
                .contains("`safe_tasks`: `setup` (`ota run setup`), `build` (`ota run build`)")
        );
        assert!(
            agents_md.contains(
                "`verify_after_changes`: `fmt` (`ota run fmt`), `check` (`ota run check`)"
            )
        );
        assert!(agents_md.contains("## Bootstrap"));
        assert!(
            agents_md.contains("Only install ota if it is missing and installation is approved.")
        );
        assert!(agents_md.contains("- `sh`: `curl -fsSL https://dist.ota.run/install.sh | sh`"));
        assert!(agents_md.contains("- `powershell`: `irm https://dist.ota.run/install.ps1 | iex`"));
        assert!(agents_md.contains("Use ota doctor first."));

        let json_output = run_with(["ota", "agents", "--write", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&json_output.stdout).unwrap();
        assert_eq!(json["mode"], "already_in_sync");
        assert_eq!(json["written"], false);
    }

    #[test]
    fn agents_json_reports_wrote_mode_when_creating_file() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
agent:
  entrypoint: setup
"#,
        );

        let output = run_with(["ota", "agents", "--write", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["mode"], "wrote");
        assert_eq!(json["written"], true);
    }

    #[test]
    fn agents_write_preserves_existing_content_and_appends_generated_block() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
agent:
  entrypoint: setup
"#,
        );
        let agents_path = fixture.dir.path().join("AGENTS.md");
        fs::write(&agents_path, "Custom guidance\n").unwrap();

        let output = run_with(["ota", "agents", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("AGENTS"));
        let agents_md = fs::read_to_string(&agents_path).unwrap();
        assert!(agents_md.starts_with("Custom guidance"));
        assert!(agents_md.contains("ota-generated-agent-guidance:start"));
        assert!(agents_md.contains("# AGENTS.md"));
        assert!(agents_md.contains("Generated from `./ota.yaml`."));
        assert!(agents_md.contains("`entrypoint`: `setup` (`ota run setup`)"));
    }

    #[test]
    fn agents_write_skips_duplicate_generated_content() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
agent:
  entrypoint: setup
  default_task: ci
"#,
        );

        let preview = run_with(["ota", "agents", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&preview.stdout).unwrap();
        let generated = json["content"].as_str().unwrap();
        let agents_path = fixture.dir.path().join("AGENTS.md");
        let original = format!("Custom guidance\n\n{generated}");
        fs::write(&agents_path, &original).unwrap();

        let output = run_with(["ota", "agents", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("already in sync"));
        assert_eq!(fs::read_to_string(&agents_path).unwrap(), original);

        let json_output = run_with(["ota", "agents", "--write", "--json", fixture.path()]);
        let json: Value = serde_json::from_str(&json_output.stdout).unwrap();
        assert_eq!(json["mode"], "already_in_sync");
        assert_eq!(json["written"], false);
    }

    #[test]
    fn init_writes_by_default_creates_full_starter_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("INIT WRITE"));
        assert!(stdout.contains(
            "Write policy: detected mode writes high- and medium-confidence fields; low-confidence fields remain excluded"
        ));
        assert!(stdout.contains("Next:"));
        assert!(stdout.contains("ota validate"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: ota-web"));
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(written.contains("notes: |"));
        assert!(written.contains("Run `ota run dev` to execute this task."));
    }

    #[test]
    fn init_writes_agent_block_for_setup_and_test_repos() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": {
    "setup": "pnpm install",
    "test": "pnpm test"
  }
}"#,
        );

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("agent:"));
        assert!(written.contains("entrypoint: setup"));
        assert!(written.contains("default_task: test"));
        assert!(written.contains("safe_tasks:"));
        assert!(written.contains("verify_after_changes:"));
        assert!(written.contains("- test"));
        assert!(written.contains("protected_paths:"));
        assert!(written.contains("- ota.yaml"));
        assert!(written.contains("notes: |"));
        assert!(written.contains("Use `ota run test` to verify changes."));
        assert!(!written.contains("entrypoint: null"));
        let agent_index = written.find("agent:").unwrap();
        let tasks_index = written.find("tasks:").unwrap();
        let tools_index = written.find("tools:").unwrap();
        assert!(agent_index > tasks_index);
        assert!(agent_index > tools_index);
    }

    #[test]
    fn init_writes_medium_confidence_starter_when_it_is_valid() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("INIT WRITE"));
        assert!(stdout.contains(
            "Write policy: detected mode writes high- and medium-confidence fields; low-confidence fields remain excluded"
        ));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: go-service"));
        assert!(written.contains("go: 1.24.0"));
    }

    #[test]
    fn init_reports_excluded_write_inferences_with_clear_spacing() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "Makefile",
            r#"build:
	@printf "build\n"
"#,
        );

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Next:"));
        assert!(stdout.contains("Excluded from automatic write:"));
        assert!(stdout.contains("Field: project.name"));
        assert!(stdout.contains("Confidence: low"));
        assert!(stdout.contains("Field: tasks.build.run"));
        assert!(!stdout.contains("▸  Excluded from automatic write:"));
        assert!(!stdout.contains("▸  ✦ Field: project.name"));
    }

    #[test]
    fn init_bootstrap_writes_full_detected_starter_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write("tclapp.tcl", "puts \"hello\"\n");

        let output = run_with(["ota", "init", "--bootstrap", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("INIT WRITE"));
        assert!(
            stdout.contains(
                "Bootstrap policy: detected mode writes the full detected starter contract"
            )
        );
        assert!(!stdout.contains("Excluded from automatic write:"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: tclapp"));
        assert!(written.contains("tclsh: '*"));
        assert!(written.contains("run: tclsh tclapp.tcl"));
        assert!(written.contains("notes: |"));
        assert!(written.contains("Run `ota run run` to execute this task."));
    }

    #[test]
    fn init_bootstrap_falls_back_to_directory_name_for_project() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "node/Makefile",
            r#"build:
	@printf "build\n"
"#,
        );

        let node_path = format!("{}/node", fixture.path());
        let output = run_with(["ota", "init", "--bootstrap", node_path.as_str()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("INIT WRITE"));
        assert!(
            stdout.contains(
                "Bootstrap policy: detected mode writes the full detected starter contract"
            )
        );
        let written = fs::read_to_string(fixture.dir.path().join("node").join("ota.yaml")).unwrap();
        assert!(written.contains("name: node"));
        let validate = run_with(["ota", "validate", node_path.as_str()]);
        assert_eq!(validate.exit_code, 0);
    }

    #[test]
    fn init_bootstrap_falls_back_to_current_directory_name_for_project() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        let node_dir = fixture.path().join("node");
        fs::create_dir_all(&node_dir).unwrap();
        fs::write(
            node_dir.join("Makefile"),
            r#"build:
	@printf "build\n"
"#,
        )
        .unwrap();

        let _guard = CWD_MUTEX.lock().unwrap();
        let _cwd = CurrentDirGuard::enter(&node_dir);
        let output = run_with(["ota", "init", "--bootstrap"]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("INIT WRITE ."));
        let written = fs::read_to_string(node_dir.join("ota.yaml")).unwrap();
        assert!(written.contains("name: node"));
    }

    #[test]
    fn init_json_reports_blank_mode() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "init", "--json", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["written"], false);
        assert_eq!(json["mode"], "blank");
    }

    #[test]
    fn init_blank_mode_text_calls_out_minimal_coverage() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "init", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Mode: blank"));
        assert!(
            stdout.contains(
                "Coverage: blank mode is a minimal starter; add runtimes, tools, env, tasks, and checks before relying on it"
            )
        );
    }

    #[test]
    fn init_refuses_to_overwrite_existing_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tasks:
  dev:
    run: npm run dev
"#,
        );

        let output = run_with(["ota", "init", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("use `ota detect --merge` to update the existing contract"));
        assert!(
            stderr.contains("review the existing contract with `ota validate` or `ota doctor`")
        );
        assert!(stderr.contains("update the existing contract with `ota detect --merge"));
    }

    #[test]
    fn init_json_refuses_to_overwrite_existing_contract_with_next() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "init", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        assert_eq!(
            json["next"],
            format!(
                "ota detect --merge {}",
                compact_path(fixture.dir.path(), ".")
            )
        );
    }

    #[test]
    fn run_executes_script_task_inputs() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    inputs:
      base_url:
        required: true
    script: |
      printf '%s' "$OTA_INPUT_BASE_URL" > prepared.txt
"#,
        );

        let output = run_with([
            "ota",
            "run",
            "setup",
            fixture.path(),
            "--base-url",
            "http://localhost:8080",
        ]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "http://localhost:8080"
        );

        let _guard = CWD_MUTEX.lock().unwrap();
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());
        let output = run_with(["ota", "run", "setup", "--base-url", "http://localhost:8080"]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("prepared.txt")).unwrap(),
            "http://localhost:8080"
        );
    }

    #[test]
    fn run_executes_task_inputs_without_explicit_path() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  bump-version:
    inputs:
      version:
        required: true
    script: |
      printf '%s' "$OTA_INPUT_VERSION" > version.txt
"#,
        );

        let _guard = CWD_MUTEX.lock().unwrap();
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());
        let output = run_with(["ota", "run", "bump-version", "--version", "0.1.3"]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(fixture.dir.path().join("version.txt")).unwrap(),
            "0.1.3"
        );
    }

    #[test]
    fn run_reports_ephemeral_lifecycle_as_advisory_note() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  setup:
    run: exit 0
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let rendered = strip_ansi(&format!(
            "{}\n{}",
            output.stdout,
            output.stderr.as_deref().unwrap_or_default()
        ));
        assert!(rendered.contains("SUMMARY"));
        assert!(rendered.contains("Mode:       native"));
        assert!(rendered.contains("Task:       setup"));
        assert!(rendered.contains("Note:       running on the host environment"));
        assert!(rendered.contains("Next:"));
        assert!(rendered.contains("ota tasks --use"));
    }

    #[cfg(unix)]
    #[test]
    fn run_reports_container_execution_banner_with_container_name() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: exit 0
"#,
        );
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_path = bin_dir.join("docker");
        install_fake_docker(&docker_path);
        let mut permissions = fs::metadata(&docker_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).unwrap();

        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(existing));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        match original_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }

        assert_eq!(output.exit_code, 0);
        let rendered = strip_ansi(&format!(
            "{}\n{}",
            output.stdout,
            output.stderr.as_deref().unwrap_or_default()
        ));
        assert!(rendered.contains("SUMMARY"));
        assert!(rendered.contains("Mode:       container"));
        assert!(rendered.contains("Task:       setup"));
        assert!(rendered.contains("Target:"));
        assert!(rendered.contains("ota-"));
        assert!(rendered.contains("Lifecycle:  persistent"));
        assert!(rendered.contains("Note:       reusing persistent container backend"));
        assert!(rendered.contains("Next:"));
        assert!(rendered.contains("ota tasks --use"));
    }

    #[test]
    fn run_backend_override_native_bypasses_container_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/test:latest
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "run", "setup", "--mode", "native", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn run_preserves_child_exit_code() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: exit 17
"#,
        );

        let output = run_with(["ota", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 17);
    }

    #[test]
    fn doctor_text_reports_ready_when_no_findings_exist() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
    }

    #[test]
    fn doctor_reports_not_ready_when_contract_has_no_tasks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("No tasks defined in contract"));
    }

    #[test]
    #[cfg(unix)]
    fn doctor_without_contract_inspects_repo_and_host_signals() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("Cargo.toml"),
            "[package]\nname = \"ota-rust\"\n",
        )
        .unwrap();
        fs::write(
            fixture.path().join("compose.yaml"),
            "services:\n  web:\n    image: nginx:latest\n",
        )
        .unwrap();

        let bin_dir = TempDir::new().unwrap();
        #[cfg(unix)]
        {
            install_fake_cargo(&bin_dir.path().join("cargo"));
        }
        let original_path = std::env::var_os("PATH");
        let _cwd = CurrentDirGuard::enter(fixture.path());
        unsafe {
            std::env::set_var("PATH", bin_dir.path());
        }

        let output = run_with(["ota", "doctor"]);

        unsafe {
            match original_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(!stdout.contains("Primary blocker:"));
        assert!(stdout.contains("No `ota.yaml` found"));
        assert!(stdout.contains("Detected Rust repo"));
        assert!(stdout.contains("Detected Docker Compose services: web"));
        assert!(stdout.contains("Host tool available: cargo"));
        assert!(stdout.contains("Missing container execution backend CLI: docker, podman"));
        assert!(stdout.contains("ota detect --dry-run"));
        assert!(stdout.contains("ota init --bootstrap"));

        let json_output = run_with(["ota", "doctor", "--json", fixture.path().to_str().unwrap()]);
        let json: Value = serde_json::from_str(&json_output.stdout).unwrap();
        assert_eq!(
            json["summary"]["primary_blocker"]["summary"],
            "No `ota.yaml` found"
        );
        assert_eq!(json["summary"]["primary_blocker"]["severity"], "error");
    }

    #[test]
    fn doctor_text_format_uses_spaced_sections_without_rule_separator() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("DOCTOR "));
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Primary Blocker"));
        assert!(stdout.contains("No tasks defined in contract"));
        assert!(!stdout.contains("\n---\n"));
    }

    #[test]
    fn doctor_missing_explicit_path_includes_contract_creation_next_steps() {
        let fixture = TempDir::new().unwrap();
        let missing = fixture.path().join("ota.yaml");

        let output = run_with(["ota", "doctor", missing.to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("contract path does not exist"));
        assert!(stderr.contains("run `ota init` to create a starter contract"));
        assert!(stderr.contains("run `ota detect --dry-run` to preview inferred fields"));
        assert!(stderr.contains("run `ota detect --write` to write a detected contract"));
    }

    #[test]
    fn check_runs_only_configured_checks() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_CHECK_REQUIRED:
    required: true
checks:
  - name: health-check
    kind: health
    severity: warn
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "check", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("CHECK"));
        assert!(stdout.contains("WARN  Check failed: health-check"));
        assert!(!stdout.contains("Missing environment variable"));
    }

    #[test]
    fn check_ready_text_surfaces_next_actions_and_compact_execution_summary() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  supported:
    - native
    - container
  lifecycle: ephemeral
  backends:
    container:
      image: rust:1.94-bookworm
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "check", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("CHECK"));
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("Next:"));
        assert!(stdout.contains("ota up"));
        assert!(stdout.contains("ota tasks --use"));
        assert!(stdout.contains("Preferred: `container`"));
        assert!(stdout.contains("Supported: `native`, `container`"));
        assert!(stdout.contains("Lifecycle: `ephemeral`"));
        assert!(stdout.contains("Container: `rust:1.94-bookworm`"));
    }

    #[test]
    fn check_json_reports_findings() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "check", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["findings"][0]["summary"], "Check failed: health-check");
    }

    #[test]
    fn check_json_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
checks:
  - name: root-health
    kind: health
    severity: warn
    run: exit 1
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
checks:
  - name: api-health
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "check", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["error_count"], 1);
        assert_eq!(json["summary"]["warn_count"], 1);
        assert_eq!(json["summary"]["info_count"], 0);
        assert_eq!(json["summary"]["verdict"], "not_ready");
        assert_eq!(json["summary"]["agent_verdict"], "not_ready");
        assert_eq!(json["findings"][0]["summary"], "Check failed: root-health");
        let members = json["members"].as_array().unwrap();
        assert_eq!(members[0]["member"], "api");
        assert_eq!(members[0]["ok"], false);
        assert_eq!(
            members[0]["findings"][0]["summary"],
            "Check failed: api-health"
        );
    }

    #[test]
    fn check_text_reports_root_monorepo_summary_with_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
checks:
  - name: api-health
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "check", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains(&format!("CHECK {}", compact_contract(&fixture.file_path()))));
        assert!(stdout.contains(&format!(
            "CHECK {} [member api]",
            compact_contract(&fixture.file_path())
        )));
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Check failed: api-health"));
        assert!(stdout.contains("Overview"));
        assert!(stdout.contains("Errors:"));
        assert!(stdout.contains("Warnings:"));
        assert!(stdout.contains("Info:"));
        assert!(stdout.rfind("Overview") > stdout.rfind("Check failed: api-health"));
        assert!(stdout.contains("\n\n"));
    }

    #[test]
    fn check_rejects_duplicate_monorepo_members() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
"#,
        );

        let output = run_with([
            "ota",
            "check",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("`--member api` was provided more than once"));
    }

    #[test]
    fn doctor_warning_only_reports_ready_with_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("Primary Finding"));
        assert!(stdout.contains("Missing tool: ota-tool-that-does-not-exist"));
    }

    #[test]
    fn doctor_warning_only_json_reports_ok_true() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["findings"][0]["severity"], "warn");
    }

    #[test]
    fn doctor_reports_ephemeral_lifecycle_warning() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("Primary Finding"));
        assert!(stdout.contains("Ephemeral lifecycle is advisory only in V1"));
    }

    #[test]
    fn doctor_text_orders_error_warn_and_info_findings() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_ORDER_REQUIRED:
    required: true
tools:
  cargo:
    version: "999.0.0"
    required: false
checks:
  - name: informational-check
    kind: health
    severity: info
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        let error_index = stdout.find("Primary Blocker").unwrap();
        let warn_index = stdout
            .find("WARN  Version mismatch for tool: cargo")
            .unwrap();
        let info_index = stdout
            .find("INFO  Check failed: informational-check")
            .unwrap();

        assert!(error_index < warn_index);
        assert!(warn_index < info_index);
    }

    #[test]
    fn up_runs_setup_and_reports_ready() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("Phase: post-setup diagnosis"));
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_reports_setup_failure_with_exit_code() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let bin_dir = TempDir::new().unwrap();
        let contract = format!(
            r#"
version: 1
project:
  name: ota
checks:
  - name: provisioned-tool
    kind: precondition
    severity: error
    run: provisioned-tool --version
tasks:
  setup:
    run: exit 7
"#
        );
        let fixture = ContractFixture::new(&contract);
        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.path().to_path_buf()];
        if let Some(path) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(path));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        unsafe {
            match original_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }

        assert_eq!(output.exit_code, 7);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("SETUP FAILED"));
        assert!(stdout.contains("Phase: setup"));
        assert!(stdout.contains("Task: setup"));
        assert!(stdout.contains("Exit code: 7"));
    }

    #[test]
    fn up_captures_setup_failure_output_in_compact_report() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
checks:
  - name: provisioned-tool
    kind: precondition
    severity: error
    run: provisioned-tool --version
tasks:
  setup:
    run: printf setup-stdout && printf setup-stderr >&2 && exit 7
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 7);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("SETUP FAILED"));
        assert!(stdout.contains("Task output:"));
        assert!(stdout.contains("setup-stdout"));
        assert!(stdout.contains("setup-stderr"));
    }

    #[test]
    #[cfg(unix)]
    fn up_runs_setup_before_preconditions_fail() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let bin_dir = TempDir::new().unwrap();
        let contract = format!(
            r#"
version: 1
project:
  name: ota
checks:
  - name: provisioned-tool
    kind: precondition
    severity: error
    run: provisioned-tool --version
tasks:
  setup:
    run: |
      mkdir -p "{bin_dir}"
      printf '%s\n' \
        '#!/bin/sh' \
        'if [ "$1" = "--version" ]; then' \
        '  echo "provisioned-tool 1.0.0"' \
        '  exit 0' \
        'fi' \
        'exit 0' > "{bin_dir}/provisioned-tool"
      chmod +x "{bin_dir}/provisioned-tool"
      printf ready > prepared.txt
"#,
            bin_dir = bin_dir.path().display()
        );
        let fixture = ContractFixture::new(&contract);
        let original_path = std::env::var_os("PATH");
        let mut path_entries = vec![bin_dir.path().to_path_buf()];
        if let Some(path) = original_path.as_ref() {
            path_entries.extend(std::env::split_paths(path));
        }
        let joined_path = std::env::join_paths(path_entries).unwrap();
        unsafe {
            std::env::set_var("PATH", &joined_path);
        }

        let output = run_with(["ota", "up", fixture.path()]);

        unsafe {
            match original_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("Phase: post-setup diagnosis"));
        assert!(fixture.dir.path().join("prepared.txt").exists());
        assert!(bin_dir.path().join("provisioned-tool").exists());
    }

    #[test]
    #[cfg(unix)]
    fn up_reports_not_ready_when_setup_does_not_fix_prerequisites() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
checks:
  - name: provisioned-tool
    kind: precondition
    severity: error
    run: provisioned-tool --version
tasks:
  setup:
    run: printf setup > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Phase: provisioning"));
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_runs_required_service_start_before_setup() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: printf service > service.txt
    healthcheck: test -f service.txt
tasks:
  setup:
    run: test -f service.txt && printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        assert!(fixture.dir.path().join("service.txt").exists());
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_starts_services_in_dependency_order() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: test -f db-ready.txt && printf api >> order.txt && printf ready > api-ready.txt
    healthcheck: test -f api-ready.txt
    depends_on:
      - postgres
  postgres:
    required: true
    start: printf db >> order.txt && printf ready > db-ready.txt
    healthcheck: test -f db-ready.txt
tasks:
  setup:
    run: test "$(cat order.txt)" = "dbapi" && printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            std::fs::read_to_string(fixture.dir.path().join("order.txt")).unwrap(),
            "dbapi"
        );
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_stops_in_services_phase_when_required_service_healthcheck_fails() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: printf service > service.txt
    healthcheck: test -f service-ready.txt
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Phase: services"));
        assert!(stdout.contains("ERROR  Service healthcheck failed: postgres"));
        assert!(fixture.dir.path().join("service.txt").exists());
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_stops_before_starting_dependents_when_dependency_is_not_ready() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  api:
    required: true
    start: printf api > api-started.txt
    healthcheck: test -f api-ready.txt
    depends_on:
      - postgres
  postgres:
    required: true
    start: printf db > db-started.txt
    healthcheck: test -f db-ready.txt
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Phase: services"));
        assert!(stdout.contains("ERROR  Service healthcheck failed: postgres"));
        assert!(fixture.dir.path().join("db-started.txt").exists());
        assert!(!fixture.dir.path().join("api-started.txt").exists());
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_json_reports_ready_status_and_phase() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["status"], "READY");
        assert_eq!(json["phase"], "post-setup diagnosis");
        assert!(json["findings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn up_dry_run_text_reports_plan_without_mutating_repo() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  lifecycle: ephemeral
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "up", "--dry-run", "."]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("UP PREVIEW"));
        assert!(stdout.contains("Mode: dry-run (no write)"));
        assert!(stdout.contains("Plan"));
        assert!(stdout.contains("run task `setup`"));
        assert!(stdout.contains("Dry run only"));
        assert!(!stdout.contains("Blocked by"));
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_dry_run_json_reports_execution_plan_without_mutation() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  lifecycle: ephemeral
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", "--json", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["dry_run"], true);
        assert_eq!(json["status"], "READY");
        assert_eq!(json["phase"], "preview");
        assert_eq!(json["execution"]["backend"], "native");
        assert_eq!(json["execution"]["lifecycle"], "ephemeral");
        assert!(json["execution"].get("image").is_none());
        assert_eq!(json["execution"]["task"], "setup");
        assert!(
            json["plan"]["actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == "run task `setup`")
        );
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_dry_run_container_preview_reports_image_and_preview_rerun() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: rust:1.94-bookworm
      engines: [docker]
tasks:
  setup:
    run: printf ready > prepared.txt
tools:
  cargo: "*"
"#,
        );
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" exit /b 1\r\necho unsupported\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  exit 1\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "up", "--dry-run", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("➤ Primary Blocker"));
        assert!(!stdout.contains("Blocked by"));
        assert!(stdout.contains("Image: `rust:1.94-bookworm`"));
        assert!(stdout.contains(
            "Why: cargo is declared in the contract but is not available inside the configured"
        ));
        assert!(!stdout.contains("container image"));
        assert!(stdout.contains("rerun `ota up --dry-run --mode container`"));
        assert!(!stdout.contains("rerun `ota doctor --mode container`"));
    }

    #[test]
    fn up_dry_run_container_json_includes_execution_image() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: rust:1.94-bookworm
      engines: [docker]
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );
        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo ok\r\n  exit /b 0\r\n)\r\necho unsupported\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  echo ok\n  exit 0\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);

        let output = run_with([
            "ota",
            "up",
            "--json",
            "--dry-run",
            "--mode",
            "container",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["execution"]["backend"], "container");
        assert_eq!(json["execution"]["image"], "rust:1.94-bookworm");
    }

    #[test]
    fn up_dry_run_service_preview_distinguishes_start_from_readiness_checks() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    healthcheck: exit 0
  redis:
    required: true
    start: docker compose up -d redis
    healthcheck: exit 0
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "up", "--dry-run", "."]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("verify service `postgres` readiness"));
        assert!(!stdout.contains("start service `postgres`"));
        assert!(stdout.contains("start service `redis`"));
        assert!(stdout.contains("verify service `redis` readiness"));
    }

    #[test]
    fn up_rejects_stream_with_json() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "up", "--json", "--stream", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap()),
            "`--stream` is only supported for text output"
        );
    }

    #[test]
    fn up_rejects_stream_with_dry_run() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "up", "--dry-run", "--stream", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap()),
            "`--stream` is only supported for mutating `ota up`"
        );
    }

    #[test]
    fn up_reports_service_start_failure_with_exit_code() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: exit 9
    healthcheck: exit 0
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 9);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("SERVICE START FAILED"));
        assert!(stdout.contains("Phase: services"));
        assert!(stdout.contains("Service: postgres"));
        assert!(stdout.contains("Exit code: 9"));
        assert!(!fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn up_json_reports_service_start_failure_details() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: exit 9
    healthcheck: exit 0
"#,
        );

        let output = run_with(["ota", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 9);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["status"], "SERVICE START FAILED");
        assert_eq!(json["phase"], "services");
        assert_eq!(json["service"], "postgres");
        assert_eq!(json["exit_code"], 9);
        assert!(json["findings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn up_reports_post_setup_diagnosis_findings() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "up", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Phase: post-setup diagnosis"));
        assert!(stdout.contains("ERROR  Check failed: health-check"));
        assert!(fixture.dir.path().join("prepared.txt").exists());
    }

    #[test]
    fn detect_dry_run_renders_yaml_and_annotations() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let output = run_with(["ota", "detect", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("project:"));
        assert!(stdout.contains("name: ota-web"));
        assert!(stdout.contains("runtimes.node"));
        assert!(stdout.contains("tasks.dev.run"));
    }

    #[test]
    fn detect_dry_run_text_format_uses_spaced_sections_without_rule_separator() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota"
version = "0.1.0"
edition = "2024"
"#,
        );

        let output = run_with(["ota", "detect", "--dry-run", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("DETECT PREVIEW "));
        assert!(stdout.contains("dry-run (no write)"));
        assert!(stdout.contains("Contract:\nversion: 1"));
        assert!(stdout.contains("Annotations:"));
        assert!(stdout.contains("Field: "));
        assert!(stdout.contains("Value: "));
        assert!(stdout.contains("Source: "));
        assert!(stdout.contains("Confidence: "));
        assert!(!stdout.contains("\n---\n"));
    }

    #[test]
    fn detect_json_reports_candidate_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--json", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["written"], false);
        assert_eq!(json["config"]["project"]["name"], "ota-web");
        assert_eq!(json["inferred"][0]["field"], "project.name");
    }

    #[test]
    fn detect_dry_run_reports_existing_contract_comparison_in_text() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  pnpm: "9"
tasks:
  dev:
    run: npm run dev
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Existing contract comparison:"));
        assert!(stdout.contains("project.name: would update `existing` -> `ota-web`"));
        assert!(stdout.contains("tools.pnpm: would update `9` -> `10.1.0`"));
        assert!(stdout.contains("tasks.dev.run: would update `npm run dev` -> `pnpm dev`"));
        assert!(stdout.contains("ota detect --merge --dry-run"));
        assert!(!stdout.contains("ota detect --write"));
        let comparison = stdout
            .find("Existing contract comparison:")
            .expect("comparison section");
        let contract = stdout.find("Contract:").expect("contract section");
        assert!(comparison < contract);
    }

    #[test]
    fn detect_json_reports_existing_contract_comparison() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--json", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert_eq!(json["comparison"]["changes"][0]["field"], "project.name");
    }

    #[test]
    fn detect_json_reports_existing_contract_drift() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  cargo: "1.78"
tasks:
  build:
    run: cargo build
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with([
            "ota",
            "detect",
            "--json",
            "--merge",
            "--dry-run",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert_eq!(json["comparison"]["removals"][0]["field"], "tools.cargo");
        assert_eq!(
            json["comparison"]["removals"][1]["field"],
            "tasks.build.run"
        );
        assert_eq!(json["comparison"]["removals"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn detect_dry_run_existing_contract_with_drift_points_to_merge_and_rewrite_review() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  cargo: "1.78"
tasks:
  build:
    run: cargo build
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Existing contract drift:"));
        assert!(stdout.contains("ota detect --merge --dry-run"));
        assert!(stdout.contains("ota detect --rewrite --dry-run"));
        assert!(!stdout.contains("ota detect --write"));
        let drift = stdout
            .find("Existing contract drift:")
            .expect("drift section");
        let contract = stdout.find("Contract:").expect("contract section");
        assert!(drift < contract);
    }

    #[test]
    fn detect_merge_requires_existing_contract() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "detect", "--merge", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("`ota detect --merge` requires an existing `ota.yaml`"));
        assert!(stderr.contains("use `ota detect --write` to write a first contract"));
        assert!(stderr.contains("use `ota detect --dry-run` to review one"));
    }

    #[test]
    fn detect_merge_json_requires_existing_contract_with_next() {
        let fixture = ContractFixture::new_dir();

        let output = run_with(["ota", "detect", "--merge", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        let expected_path = std::path::Path::new(fixture.path()).canonicalize().unwrap();
        assert_eq!(
            json["next"],
            format!("ota detect --write {}", expected_path.display())
        );
    }

    #[test]
    fn doctor_explicit_directory_does_not_walk_up_to_parent_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready
"#,
        );
        let nested = fixture.dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();

        let output = run_with(["ota", "doctor", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("No `ota.yaml` found"));
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(!stderr.contains("explicit repo path does not contain `ota.yaml`"));
    }

    #[test]
    fn diff_explicit_directory_does_not_walk_up_to_parent_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );
        let nested = fixture.dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();

        let output = run_with([
            "ota",
            "diff",
            fixture.file_path().to_str().unwrap(),
            nested.to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("explicit repo path does not contain `ota.yaml`"));
    }

    #[test]
    fn detect_merge_dry_run_requires_existing_contract() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0"
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("`ota detect --merge --dry-run` requires an existing `ota.yaml`"));
        assert!(stderr.contains("use `ota detect --dry-run` to review a first contract"));
    }

    #[test]
    fn detect_merge_writes_high_confidence_missing_fields_only() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let output = run_with(["ota", "detect", "--merge", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("MERGED"));
        assert!(stdout.contains("Applied high-confidence additions:"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(written.contains("name: existing"));
        assert!(!written.contains("name: ota-web"));
    }

    #[test]
    fn detect_merge_dry_run_marks_stale_drift_as_review_only() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  cargo: "1.78"
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0"
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--dry-run", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("ota detect --merge"));
        assert!(stdout.contains("stale entries"));
        assert!(stdout.contains("ota detect --rewrite --dry-run"));
        assert!(!stdout.contains("Applying detect merge would remove"));
    }

    #[test]
    fn detect_merge_apply_writes_only_selected_fields() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let output = run_with([
            "ota",
            "detect",
            "--merge",
            "--apply",
            "tools.pnpm",
            "--apply",
            "tasks.dev.run",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("MERGED"));
        assert!(stdout.contains("Applied selected high-confidence changes:"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(!written.contains("node: 22"));
        assert!(!written.contains("name: ota-web"));
    }

    #[test]
    fn detect_merge_apply_dot_writes_all_eligible_high_confidence_changes() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--apply-all", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Applied high-confidence additions:"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(!written.contains("name: existing"));
        assert!(written.contains("name: ota-web"));
    }

    #[test]
    fn detect_merge_apply_reports_actionable_next_when_existing_contract_is_invalid() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  cargo:
"#,
        );
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota"
version = "0.1.0"
"#,
        );

        let output = run_with([
            "ota",
            "detect",
            "--merge",
            "--apply",
            "tools.cargo",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where: ota detect --merge --apply"));
        assert!(stderr.contains("failed to parse contract"));
        assert!(stderr.contains("ota validate"));
        assert!(stderr.contains("ota detect --merge --apply <field name>"));
    }

    #[test]
    fn detect_merge_apply_reports_when_selected_field_is_not_in_current_comparison() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tools:
  cargo: '*'
"#,
        );
        fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota-web"
version = "0.1.0"
"#,
        );

        let output = run_with([
            "ota",
            "detect",
            "--merge",
            "--apply",
            "tools.cargo",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where: ota detect --merge --apply"));
        assert!(stderr.contains("selected field(s) not present in current detect comparison"));
        assert!(stderr.contains("tools.cargo"));
        assert!(stderr.contains("ota detect --dry-run ."));
    }

    #[test]
    fn detect_rewrite_requires_yes() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0"
}"#,
        );

        let output = run_with(["ota", "detect", "--rewrite", fixture.path()]);
        assert_eq!(output.exit_code, 1);
        assert!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()).contains("requires `--yes`")
        );
    }

    #[test]
    fn detect_rewrite_writes_and_creates_timestamped_backup() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tasks:
  setup:
    run: echo existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--rewrite", "--yes", fixture.path()]);
        assert_eq!(output.exit_code, 0);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("REWRITTEN"));
        assert!(body.contains("Backup:"));

        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: ota-web"));
        assert!(written.contains("pnpm: 10.1.0"));

        let backups = fs::read_dir(fixture.dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.starts_with("ota.yaml.bak-"))
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn detect_rewrite_refuses_to_overwrite_protected_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
agent:
  protected_paths:
    - ota.yaml
tasks:
  setup:
    run: echo existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--rewrite", "--yes", fixture.path()]);
        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("refusing to write protected path `ota.yaml`"));
        assert!(
            !fixture
                .dir
                .path()
                .read_dir()
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("ota.yaml.bak-"))
        );
    }

    #[test]
    fn detect_merge_refuses_to_overwrite_protected_contract() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
agent:
  protected_paths:
    - ota.yaml
tasks:
  setup:
    run: echo existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", fixture.path()]);
        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("refusing to write protected path `ota.yaml`"));
    }

    #[test]
    fn detect_merge_json_reports_written_when_additions_are_applied() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["written"], true);
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert!(
            json["comparison"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|change| change["field"] == "project.name" && change["status"] == "update")
        );
        assert!(
            !json["comparison"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|change| change["field"] == "tools.pnpm" && change["status"] == "add")
        );
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
    }

    #[test]
    fn detect_merge_json_reports_written_false_when_nothing_is_addable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  pnpm: 10.1.0
tasks:
  dev:
    run: pnpm dev
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["written"], false);
        assert_eq!(json["comparison"]["existing_contract"], true);
        assert!(
            json["comparison"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .all(|change| change["status"] != "add")
        );
    }

    #[test]
    fn detect_writes_high_confidence_contract_with_write_flag() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "engines": { "node": "20" },
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let output = run_with(["ota", "detect", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("DETECT WRITE"));
        assert!(stdout.contains("Excluded from automatic write:"));
        assert!(stdout.contains("runtimes.node"));
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        assert!(written.contains("name: ota-web"));
        assert!(written.contains("pnpm: 10.1.0"));
        assert!(written.contains("run: pnpm dev"));
        assert!(!written.contains("node:"));
    }

    #[test]
    fn detect_write_marks_verifier_tasks_safe_for_agent() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "npm@10.9.2",
  "scripts": {
    "build": "next build",
    "test": "vitest run",
    "typecheck": "tsc --noEmit"
  }
}"#,
        );

        let output = run_with(["ota", "detect", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        let yaml: YamlValue = serde_yaml::from_str(&written).unwrap();
        let tasks = yaml
            .get("tasks")
            .and_then(YamlValue::as_mapping)
            .expect("tasks must exist");
        let test_safe = tasks
            .get(&YamlValue::String(String::from("test")))
            .and_then(YamlValue::as_mapping)
            .and_then(|task| task.get(&YamlValue::String(String::from("safe_for_agent"))))
            .and_then(YamlValue::as_bool);
        let typecheck_safe = tasks
            .get(&YamlValue::String(String::from("typecheck")))
            .and_then(YamlValue::as_mapping)
            .and_then(|task| task.get(&YamlValue::String(String::from("safe_for_agent"))))
            .and_then(YamlValue::as_bool);
        let build_safe_present = tasks
            .get(&YamlValue::String(String::from("build")))
            .and_then(YamlValue::as_mapping)
            .and_then(|task| task.get(&YamlValue::String(String::from("safe_for_agent"))))
            .is_some();

        assert_eq!(test_safe, Some(true));
        assert_eq!(typecheck_safe, Some(true));
        assert!(!build_safe_present);
    }

    #[test]
    fn detect_write_existing_contract_renders_next_as_section_not_inline() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web"
}"#,
        );

        let output = run_with(["ota", "detect", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Why:"));
        assert!(stderr.contains("Next:"));
        assert!(stderr.contains("review detected changes"));
        assert!(stderr.contains("ota detect --merge --dry-run"));
        assert!(!stderr.contains("| Next: |"));
        assert!(
            !stderr.contains("\n▸  review detected changes with `ota detect --merge --dry-run")
        );
    }

    #[test]
    fn stylize_failure_splits_ansi_next_block() {
        let message = "`./ota.yaml` already exists; refusing to overwrite an existing contract | \x1b[1;38;2;242;209;170mNext:\x1b[0m | ▸  review detected changes with `ota detect --merge --dry-run .`";
        let styled = commands::stylize_text_failure("ota detect", message);
        let plain = strip_ansi(&styled);
        assert!(plain.contains("Why:"));
        assert!(plain.contains("Next:"));
        assert!(!plain.contains("| Next: |"));
    }

    #[test]
    fn detect_merge_adds_safe_for_agent_for_added_verifier_task() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota-web
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "npm@10.9.2",
  "scripts": { "test": "vitest run" }
}"#,
        );

        let output = run_with(["ota", "detect", "--merge", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let written = fs::read_to_string(fixture.file_path()).unwrap();
        let yaml: YamlValue = serde_yaml::from_str(&written).unwrap();
        let safe = yaml
            .get("tasks")
            .and_then(YamlValue::as_mapping)
            .and_then(|tasks| tasks.get(&YamlValue::String(String::from("test"))))
            .and_then(YamlValue::as_mapping)
            .and_then(|task| task.get(&YamlValue::String(String::from("safe_for_agent"))))
            .and_then(YamlValue::as_bool);
        assert_eq!(safe, Some(true));
    }

    #[test]
    fn detect_defaults_to_preview_when_contract_exists() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "detect", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("DETECT PREVIEW"));
        assert!(stdout.contains("Mode: dry-run (no write)"));
    }

    #[test]
    fn detect_json_defaults_to_preview_when_contract_exists() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
"#,
        );

        let output = run_with(["ota", "detect", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["written"], false);
        assert_eq!(json["comparison"]["existing_contract"], true);
    }

    #[test]
    fn detect_refuses_to_write_when_high_confidence_fields_are_insufficient() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "detect", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains(
            "detected high-confidence fields are not sufficient to produce a valid contract"
        ));
        assert!(
            stderr
                .contains("use `ota detect --dry-run` to review medium and low confidence fields")
        );
        assert!(stderr.contains("Excluded from automatic write:"));
        assert!(stderr.contains("project.name"));
        assert!(!fixture.file_path().exists());
    }

    #[test]
    fn detect_json_reports_next_when_high_confidence_fields_are_insufficient() {
        let fixture = ContractFixture::new_dir();
        fixture.write("go.mod", "module github.com/ota/go-service\n\ngo 1.24.0\n");

        let output = run_with(["ota", "detect", "--json", "--write", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(output.stderr.as_deref().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["written"], false);
        assert_eq!(json["next"], "ota detect --dry-run");
    }

    #[test]
    fn debug_validate_emits_trace_to_stderr() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "--debug", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("VALIDATE"));
        assert!(stdout.contains("VALID"));
        assert!(stdout.contains(&compact_contract(&fixture.file_path())));
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("DEBUG command=validate"));
        assert!(stderr.contains(&format!(
            "DEBUG contract_path={}",
            fixture.file_path().display()
        )));
    }

    #[test]
    fn debug_run_appends_trace_after_lifecycle_note() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  setup:
    run: exit 0
"#,
        );

        let output = run_with(["ota", "--debug", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let rendered = strip_ansi(&format!(
            "{}\n{}",
            output.stdout,
            output.stderr.as_deref().unwrap_or_default()
        ));
        assert!(rendered.contains("DEBUG command=run"));
        assert!(rendered.contains("DEBUG task=setup"));
        assert!(rendered.contains("Note:"));
        assert!(rendered.contains("running on the host environment"));
        assert!(rendered.contains("execution.lifecycle: ephemeral"));
    }

    #[test]
    fn workspace_validate_json_reports_success() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "validate", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["path"], fixture.workspace_file().display().to_string());
        assert_eq!(json["summary"]["error_count"], 0);
    }

    #[test]
    fn repo_commands_json_success_contract_is_stable() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: echo ready
  test:
    run: echo test
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );

        let validate = run_with(["ota", "validate", "--json", fixture.path()]);
        assert_eq!(validate.exit_code, 0);
        assert_json_top_level_keys(&validate, &["ok", "path", "summary"]);

        let tasks = run_with(["ota", "tasks", "--json", fixture.path()]);
        assert_eq!(tasks.exit_code, 0);
        assert_json_top_level_keys(&tasks, &["ok", "path", "tasks"]);

        let doctor = run_with(["ota", "doctor", "--json", fixture.path()]);
        assert_eq!(doctor.exit_code, 0);
        assert_json_top_level_keys(
            &doctor,
            &[
                "finding_groups",
                "findings",
                "mode",
                "ok",
                "path",
                "summary",
            ],
        );

        let check = run_with(["ota", "check", "--json", fixture.path()]);
        assert_eq!(check.exit_code, 0);
        assert_json_top_level_keys(&check, &["findings", "ok", "path", "summary"]);

        let up = run_with(["ota", "up", "--json", fixture.path()]);
        assert_eq!(up.exit_code, 0);
        assert_json_top_level_keys(
            &up,
            &["findings", "ok", "path", "phase", "receipt", "status"],
        );

        let detect = run_with(["ota", "detect", "--json", "--dry-run", fixture.path()]);
        assert_eq!(detect.exit_code, 0);
        assert_json_top_level_keys(
            &detect,
            &["comparison", "config", "inferred", "ok", "path", "written"],
        );

        let init_fixture = ContractFixture::new_dir();
        let init = run_with(["ota", "init", "--json", init_fixture.path()]);
        assert_eq!(init.exit_code, 0);
        assert_json_top_level_keys(
            &init,
            &["config", "inferred", "mode", "ok", "path", "written"],
        );
    }

    #[test]
    fn repo_commands_json_validation_failure_contract_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    depends_on:
      - setup
"#,
        );

        let validate = run_with(["ota", "validate", "--json", fixture.path()]);
        assert_eq!(validate.exit_code, 1);
        assert_eq!(
            json_top_level_keys_named("validate", &validate),
            vec!["errors", "ok", "path", "summary"],
            "validate failure keys"
        );

        let tasks = run_with(["ota", "tasks", "--json", fixture.path()]);
        assert_eq!(tasks.exit_code, 1);
        assert_eq!(
            json_top_level_keys_named("tasks", &tasks),
            vec!["errors", "ok", "path"],
            "tasks failure keys"
        );

        let doctor = run_with(["ota", "doctor", "--json", fixture.path()]);
        assert_eq!(doctor.exit_code, 1);
        assert_eq!(
            json_top_level_keys_named("doctor", &doctor),
            vec!["errors", "ok", "path"],
            "doctor failure keys"
        );

        let check = run_with(["ota", "check", "--json", fixture.path()]);
        assert_eq!(check.exit_code, 1);
        assert_eq!(
            json_top_level_keys_named("check", &check),
            vec!["errors", "ok", "path"],
            "check failure keys"
        );

        let up = run_with(["ota", "up", "--json", fixture.path()]);
        assert_eq!(up.exit_code, 1);
        assert_eq!(
            json_top_level_keys_named("up", &up),
            vec!["errors", "ok", "path"],
            "up failure keys"
        );
    }

    #[test]
    fn repo_commands_exit_code_contract_is_stable() {
        let usage = run_with(["ota", "validate", "--unknown-flag"]);
        assert_eq!(usage.exit_code, 2);

        let invalid = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: echo dev
    depends_on:
      - setup
"#,
        );
        let validate_invalid = run_with(["ota", "validate", invalid.path()]);
        assert_eq!(validate_invalid.exit_code, 1);

        let warning_only = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
tasks:
  test:
    run: cargo test
"#,
        );
        let doctor_warning_only = run_with(["ota", "doctor", warning_only.path()]);
        assert_eq!(doctor_warning_only.exit_code, 0);

        let no_tasks = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );
        let doctor_not_ready = run_with(["ota", "doctor", no_tasks.path()]);
        assert_eq!(doctor_not_ready.exit_code, 1);

        let run_failure = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: exit 17
"#,
        );
        let run = run_with(["ota", "run", "setup", run_failure.path()]);
        assert_eq!(run.exit_code, 17);

        let up_failure = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: exit 7
"#,
        );
        let up = run_with(["ota", "up", up_failure.path()]);
        assert_eq!(up.exit_code, 7);
    }

    #[test]
    fn repo_commands_text_status_contract_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: printf ready > prepared.txt
"#,
        );

        let validate = run_with(["ota", "validate", fixture.path()]);
        let validate_stdout = strip_ansi(&validate.stdout);
        assert_eq!(validate.exit_code, 0);
        assert!(validate_stdout.contains(&format!(
            "VALIDATE {}",
            compact_contract(&fixture.file_path())
        )));
        assert!(validate_stdout.contains("VALID"));
        assert!(!validate_stdout.contains("\n---\n"));

        let doctor = run_with(["ota", "doctor", fixture.path()]);
        let doctor_stdout = strip_ansi(&doctor.stdout);
        assert_eq!(doctor.exit_code, 0);
        assert!(doctor_stdout.contains(&format!(
            "DOCTOR {}",
            compact_contract(&fixture.file_path())
        )));
        assert!(doctor_stdout.contains("READY"));
        assert!(!doctor_stdout.contains("\n---\n"));

        let up = run_with(["ota", "up", fixture.path()]);
        let up_stdout = strip_ansi(&up.stdout);
        assert_eq!(up.exit_code, 0);
        assert!(up_stdout.contains(&format!("UP {}", compact_contract(&fixture.file_path()))));
        assert!(up_stdout.contains("READY"));
        assert!(up_stdout.contains("Phase: post-setup diagnosis"));
        assert!(!up_stdout.contains("\n---\n"));

        let detect_fixture = ContractFixture::new_dir();
        detect_fixture.write(
            "Cargo.toml",
            r#"[package]
name = "ota"
version = "0.1.0"
edition = "2024"
"#,
        );
        let detect = run_with(["ota", "detect", "--dry-run", detect_fixture.path()]);
        let detect_stdout = strip_ansi(&detect.stdout);
        assert_eq!(detect.exit_code, 0);
        assert!(detect_stdout.contains(&format!(
            "DETECT PREVIEW {}",
            compact_path(detect_fixture.dir.path(), ".")
        )));
        assert!(detect_stdout.contains("Mode: dry-run (no write)"));
        assert!(detect_stdout.contains("Contract:"));
        assert!(detect_stdout.contains("Annotations:"));
        assert!(detect_stdout.contains("Next:"));
        assert!(detect_stdout.find("Annotations:").unwrap() < detect_stdout.find("Next:").unwrap());
        assert!(!detect_stdout.contains("\n---\n"));

        let detect_contract = run_with(["ota", "detect", "--contract", detect_fixture.path()]);
        let detect_contract_stdout = strip_ansi(&detect_contract.stdout);
        assert_eq!(detect_contract.exit_code, 0);
        assert!(detect_contract_stdout.contains("DETECT CONTRACT PREVIEW"));
        assert!(!detect_contract_stdout.contains("Annotations:"));
        assert!(!detect_contract_stdout.contains("Next:"));
    }

    #[test]
    fn root_help_text_snapshot_is_stable() {
        let output = run_with(["ota", "--help"]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot(
            "help_root.txt",
            output
                .stderr
                .as_deref()
                .expect("help text should be present"),
        );
    }

    #[test]
    fn workspace_help_describes_file_flag_for_workspace_contracts() {
        let output = run_with(["ota", "workspace", "--help"]);
        let stderr = output
            .stderr
            .as_deref()
            .expect("workspace help text should be present");

        assert_eq!(output.exit_code, 0);
        assert!(stderr.contains(
            "Use an explicit ota.yaml or ota.workspace.yaml file instead of path discovery"
        ));
    }

    #[test]
    fn agents_text_snapshot_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-agents
  description: Premium AGENTS preview
tasks:
  setup:
    run: echo setup
  ci:
    run: echo ci
agent:
  entrypoint: setup
  default_task: ci
  safe_tasks: [setup, ci]
  verify_after_changes: [ci]
  writable_paths: [src, docs]
  protected_paths: [ota.yaml, Cargo.lock]
  bootstrap:
    ota:
      note: Only install ota if it is missing and installation is approved.
      sh: curl -fsSL https://dist.ota.run/install.sh | sh
      powershell: irm https://dist.ota.run/install.ps1 | iex
  notes: |
    Use ota doctor first.
    Keep output calm and exact.
"#,
        );

        let _cwd = CurrentDirGuard::enter(fixture.dir.path());
        let output = run_with(["ota", "agents", "."]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot("agents_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn doctor_text_snapshot_is_stable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-demo
execution:
  lifecycle: ephemeral
tasks:
  ci:
    run: echo ci
agent:
  entrypoint: ci
  default_task: ci
  safe_tasks: [ci]
  verify_after_changes: [ci]
  writable_paths: [src, docs]
  protected_paths: [ota.yaml, Cargo.lock]
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "premium-demo"
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);

        assert_eq!(output.exit_code, 1);
        assert_text_snapshot("doctor_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn doctor_plain_text_snapshot_is_stable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-demo
execution:
  lifecycle: ephemeral
tasks:
  ci:
    run: echo ci
agent:
  entrypoint: ci
  default_task: ci
  safe_tasks: [ci]
  verify_after_changes: [ci]
  writable_paths: [src, docs]
  protected_paths: [ota.yaml, Cargo.lock]
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "premium-demo"
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "--plain", "doctor", "."]);

        assert_eq!(output.exit_code, 1);
        assert_text_snapshot("doctor_plain_premium.txt", &output.stdout);
    }

    #[test]
    fn doctor_container_mode_reports_container_context() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: persistent
  backends:
    container:
      image: jdxcode/mise:latest
      engines: [notarealengine]
tasks:
  ci:
    run: echo ci
agent:
  entrypoint: ci
  default_task: ci
  safe_tasks: [ci]
  verify_after_changes: [ci]
  writable_paths: [src]
  protected_paths: [ota.yaml]
"#,
        );
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Mode:"));
        assert!(text.contains("Missing container execution backend CLI"));
        assert!(text.contains("Execution"));
    }

    #[test]
    fn doctor_container_mode_probes_the_container_image() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: persistent
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  ci:
    run: echo ci
agent:
  entrypoint: ci
  default_task: ci
  safe_tasks: [ci]
  verify_after_changes: [ci]
  writable_paths: [src]
  protected_paths: [ota.yaml]
runtimes:
  node: "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo v22.0.0\r\n  exit /b 0\r\n)\r\necho unsupported\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) echo 'v22.0.0'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 0);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Mode:"));
        assert!(text.contains("ready"));
        assert!(!text.contains("Version mismatch for runtime: node"));
    }

    #[test]
    fn doctor_container_mode_missing_runtime_mentions_configured_image() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: persistent
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  ci:
    run: echo ci
runtimes:
  java: "21"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" exit /b 1\r\necho unsupported\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  exit 1\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Missing runtime: java"));
        assert!(text.contains("inside the configured"));
        assert!(text.contains("Image:"));
        assert!(text.contains("`premium/test:latest`"));
        assert!(
            text.contains("update `execution.backends.container.image` so `java` is available")
        );
    }

    #[test]
    fn doctor_container_mode_reports_apt_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  curl: "8.13.0"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    curl:
      source: apt
      approved_versions:
        - "8.13.0"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"curl --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"apt-get\" >nul && (\r\n    echo E: Version '8.13.0' for 'curl' was not found 1>&2\r\n    exit /b 100\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"curl --version\"*) exit 1 ;;\n    *\"apt-get\"*) echo \"E: Version '8.13.0' for 'curl' was not found\" >&2; exit 100 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container apt cannot install pinned package version: curl"));
        assert!(text.contains("Image:"));
        assert!(text.contains("`premium/test:latest`"));
        assert!(text.contains("relax the Linux/container version pin for `curl`"));
        assert!(!text.contains("Missing tool: curl"));
    }

    #[test]
    fn doctor_container_mode_reports_brew_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"node --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"brew\" >nul && echo %* | findstr /C:\"node@22\" >nul && (\r\n    echo Error: No available formula with the name \"node@22\" 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) exit 1 ;;\n    *brew*node@22*) echo 'Error: No available formula with the name \"node@22\"' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container brew cannot install pinned version: node"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing runtime: node"));
    }

    #[test]
    fn doctor_container_mode_reports_dnf_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  jq: "1.7.1"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    jq:
      source: dnf
      approved_versions:
        - "1.7.1"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"jq --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"dnf\" >nul && echo %* | findstr /C:\"jq-1.7.1\" >nul && (\r\n    echo No match for argument: jq-1.7.1 1>&2\r\n    echo Error: Unable to find a match: jq-1.7.1 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"jq --version\"*) exit 1 ;;\n    *dnf*jq-1.7.1*) echo 'No match for argument: jq-1.7.1' >&2; echo 'Error: Unable to find a match: jq-1.7.1' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container dnf cannot install pinned version: jq"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing tool: jq"));
    }

    #[test]
    fn doctor_container_mode_reports_pacman_package_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  jq: "1.7.1"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    jq:
      source: pacman
      approved_versions:
        - "1.7.1"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"jq --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"pacman\" >nul && echo %* | findstr /C:\"-Si\" >nul && (\r\n    echo error: target not found: jq 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"jq --version\"*) exit 1 ;;\n    *pacman*'-Si'*jq*) echo 'error: target not found: jq' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container pacman cannot locate required package: jq"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing tool: jq"));
    }

    #[test]
    fn doctor_container_mode_reports_winget_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  Microsoft.VisualStudioCode: "1.88.0"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    Microsoft.VisualStudioCode:
      source: winget
      approved_versions:
        - "1.88.0"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"Microsoft.VisualStudioCode --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"winget\" >nul && echo %* | findstr /C:\"--versions\" >nul && (\r\n    echo Found Microsoft.VisualStudioCode\r\n    echo Version\r\n    echo 1.89.0\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"Microsoft.VisualStudioCode --version\"*) exit 1 ;;\n    *winget*--versions*) printf 'Found Microsoft.VisualStudioCode\\nVersion\\n1.89.0\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains(
            "Container winget cannot install pinned version: Microsoft.VisualStudioCode"
        ));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing tool: Microsoft.VisualStudioCode"));
    }

    #[test]
    fn doctor_container_mode_reports_choco_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  git: "2.47.0"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    git:
      source: choco
      approved_versions:
        - "2.47.0"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"git --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"choco\" >nul && echo %* | findstr /C:\"search\" >nul && (\r\n    echo git^|2.46.0\r\n    echo git^|2.45.0\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"git --version\"*) exit 1 ;;\n    *choco*search*) printf 'git|2.46.0\\ngit|2.45.0\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container choco cannot install pinned version: git"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing tool: git"));
    }

    #[test]
    fn doctor_container_mode_reports_scoop_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  neovim: "0.10.1"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    neovim:
      source: scoop
      approved_versions:
        - "0.10.1"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"neovim --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"scoop\" >nul && echo %* | findstr /C:\"cat\" >nul && (\r\n    echo {\"version\":\"0.10.0\"}\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"neovim --version\"*) exit 1 ;;\n    *scoop*cat*) printf '{\"version\":\"0.10.0\"}\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container scoop cannot install pinned version: neovim"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing tool: neovim"));
    }

    #[test]
    fn doctor_container_mode_reports_mise_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    node:
      source: mise
      approved_versions:
        - "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"node --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"mise\" >nul && echo %* | findstr /C:\"ls-remote\" >nul && (\r\n    echo [\"21.0.0\",\"21.1.0\"]\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) exit 1 ;;\n    *mise*ls-remote*) printf '[\"21.0.0\",\"21.1.0\"]\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container mise cannot install pinned version: node"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing runtime: node"));
    }

    #[test]
    fn doctor_container_mode_reports_asdf_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    node:
      source: asdf
      approved_versions:
        - "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"node --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"asdf\" >nul && echo %* | findstr /C:\"list\" >nul && (\r\n    echo 21.0.0\r\n    echo 21.1.0\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) exit 1 ;;\n    *asdf*list*all*) printf '21.0.0\\n21.1.0\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container asdf cannot install pinned version: node"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing runtime: node"));
    }

    #[test]
    fn doctor_container_mode_reports_sdkman_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  java: "21"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    java:
      source: sdkman
      approved_versions:
        - "21"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"java --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"sdk list java\" >nul && (\r\n    echo Available Java Versions\r\n    echo 17.0.9-tem\r\n    echo 22.0.1-tem\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"java --version\"*) exit 1 ;;\n    *\"sdk list java\"*) printf 'Available Java Versions\\n17.0.9-tem\\n22.0.1-tem\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container sdkman cannot install pinned version: java"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing runtime: java"));
    }

    #[test]
    fn doctor_container_mode_reports_uv_version_unavailable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  python: "3.12"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    python:
      source: uv
      approved_versions:
        - "3.12"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"python --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"uv\" >nul && echo %* | findstr /C:\"python list\" >nul && (\r\n    echo cpython-3.11.9-linux-x86_64-none\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"python --version\"*) exit 1 ;;\n    *uv*python*list*) printf 'cpython-3.11.9-linux-x86_64-none\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Container uv cannot install pinned version: python"));
        assert!(text.contains("ota doctor --mode container"));
        assert!(!text.contains("Missing runtime: python"));
    }

    #[test]
    fn doctor_native_mode_keeps_host_failure_for_apt_backed_policy() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: native-host
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  yq: "4.52.5"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    yq:
      source: apt
      approved_versions:
        - "4.52.5"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\nexit 0\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = bin_dir.as_os_str().to_os_string();
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Missing tool: yq"));
        assert!(!text.contains("Container apt cannot"));
    }

    #[test]
    fn doctor_container_mode_skips_host_bound_readiness_checks() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: remote
  supported: [native, container, remote]
  lifecycle: persistent
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
    remote:
      provider: ssh
      target: badtarget
env:
  OTA_CONTAINER_MODE_REQUIRED:
    required: true
checks:
  - name: failing-check
    kind: health
    severity: error
    run: exit 1
services:
  postgres:
    required: true
    healthcheck: exit 1
tasks:
  ci:
    run: echo ci
runtimes:
  node: "22"
agent:
  entrypoint: ci
  default_task: ci
  safe_tasks: [ci]
  verify_after_changes: [ci]
  writable_paths: [src]
  protected_paths: [ota.yaml]
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo v22.0.0\r\n  exit /b 0\r\n)\r\necho unsupported\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) echo 'v22.0.0'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 0);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("Host-bound readiness checks are not evaluated in container mode"));
        assert!(!text.contains("Version mismatch for runtime: node"));
        assert!(!text.contains("Missing environment variable: OTA_CONTAINER_MODE_REQUIRED"));
        assert!(!text.contains("Check failed: failing-check"));
        assert!(!text.contains("Service healthcheck failed: postgres"));
        assert!(!text.contains("Suspicious remote target for ssh"));
    }

    #[test]
    fn doctor_uses_uv_python_remediation_when_repo_signals_uv() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: uv-demo
runtimes:
  python: "3.12.4"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".python-version", "3.12.4\n");
        fixture.write("uv.lock", "version = 1\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let python_body = if cfg!(windows) {
            "@echo off\r\necho Python 3.13.2\r\n"
        } else {
            "#!/bin/sh\necho 'Python 3.13.2'\n"
        };
        write_fake_command(&bin_dir, "python", python_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `uv python install 3.12.4` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_pyenv_python_remediation_when_repo_signals_python_version() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: pyenv-demo
runtimes:
  python: "3.12.4"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".python-version", "3.12.4\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let python_body = if cfg!(windows) {
            "@echo off\r\necho Python 3.13.2\r\n"
        } else {
            "#!/bin/sh\necho 'Python 3.13.2'\n"
        };
        write_fake_command(&bin_dir, "python", python_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `pyenv install 3.12.4` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_sdkman_java_remediation_when_repo_signals_sdkman() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: java-demo
runtimes:
  java: "21.0.2-tem"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".sdkmanrc", "java=21.0.2-tem\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let java_body = if cfg!(windows) {
            "@echo off\r\necho java version \"25.0.2\"\r\n"
        } else {
            "#!/bin/sh\nprintf 'java version \"25.0.2\"\\n'\n"
        };
        write_fake_command(&bin_dir, "java", java_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `sdk install java 21.0.2-tem` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_sdkman_maven_remediation_when_repo_signals_sdkman() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: maven-demo
tools:
  maven: "3.9.9"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".sdkmanrc", "maven=3.9.9\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let mvn_body = if cfg!(windows) {
            "@echo off\r\necho Apache Maven 3.9.14\r\n"
        } else {
            "#!/bin/sh\necho 'Apache Maven 3.9.14'\n"
        };
        write_fake_command(&bin_dir, "mvn", mvn_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `sdk install maven 3.9.9` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_dotnet_install_script_when_repo_signals_global_json() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: dotnet-demo
runtimes:
  dotnet: "8.0.203"
tools:
  dotnet: "*"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(
            "global.json",
            r#"{
  "sdk": {
    "version": "8.0.203"
  }
}"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let dotnet_body = if cfg!(windows) {
            "@echo off\r\necho 9.0.100\r\n"
        } else {
            "#!/bin/sh\necho '9.0.100'\n"
        };
        write_fake_command(&bin_dir, "dotnet", dotnet_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);
        let expected_command = if cfg!(windows) {
            "powershell -ExecutionPolicy Bypass -Command \"iwr https://dot.net/v1/dotnet-install.ps1 -OutFile dotnet-install.ps1; ./dotnet-install.ps1 -Version 8.0.203\""
        } else {
            "curl -fsSL https://dot.net/v1/dotnet-install.sh -o dotnet-install.sh && bash dotnet-install.sh --version 8.0.203"
        };

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains(expected_command));
        assert!(stdout.contains("rerun `ota doctor`"));
    }

    #[test]
    fn doctor_external_contract_rewrites_next_steps_with_explicit_target() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: explicit-target-demo
runtimes:
  node: "22"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "repo-signal-demo"
}"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);

        let output = run_with(["ota", "doctor", fixture.file_path().to_str().unwrap()]);
        let stdout = strip_ansi(&output.stdout);
        let contract_path = fs::canonicalize(fixture.file_path())
            .unwrap()
            .display()
            .to_string();

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("rerun"));
        assert!(stdout.contains("ota doctor"));
        assert!(stdout.contains(&contract_path));
        assert!(stdout.contains(&format!("ota detect --merge --dry-run {contract_path}")));
    }

    #[test]
    fn explain_external_contract_rewrites_next_steps_with_explicit_target() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: explicit-target-demo
runtimes:
  node: "22"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "repo-signal-demo"
}"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);

        let output = run_with(["ota", "explain", fixture.file_path().to_str().unwrap()]);
        let stdout = strip_ansi(&output.stdout);
        let contract_path = fs::canonicalize(fixture.file_path())
            .unwrap()
            .display()
            .to_string();

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("rerun"));
        assert!(stdout.contains("ota doctor"));
        assert!(stdout.contains(&contract_path));
    }

    #[test]
    fn doctor_uses_volta_node_remediation_when_contract_provider_is_volta() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: volta-demo
runtimes:
  node:
    version: "22"
    provider: volta
tasks:
  ci:
    run: echo ci
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `volta install node@22` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_nodenv_node_remediation_when_repo_signals_node_version() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: nodenv-demo
runtimes:
  node: "22"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".node-version", "22\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `nodenv install 22` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_rbenv_ruby_remediation_when_repo_signals_ruby_version() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: rbenv-demo
runtimes:
  ruby: "3.3.0"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".ruby-version", "3.3.0\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let ruby_body = if cfg!(windows) {
            "@echo off\r\necho ruby 3.4.1\r\n"
        } else {
            "#!/bin/sh\necho 'ruby 3.4.1'\n"
        };
        write_fake_command(&bin_dir, "ruby", ruby_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `rbenv install 3.3.0` and rerun `ota doctor`"));
    }

    #[test]
    fn doctor_uses_asdf_tool_remediation_when_repo_signals_tool_versions() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: asdf-demo
tools:
  maven: "3.9.9"
tasks:
  ci:
    run: echo ci
"#,
        );
        fixture.write(".tool-versions", "maven 3.9.9\n");

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let mvn_body = if cfg!(windows) {
            "@echo off\r\necho Apache Maven 3.9.14\r\n"
        } else {
            "#!/bin/sh\necho 'Apache Maven 3.9.14'\n"
        };
        let asdf_body = if cfg!(windows) {
            "@echo off\r\necho asdf\r\n"
        } else {
            "#!/bin/sh\necho 'asdf'\n"
        };
        write_fake_command(&bin_dir, "mvn", mvn_body);
        write_fake_command(&bin_dir, "asdf", asdf_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "doctor", "."]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("run `asdf install maven 3.9.9` and rerun `ota doctor`"));
    }

    #[test]
    fn up_external_contract_rewrites_next_steps_with_explicit_target() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: explicit-target-demo
runtimes:
  node: "22"
tasks:
  setup:
    run: echo setup
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);

        let output = run_with(["ota", "up", fixture.file_path().to_str().unwrap()]);
        let stdout = strip_ansi(&output.stdout);
        let contract_path = fs::canonicalize(fixture.file_path())
            .unwrap()
            .display()
            .to_string();

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains(&format!("rerun `ota doctor {contract_path}`")));
    }

    #[test]
    fn detect_text_snapshot_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: existing
tools:
  cargo: "1.78"
tasks:
  build:
    run: cargo build
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "ota-web",
  "packageManager": "pnpm@10.1.0",
  "scripts": { "dev": "vite" }
}"#,
        );
        fixture.write(".nvmrc", "22\n");

        let _cwd = CurrentDirGuard::enter(fixture.dir.path());
        let output = run_with(["ota", "detect", "--dry-run", "."]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot("detect_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn explain_narrow_text_snapshot_is_stable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: narrow-demo
tasks:
  ci:
    run: echo ci
runtimes:
  node: "22"
tools:
  npm: "10"
"#,
        );
        fixture.write(
            "package.json",
            r#"{
  "name": "narrow-demo"
}"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let node_body = if cfg!(windows) {
            "@echo off\r\necho v24.14.1\r\n"
        } else {
            "#!/bin/sh\necho 'v24.14.1'\n"
        };
        write_fake_command(&bin_dir, "node", node_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _columns_guard = EnvVarGuard::set("COLUMNS", OsString::from("48"));
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "explain", "."]);

        assert_eq!(output.exit_code, 1);
        assert_text_snapshot("explain_narrow_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn up_container_mode_surfaces_apt_version_unavailable_finding() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
tools:
  curl: "8.13.0"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    curl:
      source: apt
      approved_versions:
        - "8.13.0"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"curl --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"apt-get\" >nul && (\r\n    echo E: Version '8.13.0' for 'curl' was not found 1>&2\r\n    exit /b 100\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"curl --version\"*) exit 1 ;;\n    *\"apt-get\"*) echo \"E: Version '8.13.0' for 'curl' was not found\" >&2; exit 100 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "up", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 100);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("PROVISION FAILED"));
        assert!(text.contains("Container apt cannot install pinned package version: curl"));
        assert!(text.contains("Task output: E: Version '8.13.0' for 'curl' was not found"));
        assert!(text.contains("ota up --mode container"));
        assert!(!text.contains("Missing tool: curl"));
    }

    #[test]
    fn up_container_mode_surfaces_generic_backend_failure_finding() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"node --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"brew\" >nul && echo %* | findstr /C:\"node@22\" >nul && (\r\n    echo Error: No available formula with the name \"node@22\" 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) exit 1 ;;\n    *brew*node@22*) echo 'Error: No available formula with the name \"node@22\"' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "up", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("PROVISION FAILED"));
        assert!(text.contains("Container brew cannot install pinned version: node"));
        assert!(text.contains("Task output: Error: No available formula with the name"));
        assert!(text.contains("node@22"));
        assert!(text.contains("ota up --mode container"));
        assert!(!text.contains("Missing runtime: node"));
    }

    #[test]
    fn up_container_mode_refines_generic_runtime_backend_failure_via_probe() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-container
execution:
  preferred: container
  supported: [native, container]
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tasks:
  setup:
    run: echo ready
runtimes:
  node: "22"
"#,
        );
        fixture.write(
            ".ota/org-policy.yaml",
            r#"
policies:
  provisioning:
    node:
      source: mise
      approved_versions:
        - "22"
"#,
        );

        let bin_dir = fixture.dir.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("create bin dir");
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"node --version\" >nul && exit /b 1\r\n  echo %* | findstr /C:\"mise\" >nul && echo %* | findstr /C:\"install\" >nul && echo %* | findstr /C:\"node@22\" >nul && (\r\n    echo mise install failed 1>&2\r\n    exit /b 1\r\n  )\r\n  echo %* | findstr /C:\"mise\" >nul && echo %* | findstr /C:\"ls-remote\" >nul && echo %* | findstr /C:\"node@22\" >nul && (\r\n    echo [\"21.0.0\",\"21.1.0\"]\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"node --version\"*) exit 1 ;;\n    *mise*install*node@22*) echo 'mise install failed' >&2; exit 1 ;;\n    *mise*ls-remote*node@22*) printf '[\"21.0.0\",\"21.1.0\"]\\n'; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);
        let path = prepend_path(&bin_dir);
        let _path_guard = EnvVarGuard::set("PATH", path);
        let _cwd = CurrentDirGuard::enter(fixture.dir.path());

        let output = run_with(["ota", "up", "--mode", "container", "."]);

        assert_eq!(output.exit_code, 1);
        let text = strip_ansi(&output.stdout);
        assert!(text.contains("PROVISION FAILED"));
        assert!(text.contains("Container mise cannot install pinned version: node"));
        assert!(text.contains("Task output: mise install failed"));
        assert!(text.contains("ota up --mode container"));
        assert!(!text.contains("Missing runtime: node"));
    }

    #[test]
    fn up_text_snapshot_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-up
execution:
  lifecycle: ephemeral
tasks:
  setup:
    run: python3 -c "from pathlib import Path; Path('prepared.txt').write_text('ready')"
"#,
        );

        let _cwd = CurrentDirGuard::enter(fixture.dir.path());
        let output = run_with(["ota", "up", "."]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot("up_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn run_failure_text_snapshot_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: premium-run
tasks:
  install-from-source:
    run: |
      printf 'build log\n'
      printf 'trace line\n' >&2
      exit 7
"#,
        );

        let output = run_with([
            "ota",
            "run",
            "install-from-source",
            fixture.file_path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 7);
        let stderr = strip_ansi(output.stderr.as_deref().expect("run failure stderr"))
            .replace("./ota.yaml", "<TMP>/ota.yaml");
        assert_text_snapshot_for_dir("run_premium_error.txt", &stderr, fixture.dir.path());
    }

    #[test]
    fn doctor_not_ready_text_status_contract_is_stable() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: ota
"#,
        );

        let output = run_with(["ota", "doctor", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);
        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains(&format!(
            "DOCTOR {}",
            compact_contract(&fixture.file_path())
        )));
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("Primary Blocker"));
        assert!(stdout.contains("No tasks defined in contract"));
        assert!(!stdout.contains("\n---\n"));
    }

    #[test]
    fn workspace_validate_text_snapshot_is_stable() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: premium-workspace
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: python3 -c "from pathlib import Path; Path('ready.txt').write_text('ok')"
"#,
        )
        .unwrap();

        let _cwd = CurrentDirGuard::enter(fixture.path());
        let output = run_with(["ota", "workspace", "validate", "."]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot(
            "workspace_validate_premium.txt",
            &strip_ansi(&output.stdout),
        );
    }

    #[test]
    fn workspace_doctor_text_snapshot_is_stable() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: premium-workspace
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let _cwd = CurrentDirGuard::enter(fixture.path());
        let output = run_with(["ota", "workspace", "doctor", "."]);

        assert_eq!(output.exit_code, 1);
        assert_text_snapshot("workspace_doctor_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn workspace_explain_text_snapshot_is_stable() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: premium-workspace
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let _cwd = CurrentDirGuard::enter(fixture.path());
        let output = run_with(["ota", "workspace", "explain", "."]);

        assert_eq!(output.exit_code, 1);
        assert_text_snapshot("workspace_explain_premium.txt", &strip_ansi(&output.stdout));
    }

    #[test]
    fn workspace_up_text_snapshot_is_stable() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: premium-workspace
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: python3 -c "from pathlib import Path; Path('ready.txt').write_text('ok')"
"#,
        )
        .unwrap();

        let _cwd = CurrentDirGuard::enter(fixture.path());
        let output = run_with(["ota", "workspace", "up", "."]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot_for_dir(
            "workspace_up_premium.txt",
            &strip_ansi(&output.stdout),
            fixture.path(),
        );
    }

    #[test]
    fn workspace_run_text_snapshot_is_stable() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: premium-workspace
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: python3 -c "from pathlib import Path; Path('ready.txt').write_text('ok')"
"#,
        )
        .unwrap();

        let _cwd = CurrentDirGuard::enter(fixture.path());
        let output = run_with(["ota", "workspace", "run", "setup", "."]);

        assert_eq!(output.exit_code, 0);
        assert_text_snapshot_for_dir(
            "workspace_run_premium.txt",
            &strip_ansi(&output.stdout),
            fixture.path(),
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_commands_json_success_contract_is_stable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let single_repo = WorkspaceFixture::new();
        let multi_repo = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(multi_repo.dir.path());

        let init_fixture = TempDir::new().unwrap();
        let web_dir = init_fixture.path().join("apps").join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo web
"#,
        )
        .unwrap();
        let workspace_init = run_with([
            "ota",
            "workspace",
            "init",
            "--json",
            init_fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(workspace_init.exit_code, 0);
        assert_json_top_level_keys(
            &workspace_init,
            &[
                "config",
                "included",
                "missing_contract",
                "mode",
                "ok",
                "path",
                "written",
            ],
        );

        let validate = run_with(["ota", "workspace", "validate", "--json", single_repo.path()]);
        assert_eq!(validate.exit_code, 0);
        assert_json_top_level_keys(&validate, &["ok", "path", "summary"]);

        let tasks = run_with(["ota", "workspace", "tasks", "--json", single_repo.path()]);
        assert_eq!(tasks.exit_code, 0);
        assert_json_top_level_keys(&tasks, &["ok", "path", "repos", "summary"]);

        let run = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--json",
            multi_repo.path(),
        ]);
        assert_eq!(run.exit_code, 0);
        assert_json_top_level_keys(&run, &["ok", "path", "receipt", "repos", "summary", "task"]);

        let check = run_with(["ota", "workspace", "check", "--json", single_repo.path()]);
        assert_eq!(check.exit_code, 0);
        assert_json_top_level_keys(&check, &["ok", "path", "repos", "summary"]);

        let doctor = run_with(["ota", "workspace", "doctor", "--json", single_repo.path()]);
        assert_eq!(doctor.exit_code, 0);
        assert_json_top_level_keys(
            &doctor,
            &["finding_groups", "ok", "path", "repos", "summary"],
        );

        let up = run_with(["ota", "workspace", "up", "--json", multi_repo.path()]);
        assert_eq!(up.exit_code, 0);
        assert_json_top_level_keys(&up, &["ok", "path", "receipt", "repos", "summary"]);
    }

    #[test]
    fn workspace_init_text_preview_and_write() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        let api_dir = fixture.path().join("services").join("api");
        let docs_dir = fixture.path().join("docs-site");
        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&api_dir).unwrap();
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo web
"#,
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: api
tasks:
  setup:
    run: echo api
"#,
        )
        .unwrap();
        fs::write(docs_dir.join("README.md"), "# docs\n").unwrap();

        let preview = run_with([
            "ota",
            "workspace",
            "detect",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ]);
        let preview_stdout = strip_ansi(&preview.stdout);
        assert_eq!(preview.exit_code, 0);
        assert!(preview_stdout.contains(&format!(
            "WORKSPACE DETECT PREVIEW {}",
            compact_path(fixture.path(), ".")
        )));
        assert!(preview_stdout.contains("Mode: dry-run (no write)"));
        assert!(preview_stdout.contains("Included repos:"));
        assert!(preview_stdout.contains("api (services/api)"));
        assert!(preview_stdout.contains("web (apps/web)"));
        assert!(!fixture.path().join("ota.workspace.yaml").exists());

        let write = run_with(["ota", "workspace", "init", fixture.path().to_str().unwrap()]);
        let write_stdout = strip_ansi(&write.stdout);
        assert_eq!(write.exit_code, 0);
        assert!(write_stdout.contains(&format!(
            "WORKSPACE INIT WRITE {}",
            compact_path(fixture.path(), ".")
        )));
        assert!(fixture.path().join("ota.workspace.yaml").exists());
    }

    #[test]
    fn workspace_detect_merge_write_auto_provisions_missing_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        let api_dir = fixture.path().join("services").join("api");
        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo web
"#,
        )
        .unwrap();
        fs::write(
            api_dir.join("package.json"),
            r#"
{
  "name": "api",
  "scripts": {
    "build": "npm run build"
  }
}
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--merge",
            "--write",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 0);
        assert!(api_dir.join("ota.yaml").is_file());

        let workspace = fs::read_to_string(fixture.path().join("ota.workspace.yaml")).unwrap();
        assert!(workspace.contains("services/api"));
    }

    #[test]
    fn workspace_init_dry_run_alias_previews_without_writing() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "init",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("WORKSPACE INIT PREVIEW"));
        assert!(!fixture.path().join("ota.workspace.yaml").exists());
    }

    #[test]
    fn workspace_init_empty_root_provides_clear_next_steps() {
        let fixture = TempDir::new().unwrap();

        let output = run_with(["ota", "workspace", "init", fixture.path().to_str().unwrap()]);
        let body = format!(
            "{}\n{}",
            strip_ansi(&output.stdout),
            strip_ansi(output.stderr.as_deref().unwrap_or(""))
        );

        assert_eq!(output.exit_code, 1);
        assert!(body.contains("workspace init could not find any repos to bootstrap"));
        assert!(body.contains("create repo contracts with `ota init <repo-path>`"));
        assert!(body.contains("preview repo contracts with `ota detect --dry-run <repo-path>`"));
        assert!(body.contains(
            "then run `ota workspace detect --write` or `ota workspace init` after repo contracts exist"
        ));
        assert!(body.contains("Next:"));
    }

    #[test]
    fn workspace_init_bootstrap_auto_provisions_missing_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("Cargo.toml"),
            r#"[package]
name = "web"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "init",
            "--bootstrap",
            fixture.path().to_str().unwrap(),
        ]);
        let body = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(fixture.path().join("ota.workspace.yaml").is_file());
        assert!(repo_dir.join("ota.yaml").is_file());
        assert!(body.contains("Auto-provisioned repo contracts"));
        assert!(body.contains("web (apps/web)"));
    }

    #[test]
    fn workspace_init_without_bootstrap_recommends_bootstrap_for_missing_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("Cargo.toml"),
            r#"[package]
name = "web"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "init", fixture.path().to_str().unwrap()]);
        let body = format!(
            "{}\n{}",
            strip_ansi(&output.stdout),
            strip_ansi(output.stderr.as_deref().unwrap_or(""))
        );

        assert_eq!(output.exit_code, 1);
        assert!(body.contains("workspace init could not find any repos with `ota.yaml`"));
        assert!(
            body.contains(
                "run `ota workspace init --bootstrap` to scaffold missing repo contracts"
            )
        );
        assert!(!repo_dir.join("ota.yaml").exists());
    }

    #[test]
    fn workspace_init_bootstrap_refuses_to_overwrite_existing_workspace_contract() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("Cargo.toml"),
            r#"[package]
name = "web"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "init",
            "--bootstrap",
            fixture.path().to_str().unwrap(),
        ]);
        let body = format!(
            "{}\n{}",
            strip_ansi(&output.stdout),
            strip_ansi(output.stderr.as_deref().unwrap_or(""))
        );

        assert_eq!(output.exit_code, 1);
        assert!(fixture.path().join("ota.workspace.yaml").is_file());
        assert!(!repo_dir.join("ota.yaml").exists());
        assert!(
            body.contains("already exists; refusing to overwrite an existing workspace contract")
        );
        assert!(body.contains("use `ota workspace detect --merge"));
        assert!(body.contains("use `ota workspace detect --rewrite --yes"));
    }

    #[test]
    fn workspace_init_writes_without_bootstrapping_missing_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        let api_dir = fixture.path().join("services").join("api");
        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();
        fs::write(
            api_dir.join("Cargo.toml"),
            r#"[package]
name = "api"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "init", fixture.path().to_str().unwrap()]);
        let body = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(fixture.path().join("ota.workspace.yaml").is_file());
        assert!(web_dir.join("ota.yaml").is_file());
        assert!(!api_dir.join("ota.yaml").exists());
        assert!(!body.contains("Auto-provisioned repo contracts"));
    }

    #[test]
    fn workspace_validate_failure_does_not_append_try_footer() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
version: 1
project:
  namex: broken
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  repo:
    path: repo
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "validate",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 1);
        let stderr = output.stderr.as_deref().unwrap_or_default();
        assert!(!stderr.contains("\nTry: `ota workspace validate`"));
    }

    #[test]
    fn workspace_validate_not_found_includes_workspace_setup_next() {
        let fixture = TempDir::new().unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "validate",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("explicit workspace path does not contain `ota.workspace.yaml`"));
        assert!(stderr.contains("Next:"));
        assert!(stderr.contains("run `ota workspace init` to create a starter workspace"));
    }

    #[test]
    fn workspace_init_merge_alias_routes_to_detect_merge() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        let api_dir = fixture.path().join("services").join("api");
        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: api\n",
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: existing
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "init",
            "--merge",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("WORKSPACE DETECT MERGE PREVIEW"));
    }

    #[test]
    fn workspace_init_refuses_overwrite_and_provides_next_steps() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo web
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: existing
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "init", fixture.path().to_str().unwrap()]);
        let body = format!(
            "{}\n{}",
            strip_ansi(&output.stdout),
            strip_ansi(output.stderr.as_deref().unwrap_or(""))
        );

        assert_eq!(output.exit_code, 1);
        assert!(
            body.contains("already exists; refusing to overwrite an existing workspace contract")
        );
        assert!(body.contains("ota workspace detect --merge"));
        assert!(body.contains("ota workspace detect --rewrite --yes"));
    }

    #[test]
    fn workspace_init_merge_requires_existing_workspace_contract() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--merge",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        assert!(
            output
                .stderr
                .as_deref()
                .unwrap_or("")
                .contains("requires an existing `ota.workspace.yaml`")
        );
    }

    #[test]
    fn workspace_detect_rewrite_requires_yes() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            "version: 1\nworkspace:\n  name: demo\nrepos:\n  web:\n    path: apps/web\n    required: true\n",
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--rewrite",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 1);
        assert!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()).contains("requires `--yes`")
        );
    }

    #[test]
    fn workspace_detect_merge_parse_failure_suggests_validation_first() {
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("qredex-java");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("ota.yaml"),
            r#"
project:
  name: qredex-java
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  qredex-java:
    path: qredex-java
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--merge",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("review the failing repo contract"));
        assert!(stderr.contains("ota validate"));
        assert!(stderr.contains("ota doctor"));
        assert!(stderr.contains("ota workspace detect --merge"));
        assert!(!stderr.contains("ota workspace detect --dry-run"));
    }

    #[test]
    fn workspace_detect_rewrite_writes_and_creates_timestamped_backup() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        fs::create_dir_all(&web_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            "version: 1\nworkspace:\n  namex: broken\nrepos:\n  web:\n    path: apps/web\n    required: true\n",
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--rewrite",
            "--yes",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 0);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("WORKSPACE DETECT REWRITTEN"));
        assert!(body.contains("Backup:"));
        assert!(!body.contains("Included repos:"));

        let backups = fs::read_dir(fixture.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|name| name.starts_with("ota.workspace.yaml.bak-"))
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn workspace_detect_rewrite_ignores_invalid_repo_contracts() {
        let fixture = TempDir::new().unwrap();
        let broken_dir = fixture.path().join("qredex-resources");
        fs::create_dir_all(&broken_dir).unwrap();
        fs::write(
            broken_dir.join("ota.yaml"),
            r#"
version: 1
project:
  namex: broken
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: existing
repos:
  qredex-resources:
    path: qredex-resources
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--rewrite",
            "--yes",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("WORKSPACE DETECT REWRITTEN"));
        assert!(fixture.path().join("ota.workspace.yaml").is_file());

        let validate = run_with([
            "ota",
            "workspace",
            "validate",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(validate.exit_code, 0);
    }

    #[test]
    fn workspace_init_merge_adds_missing_repos_without_overwriting_existing_entries() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        let api_dir = fixture.path().join("services").join("api");
        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: api\n",
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: existing
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--merge",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("WORKSPACE DETECT MERGED"));

        let written = fs::read_to_string(fixture.path().join("ota.workspace.yaml")).unwrap();
        assert!(written.contains("web:"));
        assert!(written.contains("required: false"));
        assert!(written.contains("api:"));
        assert!(written.contains("path: services/api"));
    }

    #[test]
    fn workspace_init_merge_dry_run_does_not_write_workspace_contract() {
        let fixture = TempDir::new().unwrap();
        let web_dir = fixture.path().join("apps").join("web");
        let api_dir = fixture.path().join("services").join("api");
        fs::create_dir_all(&web_dir).unwrap();
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(
            web_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: web\n",
        )
        .unwrap();
        fs::write(
            api_dir.join("ota.yaml"),
            "version: 1\nproject:\n  name: api\n",
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: existing
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        let before = fs::read_to_string(fixture.path().join("ota.workspace.yaml")).unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "detect",
            "--merge",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("WORKSPACE DETECT MERGE PREVIEW"));

        let after = fs::read_to_string(fixture.path().join("ota.workspace.yaml")).unwrap();
        assert_eq!(before, after);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_commands_json_validation_failure_contract_is_stable() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  dev:
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let validate = run_with(["ota", "workspace", "validate", "--json", fixture.path()]);
        assert_eq!(validate.exit_code, 1);
        assert_json_top_level_keys(&validate, &["errors", "ok", "path", "summary"]);

        let tasks = run_with(["ota", "workspace", "tasks", "--json", fixture.path()]);
        assert_eq!(tasks.exit_code, 1);
        assert_json_top_level_keys(&tasks, &["errors", "ok", "path"]);

        let run = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);
        assert_eq!(run.exit_code, 1);
        assert_json_top_level_keys(&run, &["ok", "path", "receipt", "repos", "summary", "task"]);

        let check = run_with(["ota", "workspace", "check", "--json", fixture.path()]);
        assert_eq!(check.exit_code, 1);
        assert_json_top_level_keys(
            &check,
            &["finding_groups", "ok", "path", "repos", "summary"],
        );

        let doctor = run_with(["ota", "workspace", "doctor", "--json", fixture.path()]);
        assert_eq!(doctor.exit_code, 1);
        assert_json_top_level_keys(
            &doctor,
            &["finding_groups", "ok", "path", "repos", "summary"],
        );

        let up = run_with(["ota", "workspace", "up", "--json", fixture.path()]);
        assert_eq!(up.exit_code, 1);
        assert_json_top_level_keys(&up, &["ok", "path", "receipt", "repos", "summary"]);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_commands_exit_code_contract_is_stable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let usage_fixture = WorkspaceFixture::new();
        let usage = run_with([
            "ota",
            "workspace",
            "up",
            "--jobs",
            "0",
            usage_fixture.path(),
        ]);
        assert_eq!(usage.exit_code, 2);

        let invalid_fixture = WorkspaceFixture::new();
        fs::write(
            invalid_fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    required: true
"#,
        )
        .unwrap();
        let validate_invalid = run_with(["ota", "workspace", "validate", invalid_fixture.path()]);
        assert_eq!(validate_invalid.exit_code, 1);

        let failing_up = WorkspaceFixture::new();
        fs::write(
            failing_up.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            failing_up
                .dir
                .path()
                .join("apps")
                .join("web")
                .join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 9
"#,
        )
        .unwrap();
        let up_failure = run_with(["ota", "workspace", "up", failing_up.path()]);
        assert_eq!(up_failure.exit_code, 1);

        let success_up = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(success_up.dir.path());
        let up_success = run_with(["ota", "workspace", "up", success_up.path()]);
        assert_eq!(up_success.exit_code, 0);
    }

    #[test]
    fn monorepo_member_json_contract_is_stable() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  setup:
    run: printf root
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: printf api
"#,
        );
        fixture.write(
            "web/ota.yaml",
            r#"
project:
  name: web
tasks:
  lint:
    run: printf web
"#,
        );

        let validate = run_with([
            "ota",
            "validate",
            "--member",
            "api",
            "--json",
            fixture.path(),
        ]);
        assert_eq!(validate.exit_code, 0);
        assert_json_top_level_keys(&validate, &["ok", "path", "summary"]);

        let tasks = run_with([
            "ota",
            "tasks",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);
        assert_eq!(tasks.exit_code, 0);
        assert_json_top_level_keys(&tasks, &["members", "ok", "path", "tasks"]);

        let doctor = run_with([
            "ota",
            "doctor",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);
        assert_eq!(doctor.exit_code, 0);
        assert_json_top_level_keys(&doctor, &["findings", "members", "ok", "path"]);

        let check = run_with([
            "ota",
            "check",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);
        assert_eq!(check.exit_code, 0);
        assert_json_top_level_keys(&check, &["findings", "members", "ok", "path"]);

        let up = run_with([
            "ota",
            "up",
            "--member",
            "api",
            "--member",
            "web",
            "--json",
            fixture.path(),
        ]);
        assert_eq!(up.exit_code, 0);
        assert_json_top_level_keys(
            &up,
            &[
                "dry_run", "findings", "members", "ok", "path", "phase", "status",
            ],
        );
    }

    #[test]
    fn monorepo_member_text_status_contract_is_stable() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
env:
  OTA_MEMBER_REQUIRED:
    required: true
tasks:
  test:
    run: printf api
"#,
        );

        let doctor = run_with(["ota", "doctor", "--member", "api", fixture.path()]);
        let doctor_stdout = strip_ansi(&doctor.stdout);
        assert_eq!(doctor.exit_code, 1);
        assert!(doctor_stdout.contains(&format!(
            "DOCTOR {} [member api]",
            compact_contract(&fixture.file_path())
        )));
        assert!(doctor_stdout.contains("NOT READY"));
        assert!(doctor_stdout.contains("Missing environment variable: OTA_MEMBER_REQUIRED"));
        assert!(!doctor_stdout.contains("\n---\n"));
    }

    #[test]
    fn monorepo_member_exit_code_contract_is_stable() {
        let fixture = ContractFixture::new_dir();
        fixture.write(
            "ota.yaml",
            r#"
version: 1
project:
  name: ota-monorepo
workspace:
  type: monorepo
  members:
    - api
tasks:
  test:
    run: 'true'
"#,
        );
        fixture.write(
            "api/ota.yaml",
            r#"
project:
  name: api
tasks:
  test:
    run: 'true'
"#,
        );

        let duplicate = run_with([
            "ota",
            "tasks",
            "--member",
            "api",
            "--member",
            "api",
            fixture.path(),
        ]);
        assert_eq!(duplicate.exit_code, 2);

        let run_ok = run_with(["ota", "run", "test", "--member", "api", fixture.path()]);
        assert_eq!(run_ok.exit_code, 0);
    }

    #[test]
    fn workspace_tasks_json_reports_dependency_order_and_tasks() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "workspace", "tasks", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["summary"]["repo_count"], 2);
        assert_eq!(json["summary"]["acquired_count"], 2);
        assert_eq!(json["summary"]["task_count"], 2);
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][0]["tasks"][0]["name"], "setup");
        assert_eq!(json["repos"][1]["name"], "api");
        assert_eq!(json["repos"][1]["depends_on"][0], "db");
    }

    #[test]
    fn workspace_list_text_and_json_report_repos() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let text = run_with(["ota", "workspace", "list", fixture.path()]);
        assert_eq!(text.exit_code, 0);
        let text_body = strip_ansi(&text.stdout);
        assert!(text_body.contains("WORKSPACE LIST"));
        assert!(text_body.contains("db [required] (ACQUIRED)"));
        assert!(text_body.contains("Status: READY"));
        assert!(text_body.contains("api [required] (ACQUIRED)"));
        assert!(text_body.contains("Execution"));
        assert!(text_body.contains("Remote: provider `ssh` target `user@host`"));

        let json = run_with(["ota", "workspace", "list", "--json", fixture.path()]);
        assert_eq!(json.exit_code, 0);
        let body: Value = serde_json::from_str(&json.stdout).unwrap();
        assert_eq!(body["ok"], true);
        assert_eq!(body["summary"]["repo_count"], 2);
        assert_eq!(body["summary"]["ready_count"], 2);
        assert_eq!(body["summary"]["not_ready_count"], 0);
        assert_eq!(body["summary"]["acquired_count"], 2);
        assert_eq!(body["summary"]["missing_contract_count"], 0);
        assert_eq!(body["repos"][0]["name"], "db");
        assert_eq!(body["repos"][0]["status"], "READY");
        assert_eq!(body["repos"][1]["name"], "api");
        assert_eq!(body["repos"][1]["status"], "READY");
        assert_eq!(body["repos"][1]["execution"]["preferred"], "remote");
        assert_eq!(body["repos"][1]["execution"]["supported"][0], "remote");
        assert_eq!(body["repos"][1]["execution"]["lifecycle"], "ephemeral");
        assert_eq!(
            body["repos"][1]["execution"]["backends"]["remote"]["provider"],
            "ssh"
        );
        assert_eq!(
            body["repos"][1]["execution"]["backends"]["remote"]["target"],
            "user@host"
        );
        assert_eq!(
            body["repos"][1]["execution"]["backends"]["remote"]["cwd"],
            fixture
                .dir
                .path()
                .join("services")
                .join("api")
                .display()
                .to_string()
        );
    }

    #[test]
    fn workspace_list_status_filter_limits_repos() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("ready")).unwrap();
        fs::create_dir_all(fixture.path().join("not-ready")).unwrap();

        fs::write(
            fixture.path().join("ready").join("ota.yaml"),
            r#"
version: 1
project:
  name: ready
tasks:
  ok:
    run: 'true'
"#,
        )
        .unwrap();

        fs::write(
            fixture.path().join("not-ready").join("ota.yaml"),
            r#"
version: 1
project:
  name: not-ready
env:
  REQUIRED_VALUE:
    required: true
"#,
        )
        .unwrap();

        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  ready:
    path: ready
    required: true
  not-ready:
    path: not-ready
    required: true
"#,
        )
        .unwrap();

        let ready = run_with([
            "ota",
            "workspace",
            "list",
            "--json",
            "--status",
            "ready",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(ready.exit_code, 0);
        let ready_json: Value = serde_json::from_str(&ready.stdout).unwrap();
        assert_eq!(ready_json["repos"].as_array().unwrap().len(), 1);
        assert_eq!(ready_json["repos"][0]["name"], "ready");
        assert_eq!(ready_json["repos"][0]["status"], "READY");

        let not_ready = run_with([
            "ota",
            "workspace",
            "list",
            "--json",
            "--status",
            "not-ready",
            fixture.path().to_str().unwrap(),
        ]);
        assert_eq!(not_ready.exit_code, 0);
        let not_ready_json: Value = serde_json::from_str(&not_ready.stdout).unwrap();
        assert_eq!(not_ready_json["repos"].as_array().unwrap().len(), 1);
        assert_eq!(not_ready_json["repos"][0]["name"], "not-ready");
        assert_eq!(not_ready_json["repos"][0]["status"], "NOT READY");
    }

    #[test]
    fn workspace_list_repo_filter_rejects_unknown_repo() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with([
            "ota",
            "workspace",
            "list",
            "--repo",
            "missing-repo",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = output.stderr.as_deref().unwrap_or_default();
        assert!(stderr.contains("unknown workspace repo `missing-repo`"));
        assert!(stderr.contains("Known repos:"));
    }

    #[test]
    fn workspace_list_marks_missing_contract_explicitly() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  api:
    path: api
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "list", fixture.path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 0);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("api [required] (ACQUIRED)"));
        assert!(body.contains("Status: NOT READY"));
        assert!(body.contains("Contract: missing (run"));
        assert!(body.contains("ota init"));
    }

    #[test]
    fn workspace_list_not_found_uses_path_where_and_actionable_next() {
        let fixture = TempDir::new().unwrap();

        let output = run_with(["ota", "workspace", "list", fixture.path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("explicit workspace path does not contain `ota.workspace.yaml`"));
        assert!(stderr.contains("Next: run `ota workspace init` to create a starter workspace"));
    }

    #[test]
    fn validate_not_found_splits_why_and_next_actions() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();

        let output = run_with(["ota", "validate", fixture.path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("explicit repo path does not contain `ota.yaml`"));
        assert!(stderr.contains("Next:"));
        assert!(stderr.contains("run `ota init` to create a starter contract"));
    }

    #[test]
    fn doctor_not_found_uses_shared_next_actions() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();

        let output = run_with(["ota", "doctor", fixture.path().to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Primary Blocker"));
        assert!(stdout.contains("Why:"));
        assert!(stdout.contains("ota.yaml"));
        assert!(stdout.contains("Next:"));
        assert!(stdout.contains("ota init"));
        assert!(!stdout.contains("create missing repo contracts with `ota init <repo-path>`"));
    }

    #[test]
    fn workspace_doctor_not_found_uses_single_workspace_next() {
        let fixture = TempDir::new().unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("explicit workspace path does not contain `ota.workspace.yaml`"));
        assert_eq!(stderr.matches("Next:").count(), 1);
        assert!(stderr.contains("run `ota workspace init` to create a starter workspace"));
        assert!(!stderr.contains("Next: `ota workspace doctor`"));
    }

    #[test]
    fn workspace_list_concise_omits_detail_lines() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "--concise", "workspace", "list", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("WORKSPACE LIST"));
        assert!(body.contains("db [required] (ACQUIRED)"));
        assert!(body.contains("Status: READY"));
        assert!(!body.contains("Path:"));
        assert!(!body.contains("Contract:"));
        assert!(!body.contains("Depends On:"));
    }

    #[test]
    fn doctor_concise_keeps_next_but_omits_why_lines() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: no-tasks
"#,
        );

        let output = run_with(["ota", "--concise", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("DOCTOR"));
        assert!(body.contains("NOT READY"));
        assert!(body.contains("Next:"));
        assert!(!body.contains("Why:"));
    }

    #[test]
    fn workspace_doctor_concise_omits_path_contract_and_why() {
        let fixture = WorkspaceFixture::new_multi_repo();
        let db_contract = fixture
            .dir
            .path()
            .join("services")
            .join("db")
            .join("ota.yaml");
        fs::write(
            db_contract,
            r#"
version: 1
project:
  name: db
"#,
        )
        .unwrap();

        let output = run_with(["ota", "--concise", "workspace", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("WORKSPACE DOCTOR"));
        assert!(body.contains("db [required] (NOT READY)"));
        assert!(body.contains("Next:"));
        assert!(!body.contains("Path:"));
        assert!(!body.contains("Contract:"));
        assert!(!body.contains("Why:"));
    }

    #[test]
    fn check_concise_keeps_next_but_omits_why_lines() {
        let fixture = ContractFixture::new(
            r#"
version: 1
project:
  name: failing-check
checks:
  - name: smoke
    kind: health
    severity: error
    run: exit 1
"#,
        );

        let output = run_with(["ota", "--concise", "check", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("CHECK"));
        assert!(body.contains("NOT READY"));
        assert!(body.contains("Next:"));
        assert!(!body.contains("Why:"));
    }

    #[test]
    fn workspace_check_concise_omits_path_contract_and_why() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let output = run_with(["ota", "--concise", "workspace", "check", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("WORKSPACE CHECK"));
        assert!(body.contains("web [required] (NOT READY)"));
        assert!(body.contains("Next:"));
        assert!(!body.contains("Path:"));
        assert!(!body.contains("Contract:"));
        assert!(!body.contains("Why:"));
    }

    #[test]
    fn workspace_tasks_reports_not_acquired_repo() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    required: true
    source:
      repo: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["acquired"], false);
        assert!(json["repos"][0]["tasks"].as_array().unwrap().is_empty());
    }

    #[test]
    fn workspace_tasks_failure_uses_concise_next_step() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
project:
  name: web
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("Next:"));
        assert!(stderr.contains("fix the failing repo contract with `ota validate`"));
        assert!(stderr.contains("rerun `ota workspace tasks`"));
    }

    #[test]
    fn workspace_tasks_discovers_workspace_from_nested_directory() {
        let fixture = WorkspaceFixture::new();
        let nested = fixture.dir.path().join("apps").join("web").join("src");
        fs::create_dir_all(&nested).unwrap();

        let _cwd = CurrentDirGuard::enter(&nested);
        let output = run_with(["ota", "workspace", "tasks"]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("WORKSPACE TASKS ./ota.workspace.yaml"));
    }

    #[test]
    fn workspace_tasks_uses_repo_path_in_command_preview() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  site:
    path: qredex-site
    required: true
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.dir.path().join("qredex-site")).unwrap();
        fs::write(
            fixture.dir.path().join("qredex-site").join("ota.yaml"),
            r#"
version: 1
project:
  name: site
tasks:
  typecheck:
    run: npm run typecheck
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "tasks", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("Use:"));
        assert!(stdout.contains("ota run typecheck"));
        assert!(stdout.contains("qredex-site"));
        assert!(!stdout.contains("workspace run typecheck --repo"));
    }

    #[test]
    fn workspace_validate_discovers_workspace_from_nested_directory() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new();
        let nested = fixture.dir.path().join("apps").join("web").join("src");
        fs::create_dir_all(&nested).unwrap();

        let _cwd = CurrentDirGuard::enter(&nested);
        let output = run_with(["ota", "workspace", "validate"]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("WORKSPACE VALIDATE"));
        assert!(stdout.contains("VALID"));
        assert!(stdout.contains("WORKSPACE VALIDATE ./ota.workspace.yaml"));
        assert!(stdout.contains("run `ota workspace doctor` to inspect readiness"));
        assert!(stdout.contains("run `ota workspace tasks` to inspect runnable task usage"));
    }

    #[test]
    fn workspace_validate_does_not_walk_past_repo_root_to_parent_workspace() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let outer = TempDir::new().unwrap();
        fs::write(
            outer.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: outer
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();

        let repo_root = outer.path().join("child-repo");
        fs::create_dir_all(repo_root.join("nested")).unwrap();
        fs::create_dir_all(repo_root.join(".git")).unwrap();

        let _cwd = CurrentDirGuard::enter(&repo_root.join("nested"));
        let output = run_with(["ota", "workspace", "validate"]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("no `ota.workspace.yaml` found"));
        assert!(!stderr.contains("VALID"));
    }

    #[test]
    fn workspace_validate_reports_invalid_repo_contract() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  dev:
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "validate", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("ERROR"));
        assert!(stderr.contains("Where:"));
        assert!(stderr.contains("Why:"));
        assert!(stderr.contains("workspace repo `web` contract"));
        assert!(!stderr.contains("/Users/"));
    }

    #[test]
    fn workspace_doctor_status_not_ready_filters_repos() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("good")).unwrap();
        fs::create_dir_all(fixture.path().join("broken")).unwrap();
        fs::write(
            fixture.path().join("good").join("ota.yaml"),
            r#"
version: 1
project:
  name: good
tasks:
  setup:
    run: echo ok
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  good:
    path: good
    required: true
  broken:
    path: broken
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--status",
            "not-ready",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let body = strip_ansi(&output.stdout);
        assert!(!body.contains("good [required]"));
        assert!(body.contains("broken [required]"));
    }

    #[test]
    fn workspace_doctor_repo_filter_scopes_output() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "workspace", "doctor", "--repo", "db", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("db [required]"));
        assert!(!body.contains("api [required]"));
    }

    #[test]
    fn workspace_doctor_severity_error_filters_findings() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join("broken")).unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: demo
repos:
  broken:
    path: broken
    required: true
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--severity",
            "error",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let body = strip_ansi(&output.stdout);
        assert!(body.contains("ERROR  Missing repo contract"));
    }

    #[test]
    fn workspace_doctor_repo_filter_rejects_unknown_repo() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--repo",
            "missing-repo",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 1);
        let stderr = output.stderr.as_deref().unwrap_or_default();
        assert!(stderr.contains("unknown workspace repo `missing-repo`"));
        assert!(stderr.contains("Known repos:"));
        let plain = strip_ansi(stderr);
        assert_eq!(plain.matches("Next:").count(), 1);
    }

    #[test]
    fn workspace_doctor_text_status_contract_is_stable() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains(&format!(
            "WORKSPACE DOCTOR {}",
            compact_workspace(&fixture.workspace_file())
        )));
        assert!(stdout.contains("NOT READY"));
        assert!(stdout.contains("No tasks defined in contract"));
        assert!(!stdout.contains("\n---\n"));
    }

    #[test]
    fn workspace_doctor_text_lists_extensions() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
tasks:
  setup:
    run: cargo build
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("Extensions:"));
        assert!(stdout.contains("demo"));
        assert!(stdout.contains("Kind: check_provider"));
    }

    #[test]
    fn workspace_doctor_text_reports_execution_metadata() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with(["ota", "workspace", "doctor", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains("Execution"));
        assert!(stdout.contains("Overview"));
        assert!(stdout.contains("»"));
        assert!(stdout.contains("Repos: 2"));
        assert!(stdout.contains("Ready: 2"));
        assert!(stdout.contains("Preferred: `remote`"));
        assert!(stdout.contains("Remote: provider `ssh` target `user@host`"));
        assert!(stdout.rfind("Overview") > stdout.rfind("Preferred: `remote`"));
    }

    #[test]
    fn workspace_doctor_explicit_directory_does_not_walk_up_to_parent_contract() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
"#,
        )
        .unwrap();
        let nested = fixture.dir.path().join("nested");
        fs::create_dir_all(&nested).unwrap();

        let output = run_with(["ota", "workspace", "doctor", nested.to_str().unwrap()]);

        assert_eq!(output.exit_code, 1);
        let stderr = strip_ansi(output.stderr.as_deref().unwrap_or_default());
        assert!(stderr.contains("explicit workspace path does not contain `ota.workspace.yaml`"));
        assert!(!stderr.contains("upward"));
    }

    #[test]
    fn workspace_check_text_reports_summary_rollup() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "check", fixture.path()]);
        let body = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(body.contains("Overview"));
        assert!(body.contains("»"));
        assert!(body.contains("Repos:"));
        assert!(body.contains("Errors:"));
        assert!(body.contains("WORKSPACE CHECK"));
    }

    #[test]
    fn workspace_doctor_stream_emits_progress_updates() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", "--stream", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("WORKSPACE DOCTOR"));
        assert!(stdout.contains("NOT READY"));
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_text_status_contract_is_stable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(fixture.dir.path());

        let output = run_with(["ota", "workspace", "up", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains(&format!(
            "WORKSPACE UP {}",
            compact_workspace(&fixture.workspace_file())
        )));
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("SUMMARY"));
        assert!(stdout.contains("Phase: post-setup diagnosis"));
        assert!(!stdout.contains("RECEIPT"));
        assert!(!stdout.contains("\n---\n"));
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_quiet_suppresses_progress_output() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(fixture.dir.path());

        let output = run_with(["ota", "workspace", "up", "--quiet", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 0);
        assert!(stdout.contains(&format!(
            "WORKSPACE UP {}",
            compact_workspace(&fixture.workspace_file())
        )));
        assert!(stdout.contains("READY"));
        assert!(output.stderr.is_none());
    }

    #[test]
    fn workspace_doctor_json_reports_repo_findings() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
execution:
  preferred: remote
  supported:
    - remote
  lifecycle: ephemeral
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: /workspace
env:
  OTA_WORKSPACE_REQUIRED:
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["summary"]["repo_count"], 1);
        assert_eq!(json["summary"]["ready_count"], 0);
        assert_eq!(json["summary"]["not_ready_count"], 1);
        assert_eq!(json["summary"]["verdict"], "not_ready");
        assert_eq!(json["summary"]["agent_verdict"], "not_ready");
        assert_eq!(json["summary"]["error_count"], 2);
        assert_eq!(json["summary"]["warn_count"], 1);
        assert_eq!(json["summary"]["info_count"], 0);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["ok"], false);
        assert_eq!(json["repos"][0]["execution"]["preferred"], "remote");
        assert_eq!(json["repos"][0]["execution"]["supported"][0], "remote");
        assert_eq!(json["repos"][0]["execution"]["lifecycle"], "ephemeral");
        assert_eq!(
            json["repos"][0]["execution"]["backends"]["remote"]["provider"],
            "ssh"
        );
        assert_eq!(
            json["repos"][0]["execution"]["backends"]["remote"]["target"],
            "user@host"
        );
        assert_eq!(
            json["repos"][0]["execution"]["backends"]["remote"]["cwd"],
            "/workspace"
        );
        assert_eq!(
            json["repos"][0]["extensions"]["demo"]["kind"],
            "check_provider"
        );
        assert_eq!(
            json["repos"][0]["extensions"]["demo"]["command"],
            "ota-ext-demo"
        );
        assert_eq!(json["repos"][0]["extensions"]["demo"]["api_version"], 1);
        assert_eq!(
            json["repos"][0]["findings"][0]["summary"],
            "Missing environment variable: OTA_WORKSPACE_REQUIRED"
        );
    }

    #[test]
    fn workspace_doctor_json_includes_policy_provenance() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: web
    required: true
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web").join(".ota")).unwrap();
        fs::write(
            fixture
                .path()
                .join("web")
                .join(".ota")
                .join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let finding = &json["repos"][0]["findings"][0];
        assert_eq!(finding["summary"], "Repo does not satisfy org policy pack");
        assert_eq!(finding["policy_outcome"], "blocked_by_policy");
        assert_eq!(finding["policy_reason"], "missing_required_sections");
        assert_eq!(finding["policy_source"], "org");
        assert_eq!(finding["install_scope"], "repo_local");
        assert_eq!(finding["mutation_allowed"], false);
    }

    #[test]
    fn workspace_doctor_json_includes_provisioning_request() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: web
    required: true
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        fs::write(
            fixture.path().join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
runtimes:
  java: "22"
tools:
  maven: "3.9"
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web").join(".ota")).unwrap();
        fs::write(
            fixture
                .path()
                .join("web")
                .join(".ota")
                .join("org-policy.yaml"),
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      source_config:
        feed: internal-jdk
      approved_versions:
        - "22"
    maven:
      source: approved-manager
      approved_versions:
        - "3.9"
  adapter_bootstrap:
    mise:
      source: brew
      approved_versions:
        - "4.4"
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_ne!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        let provisioning_finding = json["repos"][0]["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["summary"] == "Policy-backed provisioning sources are declared")
            .expect("workspace provisioning finding should be present");
        assert_eq!(provisioning_finding["severity"], "info");
        assert!(
            provisioning_finding["why"]
                .as_str()
                .unwrap()
                .contains("source_config: feed=internal-jdk")
        );
        let provisioning = json["repos"][0]["provisioning"]
            .as_object()
            .expect("workspace provisioning should be present");
        assert_eq!(provisioning["plan"]["allowed"].as_array().unwrap().len(), 2);
        let provisioning_request = provisioning["request"]
            .as_object()
            .expect("workspace provisioning request should be present");
        assert_eq!(provisioning_request["actions"].as_array().unwrap().len(), 2);
        assert_eq!(provisioning_request["actions"][0]["kind"], "select_source");
        let adapter_bootstrap = json["repos"][0]["adapter_bootstrap"]
            .as_object()
            .expect("workspace adapter bootstrap should be present");
        assert_eq!(
            adapter_bootstrap["plan"]["allowed"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            adapter_bootstrap["request"]["actions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(adapter_bootstrap["request"]["actions"][0]["source"], "brew");
    }

    #[test]
    fn workspace_doctor_downgrades_optional_repo_errors_to_warnings() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
env:
  OTA_OPTIONAL_REQUIRED:
    required: true
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "doctor", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("web [optional] (READY)"));
        assert!(stdout.contains("WARN  Missing environment variable: OTA_OPTIONAL_REQUIRED"));
    }

    #[test]
    fn workspace_validate_allows_missing_repo_path_with_source() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    source:
      repo: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "validate",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("WORKSPACE VALIDATE"));
        assert!(stdout.contains("VALID"));
        assert!(stdout.contains(&compact_workspace(
            &fixture.path().join("ota.workspace.yaml")
        )));
    }

    #[test]
    fn workspace_doctor_reports_not_acquired_repo_with_source() {
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    required: true
    source:
      repo: web
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["ok"], false);
        assert_eq!(
            json["repos"][0]["findings"][0]["summary"],
            "Repo not acquired: web"
        );
    }

    #[test]
    fn workspace_doctor_jobs_preserves_dependency_order_in_output() {
        let fixture = WorkspaceFixture::new_multi_repo();

        let output = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][1]["name"], "api");
    }

    #[cfg(unix)]
    #[test]
    fn workspace_parallel_commands_preserve_dependency_order_in_json() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(fixture.dir.path());

        let doctor = run_with([
            "ota",
            "workspace",
            "doctor",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(doctor.exit_code, 0);
        let doctor_json: Value = serde_json::from_str(&doctor.stdout).unwrap();
        assert_eq!(doctor_json["repos"][0]["name"], "db");
        assert_eq!(doctor_json["repos"][1]["name"], "api");

        let check = run_with([
            "ota",
            "workspace",
            "check",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(check.exit_code, 0);
        let check_json: Value = serde_json::from_str(&check.stdout).unwrap();
        assert_eq!(check_json["repos"][0]["name"], "db");
        assert_eq!(check_json["repos"][1]["name"], "api");

        let up = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(up.exit_code, 0);
        let up_json: Value = serde_json::from_str(&up.stdout).unwrap();
        assert_eq!(up_json["repos"][0]["name"], "db");
        assert_eq!(up_json["repos"][1]["name"], "api");

        let run = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);
        assert_eq!(run.exit_code, 0);
        let run_json: Value = serde_json::from_str(&run.stdout).unwrap();
        assert_eq!(run_json["repos"][0]["name"], "db");
        assert_eq!(run_json["repos"][1]["name"], "api");
    }

    #[test]
    fn workspace_doctor_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "doctor", "--jobs", "0", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--jobs` must be greater than zero"
        );
    }

    #[test]
    fn workspace_check_json_reports_repo_findings() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "check", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["ok"], false);
        assert_eq!(
            json["repos"][0]["findings"][0]["summary"],
            "Check failed: health-check"
        );
    }

    #[test]
    fn workspace_check_downgrades_optional_repo_errors_to_warnings() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "check", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert!(strip_ansi(&output.stdout).contains("READY"));
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("web [optional] (READY)"));
        assert!(stdout.contains("WARN  Check failed: health-check"));
    }

    #[test]
    fn workspace_check_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "check", "--jobs", "0", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--jobs` must be greater than zero"
        );
    }

    #[test]
    fn workspace_up_json_reports_required_repo_failure() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 9
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "SETUP FAILED");
        assert_eq!(json["repos"][0]["phase"], "setup");
        assert_eq!(json["repos"][0]["task"], "setup");
        assert_eq!(json["repos"][0]["exit_code"], 9);
        assert_eq!(json["receipt"]["scope"], "workspace");
    }

    #[test]
    fn workspace_up_ignores_optional_repo_failure_in_aggregate_status() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 7
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("web [optional] (WARN)"));
        assert!(stdout.contains("Exit code: 7"));
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_respects_repo_dependency_order() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(fixture.dir.path());

        let output = run_with(["ota", "workspace", "up", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let marker = fs::read_to_string(
            fixture
                .dir
                .path()
                .join("services")
                .join("api")
                .join("workspace-order.txt"),
        )
        .unwrap();
        assert_eq!(marker, "db\napi\n");
    }

    #[test]
    fn workspace_up_blocks_dependent_repo_when_dependency_fails() {
        let fixture = WorkspaceFixture::new_multi_repo();
        fs::write(
            fixture
                .dir
                .path()
                .join("services")
                .join("db")
                .join("ota.yaml"),
            r#"
version: 1
project:
  name: db
tasks:
  setup:
    run: exit 5
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "up", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][0]["status"], "SETUP FAILED");
        assert_eq!(json["repos"][1]["name"], "api");
        assert_eq!(json["repos"][1]["status"], "BLOCKED");
        assert_eq!(json["repos"][1]["phase"], "dependencies");
        assert_eq!(
            json["repos"][1]["findings"][0]["summary"],
            "Blocked by failed dependency: db"
        );
    }

    #[test]
    fn workspace_up_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with(["ota", "workspace", "up", "--jobs", "0", fixture.path()]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--jobs` must be greater than zero"
        );
    }

    #[test]
    fn workspace_up_rejects_stream_with_json() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            "--stream",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--stream` is only supported for text output"
        );
    }

    #[test]
    fn workspace_up_rejects_stream_with_parallel_jobs() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--stream",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--stream` currently requires `--jobs 1`"
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_acquires_missing_repo_from_git_source() {
        let fixture = TempDir::new().unwrap();
        let origin = init_git_repo(
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: printf ready
"#,
        );

        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            format!(
                r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
    source:
      git: {}
"#,
                origin.path().display()
            ),
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "READY");
        assert!(
            fixture
                .path()
                .join("apps")
                .join("web")
                .join("ota.yaml")
                .is_file()
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_acquires_missing_repo_from_repo_source() {
        let fixture = TempDir::new().unwrap();
        let origins = TempDir::new().unwrap();
        let origin = init_named_git_repo(
            origins.path(),
            "web-origin",
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: printf ready
"#,
        );

        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            format!(
                r#"
version: 1
workspace:
  name: ota-dev
  git_base: {}
repos:
  web:
    path: apps/web
    required: true
    source:
      repo: {}
"#,
                origins.path().display(),
                origin.file_name().unwrap().to_string_lossy()
            ),
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            fixture.path().to_str().unwrap(),
        ]);

        assert_eq!(output.exit_code, 0);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "READY");
        assert!(
            fixture
                .path()
                .join("apps")
                .join("web")
                .join("ota.yaml")
                .is_file()
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_up_jobs_runs_independent_repos_in_parallel() {
        let fixture = WorkspaceFixture::new_parallel_repos();

        let started = Instant::now();
        let output = run_with([
            "ota",
            "workspace",
            "up",
            "--json",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        assert!(started.elapsed() < Duration::from_millis(1800));

        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["repos"][0]["name"], "one");
        assert_eq!(json["repos"][0]["stdout"], "one-out");
        assert_eq!(json["repos"][0]["stderr"], "one-err");
        assert_eq!(json["repos"][1]["name"], "two");
        assert_eq!(json["repos"][1]["stdout"], "two-out");
        assert_eq!(json["repos"][1]["stderr"], "two-err");
    }

    #[test]
    fn workspace_run_json_reports_required_repo_failure() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 9
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["task"], "setup");
        assert_eq!(json["summary"]["repo_count"], 1);
        assert_eq!(json["summary"]["ready_count"], 0);
        assert_eq!(json["summary"]["not_ready_count"], 1);
        assert_eq!(json["summary"]["error_count"], 1);
        assert_eq!(json["receipt"]["scope"], "workspace");
        assert_eq!(json["receipt"]["summary"]["step_count"], 1);
        assert_eq!(json["repos"][0]["name"], "web");
        assert_eq!(json["repos"][0]["status"], "TASK FAILED");
        assert_eq!(json["repos"][0]["task"], "setup");
        assert_eq!(json["repos"][0]["exit_code"], 9);
    }

    #[test]
    fn workspace_run_text_reports_single_primary_output_block() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    script: |
      printf 'task-out\n'
      printf 'task-err\n' >&2
      exit 7
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "run", "setup", fixture.path()]);
        let stdout = strip_ansi(&output.stdout);

        assert_eq!(output.exit_code, 1);
        assert!(stdout.contains("Task output:"));
        assert!(stdout.contains("task-err"));
        assert!(!stdout.contains("Stdout:"));
        assert!(!stdout.contains("Stderr:"));
    }

    #[test]
    fn workspace_run_ignores_optional_repo_failure_in_aggregate_status() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: false
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: exit 7
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--receipt",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 0);
        let stdout = strip_ansi(&output.stdout);
        assert!(stdout.contains("READY"));
        assert!(stdout.contains("web [optional] (WARN)"));
        assert!(stdout.contains("Task: setup"));
        assert!(stdout.contains("Exit code: 7"));
        assert!(stdout.contains("Steps:"));
        assert!(stdout.contains("Summary"));
    }

    #[test]
    fn workspace_run_passes_named_inputs_to_task() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
tasks:
  setup:
    inputs:
      base_url:
        required: true
    script: |
      printf '%s' "$OTA_INPUT_BASE_URL" > version.txt
"#,
        )
        .unwrap();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            fixture.path(),
            "--base-url",
            "http://localhost:8080",
        ]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(
                fixture
                    .dir
                    .path()
                    .join("apps")
                    .join("web")
                    .join("version.txt")
            )
            .unwrap(),
            "http://localhost:8080"
        );
    }

    #[test]
    fn workspace_run_uses_workspace_policy_env_values() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.workspace_file(),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
policies:
  env:
    OTA_TEST_SHARED: workspace-policy
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
env:
  OTA_TEST_SHARED:
    required: true
tasks:
  setup:
    script: |
      printf '%s' "$OTA_TEST_SHARED" > shared.txt
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            fs::read_to_string(
                fixture
                    .dir
                    .path()
                    .join("apps")
                    .join("web")
                    .join("shared.txt")
            )
            .unwrap(),
            "workspace-policy"
        );

        let run_json = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);
        assert_eq!(run_json.exit_code, 0);
        let run_body: Value = serde_json::from_str(&run_json.stdout).unwrap();
        assert_eq!(
            run_body["receipt"]["env_sources"][0]["source"],
            "workspace policy"
        );

        let up_json = run_with(["ota", "workspace", "up", "--json", fixture.path()]);
        assert_eq!(up_json.exit_code, 0);
        let up_body: Value = serde_json::from_str(&up_json.stdout).unwrap();
        assert_eq!(
            up_body["receipt"]["env_sources"][0]["source"],
            "workspace policy"
        );
    }

    #[test]
    fn workspace_list_reports_workspace_policy_env_sources() {
        let fixture = WorkspaceFixture::new();
        fs::write(
            fixture.workspace_file(),
            r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
policies:
  env:
    OTA_TEST_SHARED: workspace-policy
"#,
        )
        .unwrap();
        fs::write(
            fixture.dir.path().join("apps").join("web").join("ota.yaml"),
            r#"
version: 1
project:
  name: web
env:
  OTA_TEST_SHARED:
    required: true
execution:
  preferred: native
  supported:
    - native
  lifecycle: persistent
tasks:
  setup:
    run: printf ok
"#,
        )
        .unwrap();

        let text = run_with(["ota", "workspace", "list", fixture.path()]);
        assert_eq!(text.exit_code, 0);
        assert!(text.stdout.contains("Execution"));
        assert!(text.stdout.contains("Env precedence:"));
        assert!(
            text.stdout
                .contains("Env: `OTA_TEST_SHARED` (workspace policy, required)")
        );

        let json = run_with(["ota", "workspace", "list", "--json", fixture.path()]);
        assert_eq!(json.exit_code, 0);
        let body: Value = serde_json::from_str(&json.stdout).unwrap();
        assert_eq!(
            body["repos"][0]["execution"]["env"][0]["policy"],
            "workspace-policy"
        );
        assert_eq!(
            body["repos"][0]["execution"]["env"][0]["source"],
            "workspace policy"
        );
    }

    #[cfg(unix)]
    #[test]
    fn workspace_run_respects_repo_dependency_order() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let fixture = WorkspaceFixture::new_multi_repo();
        let _ssh = setup_fake_ssh(fixture.dir.path());

        let output = run_with(["ota", "workspace", "run", "setup", fixture.path()]);

        assert_eq!(output.exit_code, 0);
        let marker = fs::read_to_string(
            fixture
                .dir
                .path()
                .join("services")
                .join("api")
                .join("workspace-order.txt"),
        )
        .unwrap();
        assert_eq!(marker, "db\napi\n");
    }

    #[test]
    fn workspace_run_blocks_dependent_repo_when_dependency_fails() {
        let fixture = WorkspaceFixture::new_multi_repo();
        fs::write(
            fixture
                .dir
                .path()
                .join("services")
                .join("db")
                .join("ota.yaml"),
            r#"
version: 1
project:
  name: db
tasks:
  setup:
    run: exit 5
"#,
        )
        .unwrap();

        let output = run_with(["ota", "workspace", "run", "setup", "--json", fixture.path()]);

        assert_eq!(output.exit_code, 1);
        let json: Value = serde_json::from_str(&output.stdout).unwrap();
        assert_eq!(json["ok"], false);
        assert_eq!(json["repos"][0]["name"], "db");
        assert_eq!(json["repos"][0]["status"], "TASK FAILED");
        assert_eq!(json["repos"][1]["name"], "api");
        assert_eq!(json["repos"][1]["status"], "BLOCKED");
        assert_eq!(json["repos"][1]["task"], "setup");
        assert_eq!(
            json["repos"][1]["findings"][0]["summary"],
            "Blocked by failed dependency: db"
        );
    }

    #[test]
    fn workspace_run_rejects_zero_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--jobs",
            "0",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--jobs` must be greater than zero"
        );
    }

    #[test]
    fn workspace_run_rejects_stream_with_json() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--json",
            "--stream",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--stream` is only supported for text output"
        );
    }

    #[test]
    fn workspace_run_rejects_stream_with_parallel_jobs() {
        let fixture = WorkspaceFixture::new();

        let output = run_with([
            "ota",
            "workspace",
            "run",
            "setup",
            "--stream",
            "--jobs",
            "2",
            fixture.path(),
        ]);

        assert_eq!(output.exit_code, 2);
        assert_eq!(
            strip_ansi(output.stderr.as_deref().unwrap_or_default()),
            "`--stream` currently requires `--jobs 1`"
        );
    }

    struct ContractFixture {
        dir: TempDir,
        file_path: std::path::PathBuf,
    }

    impl ContractFixture {
        fn new(contents: &str) -> Self {
            let dir = TempDir::new().unwrap();
            let file_path = dir.path().join("ota.yaml");
            fs::write(&file_path, contents.trim_start()).unwrap();

            Self { dir, file_path }
        }

        fn new_dir() -> Self {
            let dir = TempDir::new().unwrap();
            let file_path = dir.path().join("ota.yaml");
            Self { dir, file_path }
        }

        fn path(&self) -> &str {
            self.dir.path().to_str().unwrap()
        }

        fn file_path(&self) -> &std::path::Path {
            &self.file_path
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.dir.path().join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
    }

    struct WorkspaceFixture {
        dir: TempDir,
        workspace_file: std::path::PathBuf,
    }

    impl WorkspaceFixture {
        fn new() -> Self {
            let dir = TempDir::new().unwrap();
            let repo_dir = dir.path().join("apps").join("web");
            fs::create_dir_all(&repo_dir).unwrap();
            fs::write(
                repo_dir.join("ota.yaml"),
                r#"
version: 1
project:
  name: web
"#,
            )
            .unwrap();

            let workspace_file = dir.path().join("ota.workspace.yaml");
            fs::write(
                &workspace_file,
                r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
"#,
            )
            .unwrap();

            Self {
                dir,
                workspace_file,
            }
        }

        fn path(&self) -> &str {
            self.dir.path().to_str().unwrap()
        }

        fn workspace_file(&self) -> &std::path::Path {
            &self.workspace_file
        }

        fn new_multi_repo() -> Self {
            let dir = TempDir::new().unwrap();
            let api_dir = dir.path().join("services").join("api");
            let db_dir = dir.path().join("services").join("db");
            fs::create_dir_all(&api_dir).unwrap();
            fs::create_dir_all(&db_dir).unwrap();

            fs::write(
                db_dir.join("ota.yaml"),
                format!(
                    r#"
version: 1
project:
  name: db
tasks:
  setup:
    run: printf "db\n" > "{marker}"
"#,
                    marker = api_dir.join("workspace-order.txt").display()
                ),
            )
            .unwrap();

            fs::write(
                api_dir.join("ota.yaml"),
                format!(
                    r#"
version: 1
project:
  name: api
execution:
  preferred: remote
  supported:
    - remote
  lifecycle: ephemeral
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: {}
tasks:
  setup:
    run: printf "api\n" >> "{marker}"
"#,
                    api_dir.display(),
                    marker = api_dir.join("workspace-order.txt").display()
                ),
            )
            .unwrap();

            let workspace_file = dir.path().join("ota.workspace.yaml");
            fs::write(
                &workspace_file,
                r#"
version: 1
workspace:
  name: ota-stack
repos:
  api:
    path: services/api
    required: true
    depends_on:
      - db
  db:
    path: services/db
    required: true
"#,
            )
            .unwrap();

            Self {
                dir,
                workspace_file,
            }
        }

        #[cfg(unix)]
        fn new_parallel_repos() -> Self {
            let dir = TempDir::new().unwrap();
            let one_dir = dir.path().join("apps").join("one");
            let two_dir = dir.path().join("apps").join("two");
            fs::create_dir_all(&one_dir).unwrap();
            fs::create_dir_all(&two_dir).unwrap();

            fs::write(
                one_dir.join("ota.yaml"),
                r#"
version: 1
project:
  name: one
tasks:
  setup:
    script: |
      sleep 1
      printf one-out
      printf one-err >&2
"#,
            )
            .unwrap();

            fs::write(
                two_dir.join("ota.yaml"),
                r#"
version: 1
project:
  name: two
tasks:
  setup:
    script: |
      sleep 1
      printf two-out
      printf two-err >&2
"#,
            )
            .unwrap();

            let workspace_file = dir.path().join("ota.workspace.yaml");
            fs::write(
                &workspace_file,
                r#"
version: 1
workspace:
  name: ota-parallel
repos:
  one:
    path: apps/one
    required: true
  two:
    path: apps/two
    required: true
"#,
            )
            .unwrap();

            Self {
                dir,
                workspace_file,
            }
        }
    }

    #[cfg(unix)]
    fn init_git_repo(contract: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        run_git(dir.path(), &["init"]);
        run_git(dir.path(), &["config", "user.email", "ota@example.com"]);
        run_git(dir.path(), &["config", "user.name", "Ota Tests"]);
        fs::write(dir.path().join("ota.yaml"), contract.trim_start()).unwrap();
        run_git(dir.path(), &["add", "ota.yaml"]);
        run_git(dir.path(), &["commit", "-m", "initial"]);
        dir
    }

    #[cfg(unix)]
    fn init_named_git_repo(
        root: &std::path::Path,
        name: &str,
        contract: &str,
    ) -> std::path::PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        run_git(&dir, &["init"]);
        run_git(&dir, &["config", "user.email", "ota@example.com"]);
        run_git(&dir, &["config", "user.name", "Ota Tests"]);
        fs::write(dir.join("ota.yaml"), contract.trim_start()).unwrap();
        run_git(&dir, &["add", "ota.yaml"]);
        run_git(&dir, &["commit", "-m", "initial"]);
        dir
    }

    #[cfg(unix)]
    fn run_git(cwd: &std::path::Path, args: &[&str]) {
        let output = Command::new("git")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git command failed: git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn assert_json_top_level_keys(output: &super::CommandOutput, expected: &[&str]) {
        let actual = json_top_level_keys(output);
        let mut expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>();
        expected.sort();
        assert_eq!(actual, expected);
    }

    fn json_top_level_keys(output: &super::CommandOutput) -> Vec<String> {
        let json = parse_json_from_output(output);
        let object = json.as_object().expect("json output should be an object");
        let mut actual = object.keys().cloned().collect::<Vec<_>>();
        actual.sort();
        actual
    }

    fn json_top_level_keys_named(name: &str, output: &super::CommandOutput) -> Vec<String> {
        let payload = if output.stdout.trim().is_empty() {
            output
                .stderr
                .as_deref()
                .expect("json payload should be present in stderr when stdout is empty")
        } else {
            output.stdout.as_str()
        };
        let json: Value = serde_json::from_str(payload).unwrap_or_else(|error| {
            panic!(
                "{name} produced non-json payload: stdout={:?} stderr={:?} error={error}",
                output.stdout, output.stderr
            )
        });
        let object = json.as_object().expect("json output should be an object");
        let mut actual = object.keys().cloned().collect::<Vec<_>>();
        actual.sort();
        actual
    }

    fn parse_json_from_output(output: &super::CommandOutput) -> Value {
        let payload = if output.stdout.trim().is_empty() {
            output
                .stderr
                .as_deref()
                .expect("json payload should be present in stderr when stdout is empty")
        } else {
            output.stdout.as_str()
        };
        serde_json::from_str(payload).expect("output should be valid json")
    }
}
