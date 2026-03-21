use std::fs;
use std::path::Path;

use crate::schema::Contract;

#[derive(Debug, thiserror::Error)]
pub enum LoadContractError {
    #[error("failed to read contract `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse contract `{path}`: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
}

pub fn load_contract(path: &Path) -> Result<Contract, LoadContractError> {
    let contents = fs::read_to_string(path).map_err(|source| LoadContractError::Read {
        path: path.display().to_string(),
        source,
    })?;

    parse_contract_str(path, &contents)
}

pub fn parse_contract_str(path: &Path, contents: &str) -> Result<Contract, LoadContractError> {
    serde_yaml::from_str(contents).map_err(|source| LoadContractError::Parse {
        path: path.display().to_string(),
        source,
    })
}
