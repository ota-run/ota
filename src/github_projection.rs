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

use std::path::Path;

use serde::Serialize;
use serde_yaml::{Mapping, Value as YamlValue};
use sha2::{Digest, Sha256};

#[cfg(test)]
use crate::ci_projection::build_ci_projection;
use crate::ci_projection::{CiProjection, CiProjectionToolchain};
#[cfg(test)]
use crate::schema::Contract;

pub(crate) const OWNERSHIP_MARKER: &str = "# ota:managed-github-projection v1";
const GITHUB_CHECKOUT_REV: &str = "34e114876b0b11c390a56381ad16ebd13914f8d5";
const OTA_SETUP_REV: &str = "493cba84bf7f7c11c9e8e996d832d93a89c62184";
const GITHUB_SETUP_GO_REV: &str = "924ae3a1cded613372ab5595356fb5720e22ba16";
const GITHUB_SETUP_NODE_REV: &str = "a0853c24544627f65ddf259abe73b1d18a591444";
const GITHUB_SETUP_RUBY_REV: &str = "003a5c4d8d6321bd302e38f6f0ec593f77f06600";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GitHubProjection {
    pub projection: CiProjection,
    pub runner: String,
    pub provider_checks: Vec<GitHubProjectionCheck>,
    pub content_identity: String,
    pub render_identity: String,
    pub rendered: String,
}

/// A scope-qualified GitHub check that maps back to one canonical Ota merge identity.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct GitHubProjectionCheck {
    pub merge_check_id: String,
    pub provider_check_name: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CallerProjectionBinding {
    pub jobs: Vec<CallerProjectionBindingJob>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CallerProjectionBindingJob {
    pub id: String,
    pub uses: String,
    pub projection_identity: String,
    pub target_os: String,
}

#[cfg(test)]
pub(crate) fn render_github_projection(
    contract: &Contract,
    workflow_name: &str,
    runner: &str,
    mode: &str,
    target_os: &str,
) -> Result<GitHubProjection, String> {
    let projection = build_ci_projection(contract, workflow_name, mode, target_os)?;
    render_github_projection_from_projection(projection, runner)
}

pub(crate) fn render_github_projection_from_projection(
    projection: CiProjection,
    runner: &str,
) -> Result<GitHubProjection, String> {
    let check_name = &projection.merge_check_ids[0];
    let job_id = github_job_id(check_name);
    let workflow_yaml = yaml_scalar(&projection.workflow);
    let runner_yaml = yaml_scalar(runner);
    let mode_flag = format!(" --mode {}", projection.mode);
    let toolchain_setup_steps = github_toolchain_setup_steps(&projection.toolchains)?;
    let provider_checks = projection
        .merge_check_ids
        .iter()
        .map(|merge_check_id| GitHubProjectionCheck {
            merge_check_id: merge_check_id.clone(),
            provider_check_name: github_check_name(
                merge_check_id,
                &projection.mode,
                &projection.target_os,
            ),
        })
        .collect::<Vec<_>>();
    let primary_provider_check = &provider_checks[0];
    let refusal_canary_jobs = projection
        .refusal_canaries
        .iter()
        .map(|canary| {
            let target = yaml_scalar(&canary.target);
            let command = match canary.kind.as_str() {
                "task" => format!(
                    "ota run --agent --expect-refusal{mode_flag} --json {target}",
                    target = target,
                    mode_flag = mode_flag,
                ),
                "workflow" => format!(
                    "ota up --agent --expect-refusal --workflow {target}{mode_flag} --json",
                    target = target,
                    mode_flag = mode_flag,
                ),
                _ => return String::new(),
            };
            format!(
                concat!(
                    "  {job_id}:\n",
                    "    name: {check_name}\n",
                    "    runs-on: ${{{{ inputs.ota_runner }}}}\n",
                    "    steps:\n",
                    "      - uses: actions/checkout@{GITHUB_CHECKOUT_REV}\n",
                    "      - uses: ota-run/setup@{OTA_SETUP_REV}\n",
                    "        with:\n",
                    "          source: contract\n",
                    "{toolchain_setup_steps}",
                    "      - name: Verify Ota projection identity\n",
                    "        run: ota ci projection --workflow {workflow_yaml}{mode_flag} --target-os ${{{{ inputs.ota_target_os }}}} --expect-identity ${{{{ inputs.ota_projection_identity }}}} --json\n",
                    "      - name: Prove agent refusal canary\n",
                    "        run: {command}\n"
                ),
                job_id = github_job_id(&canary.merge_check_id),
                check_name = yaml_scalar(&github_check_name(
                    &canary.merge_check_id,
                    &projection.mode,
                    &projection.target_os,
                )),
                workflow_yaml = workflow_yaml,
                mode_flag = mode_flag,
                command = command,
                GITHUB_CHECKOUT_REV = GITHUB_CHECKOUT_REV,
                OTA_SETUP_REV = OTA_SETUP_REV,
                toolchain_setup_steps = toolchain_setup_steps,
            )
        })
        .collect::<String>();
    let execution_steps = if projection.proof_required {
        format!(
            concat!(
                "      - name: Prove contract runtime boundary\n",
                "        run: ota proof runtime --workflow {workflow_yaml}{mode_flag} --archive --json\n"
            ),
            workflow_yaml = workflow_yaml,
            mode_flag = mode_flag,
        )
    } else {
        format!(
            concat!(
                "      - name: Run contract lane\n",
                "        run: ota up --workflow {workflow_yaml}{mode_flag} --agent --json\n",
                "      - name: Archive contract receipt\n",
                "        run: ota receipt --workflow {workflow_yaml}{mode_flag} --archive --json\n"
            ),
            workflow_yaml = workflow_yaml,
            mode_flag = mode_flag,
        )
    };
    let rendered = format!(
        concat!(
            "{OWNERSHIP_MARKER} identity={projection_identity}\n",
            "name: Ota governance ({workflow_yaml})\n",
            "\n",
            "on:\n",
            "  workflow_call:\n",
            "    inputs:\n",
            "      ota_projection_identity:\n",
            "        description: Ota-managed projection identity\n",
            "        required: true\n",
            "        type: string\n",
            "      ota_runner:\n",
            "        description: GitHub runner selected by the human-owned caller\n",
            "        required: false\n",
            "        default: {runner_yaml}\n",
            "        type: string\n",
            "      ota_target_os:\n",
            "        description: Target operating system bound into the Ota projection\n",
            "        required: true\n",
            "        type: string\n",
            "\n",
            "jobs:\n",
            "  {job_id}:\n",
            "    name: {check_yaml}\n",
            "    runs-on: ${{{{ inputs.ota_runner }}}}\n",
            "    steps:\n",
            "      - uses: actions/checkout@{GITHUB_CHECKOUT_REV}\n",
            "      - uses: ota-run/setup@{OTA_SETUP_REV}\n",
            "        with:\n",
            "          source: contract\n",
            "{toolchain_setup_steps}",
            "      - name: Verify Ota projection identity\n",
            "        run: ota ci projection --workflow {workflow_yaml}{mode_flag} --target-os ${{{{ inputs.ota_target_os }}}} --expect-identity ${{{{ inputs.ota_projection_identity }}}} --json\n",
            "      - name: Validate contract\n",
            "        run: ota validate . --json\n",
            "      - name: Diagnose contract lane\n",
            "        run: ota doctor --workflow {workflow_yaml}{mode_flag} --json\n",
            "      - name: Discover safe execution surface\n",
            "        run: ota tasks --safe --use --json\n",
            "      - name: Preview contract lane\n",
            "        run: ota up --workflow {workflow_yaml}{mode_flag} --agent --dry-run --json\n",
            "{execution_steps}",
            "{refusal_canary_jobs}",
        ),
        OWNERSHIP_MARKER = OWNERSHIP_MARKER,
        projection_identity = projection.identity,
        workflow_yaml = workflow_yaml,
        job_id = job_id,
        check_yaml = yaml_scalar(&primary_provider_check.provider_check_name),
        runner_yaml = runner_yaml,
        mode_flag = mode_flag,
        GITHUB_CHECKOUT_REV = GITHUB_CHECKOUT_REV,
        OTA_SETUP_REV = OTA_SETUP_REV,
        toolchain_setup_steps = toolchain_setup_steps,
        execution_steps = execution_steps,
        refusal_canary_jobs = refusal_canary_jobs,
    );
    let content_identity = format!("sha256:{:x}", Sha256::digest(rendered.as_bytes()));
    let render_identity = format!(
        "sha256:{:x}",
        Sha256::digest(
            serde_json::to_vec(&(1u8, &projection.identity, runner, &content_identity))
                .map_err(|error| format!("could not serialize GitHub render identity: {error}"))?
        )
    );

    Ok(GitHubProjection {
        projection,
        runner: runner.to_string(),
        provider_checks,
        content_identity,
        render_identity,
        rendered,
    })
}

fn github_toolchain_setup_steps(toolchains: &[CiProjectionToolchain]) -> Result<String, String> {
    toolchains
        .iter()
        .map(|toolchain| match (toolchain.name.as_str(), toolchain.source.as_str()) {
            (_, "go") => Ok(format!(
                concat!(
                    "      - uses: actions/setup-go@{GITHUB_SETUP_GO_REV}\n",
                    "        with:\n",
                    "          go-version: {version}\n"
                ),
                GITHUB_SETUP_GO_REV = GITHUB_SETUP_GO_REV,
                version = yaml_scalar(&github_go_version_spec(&toolchain.version)?),
            )),
            ("node", "corepack") => Ok(format!(
                concat!(
                    "      - uses: actions/setup-node@{GITHUB_SETUP_NODE_REV}\n",
                    "        with:\n",
                    "          node-version: {version}\n"
                ),
                GITHUB_SETUP_NODE_REV = GITHUB_SETUP_NODE_REV,
                version = yaml_scalar(&github_node_version_spec(&toolchain.version)?),
            )),
            ("ruby", "ruby") => Ok(format!(
                concat!(
                    "      - uses: ruby/setup-ruby@{GITHUB_SETUP_RUBY_REV}\n",
                    "        with:\n",
                    "          ruby-version: {version}\n"
                ),
                GITHUB_SETUP_RUBY_REV = GITHUB_SETUP_RUBY_REV,
                version = yaml_scalar(&github_ruby_version_spec(&toolchain.version)?),
            )),
            (_, source) => Err(format!(
                "GitHub projection cannot provision required toolchain `{}` with source `{source}`; choose a supported provider adapter or keep the lane outside managed execution",
                toolchain.name
            )),
        })
        .collect()
}

/// `actions/setup-go` accepts a release selector, not Ota's full comparator-set syntax.
/// Preserve contract ranges in the neutral projection, then select the explicit lower release for
/// this adapter. A malformed or unsupported comparator remains a render refusal.
fn github_go_version_spec(version: &str) -> Result<String, String> {
    let trimmed = version.trim();
    let constraints = trimmed.split(',').map(str::trim).collect::<Vec<_>>();
    let candidate = if constraints.len() == 1 {
        constraints[0]
    } else {
        let Some(lower) = constraints[0].strip_prefix(">=").map(str::trim) else {
            return Err(format!(
                "GitHub projection cannot derive an actions/setup-go selector from required Go version `{version}`"
            ));
        };
        if constraints[1..].iter().any(|constraint| {
            !constraint
                .strip_prefix('<')
                .map(str::trim)
                .is_some_and(is_numeric_version)
        }) {
            return Err(format!(
                "GitHub projection cannot derive an actions/setup-go selector from required Go version `{version}`"
            ));
        }
        lower
    };
    if !is_numeric_version(candidate) {
        return Err(format!(
            "GitHub projection cannot derive an actions/setup-go selector from required Go version `{version}`"
        ));
    }
    Ok(candidate.to_string())
}

/// `actions/setup-node` consumes Node semver selectors directly, including the bounded ranges
/// Ota contracts use. Reject an empty declaration rather than falling back to the hosted image.
fn github_node_version_spec(version: &str) -> Result<String, String> {
    let version = version.trim();
    if version.is_empty() {
        return Err("GitHub projection cannot derive an actions/setup-node selector from an empty Node version".to_string());
    }
    Ok(version.to_string())
}

/// Ruby setup accepts the declared release selector directly. Reject an empty selector rather
/// than falling back to a hosted image's default Ruby.
fn github_ruby_version_spec(version: &str) -> Result<String, String> {
    let version = version.trim();
    if version.is_empty() {
        return Err(
            "GitHub projection cannot derive a ruby/setup-ruby selector from an empty Ruby version"
                .to_string(),
        );
    }
    Ok(version.to_string())
}

fn is_numeric_version(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

fn github_job_id(check_id: &str) -> String {
    format!(
        "ota_{}",
        check_id
            .chars()
            .map(|character| if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            })
            .collect::<String>()
    )
}

fn github_check_name(merge_check_id: &str, mode: &str, target_os: &str) -> String {
    format!("{merge_check_id} ({target_os}/{mode})")
}

fn yaml_scalar(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn managed_projection_identity(contents: &str) -> Option<&str> {
    let line = contents.lines().next()?;
    let value = line.strip_prefix(OWNERSHIP_MARKER)?.trim_start();
    value.strip_prefix("identity=")
}

pub(crate) fn caller_projection_binding(
    caller: &str,
    generated_path: &Path,
    contract_root: &Path,
    identity: &str,
    target_os: &str,
) -> Option<CallerProjectionBinding> {
    let relative_path = generated_path
        .strip_prefix(contract_root)
        .unwrap_or(generated_path)
        .to_string_lossy()
        .replace('\\', "/");
    let Ok(document) = serde_yaml::from_str::<YamlValue>(caller) else {
        return None;
    };
    let Some(jobs) =
        yaml_mapping_value(document.as_mapping(), "jobs").and_then(YamlValue::as_mapping)
    else {
        return None;
    };
    let mut matched_jobs = jobs
        .iter()
        .filter_map(|(job_id, job)| {
            let job_id = job_id.as_str()?;
            let Some(job) = job.as_mapping() else {
                return None;
            };
            let uses = yaml_mapping_value(Some(job), "uses").and_then(YamlValue::as_str);
            let referenced_identity = yaml_mapping_value(Some(job), "with")
                .and_then(YamlValue::as_mapping)
                .and_then(|inputs| yaml_mapping_value(Some(inputs), "ota_projection_identity"))
                .and_then(YamlValue::as_str);
            let caller_target_os = yaml_mapping_value(Some(job), "with")
                .and_then(YamlValue::as_mapping)
                .and_then(|inputs| yaml_mapping_value(Some(inputs), "ota_target_os"))
                .and_then(YamlValue::as_str);
            (uses == Some(format!("./{relative_path}").as_str())
                && referenced_identity == Some(identity)
                && caller_target_os == Some(target_os))
            .then(|| CallerProjectionBindingJob {
                id: job_id.to_string(),
                uses: uses
                    .expect("uses is present when binding matches")
                    .to_string(),
                projection_identity: referenced_identity
                    .expect("projection identity is present when binding matches")
                    .to_string(),
                target_os: caller_target_os
                    .expect("target OS is present when binding matches")
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();
    matched_jobs.sort_by(|left, right| left.id.cmp(&right.id));
    (!matched_jobs.is_empty()).then_some(CallerProjectionBinding { jobs: matched_jobs })
}

fn yaml_mapping_value<'a>(mapping: Option<&'a Mapping>, key: &str) -> Option<&'a YamlValue> {
    mapping?.get(YamlValue::String(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_is_deterministic_and_caller_requires_identity() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: fixture
toolchains:
  go:
    version: ">=1.26,<1.27"
    fulfillment:
      source: go
      mode: none
tasks:
  verify:
    run: echo verify
    execution:
      modes:
        container: {}
    requirements:
      toolchains: [go]
  publish:
    run: echo publish
workflows:
  default: verify
  verify:
    run:
      task: verify
  release:
    run:
      task: publish
agent:
  refusal_canaries:
    - task: publish
    - workflow: release
"#,
        )
        .expect("fixture contract should parse");
        let first =
            render_github_projection(&contract, "verify", "ubuntu-latest", "native", "linux")
                .expect("projection should render");
        let second =
            render_github_projection(&contract, "verify", "ubuntu-latest", "native", "linux")
                .expect("projection should render again");
        let container =
            render_github_projection(&contract, "verify", "ubuntu-latest", "container", "linux")
                .expect("container projection should render");
        let alternate_runner =
            render_github_projection(&contract, "verify", "macos-latest", "native", "linux")
                .expect("alternate runner projection should render");
        let macos_target =
            render_github_projection(&contract, "verify", "macos-latest", "native", "macos")
                .expect("macOS target projection should render");
        assert_eq!(first.projection.identity, second.projection.identity);
        assert_eq!(first.rendered, second.rendered);
        assert_ne!(first.projection.identity, container.projection.identity);
        assert_eq!(
            first.projection.identity,
            alternate_runner.projection.identity
        );
        assert_ne!(first.projection.identity, macos_target.projection.identity);
        assert_ne!(first.content_identity, alternate_runner.content_identity);
        assert_ne!(first.render_identity, alternate_runner.render_identity);
        assert_eq!(container.projection.mode, "container");
        assert!(
            container
                .rendered
                .contains("ota up --workflow 'verify' --mode container --agent")
        );
        assert!(first.rendered.contains("ota tasks --safe --use --json"));
        assert!(first.rendered.contains("ota validate . --json"));
        assert!(
            first
                .rendered
                .contains("ota run --agent --expect-refusal --mode native --json 'publish'")
        );
        assert!(
            first.rendered.contains(
                "ota up --agent --expect-refusal --workflow 'release' --mode native --json"
            )
        );
        assert!(
            first
                .rendered
                .contains("name: 'ota.refusal-canary.task.publish (linux/native)'")
        );
        assert!(
            first
                .rendered
                .contains("name: 'ota.refusal-canary.workflow.release (linux/native)'")
        );
        assert!(
            first
                .rendered
                .contains("ota_ota_refusal_canary_task_publish:")
        );
        assert!(
            first
                .rendered
                .contains("ota_ota_refusal_canary_workflow_release:")
        );
        assert_eq!(first.projection.refusal_canaries.len(), 2);
        assert_eq!(first.provider_checks.len(), 3);
        assert_eq!(
            first.provider_checks[1].provider_check_name,
            "ota.refusal-canary.task.publish (linux/native)"
        );
        assert!(
            first
                .projection
                .merge_check_ids
                .contains(&String::from("ota.refusal-canary.task.publish"))
        );
        assert!(
            first
                .projection
                .merge_check_ids
                .contains(&String::from("ota.refusal-canary.workflow.release"))
        );
        assert!(
            first
                .rendered
                .contains("actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5")
        );
        assert!(
            first
                .rendered
                .contains("ota-run/setup@493cba84bf7f7c11c9e8e996d832d93a89c62184")
        );
        assert!(
            first
                .rendered
                .contains("actions/setup-go@924ae3a1cded613372ab5595356fb5720e22ba16")
        );
        assert!(first.rendered.contains("go-version: '1.26'"));
        assert!(
            first
                .rendered
                .contains("ota up --workflow 'verify' --mode native --agent --dry-run --json")
        );
        assert!(
            first
                .rendered
                .contains("ota receipt --workflow 'verify' --mode native --archive --json")
        );
        assert!(!first.rendered.contains("ota proof runtime"));
        let _: serde_yaml::Value = serde_yaml::from_str(&first.rendered)
            .expect("the rendered GitHub workflow must be valid YAML");
        assert_eq!(
            managed_projection_identity(&first.rendered),
            Some(first.projection.identity.as_str())
        );
        let binding = caller_projection_binding(
            &format!(
                "jobs:\n  ota:\n    uses: ./.github/workflows/ota-governance.yml\n    with:\n      ota_projection_identity: {}\n      ota_target_os: {}\n",
                first.projection.identity, first.projection.target_os
            ),
            Path::new(".github/workflows/ota-governance.yml"),
            Path::new("."),
            &first.projection.identity,
            &first.projection.target_os,
        )
        .expect("caller should bind the generated projection");
        assert_eq!(binding.jobs.len(), 1);
        assert_eq!(binding.jobs[0].id, "ota");
        assert_eq!(
            binding.jobs[0].projection_identity,
            first.projection.identity
        );
        assert!(caller_projection_binding(
            &format!(
                "# uses: ./.github/workflows/ota-governance.yml\n# ota_projection_identity: {}\n",
                first.projection.identity
            ),
            Path::new(".github/workflows/ota-governance.yml"),
            Path::new("."),
            &first.projection.identity,
            &first.projection.target_os,
        )
        .is_none());

        let proof_contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: proof-fixture
tasks:
  verify:
    run: echo verify
workflows:
  default: verify
  verify:
    run:
      task: verify
    proof:
      claim: bounded
"#,
        )
        .expect("proof fixture contract should parse");
        let proof_projection = render_github_projection(
            &proof_contract,
            "verify",
            "ubuntu-latest",
            "native",
            "linux",
        )
        .expect("proof projection should render");
        assert!(proof_projection.projection.proof_required);
        assert!(
            proof_projection
                .rendered
                .contains("ota proof runtime --workflow 'verify' --mode native --archive --json")
        );
        assert!(
            !proof_projection
                .rendered
                .contains("ota up --workflow 'verify' --mode native --agent --json")
        );
    }

    #[test]
    fn github_go_version_selector_uses_the_declared_lower_bound() {
        assert_eq!(
            github_go_version_spec(">=1.26,<1.27").expect("range should project"),
            "1.26"
        );
        assert!(github_go_version_spec(">=1.26,not-a-range").is_err());
        assert!(github_go_version_spec("stable").is_err());
    }

    #[test]
    fn github_node_corepack_toolchain_preserves_the_contract_selector() {
        let rendered = github_toolchain_setup_steps(&[CiProjectionToolchain {
            name: "node".to_string(),
            source: "corepack".to_string(),
            version: "^22.12.0".to_string(),
        }])
        .expect("Node/Corepack should project");
        assert!(rendered.contains("actions/setup-node@a0853c24544627f65ddf259abe73b1d18a591444"));
        assert!(rendered.contains("node-version: '^22.12.0'"));
        assert!(github_node_version_spec(" ").is_err());
        assert!(
            github_toolchain_setup_steps(&[CiProjectionToolchain {
                name: "node".to_string(),
                source: "mise".to_string(),
                version: "22".to_string(),
            }])
            .is_err()
        );
    }

    #[test]
    fn github_ruby_toolchain_preserves_the_contract_selector() {
        let rendered = github_toolchain_setup_steps(&[CiProjectionToolchain {
            name: "ruby".to_string(),
            source: "ruby".to_string(),
            version: "3.3.11".to_string(),
        }])
        .expect("Ruby should project");
        assert!(rendered.contains("ruby/setup-ruby@003a5c4d8d6321bd302e38f6f0ec593f77f06600"));
        assert!(rendered.contains("ruby-version: '3.3.11'"));
        assert!(github_ruby_version_spec(" ").is_err());
    }
}
