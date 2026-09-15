//! Build-owned transport dependency identity for the initial secret-delivery profile.
//!
//! The build script embeds the exact Cargo metadata closure and lock identity for the Linux/x86_64
//! `ureq` transport. This module rederives every local identity before Core accepts the record.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::str::FromStr;

use cargo_platform::{Cfg, Platform};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
const ROOT_PACKAGE_NAME: &str = "ota";
const ROOT_DEPENDENCY_ALIAS: &str = "ureq";
const TRANSPORT_FEATURE: &str = "secret-delivery-pressure";
const NODE_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependency-node.v1\0";
const EDGE_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependency-edge.v1\0";
const GRAPH_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependency-feature-graph.v1\0";
const LOCK_DOMAIN: &[u8] = b"ota.secret-delivery-transport-cargo-lock.v1\0";
const RECORD_DOMAIN: &[u8] = b"ota.secret-delivery-transport-dependencies.v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryTransportDependenciesError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryTransportDependencyNodeV1 {
    pub schema_version: u32,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub enabled_features: Vec<String>,
    pub node_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryTransportDependencyEdgeV1 {
    pub schema_version: u32,
    pub kind: String,
    pub from_node_identity: String,
    pub to_node_identity: String,
    pub dependency_alias: String,
    pub dependency_kind: String,
    pub target_expression: Option<String>,
    pub edge_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryTransportDependencyFeatureGraphV1 {
    pub schema_version: u32,
    pub kind: String,
    pub target_triple: String,
    pub target_cfg: Vec<String>,
    pub root_package_name: String,
    pub root_package_version: String,
    pub selected_core_features: Vec<String>,
    pub root_dependency_alias: String,
    pub root_dependency_node_identity: String,
    pub nodes: Vec<SecretDeliveryTransportDependencyNodeV1>,
    pub edges: Vec<SecretDeliveryTransportDependencyEdgeV1>,
    pub feature_graph_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SecretDeliveryTransportDependencyRecordV1 {
    pub schema_version: u32,
    pub kind: String,
    pub cargo_lock_identity: String,
    pub feature_graph_identity: String,
    pub target_triple: String,
    pub root_package_name: String,
    pub root_package_version: String,
    pub selected_core_features: Vec<String>,
    pub record_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmbeddedSecretDeliveryTransportDependenciesV1 {
    schema_version: u32,
    kind: String,
    feature_graph: SecretDeliveryTransportDependencyFeatureGraphV1,
    record: SecretDeliveryTransportDependencyRecordV1,
}

#[derive(Serialize)]
struct NodeIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    name: &'a str,
    version: &'a str,
    source: &'a Option<String>,
    checksum: &'a Option<String>,
    enabled_features: &'a [String],
}

#[derive(Serialize)]
struct EdgeIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    from_node_identity: &'a str,
    to_node_identity: &'a str,
    dependency_alias: &'a str,
    dependency_kind: &'a str,
    target_expression: &'a Option<String>,
}

#[derive(Serialize)]
struct GraphIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    target_triple: &'a str,
    target_cfg: &'a [String],
    root_package_name: &'a str,
    root_package_version: &'a str,
    selected_core_features: &'a [String],
    root_dependency_alias: &'a str,
    root_dependency_node_identity: &'a str,
    nodes: &'a [SecretDeliveryTransportDependencyNodeV1],
    edges: &'a [SecretDeliveryTransportDependencyEdgeV1],
}

#[derive(Serialize)]
struct RecordIdentityPayload<'a> {
    schema_version: u32,
    kind: &'a str,
    cargo_lock_identity: &'a str,
    feature_graph_identity: &'a str,
    target_triple: &'a str,
    root_package_name: &'a str,
    root_package_version: &'a str,
    selected_core_features: &'a [String],
}

pub(crate) fn embedded_transport_dependency_record_v1()
-> Result<SecretDeliveryTransportDependencyRecordV1, SecretDeliveryTransportDependenciesError> {
    let embedded = serde_json::from_str::<EmbeddedSecretDeliveryTransportDependenciesV1>(
        include_str!(concat!(
            env!("OUT_DIR"),
            "/secret_delivery_transport_dependencies.json"
        )),
    )
    .map_err(|_| {
        error(
            "secret_delivery_transport_dependencies_embedded_invalid",
            "embedded transport dependency record is malformed",
        )
    })?;
    if embedded.schema_version != 1
        || embedded.kind != "secret_delivery_transport_dependencies_embedded"
    {
        return Err(error(
            "secret_delivery_transport_dependencies_embedded_invalid",
            "embedded transport dependency record has an unsupported schema",
        ));
    }
    verify_transport_dependency_feature_graph_v1(&embedded.feature_graph)?;
    verify_transport_dependency_record_v1(&embedded.record, &embedded.feature_graph)?;
    Ok(embedded.record)
}

pub(crate) fn embedded_transport_dependency_record_identity_v1()
-> Result<String, SecretDeliveryTransportDependenciesError> {
    Ok(embedded_transport_dependency_record_v1()?.record_identity)
}

pub(crate) fn verify_transport_dependency_feature_graph_v1(
    graph: &SecretDeliveryTransportDependencyFeatureGraphV1,
) -> Result<(), SecretDeliveryTransportDependenciesError> {
    if graph.schema_version != 1
        || graph.kind != "secret_delivery_transport_dependency_feature_graph"
        || graph.target_triple != TARGET_TRIPLE
        || graph.root_package_name != ROOT_PACKAGE_NAME
        || graph.root_package_version != env!("CARGO_PKG_VERSION")
        || graph.selected_core_features != [TRANSPORT_FEATURE]
        || graph.root_dependency_alias != ROOT_DEPENDENCY_ALIAS
    {
        return Err(error(
            "secret_delivery_transport_dependency_graph_invalid",
            "transport dependency graph does not match the activated build profile",
        ));
    }
    let target_cfg = canonical_cfg(&graph.target_cfg)?;
    if target_cfg != graph.target_cfg {
        return Err(error(
            "secret_delivery_transport_dependency_graph_invalid",
            "transport dependency graph target cfg is not canonical",
        ));
    }
    if graph.nodes.is_empty() || !strictly_sorted(&graph.nodes, |node| &node.node_identity) {
        return Err(error(
            "secret_delivery_transport_dependency_graph_invalid",
            "transport dependency nodes are not unique canonical order",
        ));
    }
    if graph.edges.is_empty() || !strictly_sorted(&graph.edges, |edge| &edge.edge_identity) {
        return Err(error(
            "secret_delivery_transport_dependency_graph_invalid",
            "transport dependency edges are not unique canonical order",
        ));
    }
    let node_identities = graph
        .nodes
        .iter()
        .map(|node| {
            verify_node(node)?;
            Ok(node.node_identity.as_str())
        })
        .collect::<Result<BTreeSet<_>, SecretDeliveryTransportDependenciesError>>()?;
    if !node_identities.contains(graph.root_dependency_node_identity.as_str()) {
        return Err(error(
            "secret_delivery_transport_dependency_graph_invalid",
            "transport dependency graph root is missing",
        ));
    }
    let cfg = target_cfg
        .iter()
        .map(|value| {
            Cfg::from_str(value).map_err(|_| {
                error(
                    "secret_delivery_transport_dependency_graph_invalid",
                    "transport dependency graph target cfg is invalid",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for edge in &graph.edges {
        verify_edge(edge, &node_identities, &cfg)?;
    }
    let expected_identity = domain_identity(
        GRAPH_DOMAIN,
        &GraphIdentityPayload {
            schema_version: graph.schema_version,
            kind: &graph.kind,
            target_triple: &graph.target_triple,
            target_cfg: &graph.target_cfg,
            root_package_name: &graph.root_package_name,
            root_package_version: &graph.root_package_version,
            selected_core_features: &graph.selected_core_features,
            root_dependency_alias: &graph.root_dependency_alias,
            root_dependency_node_identity: &graph.root_dependency_node_identity,
            nodes: &graph.nodes,
            edges: &graph.edges,
        },
    )?;
    if graph.feature_graph_identity != expected_identity {
        return Err(error(
            "secret_delivery_transport_dependency_graph_identity_mismatch",
            "transport dependency graph identity does not match canonical graph content",
        ));
    }
    Ok(())
}

pub(crate) fn verify_transport_dependency_record_v1(
    record: &SecretDeliveryTransportDependencyRecordV1,
    graph: &SecretDeliveryTransportDependencyFeatureGraphV1,
) -> Result<(), SecretDeliveryTransportDependenciesError> {
    if record.schema_version != 1
        || record.kind != "secret_delivery_transport_dependencies"
        || record.target_triple != graph.target_triple
        || record.root_package_name != graph.root_package_name
        || record.root_package_version != graph.root_package_version
        || record.selected_core_features != graph.selected_core_features
        || record.feature_graph_identity != graph.feature_graph_identity
    {
        return Err(error(
            "secret_delivery_transport_dependency_record_invalid",
            "transport dependency record does not reconcile to its graph",
        ));
    }
    let expected_lock_identity = bytes_identity(
        LOCK_DOMAIN,
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock")),
    );
    if record.cargo_lock_identity != expected_lock_identity {
        return Err(error(
            "secret_delivery_transport_dependency_lock_mismatch",
            "transport dependency record does not bind the compiled Cargo.lock",
        ));
    }
    let expected_identity = domain_identity(
        RECORD_DOMAIN,
        &RecordIdentityPayload {
            schema_version: record.schema_version,
            kind: &record.kind,
            cargo_lock_identity: &record.cargo_lock_identity,
            feature_graph_identity: &record.feature_graph_identity,
            target_triple: &record.target_triple,
            root_package_name: &record.root_package_name,
            root_package_version: &record.root_package_version,
            selected_core_features: &record.selected_core_features,
        },
    )?;
    if record.record_identity != expected_identity {
        return Err(error(
            "secret_delivery_transport_dependency_record_identity_mismatch",
            "transport dependency record identity does not match canonical content",
        ));
    }
    Ok(())
}

fn verify_node(
    node: &SecretDeliveryTransportDependencyNodeV1,
) -> Result<(), SecretDeliveryTransportDependenciesError> {
    if node.schema_version != 1
        || node.kind != "secret_delivery_transport_dependency_node"
        || node.name.is_empty()
        || node.version.is_empty()
        || node.enabled_features != sorted_unique(node.enabled_features.clone())
        || node.source.as_deref().is_some_and(str::is_empty)
        || node
            .checksum
            .as_deref()
            .is_some_and(|checksum| !is_lower_sha256(checksum))
        || node
            .source
            .as_deref()
            .is_some_and(|source| source.starts_with("registry+"))
            && node.checksum.is_none()
    {
        return Err(error(
            "secret_delivery_transport_dependency_node_invalid",
            "transport dependency node is not canonical",
        ));
    }
    let expected_identity = domain_identity(
        NODE_DOMAIN,
        &NodeIdentityPayload {
            schema_version: node.schema_version,
            kind: &node.kind,
            name: &node.name,
            version: &node.version,
            source: &node.source,
            checksum: &node.checksum,
            enabled_features: &node.enabled_features,
        },
    )?;
    if node.node_identity != expected_identity {
        return Err(error(
            "secret_delivery_transport_dependency_node_identity_mismatch",
            "transport dependency node identity does not match canonical content",
        ));
    }
    Ok(())
}

fn verify_edge(
    edge: &SecretDeliveryTransportDependencyEdgeV1,
    node_identities: &BTreeSet<&str>,
    target_cfg: &[Cfg],
) -> Result<(), SecretDeliveryTransportDependenciesError> {
    if edge.schema_version != 1
        || edge.kind != "secret_delivery_transport_dependency_edge"
        || !node_identities.contains(edge.from_node_identity.as_str())
        || !node_identities.contains(edge.to_node_identity.as_str())
        || edge.dependency_alias.is_empty()
        || !matches!(edge.dependency_kind.as_str(), "normal" | "build")
    {
        return Err(error(
            "secret_delivery_transport_dependency_edge_invalid",
            "transport dependency edge is not canonical",
        ));
    }
    if let Some(target_expression) = &edge.target_expression {
        let platform = Platform::from_str(target_expression).map_err(|_| {
            error(
                "secret_delivery_transport_dependency_edge_invalid",
                "transport dependency target expression is invalid",
            )
        })?;
        if platform.to_string() != *target_expression
            || !platform.matches(TARGET_TRIPLE, target_cfg)
        {
            return Err(error(
                "secret_delivery_transport_dependency_edge_invalid",
                "transport dependency target expression is not applicable",
            ));
        }
    }
    let expected_identity = domain_identity(
        EDGE_DOMAIN,
        &EdgeIdentityPayload {
            schema_version: edge.schema_version,
            kind: &edge.kind,
            from_node_identity: &edge.from_node_identity,
            to_node_identity: &edge.to_node_identity,
            dependency_alias: &edge.dependency_alias,
            dependency_kind: &edge.dependency_kind,
            target_expression: &edge.target_expression,
        },
    )?;
    if edge.edge_identity != expected_identity {
        return Err(error(
            "secret_delivery_transport_dependency_edge_identity_mismatch",
            "transport dependency edge identity does not match canonical content",
        ));
    }
    Ok(())
}

fn canonical_cfg(
    values: &[String],
) -> Result<Vec<String>, SecretDeliveryTransportDependenciesError> {
    let mut canonical = values
        .iter()
        .map(|value| {
            Cfg::from_str(value)
                .map(|cfg| cfg.to_string())
                .map_err(|_| {
                    error(
                        "secret_delivery_transport_dependency_graph_invalid",
                        "transport dependency graph target cfg is invalid",
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    canonical.sort();
    if canonical.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(error(
            "secret_delivery_transport_dependency_graph_invalid",
            "transport dependency graph target cfg contains duplicates",
        ));
    }
    Ok(canonical)
}

fn strictly_sorted<T, F>(values: &[T], identity: F) -> bool
where
    F: Fn(&T) -> &String,
{
    values
        .windows(2)
        .all(|pair| identity(&pair[0]) < identity(&pair[1]))
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, SecretDeliveryTransportDependenciesError> {
    let canonical = serde_jcs::to_vec(value).map_err(|_| {
        error(
            "secret_delivery_transport_dependencies_identity_failed",
            "failed to canonicalize transport dependency record",
        )
    })?;
    Ok(bytes_identity(domain, &canonical))
}

fn bytes_identity(domain: &[u8], bytes: &[u8]) -> String {
    let mut identity_bytes = Vec::with_capacity(domain.len() + bytes.len());
    identity_bytes.extend_from_slice(domain);
    identity_bytes.extend_from_slice(bytes);
    format!("sha256:{:x}", Sha256::digest(identity_bytes))
}

fn error(
    code: &'static str,
    message: impl Into<String>,
) -> SecretDeliveryTransportDependenciesError {
    SecretDeliveryTransportDependenciesError {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_transport_dependencies_rederive_exactly() {
        let embedded = serde_json::from_str::<EmbeddedSecretDeliveryTransportDependenciesV1>(
            include_str!(concat!(
                env!("OUT_DIR"),
                "/secret_delivery_transport_dependencies.json"
            )),
        )
        .expect("embedded transport dependencies");
        verify_transport_dependency_feature_graph_v1(&embedded.feature_graph).expect("graph");
        let record = embedded_transport_dependency_record_v1().expect("embedded record");
        assert!(is_lower_sha256(
            record
                .record_identity
                .strip_prefix("sha256:")
                .expect("identity")
        ));
    }

    #[test]
    fn embedded_transport_dependencies_refuse_node_substitution() {
        let mut embedded = serde_json::from_str::<EmbeddedSecretDeliveryTransportDependenciesV1>(
            include_str!(concat!(
                env!("OUT_DIR"),
                "/secret_delivery_transport_dependencies.json"
            )),
        )
        .expect("embedded transport dependencies");
        embedded.feature_graph.nodes[0]
            .enabled_features
            .push("forged".into());
        assert_eq!(
            verify_transport_dependency_feature_graph_v1(&embedded.feature_graph)
                .unwrap_err()
                .code,
            "secret_delivery_transport_dependency_node_invalid"
        );
    }

    #[test]
    fn embedded_transport_dependencies_refuse_edge_and_record_substitution() {
        let mut embedded = serde_json::from_str::<EmbeddedSecretDeliveryTransportDependenciesV1>(
            include_str!(concat!(
                env!("OUT_DIR"),
                "/secret_delivery_transport_dependencies.json"
            )),
        )
        .expect("embedded transport dependencies");
        let graph = embedded.feature_graph.clone();
        embedded.feature_graph.edges[0].dependency_kind = "build".into();
        assert_eq!(
            verify_transport_dependency_feature_graph_v1(&embedded.feature_graph)
                .unwrap_err()
                .code,
            "secret_delivery_transport_dependency_edge_identity_mismatch"
        );

        let mut record = embedded_transport_dependency_record_v1().expect("embedded record");
        record.cargo_lock_identity =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into();
        assert_eq!(
            verify_transport_dependency_record_v1(&record, &graph)
                .unwrap_err()
                .code,
            "secret_delivery_transport_dependency_lock_mismatch"
        );
    }
}
