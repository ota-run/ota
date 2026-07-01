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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ParsedAgentBoundaryDoc {
    pub(crate) generated_by_ota: bool,
    pub(crate) safe_tasks: Option<Vec<String>>,
    pub(crate) verify_after_changes: Option<Vec<String>>,
    pub(crate) writable_paths: Option<Vec<String>>,
    pub(crate) protected_paths: Option<Vec<String>>,
}

impl ParsedAgentBoundaryDoc {
    pub(crate) fn is_empty(&self) -> bool {
        !self.generated_by_ota
            && self.safe_tasks.is_none()
            && self.verify_after_changes.is_none()
            && self.writable_paths.is_none()
            && self.protected_paths.is_none()
    }
}

pub(crate) fn parse_agent_boundary_doc(contents: &str) -> ParsedAgentBoundaryDoc {
    let generated_by_ota =
        contents.contains("Generated from `") && contents.contains("` by `ota agents`.");
    let parse_contents = if generated_by_ota {
        contents
            .rfind("# AGENTS.md")
            .and_then(|index| contents.get(index..))
            .unwrap_or(contents)
    } else {
        contents
    };
    let mut parsed = ParsedAgentBoundaryDoc {
        generated_by_ota,
        ..ParsedAgentBoundaryDoc::default()
    };
    let lines = parse_contents.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim_end();
        if let Some(label) = parse_agent_doc_list_label(line) {
            let mut values = Vec::new();
            index += 1;
            while index < lines.len() {
                let item_line = lines[index].trim_end();
                if let Some(value) = parse_agent_doc_task_item(item_line) {
                    values.push(value);
                    index += 1;
                } else {
                    break;
                }
            }
            assign_agent_doc_list_field(&mut parsed, label, values);
            continue;
        }
        if let Some((label, values)) = parse_agent_doc_inline_list(line) {
            assign_agent_doc_list_field(&mut parsed, label, values);
        }
        index += 1;
    }
    parsed
}

fn parse_agent_doc_list_label(line: &str) -> Option<&'static str> {
    match line.trim() {
        "- `safe_tasks`:" => Some("safe_tasks"),
        "- `verify_after_changes`:" => Some("verify_after_changes"),
        _ => None,
    }
}

fn parse_agent_doc_task_item(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    let remainder = trimmed.strip_prefix("  - `")?;
    let (value, _) = remainder.split_once('`')?;
    Some(value.to_string())
}

fn parse_agent_doc_inline_list(line: &str) -> Option<(&'static str, Vec<String>)> {
    let trimmed = line.trim();
    let (label, remainder) = if let Some(remainder) = trimmed.strip_prefix("- `writable_paths`: ") {
        ("writable_paths", remainder)
    } else if let Some(remainder) = trimmed.strip_prefix("- `protected_paths`: ") {
        ("protected_paths", remainder)
    } else {
        return None;
    };

    let values = remainder
        .split(',')
        .filter_map(|segment| {
            let value = segment.trim().strip_prefix('`')?.strip_suffix('`')?;
            (!value.is_empty()).then(|| value.to_string())
        })
        .collect::<Vec<_>>();
    Some((label, values))
}

fn assign_agent_doc_list_field(
    parsed: &mut ParsedAgentBoundaryDoc,
    label: &str,
    values: Vec<String>,
) {
    match label {
        "safe_tasks" => parsed.safe_tasks = Some(values),
        "verify_after_changes" => parsed.verify_after_changes = Some(values),
        "writable_paths" => parsed.writable_paths = Some(values),
        "protected_paths" => parsed.protected_paths = Some(values),
        _ => {}
    }
}
