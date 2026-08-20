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

pub(crate) mod adapter_inputs;
pub(crate) mod agent_boundary_docs;
pub(crate) mod broker_session;
pub(crate) mod candidate_closure;
pub(crate) mod capabilities;
pub(crate) mod ci_projection;
pub mod claim_assurance;
pub mod cli;
pub(crate) mod contract_candidate;
pub(crate) mod contract_drift;
pub(crate) mod crossing;
pub(crate) mod crossing_authority;
pub(crate) mod crossing_transaction;
pub mod detector;
pub mod doctor;
pub(crate) mod execution;
pub(crate) mod execution_boundary;
pub(crate) mod github_projection;
pub(crate) mod hydration_provenance;
pub(crate) mod jsonc;
pub mod output;
pub mod parser;
pub mod policy_pack;
pub(crate) mod protected_history;
pub mod provisioning;
pub mod published_contract_schemas;
pub mod published_docs_manifest;
pub(crate) mod replay_baseline;
pub(crate) mod replay_input_policy;
pub(crate) mod replay_inputs;
pub mod runner;
pub(crate) mod sandbox_policy;
pub mod schema;
pub(crate) mod semantic_identity;
pub(crate) mod terminal;
#[cfg(test)]
pub mod test_support;
pub(crate) mod toolchains;
pub mod update;
pub mod validator;
pub mod workspace;
