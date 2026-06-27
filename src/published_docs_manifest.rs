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
use std::path::{Path, PathBuf};

use serde_json::Value as JsonValue;

pub struct PublishedDocManifest {
    pub filename: &'static str,
    pub body: &'static str,
}

const CANONICAL_DOCS_JSON: &str = r####"{
  "version": 1,
  "docs": [
    {
      "id": "contract-reference",
      "title": "ota Contract Reference",
      "kind": "reference",
      "source_path": "docs/spec/contract-reference.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/contract-reference.md",
      "public_url": "https://ota.run/docs/reference/contract",
      "canonical_for": [
        "ota.yaml field semantics",
        "repo contract validation truth"
      ]
    },
    {
      "id": "workspace-reference",
      "title": "ota Workspace Reference",
      "kind": "reference",
      "source_path": "docs/spec/workspace-reference.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/workspace-reference.md",
      "public_url": "https://ota.run/docs/reference/workspace",
      "canonical_for": [
        "ota.workspace.yaml field semantics",
        "workspace contract validation truth"
      ]
    },
    {
      "id": "command-reference",
      "title": "ota Command Reference",
      "kind": "reference",
      "source_path": "docs/spec/command-reference.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/command-reference.md",
      "public_url": "https://ota.run/docs/reference/command",
      "canonical_for": [
        "shipped CLI command behavior",
        "repo-level command usage semantics"
      ]
    },
    {
      "id": "json-output-reference",
      "title": "ota JSON Output Reference",
      "kind": "reference",
      "source_path": "docs/spec/json-output-reference.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/json-output-reference.md",
      "public_url": "https://ota.run/docs/reference/json-output",
      "canonical_for": [
        "machine-readable command output semantics",
        "JSON output contract guidance"
      ]
    },
    {
      "id": "execution-governance-loop",
      "title": "ota Execution Governance Loop",
      "kind": "reference",
      "source_path": "docs/spec/execution-governance-loop.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/execution-governance-loop.md",
      "public_url": "https://ota.run/docs/reference/execution-governance-loop",
      "canonical_for": [
        "how contract, execution, proof, diff, and policy fit together",
        "public execution-governance architecture guidance"
      ]
    },
    {
      "id": "execution-topology",
      "title": "ota Execution Topology",
      "kind": "reference",
      "source_path": "docs/spec/execution-topology.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/execution-topology.md",
      "public_url": "https://ota.run/docs/reference/execution-topology",
      "canonical_for": [
        "execution graph reasoning",
        "shared backend and runtime topology guidance"
      ]
    },
    {
      "id": "local-service-topology",
      "title": "ota Local Service Topology",
      "kind": "reference",
      "source_path": "docs/spec/local-service-topology.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/local-service-topology.md",
      "public_url": "https://ota.run/docs/reference/local-service-topology",
      "canonical_for": [
        "service endpoint ownership",
        "local runtime topology guidance"
      ]
    },
    {
      "id": "toolchains-runtimes-tools",
      "title": "ota Toolchains, Runtimes, Tools, and Orchestrators",
      "kind": "reference",
      "source_path": "docs/spec/toolchains-runtimes-tools.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/toolchains-runtimes-tools.md",
      "public_url": "https://ota.run/docs/reference/toolchains-runtimes-tools",
      "canonical_for": [
        "toolchain ownership semantics",
        "runtime and tool distinction guidance"
      ]
    },
    {
      "id": "doctor-finding-reference",
      "title": "ota Doctor Finding Reference",
      "kind": "reference",
      "source_path": "docs/spec/doctor-finding-reference.md",
      "source_url": "https://github.com/ota-run/ota/blob/main/docs/spec/doctor-finding-reference.md",
      "public_url": null,
      "canonical_for": [
        "stable doctor finding identity catalog",
        "machine-readable diagnosis code reference"
      ]
    }
  ]
}
"####;

pub fn published_doc_manifests() -> [PublishedDocManifest; 1] {
    [PublishedDocManifest {
        filename: "canonical-docs.json",
        body: CANONICAL_DOCS_JSON,
    }]
}

pub fn generated_doc_manifest(filename: &str) -> Option<JsonValue> {
    published_doc_manifests()
        .into_iter()
        .find(|manifest| manifest.filename == filename)
        .map(|manifest| parse_manifest(manifest.filename, manifest.body))
}

pub fn write_published_doc_manifests(manifest_dir: &Path) -> Result<Vec<PathBuf>, String> {
    fs::create_dir_all(manifest_dir).map_err(|error| {
        format!(
            "failed to create published doc manifest dir {}: {error}",
            manifest_dir.display()
        )
    })?;

    let mut written = Vec::new();
    for manifest in published_doc_manifests() {
        let path = manifest_dir.join(manifest.filename);
        fs::write(&path, manifest.body)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
        written.push(path);
    }

    Ok(written)
}

fn parse_manifest(filename: &str, body: &str) -> JsonValue {
    serde_json::from_str(body)
        .unwrap_or_else(|error| panic!("generated manifest {filename} should parse: {error}"))
}
