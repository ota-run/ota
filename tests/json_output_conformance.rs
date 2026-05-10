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
use std::process::Command;

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use tempfile::TempDir;

fn run_ota(args: &[&str], cwd: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");
    assert!(
        output.status.success(),
        "ota command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command should emit valid JSON")
}

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/spec/json-schemas")
}

fn load_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("JSON file should be readable");
    serde_json::from_str(&contents).expect("JSON file should parse")
}

fn decode_json_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

fn resolve_json_pointer<'a>(value: &'a Value, pointer: &str) -> &'a Value {
    if pointer.is_empty() {
        return value;
    }
    let mut current = value;
    for segment in pointer.trim_start_matches('/').split('/') {
        let segment = decode_json_pointer_segment(segment);
        current = match current {
            Value::Object(map) => map
                .get(segment.as_str())
                .unwrap_or_else(|| panic!("missing object pointer segment `{segment}`")),
            Value::Array(items) => {
                let index: usize = segment
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid array pointer segment `{segment}`"));
                items
                    .get(index)
                    .unwrap_or_else(|| panic!("missing array pointer index `{index}`"))
            }
            _ => panic!("cannot traverse pointer segment `{segment}`"),
        };
    }
    current
}

fn resolve_schema_refs(value: &Value, current_path: &Path) -> Value {
    match value {
        Value::Object(map) if map.len() == 1 && map.contains_key("$ref") => {
            let reference = map["$ref"].as_str().expect("$ref should be a string");
            let (file_part, pointer_part) = reference.split_once('#').unwrap_or((reference, ""));
            let target_path = if file_part.is_empty() {
                current_path.to_path_buf()
            } else {
                current_path
                    .parent()
                    .expect("schema path should have a parent")
                    .join(file_part)
            };
            let target_value = load_json(&target_path);
            let pointer = pointer_part.trim_start_matches('#');
            let referenced = resolve_json_pointer(&target_value, pointer);
            resolve_schema_refs(referenced, &target_path)
        }
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, inner)| (key.clone(), resolve_schema_refs(inner, current_path)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|inner| resolve_schema_refs(inner, current_path))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn assert_matches_schema(schema_name: &str, instance: &Value) {
    let schema_path = schema_dir().join(schema_name);
    let raw_schema = load_json(&schema_path);
    let schema = resolve_schema_refs(&raw_schema, &schema_path);
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .expect("schema should compile");
    if let Err(errors) = compiled.validate(instance) {
        let messages = errors.map(|error| error.to_string()).collect::<Vec<_>>();
        panic!(
            "instance did not match schema `{schema_name}`:\n{}",
            messages.join("\n")
        );
    }
}

fn write_contract(dir: &TempDir, contents: &str) {
    fs::write(dir.path().join("ota.yaml"), contents).expect("contract should be written");
}

#[test]
fn execution_topology_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: schema-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
surfaces:
  backend:
    kind: http
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 10s
tasks:
  dev:
    context: host
    run: npx --yes n8n
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
    exposes:
      - surface: backend
"#,
    );

    let json = run_ota(
        &[
            "execution",
            "topology",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("execution-topology.json", &json);
}

#[test]
fn execution_plan_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: execution-demo
execution:
  default_context: host
  contexts:
    host:
      backend: container
      lifecycle: ephemeral
      container:
        image: rust:1.94-bookworm
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "execution",
            "plan",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("execution.json", &json);
}

#[test]
fn assist_wire_setup_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: assist-demo
"#,
    );

    let json = run_ota(
        &[
            "assist",
            "wire-setup",
            "--json",
            "--copy-from",
            ".env.example",
            "--copy-to",
            ".env",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("assist-wire-setup.json", &json);
}

#[test]
fn proof_runtime_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo setup-ready
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
    );

    let json = run_ota(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["phase"], "post-up diagnosis");
    let up_log = fixture
        .path()
        .join(".ota")
        .join("proof")
        .join("app")
        .join("up.log");
    let up_log_contents = fs::read_to_string(&up_log).expect("proof up log should be written");
    assert!(
        up_log_contents.contains("setup-ready"),
        "expected captured phase output in up.log, got:\n{up_log_contents}"
    );
}

#[test]
fn tasks_json_output_with_copy_if_missing_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  setup:
    action:
      kind: copy_if_missing
      from: .env.example
      to: .env
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
}

#[test]
fn services_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: services-demo
execution:
  default_context: app
  contexts:
    app:
      backend: native
services:
  postgres:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
    healthcheck: pg_isready -h 127.0.0.1 -p 5432
"#,
    );

    let json = run_ota(
        &["services", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("services.json", &json);
}
