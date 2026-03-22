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
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::schema::{CheckKind, CheckSeverity, Contract, Lifecycle};

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

pub fn diagnose_checks_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::ChecksOnly)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorScope {
    All,
    Preconditions,
    ChecksOnly,
}

fn diagnose_contract_with_scope(
    contract: &Contract,
    contract_path: &Path,
    scope: DoctorScope,
) -> DoctorReport {
    let mut findings = Vec::new();

    if scope != DoctorScope::ChecksOnly {
        diagnose_lifecycle(contract, &mut findings);
        diagnose_env(contract, &mut findings);
        diagnose_runtimes(contract, &mut findings);
        diagnose_tools(contract, &mut findings);
        if scope == DoctorScope::All {
            diagnose_services(contract, contract_path, &mut findings);
        }
    }
    diagnose_checks(contract, contract_path, scope, &mut findings);

    findings.sort_by_key(|finding| finding.severity);

    DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        findings,
    }
}

fn diagnose_lifecycle(contract: &Contract, findings: &mut Vec<Finding>) {
    if matches!(
        contract
            .execution
            .as_ref()
            .and_then(|execution| execution.lifecycle),
        Some(Lifecycle::Ephemeral)
    ) {
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Ephemeral lifecycle is advisory only in V1"),
            why: String::from(
                "the contract requests `execution.lifecycle: ephemeral`, but current Ota execution remains shell-native and does not create isolated temporary environments",
            ),
            next: String::from(
                "treat `ephemeral` as a portability hint for now; do not rely on isolation or automatic cleanup in V1",
            ),
        });
    }
}

fn diagnose_services(contract: &Contract, contract_path: &Path, findings: &mut Vec<Finding>) {
    let working_dir = contract_working_dir(contract_path);

    for (name, service) in &contract.services {
        if let Some(healthcheck) = service.healthcheck.as_deref() {
            match run_check(healthcheck, working_dir, None) {
                CheckStatus::Passed => {}
                CheckStatus::Failed => findings.push(Finding {
                    severity: if service.required {
                        FindingSeverity::Error
                    } else {
                        FindingSeverity::Warn
                    },
                    summary: format!("Service healthcheck failed: {name}"),
                    why: format!("service `{name}` did not pass its configured healthcheck"),
                    next: match service.start.as_deref() {
                        Some(start) => format!("run `{start}` and re-run `ota doctor`"),
                        None => format!(
                            "start or repair `{name}` and re-run its healthcheck: {healthcheck}"
                        ),
                    },
                }),
                CheckStatus::TimedOut(_) => {
                    unreachable!("service healthchecks do not set timeouts")
                }
            }
            continue;
        }

        if service.required {
            let why = if service.start.is_some() {
                format!(
                    "service `{name}` is required but no `healthcheck` is configured, so Ota cannot verify readiness"
                )
            } else {
                format!(
                    "service `{name}` is required but no `healthcheck` or `start` command is configured, so Ota cannot verify or prepare it"
                )
            };

            let next = if service.start.is_some() {
                format!("add `services.{name}.healthcheck` so `ota doctor` can verify readiness")
            } else {
                format!("add `services.{name}.healthcheck` and optionally `services.{name}.start`")
            };

            findings.push(Finding {
                severity: FindingSeverity::Warn,
                summary: format!("Required service cannot be verified: {name}"),
                why,
                next,
            });
        }
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
    let working_dir = contract_working_dir(contract_path);

    for check in &contract.checks {
        if scope == DoctorScope::Preconditions && check.kind != CheckKind::Precondition {
            continue;
        }

        match run_check(&check.run, working_dir, check.timeout) {
            CheckStatus::Passed => continue,
            CheckStatus::Failed => findings.push(Finding {
                severity: map_check_severity(check.severity),
                summary: format!("Check failed: {}", check.name),
                why: format!("the configured `{}` check did not succeed", check.name),
                next: format!("run `{}` and fix the reported issue", check.run),
            }),
            CheckStatus::TimedOut(timeout) => findings.push(Finding {
                severity: map_check_severity(check.severity),
                summary: format!("Check timed out: {}", check.name),
                why: format!(
                    "the configured `{}` check did not finish within {}ms",
                    check.name, timeout
                ),
                next: format!(
                    "make `{}` complete faster or raise `checks.timeout` for `{}`",
                    check.run, check.name
                ),
            }),
        }
    }
}

enum CheckStatus {
    Passed,
    Failed,
    TimedOut(u64),
}

fn run_check(command: &str, working_dir: &Path, timeout_ms: Option<u64>) -> CheckStatus {
    let Some(timeout_ms) = timeout_ms else {
        let status = shell_command(command).current_dir(working_dir).status();
        return if matches!(status, Ok(status) if status.success()) {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };
    };

    let Ok(mut child) = shell_command(command).current_dir(working_dir).spawn() else {
        return CheckStatus::Failed;
    };

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                };
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return CheckStatus::TimedOut(timeout_ms);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return CheckStatus::Failed;
            }
        }
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

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
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

    if let Some(compatible) = requirement.strip_prefix('^') {
        return version_matches_caret(actual, compatible.trim());
    }

    actual == requirement || actual.starts_with(&format!("{requirement}."))
}

fn version_matches_caret(actual: &str, base: &str) -> bool {
    let actual_parts = match parse_version_parts(actual) {
        Some(parts) => parts,
        None => return false,
    };
    let base_parts = match parse_version_parts(base) {
        Some(parts) => parts,
        None => return false,
    };

    if compare_parts(&actual_parts, &base_parts) < 0 {
        return false;
    }

    let upper_bound = caret_upper_bound(&base_parts);
    compare_parts(&actual_parts, &upper_bound) < 0
}

fn caret_upper_bound(base: &[u64]) -> Vec<u64> {
    let mut upper = base.to_vec();
    let pivot = base.iter().position(|part| *part != 0).unwrap_or(0);

    if upper.len() <= pivot {
        upper.resize(pivot + 1, 0);
    }

    upper[pivot] += 1;
    for part in upper.iter_mut().skip(pivot + 1) {
        *part = 0;
    }

    upper
}

fn compare_version_tokens(actual: &str, minimum: &str) -> Option<i8> {
    let actual_parts = parse_version_parts(actual)?;
    let minimum_parts = parse_version_parts(minimum)?;
    Some(compare_parts(&actual_parts, &minimum_parts))
}

fn compare_parts(left: &[u64], right: &[u64]) -> i8 {
    let len = left.len().max(right.len());

    for index in 0..len {
        let left = *left.get(index).unwrap_or(&0);
        let right = *right.get(index).unwrap_or(&0);
        if left > right {
            return 1;
        }
        if left < right {
            return -1;
        }
    }

    0
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

    use super::{
        FindingSeverity, diagnose_checks_only, diagnose_contract, diagnose_preconditions,
        version_matches,
    };

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
    fn warns_when_ephemeral_lifecycle_is_only_advisory() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Ephemeral lifecycle is advisory only in V1"
        );
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

    #[test]
    fn checks_only_mode_skips_env_runtime_and_tool_diagnosis() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_REQUIRED:
    required: true
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: true
checks:
  - name: health-check
    kind: health
    severity: warn
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].summary, "Check failed: health-check");
    }

    #[test]
    fn reports_optional_tool_version_mismatches_as_warnings() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  cargo:
    version: "999.0.0"
    required: false
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Version mismatch for tool: cargo"
        );
    }

    #[test]
    fn reports_required_service_healthcheck_failures_as_errors() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    healthcheck: exit 1
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Service healthcheck failed: postgres"
        );
    }

    #[test]
    fn reports_optional_service_healthcheck_failures_as_warnings() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  cache:
    required: false
    healthcheck: exit 1
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Service healthcheck failed: cache"
        );
    }

    #[test]
    fn warns_when_required_service_has_no_healthcheck() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    start: docker compose up -d postgres
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Required service cannot be verified: postgres"
        );
    }

    #[test]
    fn sorts_errors_before_warnings_before_info() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_SORT_REQUIRED:
    required: true
tools:
  cargo:
    version: "999.0.0"
    required: false
checks:
  - name: informational-check
    kind: health
    severity: info
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 3);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(report.findings[1].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[2].severity, FindingSeverity::Info);
    }

    #[test]
    fn supports_caret_requirements_for_detected_versions() {
        assert!(version_matches("^3.11", "3.11.0"));
        assert!(version_matches("^3.11", "3.12.4"));
        assert!(!version_matches("^3.11", "4.0.0"));
        assert!(version_matches("^0.6.0", "0.6.4"));
        assert!(!version_matches("^0.6.0", "0.7.0"));
    }

    #[cfg(unix)]
    #[test]
    fn reports_timed_out_checks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
checks:
  - name: slow-check
    kind: health
    severity: warn
    run: sleep 1
    timeout: 50
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "Check timed out: slow-check");
        assert!(report.findings[0].why.contains("50ms"));
    }
}
