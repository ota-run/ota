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
    #[arg(long, requires_all = ["provider_readback", "expected_builder_artifact_identity"])]
    verify_installed: bool,
    #[arg(long, requires = "verify_installed")]
    provider_readback: Option<PathBuf>,
    #[arg(long, requires = "verify_installed")]
    expected_builder_artifact_identity: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let result = if let Some(provider_readback) = cli.provider_readback.as_deref() {
        ota::secret_delivery_pressure_fixture::verify_installed_authority_payload(
            &cli.request,
            provider_readback,
            &cli.verifier_key_identity,
            &cli.verifier_identity,
            &cli.expected_core_source_revision,
            &cli.implementation_build_identity,
            &cli.implementation_artifact_identity,
            cli.expected_builder_artifact_identity
                .as_deref()
                .expect("Clap requires builder identity"),
        )
    } else {
        ota::secret_delivery_pressure_fixture::render_authority_payload(
            &cli.request,
            &cli.verifier_key_identity,
            &cli.verifier_identity,
            &cli.expected_core_source_revision,
            &cli.implementation_build_identity,
            &cli.implementation_artifact_identity,
        )
    };
    match result {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_mode_never_falls_back_to_rendering_for_missing_inputs() {
        let base = [
            "helper",
            "--request",
            "/etc/ota/secret-delivery-pressure-request.json",
            "--verifier-key-identity",
            "key",
            "--verifier-identity",
            "verifier",
            "--expected-core-source-revision",
            "core",
            "--implementation-build-identity",
            "build",
            "--implementation-artifact-identity",
            "artifact",
        ];
        assert!(!Cli::try_parse_from(base).unwrap().verify_installed);
        for extra in [
            vec!["--verify-installed"],
            vec![
                "--provider-readback",
                "/etc/ota/secret-delivery-provider-readback.json",
            ],
            vec![
                "--verify-installed",
                "--provider-readback",
                "/etc/ota/secret-delivery-provider-readback.json",
            ],
            vec!["--expected-builder-artifact-identity", "builder"],
        ] {
            assert!(Cli::try_parse_from(base.into_iter().chain(extra)).is_err());
        }
        let complete = base.into_iter().chain([
            "--verify-installed",
            "--provider-readback",
            "/etc/ota/secret-delivery-provider-readback.json",
            "--expected-builder-artifact-identity",
            "builder",
        ]);
        assert!(Cli::try_parse_from(complete).unwrap().verify_installed);
    }
}
