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
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

use sha2::{Digest, Sha256};

use crate::schema::TaskReplayInputSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplayInputIdentityEvaluation {
    Match {
        expected_identity: String,
        observed_identity: String,
    },
    Mismatch {
        expected_identity: String,
        observed_identity: String,
    },
    Missing {
        expected_identity: String,
        error: String,
    },
}

pub(crate) fn sha256_identity(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub(crate) fn evaluate_replay_input_identity(
    input: &TaskReplayInputSpec,
    observed: Result<String, String>,
) -> Option<ReplayInputIdentityEvaluation> {
    let expected_identity = input.expected_identity.clone()?;
    Some(match observed {
        Ok(observed_identity) if observed_identity == expected_identity => {
            ReplayInputIdentityEvaluation::Match {
                expected_identity,
                observed_identity,
            }
        }
        Ok(observed_identity) => ReplayInputIdentityEvaluation::Mismatch {
            expected_identity,
            observed_identity,
        },
        Err(error) => ReplayInputIdentityEvaluation::Missing {
            expected_identity,
            error,
        },
    })
}
