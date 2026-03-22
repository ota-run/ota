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
