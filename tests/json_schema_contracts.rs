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

use serde_json::Value;

fn load_schema(path: &str) -> Value {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let contents = fs::read_to_string(&schema_path).expect("schema file should be readable");
    serde_json::from_str(&contents).expect("schema file should be valid JSON")
}

#[test]
fn tasks_schema_includes_agent_and_variant_fields() {
    let schema = load_schema("docs/spec/json-schemas/tasks.json");
    let success = &schema["oneOf"][0]["properties"];
    let task_properties = &success["tasks"]["items"]["properties"];

    assert!(success.get("agent").is_some());
    assert!(task_properties.get("selected_variant_os").is_some());
    assert!(task_properties.get("variants").is_some());
}

#[test]
fn doctor_schema_includes_agent_summary() {
    let schema = load_schema("docs/spec/json-schemas/doctor.json");
    let properties = &schema["properties"];

    assert!(properties.get("agent").is_some());
    assert!(properties.get("findings").is_some());
}

#[test]
fn detect_schema_includes_comparison_preview() {
    let schema = load_schema("docs/spec/json-schemas/detect.json");
    let success = &schema["oneOf"][0]["properties"];

    assert!(success.get("comparison").is_some());
    assert!(success.get("config").is_some());
    assert!(success.get("inferred").is_some());
}
