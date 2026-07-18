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
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

use serde::Serialize;
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

pub(crate) fn normalized_contract_snapshot_json<T: Serialize>(
    value: &T,
) -> Result<Vec<u8>, String> {
    let normalized = serde_json::to_value(value)
        .map_err(|error| format!("failed to normalize contract snapshot value: {error}"))?;
    serde_json::to_vec_pretty(&normalize_semantic_json(normalized))
        .map_err(|error| format!("failed to serialize normalized contract snapshot: {error}"))
}

pub(crate) fn semantic_contract_identity<T: Serialize>(value: &T) -> Result<String, String> {
    normalized_contract_snapshot_json(value).map(|snapshot| contract_snapshot_hash(&snapshot))
}

pub(crate) fn contract_snapshot_hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub(crate) fn normalize_semantic_json(value: JsonValue) -> JsonValue {
    prune_semantic_json("", value).unwrap_or_else(|| JsonValue::Object(Default::default()))
}

fn prune_semantic_json(path: &str, value: JsonValue) -> Option<JsonValue> {
    match value {
        JsonValue::Null => None,
        JsonValue::Bool(false) => None,
        JsonValue::String(value) if value.trim().is_empty() => None,
        JsonValue::Array(values) => {
            let values = values
                .into_iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    prune_semantic_json(&format!("{path}[{index}]"), value)
                })
                .collect::<Vec<_>>();
            (!values.is_empty()).then_some(JsonValue::Array(values))
        }
        JsonValue::Object(map) => {
            let mut normalized = serde_json::Map::new();
            for (key, value) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                if let Some(value) = prune_semantic_json(&child_path, value) {
                    normalized.insert(key, value);
                }
            }
            (!normalized.is_empty()).then_some(JsonValue::Object(normalized))
        }
        other => Some(other),
    }
}
