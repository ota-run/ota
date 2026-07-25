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

use serde_json::Value as JsonValue;

pub struct PublishedContractSchema {
    pub filename: &'static str,
    pub body: &'static str,
}

const CONTRACT_SCHEMA_JSON: &str = r########"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://dist.ota.run/spec/json-schemas/latest/contract.json",
  "title": "ota contract",
  "$defs": {
    "yamlValue": {
      "oneOf": [
        { "type": "string" },
        { "type": "number" },
        { "type": "integer" },
        { "type": "boolean" },
        {
          "type": "array",
          "items": { "$ref": "#/$defs/yamlValue" }
        },
        {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/yamlValue" }
        }
      ]
    },
    "stringMap": {
      "type": "object",
      "additionalProperties": { "type": "string" }
    },
    "stringArray": {
      "type": "array",
      "items": { "type": "string" }
    },
    "backend": {
      "enum": ["native", "container", "remote"]
    },
    "lifecycle": {
      "enum": ["persistent", "ephemeral"]
    },
    "project": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name"],
      "properties": {
        "name": { "type": "string" },
        "description": { "type": "string" },
        "type": { "type": "string" }
      }
    },
    "repoWorkspace": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "members"],
      "properties": {
        "type": { "const": "monorepo" },
        "members": { "$ref": "#/$defs/stringArray" }
      }
    },
    "runtimePlatformDetail": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "version": { "type": "string" },
        "provider": { "type": "string" },
        "distribution": { "type": "string" }
      }
    },
    "runtimeDetail": {
      "type": "object",
      "additionalProperties": false,
      "required": ["version"],
      "properties": {
        "version": { "type": "string" },
        "required": { "type": "boolean" },
        "only_on": { "$ref": "#/$defs/stringArray" },
        "provider": { "type": "string" },
        "distribution": { "type": "string" },
        "platforms": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/runtimePlatformDetail" }
        }
      }
    },
    "runtimeRequirement": {
      "oneOf": [
        { "type": "string" },
        { "$ref": "#/$defs/runtimeDetail" }
      ]
    },
    "toolPlatformDetail": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "version": { "type": "string" },
        "acquisition": { "$ref": "#/$defs/toolAcquisition" }
      }
    },
    "toolAcquisition": {
      "type": "object",
      "additionalProperties": false,
      "required": ["provider"],
      "properties": {
        "provider": { "enum": ["corepack", "command", "release-asset", "apt", "brew", "winget", "choco", "scoop"] },
        "package": { "type": "string" },
        "version": { "type": "string" },
        "shell": { "enum": ["sh", "bash", "zsh", "pwsh", "cmd"] },
        "run": { "type": "string" },
        "source_config": { "type": "object", "additionalProperties": true }
      }
    },
    "toolDetail": {
      "type": "object",
      "additionalProperties": false,
      "required": ["version"],
      "properties": {
        "version": { "type": "string" },
        "required": { "type": "boolean" },
        "only_on": { "$ref": "#/$defs/stringArray" },
        "platforms": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/toolPlatformDetail" }
        },
        "acquisition": { "$ref": "#/$defs/toolAcquisition" }
      }
    },
    "toolRequirement": {
      "oneOf": [
        { "type": "string" },
        { "$ref": "#/$defs/toolDetail" }
      ]
    },
    "toolchainFulfillment": {
      "oneOf": [
        { "enum": ["none", "run"] },
        {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "source": {
              "enum": ["rustup", "corepack", "sdkman", "uv", "go", "ruby", "dotnet", "mise"]
            },
            "mode": { "enum": ["none", "run"] }
          }
        }
      ]
    },
    "toolchainPlatform": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "version": { "type": "string" },
        "profile": { "type": "string" },
        "components": { "$ref": "#/$defs/stringArray" },
        "package_managers": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        },
        "targets": { "$ref": "#/$defs/stringArray" }
      }
    },
    "toolchainSpec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["version"],
      "properties": {
        "provider": { "enum": ["rustup", "corepack", "sdkman", "uv", "go", "ruby", "dotnet"] },
        "version": { "type": "string" },
        "required": { "type": "boolean" },
        "only_on": { "$ref": "#/$defs/stringArray" },
        "profile": { "type": "string" },
        "components": { "$ref": "#/$defs/stringArray" },
        "package_managers": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        },
        "targets": { "$ref": "#/$defs/stringArray" },
        "fulfillment": { "$ref": "#/$defs/toolchainFulfillment" },
        "platforms": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/toolchainPlatform" }
        }
      }
    },
    "orchestratorActivation": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "trust": { "type": "boolean" }
      }
    },
    "orchestratorPrepare": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "install": { "type": "boolean" }
      }
    },
    "orchestratorSpec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "const": "mise" },
        "required": { "type": "boolean" },
        "config_files": { "$ref": "#/$defs/stringArray" },
        "activation": { "$ref": "#/$defs/orchestratorActivation" },
        "prepare": { "$ref": "#/$defs/orchestratorPrepare" }
      }
    },
    "nativePrerequisiteActivation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "enum": ["visual_studio_dev_shell", "command"] },
        "arch": { "type": "string" },
        "shell": { "enum": ["sh", "bash", "zsh", "pwsh", "cmd"] },
        "run": { "type": "string" }
      }
    },
    "nativePrerequisiteVisualStudio": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "components": { "$ref": "#/$defs/stringArray" }
      }
    },
    "nativePrerequisitePlatformRequires": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "runtimes": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/runtimeRequirement" }
        },
        "tools": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/toolRequirement" }
        },
        "toolchains": { "$ref": "#/$defs/stringArray" },
        "env": { "$ref": "#/$defs/stringArray" },
        "checks": { "$ref": "#/$defs/stringArray" }
      }
    },
    "nativePrerequisitePlatform": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "check": { "type": "string" },
        "apt": { "$ref": "#/$defs/stringArray" },
        "brew": { "$ref": "#/$defs/stringArray" },
        "winget": { "$ref": "#/$defs/stringArray" },
        "choco": { "$ref": "#/$defs/stringArray" },
        "scoop": { "$ref": "#/$defs/stringArray" },
        "xcode_clt": { "type": "boolean" },
        "visual_studio_build_tools": { "type": "boolean" },
        "visual_studio": { "$ref": "#/$defs/nativePrerequisiteVisualStudio" },
        "activation": { "$ref": "#/$defs/nativePrerequisiteActivation" },
        "requires": { "$ref": "#/$defs/nativePrerequisitePlatformRequires" },
        "install": { "type": "string" },
        "note": { "type": "string" }
      }
    },
    "nativePrerequisite": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "required": { "type": "boolean" },
        "check": { "type": "string" },
        "platforms": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/nativePrerequisitePlatform" }
        }
      }
    },
    "extensionActivation": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "provider_managed_cleanup": { "type": "boolean" }
      }
    },
    "extensionSpec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "command", "api_version"],
      "properties": {
        "kind": { "enum": ["check_provider", "export_provider", "backend_provider"] },
        "command": { "type": "string" },
        "api_version": { "type": "integer", "minimum": 1 },
        "description": { "type": "string" },
        "activation": { "$ref": "#/$defs/extensionActivation" },
        "config": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/yamlValue" }
        }
      }
    },
    "executionContextRequirements": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "runtimes": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/runtimeRequirement" }
        },
        "tools": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/toolRequirement" }
        },
        "toolchains": { "$ref": "#/$defs/stringArray" }
      }
    },
    "executionContextAttachments": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "compose": { "$ref": "#/$defs/stringArray" },
        "isolated_paths": { "$ref": "#/$defs/stringArray" }
      }
    },
    "containerResources": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "memory": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "minimum": { "type": "string" },
            "default": { "type": "string" }
          }
        }
      }
    },
    "containerBackend": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "image": { "type": "string" },
        "engines": { "$ref": "#/$defs/stringArray" },
        "resources": { "$ref": "#/$defs/containerResources" }
      }
    },
    "remoteSsh": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "config_file": { "type": "string" },
        "identity_file": { "type": "string" }
      }
    },
    "remoteBackend": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "provider": { "type": "string" },
        "target": { "type": "string" },
        "cwd": { "type": "string" },
        "ssh": { "$ref": "#/$defs/remoteSsh" }
      }
    },
    "sharedBackendEnvironment": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "profile": { "type": "string" },
        "image_alias": { "type": "string" },
        "image": { "type": "string" },
        "source": { "type": "string" }
      }
    },
    "sharedBackend": {
      "type": "object",
      "additionalProperties": false,
      "required": ["scope", "backend", "lifecycle"],
      "properties": {
        "scope": { "enum": ["local", "remote"] },
        "backend": { "$ref": "#/$defs/backend" },
        "lifecycle": { "$ref": "#/$defs/lifecycle" },
        "context": { "type": "string" },
        "fulfillment": { "enum": ["none", "run"] },
        "environment": { "$ref": "#/$defs/sharedBackendEnvironment" }
      }
    },
    "executionContext": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "extends": { "type": "string" },
        "backend": { "$ref": "#/$defs/backend" },
        "only_on": { "$ref": "#/$defs/stringArray" },
        "only_arch": { "$ref": "#/$defs/stringArray" },
        "lifecycle": { "$ref": "#/$defs/lifecycle" },
        "fulfillment": { "enum": ["none", "run"] },
        "env": { "$ref": "#/$defs/stringMap" },
        "container": { "$ref": "#/$defs/containerBackend" },
        "remote": { "$ref": "#/$defs/remoteBackend" },
        "requirements": { "$ref": "#/$defs/executionContextRequirements" },
        "attachments": { "$ref": "#/$defs/executionContextAttachments" }
      }
    },
    "runtimeBoundarySharedPin": {
      "type": "object",
      "additionalProperties": false,
      "required": ["ref", "freshness"],
      "properties": {
        "ref": { "type": "string" },
        "freshness": { "enum": ["fresh", "warning", "blocking"] }
      }
    },
    "runtimeBoundaryDestinationConstraint": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "values", "source_posture", "enforcement"],
      "properties": {
        "kind": {
          "enum": [
            "callback_host_allowlist",
            "recipient_domain_allowlist",
            "downstream_host_allowlist",
            "bucket_scope",
            "tenant_scope",
            "sms_destination_allowlist"
          ]
        },
        "values": { "$ref": "#/$defs/stringArray" },
        "source_posture": {
          "enum": [
            "repo_local_authoritative",
            "shared_pinned_authoritative",
            "non_authoritative"
          ]
        },
        "enforcement": {
          "enum": [
            "authoritative_runtime_enforced",
            "authoritative_app_enforced",
            "advisory_only"
          ]
        },
        "shared_pin": { "$ref": "#/$defs/runtimeBoundarySharedPin" }
      }
    },
    "runtimeBoundaryOutboundTarget": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "value"],
      "properties": {
        "kind": { "enum": ["host", "domain", "service_alias"] },
        "value": { "type": "string" },
        "destination_shape": {
          "enum": [
            "single_purpose_host",
            "multi_tenant_host",
            "relay_host",
            "send_host"
          ]
        },
        "destination_constraint": {
          "$ref": "#/$defs/runtimeBoundaryDestinationConstraint"
        }
      }
    },
    "runtimeBoundaryNetwork": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "default": { "enum": ["deny", "allow"] },
        "outbound_targets": {
          "type": "array",
          "items": { "$ref": "#/$defs/runtimeBoundaryOutboundTarget" }
        }
      }
    },
    "runtimeBoundaryFilesystem": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "repo_root_mode": { "enum": ["read_only", "writable"] },
        "writable_paths": { "$ref": "#/$defs/stringArray" },
        "protected_paths": { "$ref": "#/$defs/stringArray" }
      }
    },
    "runtimeBoundary": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "filesystem": { "$ref": "#/$defs/runtimeBoundaryFilesystem" },
        "network": { "$ref": "#/$defs/runtimeBoundaryNetwork" }
      }
    },
    "execution": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "preferred": { "$ref": "#/$defs/backend" },
        "supported": {
          "type": "array",
          "items": { "$ref": "#/$defs/backend" }
        },
        "lifecycle": { "$ref": "#/$defs/lifecycle" },
        "backends": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "container": { "$ref": "#/$defs/containerBackend" },
            "remote": { "$ref": "#/$defs/remoteBackend" }
          }
        },
        "default_context": { "type": "string" },
        "contexts": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/executionContext" }
        },
        "runtime_boundary": { "$ref": "#/$defs/runtimeBoundary" },
        "shared_backends": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/sharedBackend" }
        }
      }
    },
    "envRequirement": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "required": { "type": "boolean" },
        "secret": { "type": "boolean" },
        "default": { "type": "string" },
        "allowed": { "$ref": "#/$defs/stringArray" },
        "prepend": { "$ref": "#/$defs/stringArray" },
        "append": { "$ref": "#/$defs/stringArray" }
      }
    },
    "envSource": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "path"],
      "properties": {
        "kind": { "enum": ["dotenv", "properties", "json", "yaml", "toml"] },
        "path": { "type": "string" },
        "must_exist": { "type": "boolean" }
      }
    },
    "envProfileDotenvRender": {
      "type": "object",
      "additionalProperties": false,
      "required": ["path"],
      "properties": {
        "path": { "type": "string" },
        "template": { "type": "string" },
        "include": { "$ref": "#/$defs/stringArray" }
      }
    },
    "envProfileStructuredFileRender": {
      "type": "object",
      "additionalProperties": false,
      "required": ["path", "format"],
      "properties": {
        "path": { "type": "string" },
        "format": { "enum": ["json", "toml"] },
        "sources": { "$ref": "#/$defs/stringArray" },
        "merge_into_existing": { "type": "boolean" }
      }
    },
    "envProfileRender": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "dotenv": { "$ref": "#/$defs/envProfileDotenvRender" },
        "files": {
          "type": "array",
          "items": { "$ref": "#/$defs/envProfileStructuredFileRender" }
        }
      }
    },
    "envProfile": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "sources": {
          "type": "array",
          "items": { "$ref": "#/$defs/envSource" }
        },
        "env_files": { "$ref": "#/$defs/stringArray" },
        "env": { "$ref": "#/$defs/stringMap" },
        "render": { "$ref": "#/$defs/envProfileRender" }
      }
    },
    "envConfig": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "vars": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/envRequirement" }
        },
        "sources": {
          "type": "array",
          "items": { "$ref": "#/$defs/envSource" }
        },
        "profiles": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/envProfile" }
        }
      }
    },
    "httpMethod": {
      "enum": ["GET", "HEAD"]
    },
    "httpSuccess": {
      "type": "object",
      "additionalProperties": false,
      "required": ["status"],
      "properties": {
        "status": {
          "type": "array",
          "items": { "type": "integer", "minimum": 100, "maximum": 599 }
        }
      }
    },
    "httpBody": {
      "type": "object",
      "additionalProperties": false,
      "required": ["contains"],
      "properties": {
        "contains": { "type": "string" }
      }
    },
    "readinessProbeObserver": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "kind": { "enum": ["command_host", "task"] },
        "task": { "type": "string" }
      }
    },
    "readinessProbeTarget": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "name"],
      "properties": {
        "kind": { "enum": ["task", "service"] },
        "name": { "type": "string" },
        "listener": { "type": "string" },
        "address_view": { "enum": ["topology", "host", "internal"] },
        "observer": { "$ref": "#/$defs/readinessProbeObserver" },
        "endpoint": { "type": "string" }
      }
    },
    "readinessProbe": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "enum": ["http", "tcp"] },
        "url": { "type": "string" },
        "target": { "$ref": "#/$defs/readinessProbeTarget" },
        "method": { "$ref": "#/$defs/httpMethod" },
        "path": { "type": "string" },
        "headers": { "$ref": "#/$defs/stringMap" },
        "success": { "$ref": "#/$defs/httpSuccess" },
        "body": { "$ref": "#/$defs/httpBody" },
        "expect_status": { "type": "integer", "minimum": 100, "maximum": 599 },
        "timeout": { "type": "integer", "minimum": 0 }
      }
    },
    "surfaceReadiness": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "enum": ["http", "tcp"] },
        "method": { "$ref": "#/$defs/httpMethod" },
        "path": { "type": "string" },
        "headers": { "$ref": "#/$defs/stringMap" },
        "success": { "$ref": "#/$defs/httpSuccess" },
        "body": { "$ref": "#/$defs/httpBody" },
        "interval": { "type": "string" },
        "timeout": { "type": "string" },
        "retries": { "type": "integer", "minimum": 0 },
        "start_period": { "type": "string" }
      }
    },
    "surfaceSpec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "port"],
      "properties": {
        "kind": { "enum": ["http", "https", "tcp"] },
        "port": { "type": "integer", "minimum": 1, "maximum": 65535 },
        "label": { "type": "string" },
        "purpose": { "type": "string" },
        "visibility": { "enum": ["public", "internal"] },
        "path": { "type": "string" },
        "readiness": { "$ref": "#/$defs/surfaceReadiness" }
      }
    },
    "serviceProducer": {
      "type": "object",
      "additionalProperties": false,
      "required": ["repo", "task"],
      "properties": {
        "repo": { "type": "string" },
        "task": { "type": "string" },
        "listener": { "type": "string" },
        "address_view": { "enum": ["topology", "host", "internal"] }
      }
    },
    "serviceEndpoint": {
      "type": "object",
      "additionalProperties": false,
      "required": ["address", "port"],
      "properties": {
        "context": { "type": "string" },
        "address": { "type": "string" },
        "port": { "type": "integer", "minimum": 1, "maximum": 65535 }
      }
    },
    "serviceManager": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "enum": ["compose", "host"] },
        "engine": { "enum": ["docker", "podman"] },
        "name": { "type": "string" },
        "file": { "type": "string" },
        "files": { "$ref": "#/$defs/stringArray" },
        "env_file": { "type": "string" },
        "env_files": { "$ref": "#/$defs/stringArray" },
        "profiles": { "$ref": "#/$defs/stringArray" },
        "service": { "type": "string" },
        "host": {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "unit"],
          "properties": {
            "kind": { "enum": ["systemd"] },
            "unit": { "type": "string" },
            "scope": { "enum": ["system", "user"] }
          }
        },
        "start": { "$ref": "#/$defs/taskCommand" },
        "stop": { "$ref": "#/$defs/taskCommand" }
      }
    },
    "serviceReadiness": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "from": { "type": "string" },
        "endpoint": { "type": "string" },
        "run": { "type": "string" },
        "probe": { "type": "string" },
        "kind": { "enum": ["http", "tcp", "compose_health", "systemd_active"] },
        "method": { "$ref": "#/$defs/httpMethod" },
        "path": { "type": "string" },
        "headers": { "$ref": "#/$defs/stringMap" },
        "success": { "$ref": "#/$defs/httpSuccess" },
        "body": { "$ref": "#/$defs/httpBody" },
        "interval": { "type": "string" },
        "timeout": { "type": "string" },
        "retries": { "type": "integer", "minimum": 0 },
        "start_period": { "type": "string" }
      }
    },
    "serviceLifecycle": {
      "type": "object",
      "additionalProperties": false,
      "required": ["teardown_assertion"],
      "properties": {
        "teardown_assertion": { "enum": ["manager_inactive", "boundary_terminated"] }
      }
    },
    "serviceSpec": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "required": { "type": "boolean" },
        "manager": { "$ref": "#/$defs/serviceManager" },
        "provider": { "type": "string" },
        "start": { "type": "string" },
        "stop": { "type": "string" },
        "producer": { "$ref": "#/$defs/serviceProducer" },
        "endpoints": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/serviceEndpoint" }
        },
        "healthcheck": { "type": "string" },
        "readiness": { "$ref": "#/$defs/serviceReadiness" },
        "lifecycle": { "$ref": "#/$defs/serviceLifecycle" },
        "depends_on": { "$ref": "#/$defs/stringArray" },
        "timeout": { "type": "integer", "minimum": 0 }
      }
    },
    "workflowTaskRef": {
      "type": "object",
      "additionalProperties": false,
      "required": ["task"],
      "properties": {
        "task": { "type": "string" }
      }
    },
    "workflowInstanceTopology": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "requires_instances": { "$ref": "#/$defs/stringArray" }
      }
    },
    "workflowInstanceTaskOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "env": { "$ref": "#/$defs/stringMap" },
        "adapter_inputs": { "$ref": "#/$defs/taskAdapterInputs" },
        "runtime": { "$ref": "#/$defs/workflowInstanceTaskRuntimeOverlay" }
      }
    },
    "workflowInstanceTaskRuntimePortOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "mode": { "enum": ["fixed", "discover", "auto"] },
        "value": { "type": "integer", "minimum": 1, "maximum": 65535 },
        "stride": { "type": "integer", "minimum": 1, "maximum": 65535 }
      }
    },
    "workflowInstanceTaskRuntimeHostPortOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "mode": { "enum": ["fixed", "auto"] },
        "value": { "type": "integer", "minimum": 1, "maximum": 65535 },
        "stride": { "type": "integer", "minimum": 1, "maximum": 65535 }
      }
    },
    "workflowInstanceTaskRuntimeBindOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "address": { "type": "string" },
        "port": { "$ref": "#/$defs/workflowInstanceTaskRuntimePortOverlay" }
      }
    },
    "workflowInstanceTaskRuntimeHostProjectionOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "address": { "type": "string" },
        "port": { "$ref": "#/$defs/workflowInstanceTaskRuntimeHostPortOverlay" },
        "primary": { "type": "boolean" },
        "path": { "type": "string" }
      }
    },
    "workflowInstanceTaskRuntimeProjectionOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "host": { "$ref": "#/$defs/workflowInstanceTaskRuntimeHostProjectionOverlay" }
      }
    },
    "workflowInstanceTaskRuntimeListenerOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "bind": { "$ref": "#/$defs/workflowInstanceTaskRuntimeBindOverlay" },
        "project": { "$ref": "#/$defs/workflowInstanceTaskRuntimeProjectionOverlay" }
      }
    },
    "workflowInstanceTaskRuntimeOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "backend_binding": { "type": "string" },
        "readiness": { "$ref": "#/$defs/taskRuntimeReadiness" },
        "listeners": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/workflowInstanceTaskRuntimeListenerOverlay"
          }
        }
      }
    },
    "workflowInstanceSurfaceOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "port": { "type": "integer", "minimum": 1, "maximum": 65535 },
        "port_stride": { "type": "integer", "minimum": 1, "maximum": 65535 },
        "path": { "type": "string" }
      }
    },
    "workflowInstance": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "notes": { "type": "string" },
        "topology": { "$ref": "#/$defs/workflowInstanceTopology" },
        "env": { "$ref": "#/$defs/stringMap" },
        "tasks": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/workflowInstanceTaskOverlay" }
        },
        "surfaces": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/workflowInstanceSurfaceOverlay" }
        }
      }
    },
    "workflowGeneratedInstance": {
      "type": "object",
      "additionalProperties": false,
      "required": ["prefix", "start", "end", "template"],
      "properties": {
        "prefix": { "type": "string" },
        "start": { "type": "integer", "minimum": 0, "maximum": 65535 },
        "end": { "type": "integer", "minimum": 0, "maximum": 65535 },
        "template": { "$ref": "#/$defs/workflowInstance" }
      }
    },
    "workflowInstances": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/workflowInstance" },
      "properties": {
        "default": { "type": "string" },
        "generated": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/workflowGeneratedInstance" }
        }
      },
      "required": ["default"]
    },
    "workflowEnv": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "profile": { "type": "string" },
        "compose_env_file_services": { "$ref": "#/$defs/stringArray" },
        "adapter_inputs": { "$ref": "#/$defs/taskAdapterInputs" },
        "compose_files": { "$ref": "#/$defs/stringArray" },
        "compose_project_name": { "type": "string" }
      }
    },
    "workflowServices": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "required": { "$ref": "#/$defs/stringArray" }
      }
    },
    "workflowReadinessSignal": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "checks": { "$ref": "#/$defs/stringArray" },
        "probes": { "$ref": "#/$defs/stringArray" },
        "surfaces": { "$ref": "#/$defs/stringArray" }
      }
    },
    "workflowReadiness": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "checks": { "$ref": "#/$defs/stringArray" },
        "probes": { "$ref": "#/$defs/stringArray" },
        "surfaces": { "$ref": "#/$defs/stringArray" },
        "signal": { "$ref": "#/$defs/workflowReadinessSignal" }
      }
    },
    "workflowNegativeControlIntervention": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": {
          "enum": ["dependency_disruption", "dependency_endpoint_override", "null_substitution"]
        }
      }
    },
    "workflowNegativeControl": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "dependency", "obligation", "task", "intervention", "expected_failure"],
      "properties": {
        "id": { "type": "string" },
        "dependency": { "type": "string" },
        "obligation": { "type": "string" },
        "task": { "type": "string" },
        "intervention": { "$ref": "#/$defs/workflowNegativeControlIntervention" },
        "expected_failure": { "enum": ["dependency_unavailable"] }
      }
    },
    "workflowSeamObservation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "dependency", "producer_task", "task", "marker_env"],
      "properties": {
        "id": { "type": "string" },
        "dependency": { "type": "string" },
        "producer_task": { "type": "string" },
        "task": { "type": "string" },
        "marker_env": { "type": "string" }
      }
    },
    "workflowLifecycleProofAssertion": {
      "type": "object",
      "additionalProperties": false,
      "required": ["task"],
      "properties": {
        "task": { "type": "string" }
      }
    },
    "workflowLifecycleProof": {
      "type": "object",
      "additionalProperties": false,
      "required": ["services"],
      "properties": {
        "services": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": { "type": "string" }
        },
        "assertion": { "$ref": "#/$defs/workflowLifecycleProofAssertion" }
      }
    },
    "workflowProof": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "claim": { "enum": ["bounded"] },
        "negative_controls": {
          "type": "array",
          "items": { "$ref": "#/$defs/workflowNegativeControl" }
        },
        "seam_observations": {
          "type": "array",
          "items": { "$ref": "#/$defs/workflowSeamObservation" }
        },
        "lifecycle": {
          "$ref": "#/$defs/workflowLifecycleProof"
        }
      }
    },
    "workflowExpose": {
      "oneOf": [
        { "type": "string" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["surface"],
          "properties": {
            "surface": { "type": "string" }
          }
        }
      ]
    },
    "workflowSpec": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "intent": { "type": "string" },
        "description": { "type": "string" },
        "notes": { "type": "string" },
        "runtime_boundary": { "$ref": "#/$defs/runtimeBoundary" },
        "adapter_inputs": { "$ref": "#/$defs/taskAdapterInputs" },
        "instances": { "$ref": "#/$defs/workflowInstances" },
        "env": { "$ref": "#/$defs/workflowEnv" },
        "prepare": { "$ref": "#/$defs/workflowTaskRef" },
        "setup": { "$ref": "#/$defs/workflowTaskRef" },
        "run": { "$ref": "#/$defs/workflowTaskRef" },
        "attach": { "$ref": "#/$defs/workflowTaskRef" },
        "services": { "$ref": "#/$defs/workflowServices" },
        "readiness": { "$ref": "#/$defs/workflowReadiness" },
        "proof": { "$ref": "#/$defs/workflowProof" },
        "exposes": {
          "type": "array",
          "items": { "$ref": "#/$defs/workflowExpose" }
        }
      }
    },
    "taskServiceEnvBinding": {
      "type": "object",
      "additionalProperties": false,
      "required": ["service"],
      "properties": {
        "service": { "type": "string" },
        "view": { "type": "string" },
        "endpoint": { "type": "string" },
        "format": { "enum": ["url", "host", "port", "host_port"] },
        "scheme": { "type": "string" },
        "username": { "type": "string" },
        "password": { "type": "string" },
        "password_env": { "type": "string" },
        "database": { "type": "string" }
      }
    },
    "taskEnvBinding": {
      "type": "object",
      "additionalProperties": false,
      "required": ["from_service"],
      "properties": {
        "from_service": { "$ref": "#/$defs/taskServiceEnvBinding" }
      }
    },
    "taskInput": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "required": { "type": "boolean" },
        "default": { "type": "string" },
        "allowed": { "$ref": "#/$defs/stringArray" }
      }
    },
    "taskTargetActivation": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "mode": { "enum": ["manual", "ensure_started", "restart_ready", "ensure_ready", "ensure_running"] }
      }
    },
    "taskTargetServiceRef": {
      "type": "object",
      "additionalProperties": false,
      "required": ["task"],
      "properties": {
        "member": { "type": "string" },
        "repo": { "type": "string" },
        "task": { "type": "string" },
        "listener": { "type": "string" },
        "address_view": { "enum": ["topology", "host", "internal"] }
      }
    },
    "taskTarget": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "service": { "$ref": "#/$defs/taskTargetServiceRef" },
        "url": { "type": "string" },
        "override_input": { "type": "string" },
        "activation": { "$ref": "#/$defs/taskTargetActivation" }
      }
    },
    "taskEnsureEnvRandom": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "bytes": { "type": "integer", "minimum": 1 },
        "encoding": { "enum": ["hex", "base64"] }
      }
    },
    "taskEnsureEnvVar": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "value": { "type": "string" },
        "random": { "$ref": "#/$defs/taskEnsureEnvRandom" },
        "from_env": { "type": "string" },
        "mode": { "enum": ["missing", "replace", "remove"] }
      }
    },
    "taskActionStep": {
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "from", "to"],
          "properties": {
            "kind": { "const": "copy_if_missing" },
            "from": { "type": "string" },
            "to": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "path", "vars"],
          "properties": {
            "kind": { "const": "ensure_env_file" },
            "path": { "type": "string" },
            "template": { "type": "string" },
            "template_mode": { "enum": ["missing", "replace"] },
            "vars": {
              "type": "object",
              "additionalProperties": { "$ref": "#/$defs/taskEnsureEnvVar" }
            }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "path"],
          "properties": {
            "kind": { "const": "ensure_file" },
            "path": { "type": "string" },
            "template": { "type": "string" },
            "value": { "type": "string" },
            "random": { "$ref": "#/$defs/taskEnsureEnvRandom" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "path"],
          "properties": {
            "kind": { "const": "ensure_directory" },
            "path": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "path"],
          "properties": {
            "kind": { "const": "ensure_virtualenv" },
            "path": { "type": "string" },
            "provider": { "enum": ["uv"] },
            "python": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "path", "source"],
          "properties": {
            "kind": { "const": "ensure_git_checkout" },
            "path": { "type": "string" },
            "source": {
              "type": "object",
              "additionalProperties": false,
              "required": ["git"],
              "properties": {
                "git": { "type": "string" },
                "ref": { "type": "string" }
              }
            },
            "remotes": {
              "type": "array",
              "items": {
                "type": "object",
                "additionalProperties": false,
                "required": ["name", "git"],
                "properties": {
                  "name": { "type": "string" },
                  "git": { "type": "string" }
                }
              }
            }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "path", "source"],
          "properties": {
            "kind": { "const": "ensure_git_template" },
            "path": { "type": "string" },
            "source": {
              "type": "object",
              "additionalProperties": false,
              "required": ["git"],
              "properties": {
                "git": { "type": "string" },
                "ref": { "type": "string" }
              }
            }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "checkouts"],
          "properties": {
            "kind": { "const": "ensure_git_checkouts" },
            "checkouts": {
              "type": "array",
              "items": {
                "type": "object",
                "additionalProperties": false,
                "required": ["path", "source"],
                "properties": {
                  "path": { "type": "string" },
                  "source": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["git"],
                    "properties": {
                      "git": { "type": "string" },
                      "ref": { "type": "string" }
                    }
                  },
                  "remotes": {
                    "type": "array",
                    "items": {
                      "type": "object",
                      "additionalProperties": false,
                      "required": ["name", "git"],
                      "properties": {
                        "name": { "type": "string" },
                        "git": { "type": "string" }
                      }
                    }
                  }
                }
              }
            }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "name"],
          "properties": {
            "kind": { "const": "ensure_container_network" },
            "provider": { "enum": ["docker"] },
            "name": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "service", "volume"],
          "properties": {
            "kind": { "const": "reset_compose_service_volume" },
            "provider": { "enum": ["docker"] },
            "service": { "type": "string" },
            "volume": { "type": "string" },
            "compose": { "$ref": "#/$defs/taskComposeAdapterInputs" }
          }
        }
      ]
    },
    "taskAction": {
      "oneOf": [
        { "$ref": "#/$defs/taskActionStep" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "file", "context", "tag"],
          "properties": {
            "kind": { "const": "build_container_image" },
            "provider": { "enum": ["docker"] },
            "file": { "type": "string" },
            "context": { "type": "string" },
            "tag": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "steps"],
          "properties": {
            "kind": { "const": "ensure_bundle" },
            "steps": {
              "type": "array",
              "items": { "$ref": "#/$defs/taskActionStep" }
            }
          }
        }
      ]
    },
    "taskLaunchVolume": {
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["source", "target"],
          "properties": {
            "kind": { "enum": ["named"] },
            "source": { "type": "string" },
            "target": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["name", "target"],
          "properties": {
            "kind": { "enum": ["named"] },
            "name": { "type": "string" },
            "target": { "type": "string" }
          }
        }
      ]
    },
    "taskCommand": {
      "type": "object",
      "additionalProperties": false,
      "required": ["exe"],
      "properties": {
        "exe": { "type": "string" },
        "args": { "$ref": "#/$defs/stringArray" },
        "cwd": { "type": "string" },
        "runtime_projection": { "$ref": "#/$defs/taskCommandRuntimeProjection" },
        "interaction": { "enum": ["auto", "forbidden", "required"], "default": "auto" }
      }
    },
    "taskCompose": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "enum": ["exec", "run", "attach", "up", "down", "build", "stop", "restart", "rm", "logs", "ps"] },
        "engine": { "enum": ["docker", "podman"] },
        "service": { "type": "string" },
        "services": { "$ref": "#/$defs/stringArray" },
        "workdir": { "type": "string" },
        "rm": { "type": "boolean" },
        "detach": { "type": "boolean" },
        "force_recreate": { "type": "boolean" },
        "force": { "type": "boolean" },
        "follow": { "type": "boolean" },
        "remove_volumes": { "type": "boolean" },
        "timeout_seconds": { "type": "integer", "minimum": 0 },
        "tty": { "type": "boolean" },
        "exe": { "type": "string" },
        "args": { "$ref": "#/$defs/stringArray" }
      }
    },
    "taskComposeInvocation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "service"],
      "properties": {
        "kind": { "enum": ["exec", "run"] },
        "engine": { "enum": ["docker", "podman"] },
        "service": { "type": "string" },
        "workdir": { "type": "string" },
        "rm": { "type": "boolean" },
        "tty": { "type": "boolean" }
      }
    },
    "taskLaunch": {
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "exe"],
          "properties": {
            "kind": { "const": "command" },
            "exe": { "type": "string" },
            "args": { "$ref": "#/$defs/stringArray" },
            "cwd": { "type": "string" },
            "runtime_projection": { "$ref": "#/$defs/taskCommandRuntimeProjection" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "action"],
          "properties": {
            "kind": { "const": "compose" },
            "engine": { "enum": ["docker", "podman"] },
            "action": { "const": "up" },
            "services": { "$ref": "#/$defs/stringArray" },
            "detach": { "type": "boolean" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "image"],
          "properties": {
            "kind": { "const": "container" },
            "image": { "type": "string" },
            "engine": { "type": "string" },
            "args": { "$ref": "#/$defs/stringArray" },
            "name": { "type": "string" },
            "remove": { "type": "boolean" },
            "volumes": {
              "type": "array",
              "items": { "$ref": "#/$defs/taskLaunchVolume" }
            }
          }
        }
      ]
    },
    "taskCommandRuntimeProjection": {
      "type": "object",
      "additionalProperties": false,
      "required": ["listener", "adapter"],
      "properties": {
        "listener": { "type": "string" },
        "adapter": { "enum": ["uvicorn", "rails", "nextjs"] }
      }
    },
    "taskPrepareSource": {
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "docker_compose" },
            "cwd": { "type": "string" },
            "file": { "type": "string" },
            "files": { "$ref": "#/$defs/stringArray" },
            "env_files": { "$ref": "#/$defs/stringArray" },
            "engine": { "enum": ["docker", "podman"] }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd", "manager", "mode"],
          "properties": {
            "kind": { "const": "node_package_manager" },
            "cwd": { "type": "string" },
            "manager": { "enum": ["npm", "pnpm", "yarn", "bun"] },
            "mode": { "enum": ["install", "ci"] },
            "filter": { "type": "string" },
            "frozen_lockfile": { "type": "boolean" },
            "inline_builds": { "type": "boolean" },
            "force": { "type": "boolean" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "bundler" },
            "cwd": { "type": "string" },
            "path": { "type": "string" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "poetry" },
            "cwd": { "type": "string" },
            "groups": { "$ref": "#/$defs/stringArray" },
            "group_mode": { "enum": ["with", "only"] },
            "no_root": { "type": "boolean" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "uv" },
            "cwd": { "type": "string" },
            "mode": { "enum": ["sync", "pip_requirements", "pip_local_project"] },
            "requirements_file": { "type": "string" },
            "local_project": {
              "type": "object",
              "additionalProperties": false,
              "required": ["path"],
              "properties": {
                "path": { "type": "string" },
                "editable": { "type": "boolean" },
                "extras": { "$ref": "#/$defs/stringArray" },
                "groups": { "$ref": "#/$defs/stringArray" },
                "lockfile": { "type": "string" }
              }
            },
            "default_index": { "type": "string" },
            "indexes": { "type": "array", "items": { "type": "string" } },
            "offline": { "type": "boolean" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "go_modules" },
            "cwd": { "type": "string" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "helm" },
            "cwd": { "type": "string" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "cargo" },
            "cwd": { "type": "string" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "dotnet_restore" },
            "cwd": { "type": "string" },
            "config_file": { "type": "string" },
            "sources": { "$ref": "#/$defs/stringArray" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd", "wrapper"],
          "properties": {
            "kind": { "const": "gradle" },
            "cwd": { "type": "string" },
            "wrapper": { "type": "boolean" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "cwd"],
          "properties": {
            "kind": { "const": "maven" },
            "cwd": { "type": "string" },
            "wrapper": { "type": "boolean" },
            "mode": { "enum": ["resolve", "go_offline"] },
            "skip_tests": { "type": "boolean" },
            "compose": { "$ref": "#/$defs/taskComposeInvocation" }
          }
        }
      ]
    },
    "taskPrepare": {
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "medium", "source"],
          "properties": {
            "kind": { "const": "dependency_hydration" },
            "medium": { "enum": ["container_images", "package_dependencies"] },
            "source": { "$ref": "#/$defs/taskPrepareSource" },
            "targets": { "$ref": "#/$defs/stringArray" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "tool", "source"],
          "properties": {
            "kind": { "const": "tool_bootstrap" },
            "tool": { "enum": ["uv", "playwright_browsers", "cypress_browsers"] },
            "browsers": {
              "type": "array",
              "items": { "enum": ["chromium", "firefox", "webkit", "chrome", "msedge"] }
            },
            "with_deps": { "type": "boolean" },
            "source": {
              "oneOf": [
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["kind", "exe"],
                  "properties": {
                    "kind": { "const": "pip" },
                    "exe": { "type": "string" }
                  }
                },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["kind", "cwd"],
                  "properties": {
                    "kind": { "const": "poetry" },
                    "cwd": { "type": "string" }
                  }
                },
                {
                  "type": "object",
                  "additionalProperties": false,
                  "required": ["kind", "cwd", "manager"],
                  "properties": {
                    "kind": { "const": "node_package_manager" },
                    "cwd": { "type": "string" },
                    "manager": { "enum": ["npm", "pnpm", "yarn", "bun"] },
                    "filter": { "type": "string" }
                  }
                }
              ]
            }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["kind", "steps"],
          "properties": {
            "kind": { "const": "sequence" },
            "steps": {
              "type": "array",
              "items": { "$ref": "#/$defs/taskPrepareSequenceStep" }
            }
          }
        }
      ]
    },
    "taskPrepareSequenceStep": {
      "oneOf": [
        { "$ref": "#/$defs/taskPrepare" },
        { "$ref": "#/$defs/taskActionStep" }
      ]
    },
    "taskEffects": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "writes": { "$ref": "#/$defs/stringArray" },
        "workspace_writes": { "$ref": "#/$defs/stringArray" },
        "network": { "type": "boolean" },
        "network_kind": { "enum": ["broad", "dependency_hydration", "container_image_hydration", "service_readiness", "integration_test", "tool_bootstrap"] },
        "adapter_state": { "$ref": "#/$defs/stringArray" },
        "external_state": { "$ref": "#/$defs/stringArray" }
      }
    },
    "taskRequirementAnyOfWhen": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "backend": { "$ref": "#/$defs/backend" },
        "context": { "type": "string" }
      }
    },
    "taskRequirementAnyOf": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "label": { "type": "string" },
        "when": { "$ref": "#/$defs/taskRequirementAnyOfWhen" },
        "runtimes": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/runtimeRequirement" }
        },
        "tools": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/toolRequirement" }
        },
        "toolchains": { "$ref": "#/$defs/stringArray" },
        "native": { "$ref": "#/$defs/stringArray" },
        "env": { "$ref": "#/$defs/stringArray" },
        "checks": { "$ref": "#/$defs/stringArray" }
      }
    },
    "taskRequirements": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "runtimes": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/runtimeRequirement" }
        },
        "tools": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/toolRequirement" }
        },
        "toolchains": { "$ref": "#/$defs/stringArray" },
        "native": { "$ref": "#/$defs/stringArray" },
        "env": { "$ref": "#/$defs/stringArray" },
        "checks": { "$ref": "#/$defs/stringArray" },
        "any_of": {
          "type": "array",
          "items": { "$ref": "#/$defs/taskRequirementAnyOf" }
        }
      }
    },
    "taskRuntimePort": {
      "type": "object",
      "additionalProperties": false,
      "required": ["mode"],
      "properties": {
        "mode": { "enum": ["fixed", "discover", "auto"] },
        "value": { "type": "integer", "minimum": 1, "maximum": 65535 }
      }
    },
    "taskRuntimeHostPort": {
      "type": "object",
      "additionalProperties": false,
      "required": ["mode"],
      "properties": {
        "mode": { "enum": ["fixed", "auto"] },
        "value": { "type": "integer", "minimum": 1, "maximum": 65535 }
      }
    },
    "taskRuntimeHostProjection": {
      "type": "object",
      "additionalProperties": false,
      "required": ["address", "port"],
      "properties": {
        "address": { "type": "string" },
        "port": { "$ref": "#/$defs/taskRuntimeHostPort" },
        "primary": { "type": "boolean" },
        "path": { "type": "string" }
      }
    },
    "taskRuntimeProjection": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "host": { "$ref": "#/$defs/taskRuntimeHostProjection" }
      }
    },
    "taskRuntimeBind": {
      "type": "object",
      "additionalProperties": false,
      "required": ["address", "port"],
      "properties": {
        "address": { "type": "string" },
        "port": { "$ref": "#/$defs/taskRuntimePort" }
      }
    },
    "taskRuntimeListenerFull": {
      "type": "object",
      "additionalProperties": false,
      "required": ["protocol", "bind"],
      "properties": {
        "protocol": { "enum": ["http", "https", "tcp"] },
        "bind": { "$ref": "#/$defs/taskRuntimeBind" },
        "project": { "$ref": "#/$defs/taskRuntimeProjection" }
      }
    },
    "taskRuntimeListener": {
      "oneOf": [
        { "$ref": "#/$defs/taskRuntimeListenerFull" },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["http"],
          "properties": {
            "http": { "type": "integer", "minimum": 1, "maximum": 65535 }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["tcp"],
          "properties": {
            "tcp": { "type": "integer", "minimum": 1, "maximum": 65535 }
          }
        }
      ]
    },
    "taskRuntimeSurfaceBindOverride": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "address": { "type": "string" },
        "port": { "$ref": "#/$defs/taskRuntimePort" }
      }
    },
    "taskRuntimeSurfaceHostOverride": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "address": { "type": "string" },
        "port": { "$ref": "#/$defs/taskRuntimeHostPort" },
        "primary": { "type": "boolean" },
        "path": { "type": "string" }
      }
    },
    "taskRuntimeSurfaceAttachment": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "bind": { "$ref": "#/$defs/taskRuntimeSurfaceBindOverride" },
        "project": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "host": { "$ref": "#/$defs/taskRuntimeSurfaceHostOverride" }
          }
        }
      }
    },
    "taskRuntimeSurfaces": {
      "oneOf": [
        {
          "type": "array",
          "items": { "type": "string" }
        },
        {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskRuntimeSurfaceAttachment" }
        }
      ]
    },
    "taskRuntimeReadiness": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "probe": { "type": "string" },
        "signal_probes": { "$ref": "#/$defs/stringArray" },
        "kind": { "enum": ["http", "tcp"] },
        "listener": { "type": "string" },
        "method": { "$ref": "#/$defs/httpMethod" },
        "path": { "type": "string" },
        "headers": { "$ref": "#/$defs/stringMap" },
        "success": { "$ref": "#/$defs/httpSuccess" },
        "body": { "$ref": "#/$defs/httpBody" },
        "interval": { "type": "string" },
        "timeout": { "type": "string" },
        "retries": { "type": "integer", "minimum": 0 },
        "start_period": { "type": "string" }
      }
    },
    "taskRuntime": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "const": "service" },
        "backend_binding": { "type": "string" },
        "readiness": { "$ref": "#/$defs/taskRuntimeReadiness" },
        "surfaces": { "$ref": "#/$defs/taskRuntimeSurfaces" },
        "listeners": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskRuntimeListener" }
        }
      }
    },
    "taskVariant": {
      "type": "object",
      "additionalProperties": false,
      "required": ["when"],
      "properties": {
        "when": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "os": { "type": "string" }
          }
        },
        "run": { "type": "string" },
        "script": { "type": "string" },
        "command": { "$ref": "#/$defs/taskCommand" }
      }
    },
    "taskExecutionOrchestrator": {
      "type": "object",
      "additionalProperties": false,
      "required": ["ref", "mode"],
      "properties": {
        "ref": { "type": "string" },
        "mode": { "enum": ["task", "exec"] }
      }
    },
    "taskComposeAdapterInputs": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "cwd": { "type": "string" },
        "env_files": { "$ref": "#/$defs/stringArray" },
        "files": { "$ref": "#/$defs/stringArray" },
        "profiles": { "$ref": "#/$defs/stringArray" },
        "project_name": { "type": "string" }
      }
    },
    "taskBakeAdapterInputs": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "cwd": { "type": "string" },
        "files": { "$ref": "#/$defs/stringArray" }
      }
    },
    "taskAdapterOverlay": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "cwd": { "type": "string" },
        "env_files": { "$ref": "#/$defs/stringArray" },
        "files": { "$ref": "#/$defs/stringArray" },
        "profiles": { "$ref": "#/$defs/stringArray" },
        "project_name": { "type": "string" },
        "values_files": { "$ref": "#/$defs/stringArray" },
        "chart": { "type": "string" },
        "release_name": { "type": "string" },
        "namespace": { "type": "string" }
      }
    },
    "taskHelmAdapterInputs": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "cwd": { "type": "string" },
        "values_files": { "$ref": "#/$defs/stringArray" },
        "chart": { "type": "string" },
        "release_name": { "type": "string" },
        "namespace": { "type": "string" }
      }
    },
    "taskAdapterInputs": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "overlays": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskAdapterOverlay" }
        },
        "compose": { "$ref": "#/$defs/taskComposeAdapterInputs" },
        "bake": { "$ref": "#/$defs/taskBakeAdapterInputs" },
        "helm": { "$ref": "#/$defs/taskHelmAdapterInputs" }
      }
    },
    "taskModeBranch": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "context": { "type": "string" },
        "orchestrator": { "$ref": "#/$defs/taskExecutionOrchestrator" },
        "lifecycle": { "$ref": "#/$defs/lifecycle" },
        "env": { "$ref": "#/$defs/stringMap" },
        "env_files": { "$ref": "#/$defs/stringArray" },
        "env_bindings": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskEnvBinding" }
        },
        "adapter_inputs": { "$ref": "#/$defs/taskAdapterInputs" },
        "run": { "type": "string" },
        "script": { "type": "string" },
        "command": { "$ref": "#/$defs/taskCommand" },
        "compose": { "$ref": "#/$defs/taskCompose" },
        "prepare": { "$ref": "#/$defs/taskPrepare" },
        "launch": { "$ref": "#/$defs/taskLaunch" },
        "runtime": { "$ref": "#/$defs/taskRuntime" }
      }
    },
    "taskExecutionModes": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "native": { "$ref": "#/$defs/taskModeBranch" },
        "container": { "$ref": "#/$defs/taskModeBranch" },
        "remote": { "$ref": "#/$defs/taskModeBranch" }
      }
    },
    "taskExecution": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "default_mode": { "$ref": "#/$defs/backend" },
        "orchestrator": { "$ref": "#/$defs/taskExecutionOrchestrator" },
        "modes": { "$ref": "#/$defs/taskExecutionModes" }
      }
    },
    "taskWhen": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "os": { "type": "string" }
      }
    },
    "taskAggregate": {
      "type": "object",
      "additionalProperties": false,
      "required": ["tasks"],
      "properties": {
        "tasks": { "$ref": "#/$defs/stringArray" }
      }
    },
    "generatedArtifact": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "producer", "paths"],
      "properties": {
        "kind": { "const": "generated_source" },
        "producer": { "type": "string" },
        "paths": { "$ref": "#/$defs/stringArray" },
        "inputs": { "$ref": "#/$defs/stringArray" }
      }
    },
    "taskSpec": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "notes": { "type": "string" },
        "category": { "type": "string" },
        "context": { "type": "string" },
        "runtime_boundary": { "$ref": "#/$defs/runtimeBoundary" },
        "env": { "$ref": "#/$defs/stringMap" },
        "env_files": { "$ref": "#/$defs/stringArray" },
        "env_bindings": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskEnvBinding" }
        },
        "adapter_inputs": { "$ref": "#/$defs/taskAdapterInputs" },
        "inputs": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskInput" }
        },
        "replay_inputs": {
          "type": "array",
          "items": { "$ref": "#/$defs/taskReplayInput" }
        },
        "witnessed_observations": { "$ref": "#/$defs/taskWitnessedObservations" },
        "targets": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/taskTarget" }
        },
        "run": { "type": "string" },
        "script": { "type": "string" },
        "command": { "$ref": "#/$defs/taskCommand" },
        "compose": { "$ref": "#/$defs/taskCompose" },
        "prepare": { "$ref": "#/$defs/taskPrepare" },
        "launch": { "$ref": "#/$defs/taskLaunch" },
        "action": { "$ref": "#/$defs/taskAction" },
        "aggregate": { "$ref": "#/$defs/taskAggregate" },
        "effects": { "$ref": "#/$defs/taskEffects" },
        "requirements": { "$ref": "#/$defs/taskRequirements" },
        "depends_on": { "$ref": "#/$defs/stringArray" },
        "requires_services": { "$ref": "#/$defs/stringArray" },
        "requires_artifacts": { "$ref": "#/$defs/stringArray" },
        "runtime": { "$ref": "#/$defs/taskRuntime" },
        "after_success": { "$ref": "#/$defs/stringArray" },
        "after_failure": { "$ref": "#/$defs/stringArray" },
        "after_always": { "$ref": "#/$defs/stringArray" },
        "safe_for_agent": { "type": "boolean" },
        "only_on": { "$ref": "#/$defs/stringArray" },
        "internal": { "type": "boolean" },
        "variants": {
          "type": "array",
          "items": { "$ref": "#/$defs/taskVariant" }
        },
        "execution": { "$ref": "#/$defs/taskExecution" },
        "when": { "$ref": "#/$defs/taskWhen" }
      }
    },
    "taskReplayInput": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "kind", "path"],
      "properties": {
        "id": { "type": "string" },
        "kind": { "enum": ["static_file", "presentation_profile", "comparator_profile"] },
        "path": { "type": "string" },
        "expected_identity": { "type": "string", "pattern": "^sha256:[0-9a-f]{64}$" }
      }
    },
    "taskWitnessedObservations": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "query_traces": {
          "type": "array",
          "items": { "$ref": "#/$defs/taskQueryTraceObservation" }
        }
      }
    },
    "taskQueryTraceObservation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "path"],
      "properties": {
        "id": { "type": "string" },
        "path": { "type": "string" }
      }
    },
    "envCheckHostAssertion": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "policy": { "enum": ["not_loopback"] },
        "allowed": { "$ref": "#/$defs/stringArray" }
      }
    },
    "envCheckAssertion": {
      "type": "object",
      "additionalProperties": false,
      "required": ["key"],
      "properties": {
        "key": { "type": "string" },
        "equals": { "type": "string" },
        "not_equals": { "$ref": "#/$defs/stringArray" },
        "state": { "enum": ["present", "missing"] },
        "host": { "$ref": "#/$defs/envCheckHostAssertion" },
        "url_host": { "$ref": "#/$defs/envCheckHostAssertion" }
      }
    },
    "envCheck": {
      "type": "object",
      "additionalProperties": false,
      "required": ["path"],
      "properties": {
        "path": { "type": "string" },
        "assertions": {
          "type": "array",
          "items": { "$ref": "#/$defs/envCheckAssertion" }
        }
      }
    },
    "changedFilesCheck": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "paths": { "$ref": "#/$defs/stringArray" },
        "base_ref": { "type": "string" },
        "head_ref": { "type": "string" },
        "include_untracked": { "type": "boolean" }
      }
    },
    "checkSpec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "kind", "severity"],
      "properties": {
        "name": { "type": "string" },
        "kind": { "enum": ["precondition", "health", "file", "env", "changed_files"] },
        "severity": { "enum": ["error", "warn", "info"] },
        "run": { "type": "string" },
        "probe": { "type": "string" },
        "path": { "type": "string" },
        "scope": { "enum": ["repo", "workspace"] },
        "expect": { "enum": ["exists", "file", "directory", "missing"] },
        "timeout": { "type": "integer", "minimum": 0 },
        "changed_files": { "$ref": "#/$defs/changedFilesCheck" },
        "env": { "$ref": "#/$defs/envCheck" }
      }
    },
    "agentBootstrapTarget": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "note": { "type": "string" },
        "sh": { "type": "string" },
        "powershell": { "type": "string" }
      }
    },
    "agentBootstrap": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "ota": { "$ref": "#/$defs/agentBootstrapTarget" }
      }
    },
    "agentBoundaryProvenance": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "writable_paths": { "$ref": "#/$defs/stringArray" },
        "protected_paths": { "$ref": "#/$defs/stringArray" }
      }
    },
    "agentInferredBoundary": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "reviewed": { "type": "boolean" },
        "provenance": { "$ref": "#/$defs/agentBoundaryProvenance" }
      }
    },
    "agentExceptions": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "sensitive_writes": { "$ref": "#/$defs/stringArray" }
      }
    },
    "agentRefusalCanary": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "task": { "type": "string" },
        "workflow": { "type": "string" }
      },
      "oneOf": [
        { "required": ["task"], "not": { "required": ["workflow"] } },
        { "required": ["workflow"], "not": { "required": ["task"] } }
      ]
    },
    "agentConfig": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "posture": { "enum": ["readiness_strict", "contract_authoring", "infra_authoring"] },
        "entrypoint": { "type": "string" },
        "default_task": { "type": "string" },
        "safe_tasks": { "$ref": "#/$defs/stringArray" },
        "refusal_canaries": {
          "type": "array",
          "uniqueItems": true,
          "items": { "$ref": "#/$defs/agentRefusalCanary" }
        },
        "verify_after_changes": { "$ref": "#/$defs/stringArray" },
        "writable_paths": { "$ref": "#/$defs/stringArray" },
        "exceptions": { "$ref": "#/$defs/agentExceptions" },
        "acknowledged_sensitive_writable_paths": { "$ref": "#/$defs/stringArray" },
        "protected_paths": { "$ref": "#/$defs/stringArray" },
        "inferred_boundary": { "$ref": "#/$defs/agentInferredBoundary" },
        "bootstrap": { "$ref": "#/$defs/agentBootstrap" },
        "notes": { "type": "string" }
      }
    }
  },
  "type": "object",
  "additionalProperties": false,
  "required": ["version", "project"],
  "properties": {
    "version": { "const": 1 },
    "project": { "$ref": "#/$defs/project" },
    "workspace": { "$ref": "#/$defs/repoWorkspace" },
    "execution": { "$ref": "#/$defs/execution" },
    "extensions": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/extensionSpec" }
    },
    "runtimes": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/runtimeRequirement" }
    },
    "tools": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/toolRequirement" }
    },
    "toolchains": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/toolchainSpec" }
    },
    "orchestrators": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/orchestratorSpec" }
    },
    "native_prerequisites": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/nativePrerequisite" }
    },
    "env": { "$ref": "#/$defs/envConfig" },
    "readiness": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "probes": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/readinessProbe" }
        }
      }
    },
    "surfaces": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/surfaceSpec" }
    },
    "services": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/serviceSpec" }
    },
    "workflows": {
      "type": "object",
      "required": ["default"],
      "properties": {
        "default": { "type": "string" }
      },
      "patternProperties": {
        "^(?!default$).+": { "$ref": "#/$defs/workflowSpec" }
      },
      "additionalProperties": false
    },
    "tasks": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/taskSpec" }
    },
    "artifacts": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/generatedArtifact" }
    },
    "checks": {
      "type": "array",
      "items": { "$ref": "#/$defs/checkSpec" }
    },
    "exports": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/yamlValue" }
    },
    "policies": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/yamlValue" }
    },
    "metadata": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/yamlValue" }
    },
    "agent": { "$ref": "#/$defs/agentConfig" }
  }
}
"########;
const WORKSPACE_CONTRACT_SCHEMA_JSON: &str = r########"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://dist.ota.run/spec/json-schemas/latest/workspace-contract.json",
  "title": "ota workspace contract",
  "$defs": {
    "yamlValue": {
      "oneOf": [
        { "type": "string" },
        { "type": "number" },
        { "type": "integer" },
        { "type": "boolean" },
        {
          "type": "array",
          "items": { "$ref": "#/$defs/yamlValue" }
        },
        {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/yamlValue" }
        }
      ]
    },
    "stringArray": {
      "type": "array",
      "items": { "type": "string" }
    },
    "workspaceInfo": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name"],
      "properties": {
        "name": { "type": "string" },
        "description": { "type": "string" },
        "git_base": { "type": "string" },
        "policy": { "type": "string" }
      }
    },
    "workspaceRepoSource": {
      "oneOf": [
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["git"],
          "properties": {
            "git": { "type": "string" },
            "ref": { "type": "string" }
          }
        },
        {
          "type": "object",
          "additionalProperties": false,
          "required": ["repo"],
          "properties": {
            "repo": { "type": "string" },
            "ref": { "type": "string" }
          }
        }
      ]
    },
    "workspaceRepoSpec": {
      "type": "object",
      "additionalProperties": false,
      "required": ["path"],
      "properties": {
        "path": { "type": "string" },
        "contract": { "type": "string" },
        "workflow": { "type": "string" },
        "required": { "type": "boolean" },
        "depends_on": { "$ref": "#/$defs/stringArray" },
        "source": { "$ref": "#/$defs/workspaceRepoSource" }
      }
    },
    "workspaceRepos": {
      "type": "object",
      "minProperties": 1,
      "additionalProperties": { "$ref": "#/$defs/workspaceRepoSpec" }
    }
  },
  "type": "object",
  "additionalProperties": false,
  "required": ["version", "workspace", "repos"],
  "properties": {
    "version": { "const": 1 },
    "workspace": { "$ref": "#/$defs/workspaceInfo" },
    "repos": { "$ref": "#/$defs/workspaceRepos" },
    "policies": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/yamlValue" }
    }
  }
}
"########;

pub fn published_contract_schemas() -> [PublishedContractSchema; 2] {
    [
        PublishedContractSchema {
            filename: "contract.json",
            body: CONTRACT_SCHEMA_JSON,
        },
        PublishedContractSchema {
            filename: "workspace-contract.json",
            body: WORKSPACE_CONTRACT_SCHEMA_JSON,
        },
    ]
}

pub fn generated_contract_schema(filename: &str) -> Option<JsonValue> {
    published_contract_schemas()
        .into_iter()
        .find(|schema| schema.filename == filename)
        .map(|schema| parse_schema(schema.filename, schema.body))
}

pub fn write_published_contract_schemas(schema_dir: &Path) -> Result<Vec<PathBuf>, String> {
    fs::create_dir_all(schema_dir).map_err(|error| {
        format!(
            "failed to create schema dir {}: {error}",
            schema_dir.display()
        )
    })?;

    let mut written = Vec::new();
    for schema in published_contract_schemas() {
        let path = schema_dir.join(schema.filename);
        fs::write(&path, schema.body)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
        written.push(path);
    }

    Ok(written)
}

fn parse_schema(filename: &str, body: &str) -> JsonValue {
    serde_json::from_str(body)
        .unwrap_or_else(|error| panic!("generated schema {filename} should parse: {error}"))
}
