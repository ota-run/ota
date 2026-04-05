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
    #[serde(default)]
    pub adapter_bootstrap: BTreeMap<String, PolicyAdapterBootstrapRule>,
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
    pub source_config: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(default)]
    pub approved_versions: Vec<String>,
    #[serde(default)]
    pub platforms: BTreeMap<String, PolicyPlatformProvisioningRule>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyPlatformProvisioningRule {
    pub source: String,
    #[serde(default)]
    pub source_config: Option<BTreeMap<String, serde_yaml::Value>>,
    #[serde(default)]
    pub approved_versions: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyAdapterBootstrapRule {
    pub source: String,
    #[serde(default)]
    pub approved_versions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdapterBootstrapDecision {
    pub name: String,
    pub source: String,
    pub approved_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdapterBootstrapPlanEntry {
    pub name: String,
    pub source: Option<String>,
    pub approved_version: Option<String>,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct AdapterBootstrapPlan {
    pub allowed: Vec<AdapterBootstrapPlanEntry>,
    pub blocked: Vec<AdapterBootstrapPlanEntry>,
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
    pub source_config: Option<BTreeMap<String, serde_yaml::Value>>,
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
    pub source_config: Option<BTreeMap<String, serde_yaml::Value>>,
    pub approved_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvisioningBackendRequest {
    pub actions: Vec<ProvisioningAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvisioningPlanEntry {
    pub kind: ProvisioningTargetKind,
    pub name: String,
    pub requested_version: String,
    pub source: Option<String>,
    pub source_config: Option<BTreeMap<String, serde_yaml::Value>>,
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
        validate_source_rules(
            &self.policies.provisioning,
            "policy-backed provisioning source",
        )?;
        validate_platform_rules(
            &self.policies.provisioning,
            "policy-backed provisioning platform source",
        )?;
        validate_source_rules(
            &self.policies.adapter_bootstrap,
            "policy-backed adapter bootstrap source",
        )?;

        Ok(())
    }

    pub fn resolve_provisioning(
        &self,
        kind: ProvisioningTargetKind,
        name: &str,
        requested_version: &str,
    ) -> Result<Option<ProvisioningDecision>, String> {
        self.resolve_provisioning_for_os(current_os(), kind, name, requested_version)
    }

    pub fn resolve_provisioning_for_os(
        &self,
        os: &str,
        kind: ProvisioningTargetKind,
        name: &str,
        requested_version: &str,
    ) -> Result<Option<ProvisioningDecision>, String> {
        let Some(rule) = self.policies.provisioning.get(name) else {
            return Ok(None);
        };

        let rule = effective_provisioning_rule(rule, os);

        if rule.source().trim().is_empty() {
            return Err(format!(
                "policy-backed provisioning source `{name}` must not be empty"
            ));
        }

        if !rule.approved_versions().is_empty()
            && !rule
                .approved_versions()
                .iter()
                .any(|version| version == requested_version)
        {
            return Err(format!(
                "{} `{name}` version `{requested_version}` is not approved by policy; expected one of: {}",
                kind.as_str(),
                rule.approved_versions().join(", ")
            ));
        }

        Ok(Some(ProvisioningDecision {
            kind,
            name: name.to_string(),
            requested_version: requested_version.to_string(),
            source: rule.source().to_string(),
            source_config: rule.source_config().cloned(),
            approved_version: rule
                .approved_versions()
                .iter()
                .find(|version| version.as_str() == requested_version)
                .cloned(),
        }))
    }

    pub fn resolve_adapter_bootstrap(
        &self,
        adapter: &str,
    ) -> Result<Option<AdapterBootstrapDecision>, String> {
        let Some(rule) = self.policies.adapter_bootstrap.get(adapter) else {
            return Ok(None);
        };

        if rule.source.trim().is_empty() {
            return Err(format!(
                "policy-backed adapter bootstrap source `{adapter}` must not be empty"
            ));
        }

        Ok(Some(AdapterBootstrapDecision {
            name: adapter.to_string(),
            source: rule.source.clone(),
            approved_version: rule.approved_versions.first().cloned(),
        }))
    }

    pub fn adapter_bootstrap_plan(&self, missing_adapters: &[&str]) -> AdapterBootstrapPlan {
        let mut plan = AdapterBootstrapPlan::default();

        for adapter in missing_adapters {
            match self.resolve_adapter_bootstrap(adapter) {
                Ok(Some(decision)) => plan.allowed.push(AdapterBootstrapPlanEntry {
                    name: decision.name,
                    source: Some(decision.source),
                    approved_version: decision.approved_version,
                    blocked_reason: None,
                }),
                Ok(None) => plan.blocked.push(AdapterBootstrapPlanEntry {
                    name: (*adapter).to_string(),
                    source: None,
                    approved_version: None,
                    blocked_reason: Some(format!(
                        "no approved adapter bootstrap source declared for `{adapter}`"
                    )),
                }),
                Err(message) => plan.blocked.push(AdapterBootstrapPlanEntry {
                    name: (*adapter).to_string(),
                    source: None,
                    approved_version: None,
                    blocked_reason: Some(message),
                }),
            }
        }

        plan
    }

    pub fn adapter_bootstrap_backend_request(
        &self,
        missing_adapters: &[&str],
    ) -> ProvisioningBackendRequest {
        let plan = self.adapter_bootstrap_plan(missing_adapters);
        let actions = plan
            .allowed
            .into_iter()
            .map(|entry| ProvisioningAction {
                kind: ProvisioningActionKind::SelectSource,
                target_kind: ProvisioningTargetKind::Tool,
                name: entry.name,
                requested_version: entry
                    .approved_version
                    .clone()
                    .unwrap_or_else(|| String::from("latest")),
                source: entry.source.unwrap_or_default(),
                source_config: None,
                approved_version: entry.approved_version,
            })
            .collect();

        ProvisioningBackendRequest { actions }
    }

    pub fn provisioning_plan(&self, contract: &crate::schema::Contract) -> ProvisioningPlan {
        self.provisioning_plan_for_os(current_os(), contract)
    }

    pub fn provisioning_plan_for_os(
        &self,
        os: &str,
        contract: &crate::schema::Contract,
    ) -> ProvisioningPlan {
        let mut plan = ProvisioningPlan::default();

        for (name, requirement) in &contract.runtimes {
            Self::push_plan_entry(
                &mut plan,
                ProvisioningTargetKind::Runtime,
                name,
                requirement.version(),
                self.resolve_provisioning_for_os(
                    os,
                    ProvisioningTargetKind::Runtime,
                    name,
                    requirement.version(),
                ),
            );
        }

        for (name, requirement) in &contract.tools {
            Self::push_plan_entry(
                &mut plan,
                ProvisioningTargetKind::Tool,
                name,
                requirement.version(),
                self.resolve_provisioning_for_os(
                    os,
                    ProvisioningTargetKind::Tool,
                    name,
                    requirement.version(),
                ),
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
                    source_config: entry.source_config.clone(),
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
        self.selected_provisioning_sources_for_os(current_os(), contract)
    }

    pub fn selected_provisioning_sources_for_os(
        &self,
        os: &str,
        contract: &crate::schema::Contract,
    ) -> Vec<ProvisioningDecision> {
        self.selected_provisioning_actions_for_os(os, contract)
            .into_iter()
            .map(|action| ProvisioningDecision {
                kind: action.target_kind,
                name: action.name,
                requested_version: action.requested_version,
                source: action.source,
                source_config: action.source_config,
                approved_version: action.approved_version,
            })
            .collect()
    }

    pub fn selected_provisioning_actions(
        &self,
        contract: &crate::schema::Contract,
    ) -> Vec<ProvisioningAction> {
        self.selected_provisioning_actions_for_os(current_os(), contract)
    }

    pub fn selected_provisioning_actions_for_os(
        &self,
        os: &str,
        contract: &crate::schema::Contract,
    ) -> Vec<ProvisioningAction> {
        self.provisioning_plan_for_os(os, contract).actions
    }

    pub fn provisioning_backend_request(
        &self,
        contract: &crate::schema::Contract,
    ) -> ProvisioningBackendRequest {
        self.provisioning_backend_request_for_os(current_os(), contract)
    }

    pub fn provisioning_backend_request_for_os(
        &self,
        os: &str,
        contract: &crate::schema::Contract,
    ) -> ProvisioningBackendRequest {
        ProvisioningBackendRequest {
            actions: self.selected_provisioning_actions_for_os(os, contract),
        }
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
                source_config: decision.source_config.clone(),
                approved_version: decision.approved_version,
                blocked_reason: None,
            }),
            Ok(None) => plan.blocked.push(ProvisioningPlanEntry {
                kind,
                name: name.to_string(),
                requested_version: requested_version.to_string(),
                source: None,
                source_config: None,
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
                source_config: None,
                approved_version: None,
                blocked_reason: Some(message),
            }),
        }
    }
}

fn validate_source_rules<T>(rules: &BTreeMap<String, T>, label: &str) -> Result<(), String>
where
    T: SourceRule,
{
    for (name, rule) in rules {
        if rule.source().trim().is_empty() {
            return Err(format!("{label} `{name}` must not be empty"));
        }

        if rule
            .approved_versions()
            .iter()
            .any(|version| version.trim().is_empty())
        {
            return Err(format!(
                "{label} `{name}` must not contain empty approved versions"
            ));
        }
    }

    Ok(())
}

fn validate_platform_rules(
    rules: &BTreeMap<String, PolicyProvisioningRule>,
    label: &str,
) -> Result<(), String> {
    for (name, rule) in rules {
        for (platform, platform_rule) in &rule.platforms {
            if !matches!(platform.as_str(), "linux" | "macos" | "windows") {
                return Err(format!(
                    "{label} `{name}` has unsupported platform `{platform}`; expected one of: linux, macos, windows"
                ));
            }
            if platform_rule.source.trim().is_empty() {
                return Err(format!(
                    "{label} `{name}` platform `{platform}` must not be empty"
                ));
            }
            if platform_rule
                .approved_versions
                .iter()
                .any(|version| version.trim().is_empty())
            {
                return Err(format!(
                    "{label} `{name}` platform `{platform}` must not contain empty approved versions"
                ));
            }
        }
    }

    Ok(())
}

fn effective_provisioning_rule<'a>(
    rule: &'a PolicyProvisioningRule,
    os: &str,
) -> &'a dyn ProvisioningSourceRule {
    rule.platforms
        .get(os)
        .map(|platform| platform as &dyn ProvisioningSourceRule)
        .unwrap_or(rule)
}

#[cfg(target_os = "windows")]
fn current_os() -> &'static str {
    "windows"
}

#[cfg(target_os = "macos")]
fn current_os() -> &'static str {
    "macos"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn current_os() -> &'static str {
    "linux"
}

trait SourceRule {
    fn source(&self) -> &str;
    fn approved_versions(&self) -> &[String];
}

impl SourceRule for PolicyProvisioningRule {
    fn source(&self) -> &str {
        &self.source
    }

    fn approved_versions(&self) -> &[String] {
        &self.approved_versions
    }
}

impl SourceRule for PolicyPlatformProvisioningRule {
    fn source(&self) -> &str {
        &self.source
    }

    fn approved_versions(&self) -> &[String] {
        &self.approved_versions
    }
}

trait ProvisioningSourceRule: SourceRule {
    fn source_config(&self) -> Option<&BTreeMap<String, serde_yaml::Value>>;
}

impl ProvisioningSourceRule for PolicyProvisioningRule {
    fn source_config(&self) -> Option<&BTreeMap<String, serde_yaml::Value>> {
        self.source_config.as_ref()
    }
}

impl ProvisioningSourceRule for PolicyPlatformProvisioningRule {
    fn source_config(&self) -> Option<&BTreeMap<String, serde_yaml::Value>> {
        self.source_config.as_ref()
    }
}

impl SourceRule for PolicyAdapterBootstrapRule {
    fn source(&self) -> &str {
        &self.source
    }

    fn approved_versions(&self) -> &[String] {
        &self.approved_versions
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
    pack.validate()
        .map_err(|message| LoadPolicyPackError::Validate {
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

    use super::{
        OrgPolicyPack, ProvisioningActionKind, ProvisioningTargetKind, load_org_policy_pack_auto,
    };

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
  adapter_bootstrap:
    mise:
      source: approved-manager
      approved_versions:
        - "2024.12"
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
        assert_eq!(
            policy.policies.adapter_bootstrap["mise"].source,
            "approved-manager"
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
    fn rejects_empty_policy_adapter_bootstrap_source() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  adapter_bootstrap:
    mise:
      source: " "
"#,
        )
        .unwrap();

        let error = policy.validate().unwrap_err();
        assert!(error.contains("must not be empty"));
    }

    #[test]
    fn resolves_approved_policy_adapter_bootstrap_source() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  adapter_bootstrap:
    mise:
      source: approved-manager
      approved_versions:
        - "2024.12"
"#,
        )
        .unwrap();

        let decision = policy.resolve_adapter_bootstrap("mise").unwrap().unwrap();

        assert_eq!(decision.name, "mise");
        assert_eq!(decision.source, "approved-manager");
        assert_eq!(decision.approved_version.as_deref(), Some("2024.12"));
    }

    #[test]
    fn builds_adapter_bootstrap_plan_for_missing_adapters() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  adapter_bootstrap:
    mise:
      source: approved-manager
      approved_versions:
        - "2024.12"
"#,
        )
        .unwrap();

        let plan = policy.adapter_bootstrap_plan(&["mise", "brew"]);
        assert_eq!(plan.allowed.len(), 1);
        assert_eq!(plan.blocked.len(), 1);
        assert_eq!(plan.allowed[0].name, "mise");
        assert_eq!(plan.allowed[0].source.as_deref(), Some("approved-manager"));
        assert_eq!(plan.allowed[0].approved_version.as_deref(), Some("2024.12"));
        assert_eq!(plan.blocked[0].name, "brew");
        assert!(
            plan.blocked[0]
                .blocked_reason
                .as_ref()
                .unwrap()
                .contains("no approved adapter bootstrap source")
        );
    }

    #[test]
    fn builds_adapter_bootstrap_backend_request_for_allowed_adapters() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  adapter_bootstrap:
    mise:
      source: approved-manager
      approved_versions:
        - "2024.12"
"#,
        )
        .unwrap();

        let request = policy.adapter_bootstrap_backend_request(&["mise"]);
        assert_eq!(request.actions.len(), 1);
        assert_eq!(
            request.actions[0].kind,
            ProvisioningActionKind::SelectSource
        );
        assert_eq!(request.actions[0].target_kind, ProvisioningTargetKind::Tool);
        assert_eq!(request.actions[0].name, "mise");
        assert_eq!(request.actions[0].source, "approved-manager");
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
    fn resolves_platform_specific_policy_provisioning_source() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
      platforms:
        linux:
          source: apt
          approved_versions:
            - "22"
        windows:
          source: choco
          approved_versions:
            - "22"
"#,
        )
        .unwrap();

        let macos = policy
            .resolve_provisioning_for_os("macos", ProvisioningTargetKind::Tool, "node", "22")
            .unwrap()
            .unwrap();
        let linux = policy
            .resolve_provisioning_for_os("linux", ProvisioningTargetKind::Tool, "node", "22")
            .unwrap()
            .unwrap();
        let windows = policy
            .resolve_provisioning_for_os("windows", ProvisioningTargetKind::Tool, "node", "22")
            .unwrap()
            .unwrap();

        assert_eq!(macos.source, "brew");
        assert_eq!(linux.source, "apt");
        assert_eq!(windows.source, "choco");
    }

    #[test]
    fn resolves_approved_policy_provisioning_choco_feed() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    node:
      source: choco
      source_config:
        feed: internal-choco
      approved_versions:
        - "22"
"#,
        )
        .unwrap();

        let decision = policy
            .resolve_provisioning(ProvisioningTargetKind::Tool, "node", "22")
            .unwrap()
            .unwrap();

        assert_eq!(decision.source, "choco");
        assert_eq!(
            decision
                .source_config
                .as_ref()
                .and_then(|config| config.get("feed"))
                .and_then(|value| value.as_str()),
            Some("internal-choco")
        );
    }

    #[test]
    fn rejects_unsupported_policy_provisioning_platform() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
      platforms:
        mac:
          source: choco
          approved_versions:
            - "22"
"#,
        )
        .unwrap();

        let error = policy.validate().unwrap_err();
        assert!(error.contains("unsupported platform"));
    }

    #[test]
    fn preserves_open_policy_provisioning_source_config() {
        let policy: OrgPolicyPack = serde_yaml::from_str(
            r#"
policies:
  provisioning:
    java:
      source: brew
      source_config:
        channel: stable
        mirror: internal-brew
      approved_versions:
        - "21"
"#,
        )
        .unwrap();

        let decision = policy
            .resolve_provisioning(ProvisioningTargetKind::Runtime, "java", "21")
            .unwrap()
            .unwrap();

        assert_eq!(decision.source, "brew");
        assert_eq!(
            decision
                .source_config
                .as_ref()
                .and_then(|config| config.get("channel"))
                .and_then(|value| value.as_str()),
            Some("stable")
        );
        assert_eq!(
            decision
                .source_config
                .as_ref()
                .and_then(|config| config.get("mirror"))
                .and_then(|value| value.as_str()),
            Some("internal-brew")
        );
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
        assert!(
            plan.blocked[0]
                .blocked_reason
                .as_ref()
                .unwrap()
                .contains("no approved provisioning source")
        );
        assert_eq!(plan.blocked[1].name, "maven");
        assert!(
            plan.blocked[1]
                .blocked_reason
                .as_ref()
                .unwrap()
                .contains("no approved provisioning source")
        );
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

    #[test]
    fn builds_provisioning_backend_request_from_allowed_targets() {
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

        let request = policy.provisioning_backend_request(&contract);
        assert_eq!(request.actions.len(), 1);
        assert_eq!(
            request.actions[0].kind,
            ProvisioningActionKind::SelectSource
        );
        assert_eq!(
            request.actions[0].target_kind,
            ProvisioningTargetKind::Runtime
        );
        assert_eq!(request.actions[0].name, "java");
        assert_eq!(request.actions[0].source, "org-mirror");
    }
}
