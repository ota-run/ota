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

use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::schema::{CheckKind, CheckSeverity, Contract};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub findings: Vec<Finding>,
}

pub fn diagnose_contract(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::All)
}

pub fn diagnose_preconditions(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::Preconditions)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorScope {
    All,
    Preconditions,
}

fn diagnose_contract_with_scope(
    contract: &Contract,
    contract_path: &Path,
    scope: DoctorScope,
) -> DoctorReport {
    let mut findings = Vec::new();

    diagnose_env(contract, &mut findings);
    diagnose_runtimes(contract, &mut findings);
    diagnose_tools(contract, &mut findings);
    diagnose_checks(contract, contract_path, scope, &mut findings);

    findings.sort_by_key(|finding| finding.severity);

    DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        findings,
    }
}

fn diagnose_env(contract: &Contract, findings: &mut Vec<Finding>) {
    for (name, requirement) in &contract.env {
        let value = std::env::var(name)
            .ok()
            .or_else(|| requirement.default.clone());

        match value {
            Some(value) => {
                if !requirement.allowed.is_empty()
                    && !requirement.allowed.iter().any(|allowed| allowed == &value)
                {
                    findings.push(Finding {
                        severity: FindingSeverity::Error,
                        summary: format!("Invalid environment value: {name}"),
                        why: format!(
                            "{name} resolved to `{value}`, which is outside the allowed values"
                        ),
                        next: format!("set {name} to one of: {}", requirement.allowed.join(", ")),
                    });
                }
            }
            None if requirement.required => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Missing environment variable: {name}"),
                why: format!("{name} is required by this repo contract"),
                next: format!("set {name} in your environment before running tasks"),
            }),
            None => {}
        }
    }
}

fn diagnose_runtimes(contract: &Contract, findings: &mut Vec<Finding>) {
    for (name, requirement) in &contract.runtimes {
        diagnose_command_version("runtime", name, requirement.version(), true, findings);
    }
}

fn diagnose_tools(contract: &Contract, findings: &mut Vec<Finding>) {
    for (name, requirement) in &contract.tools {
        let required = match requirement {
            crate::schema::ToolRequirement::Simple(_) => true,
            crate::schema::ToolRequirement::Detailed(detail) => detail.required,
        };

        diagnose_command_version("tool", name, requirement.version(), required, findings);
    }
}

fn diagnose_command_version(
    kind: &str,
    name: &str,
    requirement: &str,
    required: bool,
    findings: &mut Vec<Finding>,
) {
    let Some(actual) = command_version(name) else {
        findings.push(Finding {
            severity: if required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Missing {kind}: {name}"),
            why: format!("{name} is declared in the contract but is not available on PATH"),
            next: format!("install {name} and make it available on PATH"),
        });
        return;
    };

    if version_matches(requirement, &actual) {
        return;
    }

    findings.push(Finding {
        severity: if required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary: format!("Version mismatch for {kind}: {name}"),
        why: format!("{name} resolved to `{actual}` but the contract requires `{requirement}`"),
        next: format!("install a compatible {name} version that satisfies `{requirement}`"),
    });
}

fn diagnose_checks(
    contract: &Contract,
    contract_path: &Path,
    scope: DoctorScope,
    findings: &mut Vec<Finding>,
) {
    let working_dir = contract_path.parent().unwrap_or_else(|| Path::new("."));

    for check in &contract.checks {
        if scope == DoctorScope::Preconditions && check.kind != CheckKind::Precondition {
            continue;
        }

        let status = shell_command(&check.run).current_dir(working_dir).status();
        let passed = matches!(status, Ok(status) if status.success());

        if passed {
            continue;
        }

        findings.push(Finding {
            severity: map_check_severity(check.severity),
            summary: format!("Check failed: {}", check.name),
            why: format!("the configured `{}` check did not succeed", check.name),
            next: format!("run `{}` and fix the reported issue", check.run),
        });
    }
}

fn map_check_severity(severity: CheckSeverity) -> FindingSeverity {
    match severity {
        CheckSeverity::Error => FindingSeverity::Error,
        CheckSeverity::Warn => FindingSeverity::Warn,
        CheckSeverity::Info => FindingSeverity::Info,
    }
}

fn command_version(name: &str) -> Option<String> {
    let output = Command::new(name).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    extract_version_token(&combined)
}

fn extract_version_token(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .find(|token| token.chars().any(|ch| ch.is_ascii_digit()))
        .map(|token| {
            token
                .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.' && ch != '-')
                .trim_start_matches('v')
                .to_string()
        })
        .filter(|token| !token.is_empty())
}

fn version_matches(requirement: &str, actual: &str) -> bool {
    let requirement = requirement.trim();
    if requirement == "*" {
        return true;
    }

    if let Some(minimum) = requirement.strip_prefix(">=") {
        return compare_version_tokens(actual, minimum.trim())
            .is_some_and(|ordering| ordering >= 0);
    }

    actual == requirement || actual.starts_with(&format!("{requirement}."))
}

fn compare_version_tokens(actual: &str, minimum: &str) -> Option<i8> {
    let actual_parts = parse_version_parts(actual)?;
    let minimum_parts = parse_version_parts(minimum)?;
    let len = actual_parts.len().max(minimum_parts.len());

    for index in 0..len {
        let left = *actual_parts.get(index).unwrap_or(&0);
        let right = *minimum_parts.get(index).unwrap_or(&0);
        if left > right {
            return Some(1);
        }
        if left < right {
            return Some(-1);
        }
    }

    Some(0)
}

fn parse_version_parts(input: &str) -> Option<Vec<u64>> {
    let parts = input
        .trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            let digits = part
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>();
            if digits.is_empty() {
                None
            } else {
                digits.parse::<u64>().ok()
            }
        })
        .collect::<Option<Vec<_>>>()?;

    if parts.is_empty() { None } else { Some(parts) }
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("sh");
    shell.arg("-lc").arg(command);
    shell
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("cmd");
    shell.arg("/C").arg(command);
    shell
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::parser::parse_contract_str;

    use super::{FindingSeverity, diagnose_contract, diagnose_preconditions};

    #[test]
    fn prioritizes_blocking_env_errors_before_warnings() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
env:
  OTA_DOCTOR_REQUIRED_MISSING:
    required: true
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

        assert!(!report.ok);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Missing environment variable: OTA_DOCTOR_REQUIRED_MISSING"
        );
        assert_eq!(report.findings[1].severity, FindingSeverity::Warn);
    }

    #[test]
    fn reports_allowed_env_value_mismatches() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_ALLOWED:
    required: false
    default: prod
    allowed:
      - development
      - test
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
    }

    #[test]
    fn precondition_mode_skips_health_checks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: wait-for-db
    kind: health
    severity: error
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert!(report.findings.is_empty());
    }
}
