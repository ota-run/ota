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

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "ota-secret-delivery-pressure-authority")]
struct Cli {
    #[arg(long)]
    request: PathBuf,
    #[arg(long)]
    verifier_key_identity: String,
    #[arg(long)]
    verifier_identity: String,
    #[arg(long)]
    expected_core_source_revision: String,
    #[arg(long)]
    implementation_build_identity: String,
    #[arg(long)]
    implementation_artifact_identity: String,
}

fn main() {
    let cli = Cli::parse();
    match ota::secret_delivery_pressure_fixture::render_authority_payload(
        &cli.request,
        &cli.verifier_key_identity,
        &cli.verifier_identity,
        &cli.expected_core_source_revision,
        &cli.implementation_build_identity,
        &cli.implementation_artifact_identity,
    ) {
        Ok(payload) => {
            use std::io::Write;
            if std::io::stdout().write_all(&payload).is_err() {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("ota-secret-delivery-pressure-authority: {error}");
            std::process::exit(1);
        }
    }
}
