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

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use cargo_platform::{Cfg, Platform};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const TRANSPORT_TARGET: &str = "x86_64-unknown-linux-gnu";
const TRANSPORT_FEATURE: &str = "secret-delivery-pressure";
const ROOT_DEPENDENCY_ALIAS: &str = "ureq";
const NODE_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependency-node.v1\0";
const EDGE_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependency-edge.v1\0";
const GRAPH_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependency-feature-graph.v1\0";
const LOCK_DOMAIN: &[u8] = b"ota.secret-delivery-transport-cargo-lock.v1\0";
const RECORD_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependencies.v1\0";

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    resolve: CargoResolve,
}

#[derive(Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
}

#[derive(Deserialize)]
struct CargoResolve {
    root: Option<String>,
    nodes: Vec<CargoNode>,
}

#[derive(Deserialize)]
struct CargoNode {
    id: String,
    deps: Vec<CargoDependency>,
    features: Vec<String>,
}

#[derive(Deserialize)]
struct CargoDependency {
    name: String,
    pkg: String,
    dep_kinds: Vec<CargoDependencyKind>,
}

#[derive(Deserialize)]
struct CargoDependencyKind {
    kind: Option<String>,
    target: Option<String>,
}

#[derive(Clone, Serialize)]
struct TransportDependencyNode {
    schema_version: u32,
    kind: &'static str,
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    enabled_features: Vec<String>,
    node_identity: String,
}

#[derive(Clone, Serialize)]
struct TransportDependencyEdge {
    schema_version: u32,
    kind: &'static str,
    from_node_identity: String,
    to_node_identity: String,
    dependency_alias: String,
    dependency_kind: String,
    target_expression: Option<String>,
    edge_identity: String,
}

#[derive(Serialize)]
struct TransportDependencyFeatureGraph {
    schema_version: u32,
    kind: &'static str,
    target_triple: &'static str,
    target_cfg: Vec<String>,
    root_package_name: String,
    root_package_version: String,
    selected_core_features: Vec<String>,
    root_dependency_alias: &'static str,
    root_dependency_node_identity: String,
    nodes: Vec<TransportDependencyNode>,
    edges: Vec<TransportDependencyEdge>,
    feature_graph_identity: String,
}

#[derive(Serialize)]
struct TransportDependencyRecord {
    schema_version: u32,
    kind: &'static str,
    cargo_lock_identity: String,
    feature_graph_identity: String,
    target_triple: &'static str,
    root_package_name: String,
    root_package_version: String,
    selected_core_features: Vec<String>,
    record_identity: String,
}

#[derive(Serialize)]
struct EmbeddedTransportDependencies {
    schema_version: u32,
    kind: &'static str,
    feature_graph: TransportDependencyFeatureGraph,
    record: TransportDependencyRecord,
}

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/packed-refs");

    if let Some(head_ref) = git_output(["symbolic-ref", "-q", "HEAD"]) {
        let git_ref_path = Path::new(".git").join(head_ref.trim());
        println!("cargo:rerun-if-changed={}", git_ref_path.display());
    }

    if let Some(commit) = git_output(["rev-parse", "HEAD"]) {
        println!("cargo:rustc-env=OTA_BUILD_SOURCE=1");
        println!("cargo:rustc-env=OTA_BUILD_COMMIT={}", commit.trim());
    }

    if git_is_dirty() {
        println!("cargo:rustc-env=OTA_BUILD_DIRTY=1");
    }

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    write_transport_dependency_record();
}

fn write_transport_dependency_record() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let metadata = cargo_metadata(&manifest_dir);
    let target_cfg = rustc_target_cfg();
    let feature_graph =
        derive_transport_dependency_feature_graph(&metadata, &target_cfg, &manifest_dir);
    let cargo_lock = fs::read(manifest_dir.join("Cargo.lock")).expect("read Cargo.lock");
    let root_package_name = feature_graph.root_package_name.clone();
    let root_package_version = feature_graph.root_package_version.clone();
    let selected_core_features = feature_graph.selected_core_features.clone();
    let mut record = TransportDependencyRecord {
        schema_version: 1,
        kind: "secret_delivery_transport_dependencies",
        cargo_lock_identity: bytes_identity(LOCK_DOMAIN, &cargo_lock),
        feature_graph_identity: feature_graph.feature_graph_identity.clone(),
        target_triple: TRANSPORT_TARGET,
        root_package_name,
        root_package_version,
        selected_core_features,
        record_identity: String::new(),
    };
    record.record_identity = canonical_identity(RECORD_DOMAIN, &record, "record_identity");
    let embedded = EmbeddedTransportDependencies {
        schema_version: 1,
        kind: "secret_delivery_transport_dependencies_embedded",
        feature_graph,
        record,
    };
    let output = serde_json::to_vec(&embedded).expect("serialize transport dependency record");
    let path = PathBuf::from(env::var("OUT_DIR").expect("output directory"))
        .join("secret_delivery_transport_dependencies.json");
    fs::write(path, output).expect("write transport dependency record");
}

fn cargo_metadata(manifest_dir: &Path) -> CargoMetadata {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(manifest_dir)
        .args([
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            TRANSPORT_TARGET,
            "--features",
            TRANSPORT_FEATURE,
        ])
        .output()
        .expect("run cargo metadata for transport dependency record");
    if !output.status.success() {
        panic!(
            "cargo metadata for transport dependency record failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    serde_json::from_slice(&output.stdout)
        .expect("parse cargo metadata for transport dependency record")
}

fn rustc_target_cfg() -> Vec<Cfg> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = Command::new(rustc)
        .args(["--print", "cfg", "--target", TRANSPORT_TARGET])
        .output()
        .expect("run rustc target cfg");
    if !output.status.success() {
        panic!(
            "rustc target cfg failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let value = std::str::from_utf8(line).expect("rustc target cfg UTF-8");
            Cfg::from_str(value).expect("parse rustc target cfg")
        })
        .collect()
}

fn derive_transport_dependency_feature_graph(
    metadata: &CargoMetadata,
    target_cfg: &[Cfg],
    manifest_dir: &Path,
) -> TransportDependencyFeatureGraph {
    let root_id = metadata.resolve.root.as_ref().expect("cargo metadata root");
    let root_package = metadata
        .packages
        .iter()
        .find(|package| package.id == *root_id)
        .expect("root package");
    if root_package.name != "ota" {
        panic!("transport dependency graph root must be ota");
    }
    let nodes = metadata
        .resolve
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let packages = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let checksums = cargo_lock_checksums(manifest_dir);
    let root = nodes.get(root_id.as_str()).expect("root resolve node");
    let root_dependencies = root_transport_dependencies(applicable_dependencies(root, target_cfg));

    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::from([root_dependencies[0].pkg.clone()]);
    while let Some(package_id) = queue.pop_front() {
        if !reachable.insert(package_id.clone()) {
            continue;
        }
        let node = nodes
            .get(package_id.as_str())
            .unwrap_or_else(|| panic!("missing resolve node for {package_id}"));
        for dependency in applicable_dependencies(node, target_cfg) {
            queue.push_back(dependency.pkg.clone());
        }
    }

    let mut resolved_nodes = BTreeMap::new();
    for package_id in &reachable {
        let package = packages
            .get(package_id.as_str())
            .unwrap_or_else(|| panic!("missing package metadata for {package_id}"));
        let resolve_node = nodes
            .get(package_id.as_str())
            .unwrap_or_else(|| panic!("missing resolve node for {package_id}"));
        let checksum = package.source.as_ref().and_then(|source| {
            checksums
                .get(&(
                    package.name.clone(),
                    package.version.clone(),
                    source.clone(),
                ))
                .cloned()
        });
        if package
            .source
            .as_deref()
            .is_some_and(|source| source.starts_with("registry+"))
            && checksum.is_none()
        {
            panic!(
                "missing registry checksum for {} {}",
                package.name, package.version
            );
        }
        let mut node = TransportDependencyNode {
            schema_version: 1,
            kind: "secret_delivery_transport_dependency_node",
            name: package.name.clone(),
            version: package.version.clone(),
            source: package.source.clone(),
            checksum,
            enabled_features: sorted_unique(resolve_node.features.clone(), "enabled feature"),
            node_identity: String::new(),
        };
        node.node_identity = canonical_identity(NODE_DOMAIN, &node, "node_identity");
        if resolved_nodes.insert(package_id.clone(), node).is_some() {
            panic!("duplicate transport dependency package id");
        }
    }

    let mut edges = Vec::new();
    for package_id in &reachable {
        let node = nodes
            .get(package_id.as_str())
            .unwrap_or_else(|| panic!("missing resolve node for {package_id}"));
        let from_node_identity = resolved_nodes
            .get(package_id)
            .expect("resolved transport node")
            .node_identity
            .clone();
        for dependency in applicable_dependencies(node, target_cfg) {
            if !reachable.contains(&dependency.pkg) {
                panic!("transport dependency closure has dangling edge");
            }
            let to_node_identity = resolved_nodes
                .get(&dependency.pkg)
                .expect("resolved transport dependency")
                .node_identity
                .clone();
            let mut edge = TransportDependencyEdge {
                schema_version: 1,
                kind: "secret_delivery_transport_dependency_edge",
                from_node_identity: from_node_identity.clone(),
                to_node_identity,
                dependency_alias: dependency.name.clone(),
                dependency_kind: dependency.kind.clone(),
                target_expression: dependency.target_expression.clone(),
                edge_identity: String::new(),
            };
            edge.edge_identity = canonical_identity(EDGE_DOMAIN, &edge, "edge_identity");
            edges.push(edge);
        }
    }
    edges.sort_by(|left, right| left.edge_identity.cmp(&right.edge_identity));
    if edges
        .windows(2)
        .any(|pair| pair[0].edge_identity == pair[1].edge_identity)
    {
        panic!("duplicate transport dependency edge identity");
    }
    let mut graph_nodes = resolved_nodes.into_values().collect::<Vec<_>>();
    graph_nodes.sort_by(|left, right| left.node_identity.cmp(&right.node_identity));
    if graph_nodes
        .windows(2)
        .any(|pair| pair[0].node_identity == pair[1].node_identity)
    {
        panic!("duplicate transport dependency node identity");
    }
    let root_dependency_node_identity = resolved_nodes_for_root_dependency(
        &graph_nodes,
        &root_dependencies[0].pkg,
        &reachable,
        metadata,
    );
    let mut graph = TransportDependencyFeatureGraph {
        schema_version: 1,
        kind: "secret_delivery_transport_dependency_feature_graph",
        target_triple: TRANSPORT_TARGET,
        target_cfg: sorted_unique(
            target_cfg.iter().map(ToString::to_string).collect(),
            "target cfg",
        ),
        root_package_name: root_package.name.clone(),
        root_package_version: root_package.version.clone(),
        selected_core_features: vec![TRANSPORT_FEATURE.to_string()],
        root_dependency_alias: ROOT_DEPENDENCY_ALIAS,
        root_dependency_node_identity,
        nodes: graph_nodes,
        edges,
        feature_graph_identity: String::new(),
    };
    graph.feature_graph_identity =
        canonical_identity(GRAPH_DOMAIN, &graph, "feature_graph_identity");
    graph
}

fn resolved_nodes_for_root_dependency(
    nodes: &[TransportDependencyNode],
    root_dependency_id: &str,
    reachable: &BTreeSet<String>,
    metadata: &CargoMetadata,
) -> String {
    if !reachable.contains(root_dependency_id) {
        panic!("root transport dependency is not reachable");
    }
    let package = metadata
        .packages
        .iter()
        .find(|package| package.id == root_dependency_id)
        .expect("root transport dependency package");
    let matching = nodes
        .iter()
        .filter(|node| {
            node.name == package.name
                && node.version == package.version
                && node.source == package.source
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        panic!("root transport dependency node identity is ambiguous");
    }
    matching[0].node_identity.clone()
}

struct ApplicableDependency {
    name: String,
    pkg: String,
    kind: String,
    target_expression: Option<String>,
}

fn applicable_dependencies(node: &CargoNode, target_cfg: &[Cfg]) -> Vec<ApplicableDependency> {
    let mut result = Vec::new();
    for dependency in &node.deps {
        for dep_kind in &dependency.dep_kinds {
            let kind = match dep_kind.kind.as_deref() {
                None => "normal",
                Some("build") => "build",
                Some("dev") => continue,
                Some(other) => panic!("unsupported Cargo dependency kind {other}"),
            };
            let target_expression = dep_kind
                .target
                .as_ref()
                .map(|target| canonical_target_expression(target, target_cfg));
            result.push(ApplicableDependency {
                name: dependency.name.clone(),
                pkg: dependency.pkg.clone(),
                kind: kind.to_string(),
                target_expression,
            });
        }
    }
    result.sort_by(|left, right| {
        (&left.name, &left.pkg, &left.kind, &left.target_expression).cmp(&(
            &right.name,
            &right.pkg,
            &right.kind,
            &right.target_expression,
        ))
    });
    result
}

fn root_transport_dependencies(
    dependencies: Vec<ApplicableDependency>,
) -> Vec<ApplicableDependency> {
    let dependencies = dependencies
        .into_iter()
        .filter(|dependency| {
            dependency.name == ROOT_DEPENDENCY_ALIAS && dependency.kind == "normal"
        })
        .collect::<Vec<_>>();
    if dependencies.len() != 1 {
        panic!("root must have one applicable normal ureq dependency");
    }
    dependencies
}

fn canonical_target_expression(target: &str, target_cfg: &[Cfg]) -> String {
    let platform = Platform::from_str(target)
        .unwrap_or_else(|_| panic!("invalid Cargo target expression {target}"));
    if !platform.matches(TRANSPORT_TARGET, target_cfg) {
        panic!("inapplicable Cargo target expression {target}");
    }
    let canonical = platform.to_string();
    if canonical != target {
        panic!("non-canonical Cargo target expression {target}");
    }
    canonical
}

fn cargo_lock_checksums(manifest_dir: &Path) -> BTreeMap<(String, String, String), String> {
    let lock = fs::read_to_string(manifest_dir.join("Cargo.lock")).expect("read Cargo.lock text");
    let lock = toml::from_str::<toml::Value>(&lock).expect("parse Cargo.lock");
    lock.get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock package table")
        .iter()
        .filter_map(|package| {
            let table = package.as_table()?;
            Some((
                table.get("name")?.as_str()?.to_string(),
                table.get("version")?.as_str()?.to_string(),
                table.get("source")?.as_str()?.to_string(),
                table.get("checksum")?.as_str()?.to_string(),
            ))
        })
        .map(|(name, version, source, checksum)| ((name, version, source), checksum))
        .collect()
}

fn sorted_unique(values: Vec<String>, label: &str) -> Vec<String> {
    let mut values = values;
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        panic!("duplicate {label}");
    }
    values
}

fn canonical_identity<T: Serialize>(domain: &[u8], value: &T, identity_field: &str) -> String {
    let mut value = serde_json::to_value(value).expect("identity value");
    value
        .as_object_mut()
        .expect("identity record object")
        .remove(identity_field)
        .expect("identity field");
    domain_identity(domain, &value)
}

fn domain_identity<T: Serialize>(domain: &[u8], value: &T) -> String {
    let canonical = serde_jcs::to_vec(value).expect("canonical identity");
    let mut bytes = Vec::with_capacity(domain.len() + canonical.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(&canonical);
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn bytes_identity(domain: &[u8], bytes: &[u8]) -> String {
    let mut identity_bytes = Vec::with_capacity(domain.len() + bytes.len());
    identity_bytes.extend_from_slice(domain);
    identity_bytes.extend_from_slice(bytes);
    format!("sha256:{:x}", Sha256::digest(identity_bytes))
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn git_is_dirty() -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .is_some_and(|output| {
            output.status.success() && porcelain_has_source_changes(&output.stdout)
        })
}

fn porcelain_has_source_changes(output: &[u8]) -> bool {
    output
        .split(|byte| *byte == b'\n')
        .any(|line| !line.is_empty() && line != b"?? .cargo-ok")
}

#[cfg(test)]
mod tests {
    use super::{
        ApplicableDependency, Cfg, canonical_target_expression, porcelain_has_source_changes,
        root_transport_dependencies,
    };
    use std::str::FromStr;

    #[test]
    fn cargo_checkout_marker_does_not_dirty_source_identity() {
        assert!(!porcelain_has_source_changes(b"?? .cargo-ok\n"));
        assert!(porcelain_has_source_changes(
            b"?? .cargo-ok\n M src/main.rs\n"
        ));
        assert!(porcelain_has_source_changes(b"?? src/new.rs\n"));
    }

    #[test]
    fn transport_root_requires_one_normal_ureq_edge() {
        let build_only = vec![ApplicableDependency {
            name: "ureq".into(),
            pkg: "ureq 3.4.2 (registry+example)".into(),
            kind: "build".into(),
            target_expression: None,
        }];
        assert!(std::panic::catch_unwind(|| root_transport_dependencies(build_only)).is_err());
    }

    #[test]
    fn transport_target_expression_requires_canonical_round_trip() {
        let cfg = vec![Cfg::from_str("unix").expect("cfg")];
        assert_eq!(canonical_target_expression("cfg(unix)", &cfg), "cfg(unix)");
        assert!(
            std::panic::catch_unwind(|| { canonical_target_expression("cfg( unix )", &cfg) })
                .is_err()
        );
    }
}
