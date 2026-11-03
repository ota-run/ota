# YAML Backend Rust API Migration

This Unreleased change replaces `serde_yaml` with `yaml_serde` 0.10.7
and `libyaml-rs` 0.3.0. The Cargo alias preserves Core's imports, not
downstream Rust type identity. No CLI, YAML contract, or JSON schema change
is intended.

## Rust API Compatibility

Rust callers matching `LoadContractError::Parse`, `LoadWorkspaceError::Parse`,
or `LoadPolicyPackError::Parse` against the old concrete error type, including
through downcasts, must use `yaml_serde::Error` instead. Callers passing or
constructing YAML values at these public boundaries must use
`yaml_serde::Value` instead of the old crate's value type:

- `schema::Contract::{exports, policies, metadata}`,
  `schema::ExtensionSpec::config`, and `schema::ToolAcquisitionSpec::source_config`.
- `workspace::WorkspaceContract::policies`.
- `policy_pack::{PolicyProvisioningRule, PolicyPlatformProvisioningRule,
  ProvisioningDecision, ProvisioningAction, ProvisioningPlanEntry}::source_config`.

For affected consumers, switch the dependency to `yaml_serde` 0.10.7, or
use the Cargo alias `serde_yaml = { package = "yaml_serde", version = "=0.10.7" }`
to retain import spelling. Values from the old crate are not interchangeable
with values from the new crate. Do not ship this as an undocumented compatible
patch. This requires a breaking release for the Rust library API.
