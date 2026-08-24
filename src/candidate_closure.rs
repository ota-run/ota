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
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

//! Conservative, source-bound closure resolution for contract candidates.
//!
//! This resolver recognizes only direct, finite verifier commands. It does not infer effects or
//! agent safety from command names; unknown effects remain explicit and prevent safe promotion.

use std::collections::{BTreeMap, BTreeSet};

use crate::contract_candidate::{
    CandidateExecutionClosure, ClosureEvidence, ExecutionClosureEdge, ExecutionClosureNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateTaskClassification {
    Runnable,
    Unknown,
}

#[derive(Debug, Clone)]
pub(crate) struct CandidateClosureInput<'a> {
    pub task_name: &'a str,
    pub task_command: &'a str,
    pub command_body: &'a str,
    pub package_manager: Option<&'a str>,
    pub package_scripts: Option<&'a BTreeMap<String, String>>,
    pub root_script_name: Option<&'a str>,
    pub platform: &'a str,
    pub requirements: Vec<ExecutionClosureNode>,
    pub source_is_execution_authoritative: bool,
    pub evidence: Vec<ClosureEvidence>,
}

#[derive(Debug, Clone)]
pub(crate) struct CandidateClosureResolution {
    pub classification: CandidateTaskClassification,
    pub closure: CandidateExecutionClosure,
}

pub(crate) fn resolve_candidate_task_closure(
    input: CandidateClosureInput<'_>,
) -> CandidateClosureResolution {
    let task_id = format!("task:{}", input.task_name);
    let task_node = |classification: &str| ExecutionClosureNode {
        id: task_id.clone(),
        kind: String::from("task"),
        value: input.task_command.to_string(),
        classification: classification.to_string(),
        evidence: input.evidence.clone(),
    };

    let package_script_chain = match (
        input.package_manager,
        input.package_scripts,
        input.root_script_name,
    ) {
        (Some(manager), Some(scripts), Some(root_script_name)) => {
            package_script_chain(scripts, manager, root_script_name)
        }
        (None, None, None) => Some(Vec::new()),
        _ => None,
    };
    let command_body = package_script_chain
        .as_ref()
        .and_then(|chain| chain.last())
        .map(|(_, body)| body.as_str())
        .unwrap_or(input.command_body);
    let Some(executable) = input
        .source_is_execution_authoritative
        .then(|| {
            package_script_chain
                .as_ref()
                .and_then(|_| direct_verifier_executable(command_body))
        })
        .flatten()
    else {
        return CandidateClosureResolution {
            classification: CandidateTaskClassification::Unknown,
            closure: CandidateExecutionClosure {
                identity: String::new(),
                working_directory: String::from("."),
                platform: input.platform.to_string(),
                nodes: vec![task_node("unknown")],
                edges: Vec::new(),
                requirements: input.requirements,
                effects: Vec::new(),
                unresolved_reasons: vec![String::from("execution_closure_unresolved")],
            },
        };
    };

    let executable_id = format!("executable:{executable}");
    let executable_node = ExecutionClosureNode {
        id: executable_id.clone(),
        kind: String::from("executable"),
        value: executable.to_string(),
        classification: String::from("required"),
        evidence: input.evidence.clone(),
    };
    let mut nodes = vec![task_node("runnable")];
    let mut edges = Vec::new();
    let mut requirements = input.requirements;
    requirements.push(executable_node.clone());
    let executable_parent = if let Some(package_manager) = input.package_manager {
        let Some(script_chain) = package_script_chain else {
            unreachable!("package script chain was required before executable resolution");
        };
        let manager_id = format!("package_manager:{package_manager}");
        let manager_node = ExecutionClosureNode {
            id: manager_id.clone(),
            kind: String::from("package_manager"),
            value: package_manager.to_string(),
            classification: String::from("required"),
            evidence: input.evidence.clone(),
        };
        edges.push(ExecutionClosureEdge {
            from: task_id.clone(),
            to: manager_id,
            kind: String::from("invokes"),
            evidence: input.evidence.clone(),
        });
        edges.push(ExecutionClosureEdge {
            from: manager_node.id.clone(),
            to: format!("script:{}", script_chain[0].0),
            kind: String::from("runs_script"),
            evidence: input.evidence.clone(),
        });
        nodes.push(manager_node.clone());
        requirements.push(manager_node);
        for (index, (script_name, body)) in script_chain.iter().enumerate() {
            let script_id = format!("script:{script_name}");
            nodes.push(ExecutionClosureNode {
                id: script_id.clone(),
                kind: String::from("package_script"),
                value: body.clone(),
                classification: String::from("runnable"),
                evidence: input.evidence.clone(),
            });
            if let Some((next_script_name, _)) = script_chain.get(index + 1) {
                edges.push(ExecutionClosureEdge {
                    from: script_id,
                    to: format!("script:{next_script_name}"),
                    kind: String::from("invokes_script"),
                    evidence: input.evidence.clone(),
                });
            }
        }
        format!(
            "script:{}",
            script_chain.last().expect("non-empty script chain").0
        )
    } else {
        task_id.clone()
    };
    edges.push(ExecutionClosureEdge {
        from: executable_parent,
        to: executable_id,
        kind: String::from("executes"),
        evidence: input.evidence.clone(),
    });
    nodes.push(executable_node.clone());
    CandidateClosureResolution {
        classification: CandidateTaskClassification::Runnable,
        // A direct command body establishes execution shape, not effect safety. The unknown
        // effect below prevents the detector from emitting an agent-safe candidate.
        closure: CandidateExecutionClosure {
            identity: String::new(),
            working_directory: String::from("."),
            platform: input.platform.to_string(),
            nodes,
            edges,
            requirements,
            effects: vec![ExecutionClosureNode {
                id: String::from("effect:unclassified"),
                kind: String::from("effect"),
                value: String::from("direct_command"),
                classification: String::from("unknown"),
                evidence: input.evidence,
            }],
            unresolved_reasons: vec![String::from("effect_classification_unresolved")],
        },
    }
}

fn package_script_chain(
    scripts: &BTreeMap<String, String>,
    package_manager: &str,
    root_script_name: &str,
) -> Option<Vec<(String, String)>> {
    let mut names = BTreeSet::new();
    let mut chain = Vec::new();
    let mut current = root_script_name.to_string();
    loop {
        if !names.insert(current.clone()) {
            return None;
        }
        let body = scripts.get(&current)?.clone();
        let next = package_script_reference(&body, package_manager).map(str::to_string);
        chain.push((current, body));
        let Some(next) = next else {
            return Some(chain);
        };
        current = next;
    }
}

fn package_script_reference<'a>(body: &'a str, package_manager: &str) -> Option<&'a str> {
    let tokens = body.split_ascii_whitespace().collect::<Vec<_>>();
    match (package_manager, tokens.as_slice()) {
        ("npm", ["npm", "run", script])
        | ("pnpm", ["pnpm", script])
        | ("pnpm", ["pnpm", "run", script])
        | ("yarn", ["yarn", script])
        | ("yarn", ["yarn", "run", script])
        | ("bun", ["bun", "run", script])
            if !script.starts_with('-') =>
        {
            Some(*script)
        }
        _ => None,
    }
}

fn direct_verifier_executable(command: &str) -> Option<&str> {
    let command = command.trim();
    if command.is_empty()
        || command.contains([
            '\n', '\r', ';', '|', '&', '$', '`', '<', '>', '\\', '"', '\'',
        ])
    {
        return None;
    }
    let executable = command.split_ascii_whitespace().next()?;
    matches!(
        executable,
        "biome"
            | "cabal"
            | "cargo"
            | "dotnet"
            | "eslint"
            | "go"
            | "jest"
            | "mix"
            | "mvn"
            | "prettier"
            | "pytest"
            | "ruff"
            | "tsc"
            | "vitest"
    )
    .then_some(executable)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        CandidateClosureInput, CandidateTaskClassification, resolve_candidate_task_closure,
    };

    #[test]
    fn resolves_direct_cargo_verifier_without_claiming_agent_safety() {
        let resolution = resolve_candidate_task_closure(CandidateClosureInput {
            task_name: "test",
            task_command: "cargo test --workspace",
            command_body: "cargo test --workspace",
            package_manager: None,
            package_scripts: None,
            root_script_name: None,
            platform: "unknown",
            requirements: Vec::new(),
            source_is_execution_authoritative: true,
            evidence: Vec::new(),
        });

        assert_eq!(
            resolution.classification,
            CandidateTaskClassification::Runnable
        );
        assert_eq!(resolution.closure.nodes[1].id, "executable:cargo");
        assert_eq!(
            resolution.closure.unresolved_reasons,
            vec![String::from("effect_classification_unresolved")]
        );
    }

    #[test]
    fn refuses_wrapper_and_ci_only_commands_as_unresolved() {
        for (command, source_is_execution_authoritative) in [
            ("pnpm test", true),
            ("cargo test && cargo clippy", true),
            ("cargo test", false),
        ] {
            let resolution = resolve_candidate_task_closure(CandidateClosureInput {
                task_name: "test",
                task_command: command,
                command_body: command,
                package_manager: None,
                package_scripts: None,
                root_script_name: None,
                platform: "unknown",
                requirements: Vec::new(),
                source_is_execution_authoritative,
                evidence: Vec::new(),
            });
            assert_eq!(
                resolution.classification,
                CandidateTaskClassification::Unknown
            );
            assert_eq!(
                resolution.closure.unresolved_reasons,
                vec![String::from("execution_closure_unresolved")]
            );
        }
    }

    #[test]
    fn resolves_same_manager_package_script_chain_and_refuses_cycles() {
        let scripts = BTreeMap::from([
            (String::from("test"), String::from("pnpm run verify")),
            (String::from("verify"), String::from("vitest run")),
        ]);
        let resolution = resolve_candidate_task_closure(CandidateClosureInput {
            task_name: "test",
            task_command: "pnpm test",
            command_body: "pnpm run verify",
            package_manager: Some("pnpm"),
            package_scripts: Some(&scripts),
            root_script_name: Some("test"),
            platform: "unknown",
            requirements: Vec::new(),
            source_is_execution_authoritative: true,
            evidence: Vec::new(),
        });
        assert_eq!(
            resolution.classification,
            CandidateTaskClassification::Runnable
        );
        assert!(
            resolution
                .closure
                .edges
                .iter()
                .any(|edge| edge.from == "script:test" && edge.to == "script:verify")
        );
        assert!(
            resolution
                .closure
                .edges
                .iter()
                .any(|edge| edge.from == "script:verify" && edge.to == "executable:vitest")
        );

        let cycle = BTreeMap::from([
            (String::from("test"), String::from("pnpm run verify")),
            (String::from("verify"), String::from("pnpm test")),
        ]);
        let resolution = resolve_candidate_task_closure(CandidateClosureInput {
            task_name: "test",
            task_command: "pnpm test",
            command_body: "pnpm run verify",
            package_manager: Some("pnpm"),
            package_scripts: Some(&cycle),
            root_script_name: Some("test"),
            platform: "unknown",
            requirements: Vec::new(),
            source_is_execution_authoritative: true,
            evidence: Vec::new(),
        });
        assert_eq!(
            resolution.classification,
            CandidateTaskClassification::Unknown
        );
    }
}
