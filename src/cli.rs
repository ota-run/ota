use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::parser::{LoadContractError, load_contract};
use crate::validator::{ValidationErrors, validate_contract};

const DEFAULT_CONTRACT_FILE: &str = "ota.yaml";

#[derive(Debug, Parser)]
#[command(name = "ota")]
#[command(about = "Open repo readiness CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Validate an Ota contract.
    Validate {
        /// Path to an ota.yaml file or a directory containing one.
        path: Option<PathBuf>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Load(#[from] LoadContractError),
    #[error(transparent)]
    Validation(#[from] ValidationErrors),
}

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path } => validate(path.as_deref()),
    }
}

fn validate(path: Option<&Path>) -> Result<(), CliError> {
    let resolved_path = resolve_contract_path(path);
    let contract = load_contract(&resolved_path)?;
    validate_contract(&contract)?;

    println!("VALID {}", resolved_path.display());
    Ok(())
}

fn resolve_contract_path(path: Option<&Path>) -> PathBuf {
    match path {
        Some(path) if path.is_dir() => path.join(DEFAULT_CONTRACT_FILE),
        Some(path) => path.to_path_buf(),
        None => PathBuf::from(DEFAULT_CONTRACT_FILE),
    }
}
