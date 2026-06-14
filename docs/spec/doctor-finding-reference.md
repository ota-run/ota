<!--
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Doctor Finding Reference

Status: generated reference.

This document is generated from the shipped doctor and workspace finding identity catalogs in
`src/doctor.rs` and `src/workspace.rs`.
Do not edit the rows by hand; update the catalog and rerun the sync tests.

`owner_surface` and `provenance_key_surface` use `|` when the emitted value depends on the
selected execution target instead of one fixed owner or provenance lane. `omitted` means the
field is not emitted for that finding family.

## Contract

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_AGENT_BOUNDARY_UNREVIEWED` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_AGENT_BOOTSTRAP_UNPINNED` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_DEPENDENCY_HYDRATION` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_EXTERNAL_STATE` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_NETWORK` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DEPENDS_ON_BOUNDARY` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_FILES_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_ENV_FILES_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROFILES_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROJECT_NAME_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_RENDERED_ENV_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_ISOLATED_YARN_RELEASE_SHADOW` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_LEGACY_NODE_RUNTIME_TOOL_SPLIT` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_LEGACY_STANDALONE_POETRY` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_LIKELY_UNUSED_ATTACHMENT` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_REPLACEABLE_BAKE_FILE_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_REPLACEABLE_COMPOSE_ENV_FILE_OWNERSHIP` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_ENV_CHECK` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_ENV_MUTATION` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_FILE_CHECK` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_SENSITIVE_AGENT_WRITABLE_PATH` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_SENSITIVE_WRITE_EXCEPTION` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_SERVICE_OPAQUE_SHELL_START` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_ADVISORY_TASK_MUTATES_MANAGED_ISOLATED_PATH` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACT_DRIFT` | `contract` | `repo_contract` | `repo_signals` |
| `OTA_CONTRACTLESS_REPO_CONTRACT_MISSING` | `contract` | `repo_contract` | `repo_signals` |
| `OTA_CONTRACTLESS_SIGNAL` | `contract` | `repo_signals` | `repo_signals` |
| `OTA_CONTRACTLESS_SIGNAL_INSPECTION_FAILED` | `contract` | `repo_signals` | `repo_signals` |
| `OTA_DEVCONTAINER_PACKAGE_MANAGER_DRIFT` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_DEVCONTAINER_RUNTIME_DRIFT` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_REPO_HYGIENE_GITIGNORE_UNREADABLE` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_REPO_HYGIENE_OTA_STATE_GITIGNORE` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_SELECTED_TASK_PATH_DEPENDENCY_HYDRATION` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_SELECTED_TASK_PATH_EXTERNAL_STATE` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_SELECTED_TASK_PATH_NETWORK_REQUIRED` | `contract` | `repo_contract` | `repo_contract` |
| `OTA_TASKS_MISSING` | `contract` | `repo_contract` | `repo_contract` |

## Execution

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_BACKEND_CLI_MISSING` | `execution` | `host` | `repo_contract` |
| `OTA_CHECK_FAILED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_CHECK_TIMED_OUT` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_CONTAINER_BACKEND_CLI_MISSING` | `execution` | `host` | `repo_contract` |
| `OTA_CONTAINER_BACKEND_UNAVAILABLE` | `execution` | `host` | `repo_contract` |
| `OTA_CONTAINER_DOCTOR_HOST_SCOPE_NOTE` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_CONTAINER_IMAGE_UNAVAILABLE` | `execution` | `container_target` | `repo_contract` |
| `OTA_CONTAINER_MODE_NOT_CONFIGURED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_CONTEXT_HOST_PLATFORM_UNSUPPORTED` | `execution` | `host` | `repo_contract` |
| `OTA_FILE_CHECK_FAILED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_FILE_CHECK_TIMED_OUT` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_LIFECYCLE_EPHEMERAL_ADVISORY` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_PROBE_FAILED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_PROBE_TIMED_OUT` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SIGNAL_PROBE_FAILED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SURFACE_READINESS_FAILED` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT` | `execution` | `repo_contract` | `repo_contract` |
| `OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE` | `execution` | `repo_contract` | `repo_contract` |

## Remote

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED` | `remote` | `remote_backend` | `repo_contract` |
| `OTA_REMOTE_CONTEXT_UNEXECUTABLE` | `remote` | `remote_backend` | `repo_contract` |
| `OTA_REMOTE_DOCTOR_HOST_SCOPE_NOTE` | `remote` | `remote_backend` | `repo_contract` |
| `OTA_REMOTE_DOCTOR_PARTIAL` | `remote` | `remote_backend` | `repo_contract` |
| `OTA_REMOTE_MODE_NOT_CONFIGURED` | `remote` | `repo_contract` | `repo_contract` |
| `OTA_REMOTE_TARGET_OS_UNDETERMINED` | `remote` | `remote_backend` | `repo_contract` |
| `OTA_REMOTE_TARGET_SUSPICIOUS` | `remote` | `remote_backend` | `repo_contract` |

## Service

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_SERVICE_CHECK_FAILED` | `service` | `service` | `repo_contract` |
| `OTA_SERVICE_CHECK_TIMED_OUT` | `service` | `service` | `repo_contract` |
| `OTA_SERVICE_READINESS_CONTEXT_UNEXECUTABLE` | `service` | `service` | `repo_contract` |
| `OTA_SERVICE_READINESS_FAILED` | `service` | `service` | `repo_contract` |
| `OTA_SERVICE_UNVERIFIABLE` | `service` | `service` | `repo_contract` |

## Environment

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_ENV_INVALID` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_ENV_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_ENV_SOURCE_INVALID_STRUCTURE` | `environment` | `repo_contract` | `repo_contract` |
| `OTA_ENV_SOURCE_KEY_COLLISION` | `environment` | `repo_contract` | `repo_contract` |
| `OTA_ENV_SOURCE_MISSING_REQUIRED` | `environment` | `repo_contract` | `repo_contract` |
| `OTA_ENV_SOURCE_PARSE_FAILED` | `environment` | `repo_contract` | `repo_contract` |
| `OTA_CONTRACTLESS_HOST_TOOL_AVAILABLE` | `environment` | `host` | `repo_signals` |
| `OTA_CONTRACTLESS_HOST_TOOL_MISSING` | `environment` | `host` | `repo_signals` |
| `OTA_NATIVE_PREREQUISITE_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_NATIVE_PREREQUISITE_TIMED_OUT` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_RUNTIME_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_RUNTIME_PROBE_FAILED` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_RUNTIME_VERSION_MISMATCH` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_RUNTIME_VERSION_UNPARSEABLE` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOLCHAIN_COMPONENT_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED` | `environment` | `host\|container_target\|remote_target` | `repo_signals` |
| `OTA_TOOLCHAIN_PROVIDER_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOLCHAIN_PROVIDER_PROBE_FAILED` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOLCHAIN_TARGET_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOL_ACTIVATION_PROVIDER_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOL_MISSING` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOL_PROBE_FAILED` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOL_VERSION_MISMATCH` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |
| `OTA_TOOL_VERSION_UNPARSEABLE` | `environment` | `host\|container_target\|remote_target` | `repo_contract` |

## Provisioning

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_CONTAINER_APT_INDEX_UNAVAILABLE` | `provisioning` | `container_target` | `org_policy` |
| `OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE` | `provisioning` | `container_target` | `org_policy` |
| `OTA_CONTAINER_APT_VERSION_UNAVAILABLE` | `provisioning` | `container_target` | `org_policy` |
| `OTA_CONTAINER_PROVISIONING_BACKEND_FAILED` | `provisioning` | `container_target` | `org_policy` |
| `OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE` | `provisioning` | `container_target` | `org_policy` |
| `OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE` | `provisioning` | `container_target` | `org_policy` |
| `OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE` | `provisioning` | `container_target` | `org_policy` |
| `OTA_HOST_PROVISIONING_BACKEND_FAILED` | `provisioning` | `host` | `org_policy` |
| `OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE` | `provisioning` | `host` | `org_policy` |
| `OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE` | `provisioning` | `host` | `org_policy` |
| `OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE` | `provisioning` | `host` | `org_policy` |
| `OTA_ADAPTER_BOOTSTRAP_FAILED` | `provisioning` | `repo_contract` | `repo_contract` |
| `OTA_REMOTE_APT_INDEX_UNAVAILABLE` | `provisioning` | `remote_target` | `org_policy` |
| `OTA_REMOTE_APT_PACKAGE_UNAVAILABLE` | `provisioning` | `remote_target` | `org_policy` |
| `OTA_REMOTE_APT_VERSION_UNAVAILABLE` | `provisioning` | `remote_target` | `org_policy` |
| `OTA_REMOTE_PROVISIONING_BACKEND_FAILED` | `provisioning` | `remote_target` | `org_policy` |
| `OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE` | `provisioning` | `remote_target` | `org_policy` |
| `OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE` | `provisioning` | `remote_target` | `org_policy` |
| `OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE` | `provisioning` | `remote_target` | `org_policy` |

## Policy

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_EFFECT_ALLOWED` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_EFFECT_DENIED` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_EFFECT_WARNED` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_BACKED_PROVISIONING_DECLARED` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_BACKED_VERSION_RULES_DECLARED` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_INSTALLED_VERSION_NONCOMPLIANT` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_PACK_INVALID` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_PACK_VIOLATION` | `policy` | `org_policy` | `org_policy` |
| `OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING` | `policy` | `org_policy` | `org_policy` |

## Workspace

| Code | Category | Owner Surface | Provenance Key Surface |
| --- | --- | --- | --- |
| `OTA_WORKSPACE_REPO_NOT_ACQUIRED` | `workspace` | `workspace_acquisition` | `omitted` |
| `OTA_WORKSPACE_REPO_CONTRACT_INVALID` | `workspace` | `repo_contract` | `omitted` |
| `OTA_WORKSPACE_REPO_CONTRACT_MISSING` | `workspace` | `repo_contract` | `omitted` |
| `OTA_WORKSPACE_REPO_CONTRACT_UNREADABLE` | `workspace` | `repo_contract` | `omitted` |
