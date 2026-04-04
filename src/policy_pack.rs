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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum LoadPolicyPackError {
    #[error("failed to read policy pack `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse policy pack `{path}`: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("failed to validate policy pack `{path}`: {message}")]
    Validate { path: String, message: String },
}

impl LoadPolicyPackError {
    pub fn path(&self) -> &str {
        match self {
            Self::Read { path, .. } | Self::Parse { path, .. } | Self::Validate { path, .. } => {
                path
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrgPolicyPack {
    pub policies: PolicyRules,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRules {
    #[serde(default)]
    pub required_sections: Vec<String>,
    #[serde(default)]
    pub required_files: Vec<String>,
    #[serde(default)]
    pub strict_versions: bool,
    #[serde(default)]
    pub agent: Option<PolicyAgentRules>,
    #[serde(default)]
    pub exports: Option<PolicyExportsRules>,
    #[serde(default)]
    pub provisioning: BTreeMap<String, PolicyProvisioningRule>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyAgentRules {
    #[serde(default)]
    pub require_safe_tasks: bool,
    #[serde(default)]
    pub require_writable_paths: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyExportsRules {
    #[serde(default)]
    pub require_agents_md: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyProvisioningRule {
    pub source: String,
    #[serde(default)]
    pub approved_versions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningTargetKind {
    Runtime,
    Tool,
}

impl ProvisioningTargetKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Tool => "tool",
        }
    }
}

impl std::fmt::Display for ProvisioningTargetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvisioningDecision {
    pub kind: ProvisioningTargetKind,
    pub name: String,
    pub requested_version: String,
    pub source: String,
    pub approved_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningActionKind {
    SelectSource,
    Install,
    Verify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvisioningAction {
    pub kind: ProvisioningActionKind,
    pub target_kind: ProvisioningTargetKind,
    pub name: String,
    pub requested_version: String,
    pub source: String,
    pub approved_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvisioningPlanEntry {
    pub kind: ProvisioningTargetKind,
    pub name: String,
    pub requested_version: String,
    pub source: Option<String>,
    pub approved_version: Option<String>,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct ProvisioningPlan {
    pub allowed: Vec<ProvisioningPlanEntry>,
    pub blocked: Vec<ProvisioningPlanEntry>,
    pub actions: Vec<ProvisioningAction>,
}

impl OrgPolicyPack {
    pub fn missing_required_sections(&self, contract: &crate::schema::Contract) -> Vec<String> {
        self.policies
            .required_sections
            .iter()
            .filter(|section| !section_present(contract, section))
            .cloned()
            .collect()
    }

    pub fn missing_required_files(&self, contract_root: &Path) -> Vec<String> {
        self.policies
            .required_files
            .iter()
            .filter(|file| !contract_root.join(file).is_file())
            .cloned()
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        for (name, rule) in &self.policies.provisioning {
            if rule.source.trim().is_empty() {
                return Err(format!(
                    "policy-backed provisioning source `{name}` must not be empty"
                ));
            }

            if rule.approved_versions.iter().any(|version| version.trim().is_empty()) {
                return Err(format!(
                    "policy-backed provisioning source `{name}` must not contain empty approved versions"
                ));
            }
        }

        Ok(())
    }

    pub fn resolve_provisioning(
        &self,
        kind: ProvisioningTargetKind,
        name: &str,
        requested_version: &str,
    ) -> Result<Option<ProvisioningDecision>, String> {
        let Some(rule) = self.policies.provisioning.get(name) else {
            return Ok(None);
        };

        if rule.source.trim().is_empty() {
            return Err(format!(
                "policy-backed provisioning source `{name}` must not be empty"
            ));
        }

        if !rule.approved_versions.is_empty()
            && !rule.approved_versions.iter().any(|version| version == requested_version)
        {
            return Err(format!(
                "{} `{name}` version `{requested_version}` is not approved by policy; expected one of: {}",
                kind.as_str(),
                rule.approved_versions.join(", ")
            ));
        }

        Ok(Some(ProvisioningDecision {
            kind,
            name: name.to_string(),
            requested_version: requested_version.to_string(),
            source: rule.source.clone(),
            approved_version: rule
                .approved_versions
                .iter()
                .find(|version| version.as_str() == requested_version)
                .cloned(),
        }))
    }

    pub fn provisioning_plan(&self, contract: &crate::schema::Contract) -> ProvisioningPlan {
        let mut plan = ProvisioningPlan::default();

        for (name, requirement) in &contract.runtimes {
            Self::push_plan_entry(
                &mut plan,
                ProvisioningTargetKind::Runtime,
                name,
                requirement.version(),
                self.resolve_provisioning(ProvisioningTargetKind::Runtime, name, requirement.version()),
            );
        }

        for (name, requirement) in &contract.tools {
            Self::push_plan_entry(
                &mut plan,
                ProvisioningTargetKind::Tool,
                name,
                requirement.version(),
                self.resolve_provisioning(ProvisioningTargetKind::Tool, name, requirement.version()),
            );
        }

        plan.actions = plan
            .allowed
            .iter()
            .filter_map(|entry| {
                entry.source.as_ref().map(|source| ProvisioningAction {
                    kind: ProvisioningActionKind::SelectSource,
                    target_kind: entry.kind,
                    name: entry.name.clone(),
                    requested_version: entry.requested_version.clone(),
                    source: source.clone(),
                    approved_version: entry.approved_version.clone(),
                })
            })
            .collect();

        plan
    }

    pub fn selected_provisioning_sources(
        &self,
        contract: &crate::schema::Contract,
    ) -> Vec<ProvisioningDecision> {
        self.selected_provisioning_actions(contract)
            .into_iter()
            .map(|action| ProvisioningDecision {
                kind: action.target_kind,
                name: action.name,
                requested_version: action.requested_version,
                source: action.source,
                approved_version: action.approved_version,
            })
            .collect()
    }

    pub fn selected_provisioning_actions(
        &self,
        contract: &crate::schema::Contract,
    ) -> Vec<ProvisioningAction> {
        self.provisioning_plan(contract).actions
    }

    fn push_plan_entry(
        plan: &mut ProvisioningPlan,
        kind: ProvisioningTargetKind,
        name: &str,
        requested_version: &str,
        resolution: Result<Option<ProvisioningDecision>, String>,
    ) {
        match resolution {
            Ok(Some(decision)) => plan.allowed.push(ProvisioningPlanEntry {
                kind,
                name: name.to_string(),
                requested_version: requested_version.to_string(),
                source: Some(decision.source),
                approved_version: decision.approved_version,
                blocked_reason: None,
            }),
            Ok(None) => plan.blocked.push(ProvisioningPlanEntry {
                kind,
                name: name.to_string(),
                requested_version: requested_version.to_string(),
                source: None,
                approved_version: None,
                blocked_reason: Some(format!(
                    "no approved provisioning source declared for {kind} `{name}`"
                )),
            }),
            Err(message) => plan.blocked.push(ProvisioningPlanEntry {
                kind,
                name: name.to_string(),
                requested_version: requested_version.to_string(),
                source: None,
                approved_version: None,
                blocked_reason: Some(message),
            }),
        }
    }
}

pub fn load_org_policy_pack_auto(
    contract_path: &Path,
) -> Result<Option<(OrgPolicyPack, PathBuf)>, LoadPolicyPackError> {
    let Some(policy_path) = find_org_policy_pack_path(contract_path) else {
        return Ok(None);
    };

    let contents =
        fs::read_to_string(&policy_path).map_err(|source| LoadPolicyPackError::Read {
            path: policy_path.display().to_string(),
            source,
        })?;

    let pack: OrgPolicyPack =
        serde_yaml::from_str(&contents).map_err(|source| LoadPolicyPackError::Parse {
            path: policy_path.display().to_string(),
            source,
        })?;
    pack.validate().map_err(|message| LoadPolicyPackError::Validate {
        path: policy_path.display().to_string(),
        message,
    })?;

    Ok(Some((pack, policy_path)))
}

fn find_org_policy_pack_path(contract_path: &Path) -> Option<PathBuf> {
    let mut current = Some(contract_path.parent().unwrap_or_else(|| Path::new(".")));

    while let Some(dir) = current {
        let candidate = dir.join(".ota").join("org-policy.yaml");
        if candidate.is_file() {
            return Some(candidate);
        }
        current = dir.parent();
    }

    None
}

fn section_present(contract: &crate::schema::Contract, section: &str) -> bool {
    match section {
        "version" => true,
        "project" => true,
        "workspace" => contract.workspace.is_some(),
        "execution" => contract.execution.is_some(),
        "runtimes" => !contract.runtimes.is_empty(),
        "tools" => !contract.tools.is_empty(),
        "env" => !contract.env.is_empty(),
        "services" => !contract.services.is_empty(),
        "tasks" => !contract.tasks.is_empty(),
        "checks" => !contract.checks.is_empty(),
        "agent" => contract.agent.is_some(),
        "exports" => !contract.exports.is_empty(),
        "policies" => !contract.policies.is_empty(),
        "metadata" => !contract.metadata.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{OrgPolicyPack, ProvisioningActionKind, ProvisioningTargetKind, load_org_policy_pack_auto};

    fn write_contract(dir: &TempDir, body: &str) {
        fs::write(dir.path().join("ota.yaml"), body).unwrap();
    }

    #[test]
    fn loads_policy_pack_from_ancestor_ota_directory() {
        let fixture = TempDir::new().unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        write_contract(
            &fixture,
            r#"
version: 1
project:
  name: app
"#,
        );
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
"#,
        )
        .unwrap();

        let loaded = load_org_policy_pack_auto(&fixture.path().join("ota.yaml")).unwrap();

        assert!(loaded.is_some());
    }

    #[test]
    fn parses_policy_pack_shape_strictly() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  required_sections:
    - tasks
  required_files:
    - AGENTS.md
  strict_versions: true
  agent:
    require_safe_tasks: true
    require_writable_paths: true
  exports:
    require_agents_md: true
  provisioning:
    java:
      source: org-mirror
      approved_versions:
        - "22"
    maven:
      source: approved-manager
      approved_versions:
        - "3.9"
"#,
        )
        .unwrap();

        assert!(policy.policies.strict_versions);
        assert!(policy.policies.agent.as_ref().unwrap().require_safe_tasks);
        assert_eq!(
            policy.policies.required_files,
            vec![String::from("AGENTS.md")]
        );
        assert_eq!(policy.policies.provisioning["java"].source, "org-mirror");
        assert_eq!(
            policy.policies.provisioning["maven"].approved_versions,
            vec![String::from("3.9")]
        );
    }

    #[test]
    fn rejects_empty_policy_provisioning_source() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: " "
"#,
        )
        .unwrap();

        let error = policy.validate().unwrap_err();
        assert!(error.contains("must not be empty"));
    }

    #[test]
    fn resolves_approved_policy_provisioning_source() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      approved_versions:
        - "22"
"#,
        )
        .unwrap();

        let decision = policy
            .resolve_provisioning(ProvisioningTargetKind::Runtime, "java", "22")
            .unwrap()
            .unwrap();

        assert_eq!(decision.source, "org-mirror");
        assert_eq!(decision.approved_version.as_deref(), Some("22"));
    }

    #[test]
    fn resolves_approved_policy_provisioning_tool_source() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    maven:
      source: approved-manager
      approved_versions:
        - "3.9"
"#,
        )
        .unwrap();

        let decision = policy
            .resolve_provisioning(ProvisioningTargetKind::Tool, "maven", "3.9")
            .unwrap()
            .unwrap();

        assert_eq!(decision.source, "approved-manager");
        assert_eq!(decision.approved_version.as_deref(), Some("3.9"));
    }

    #[test]
    fn rejects_unapproved_policy_provisioning_version() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      approved_versions:
        - "22"
"#,
        )
        .unwrap();

        let error = policy
            .resolve_provisioning(ProvisioningTargetKind::Runtime, "java", "21")
            .unwrap_err();

        assert!(error.contains("is not approved by policy"));
    }

    #[test]
    fn builds_provisioning_plan_for_contract_targets() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      approved_versions:
        - "22"
"#,
        )
        .unwrap();
        let contract: crate::schema::Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
runtimes:
  java: "22"
  python: "3.12"
tools:
  maven: "3.9"
"#,
        )
        .unwrap();

        let plan = policy.provisioning_plan(&contract);
        assert_eq!(plan.allowed.len(), 1);
        assert_eq!(plan.blocked.len(), 2);
        assert_eq!(plan.allowed[0].name, "java");
        assert_eq!(plan.allowed[0].source.as_deref(), Some("org-mirror"));
        assert_eq!(plan.blocked[0].name, "python");
        assert!(plan.blocked[0].blocked_reason.as_ref().unwrap().contains("no approved provisioning source"));
        assert_eq!(plan.blocked[1].name, "maven");
        assert!(plan.blocked[1].blocked_reason.as_ref().unwrap().contains("no approved provisioning source"));
    }

    #[test]
    fn selects_provisioning_sources_for_allowed_targets() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      approved_versions:
        - "22"
"#,
        )
        .unwrap();
        let contract: crate::schema::Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
runtimes:
  java: "22"
tools:
  maven: "3.9"
"#,
        )
        .unwrap();

        let selections = policy.selected_provisioning_sources(&contract);
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].name, "java");
        assert_eq!(selections[0].source, "org-mirror");
    }

    #[test]
    fn builds_provisioning_actions_for_allowed_targets() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: org-mirror
      approved_versions:
        - "22"
"#,
        )
        .unwrap();
        let contract: crate::schema::Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
runtimes:
  java: "22"
"#,
        )
        .unwrap();

        let actions = policy.selected_provisioning_actions(&contract);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].kind, ProvisioningActionKind::SelectSource);
        assert_eq!(actions[0].target_kind, ProvisioningTargetKind::Runtime);
        assert_eq!(actions[0].name, "java");
        assert_eq!(actions[0].source, "org-mirror");
    }
}
