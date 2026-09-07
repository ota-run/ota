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

//! Provider-free GitHub Actions OIDC request-endpoint compatibility model.
//!
//! This sealed model validates a runner-supplied request URL against the retained discovery
//! profile. It does not read environment variables, accept a bearer, issue a request, validate a
//! JWT, contact Google, or make the endpoint an identity authority.

#![allow(dead_code)]

use semver::Version;
use serde::Serialize;
use sha2::{Digest, Sha256};

const PROFILE_DOMAIN: &[u8] = b"ota.github-actions-oidc-request-endpoint-profile.v1\0";
const OBSERVATION_DOMAIN: &[u8] = b"ota.github-actions-oidc-request-endpoint-observation.v1\0";
const PROFILE_ID: &str = "github_actions_oidc_request_endpoint_v1";
const REQUEST_SCHEME: &str = "https";
const REQUEST_HOST: &str = "run-actions-1-azure-eastus.actions.githubusercontent.com";
const PATH_SHAPE: &str = "/{decimal}//idtoken/{uuid}/{uuid}";
const QUERY_SHAPE: &str = "api-version=2.0";
const AUDIENCE_KEY: &str = "audience";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GithubOidcEndpointError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GithubActionsOidcRequestEndpointProfileV1 {
    pub schema_version: u32,
    pub identity: String,
    pub profile_id: String,
    pub scheme: String,
    pub host: String,
    pub explicit_port: bool,
    pub path_shape: String,
    pub existing_query_shape: String,
    pub audience_append_key: String,
    pub redirects_allowed: bool,
    pub alternate_origins_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GithubActionsOidcEndpointObservationInputV1 {
    pub schema_version: u32,
    pub request_url: String,
    pub runner_environment: String,
    pub runner_os: String,
    pub runner_architecture: String,
    pub runner_version: String,
    pub protected_launcher_capability_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ResolvedGithubActionsOidcEndpointObservationV1 {
    pub schema_version: u32,
    pub identity: String,
    pub endpoint_profile_identity: String,
    pub runner_environment: String,
    pub runner_os: String,
    pub runner_architecture: String,
    pub runner_version: String,
    pub protected_launcher_capability_identity: String,
    pub request_scheme: String,
    pub request_host: String,
    pub request_path_shape: String,
    pub request_query_shape: String,
}

#[derive(Serialize)]
struct ProfileIdentityPayload<'a> {
    schema_version: u32,
    profile_id: &'a str,
    scheme: &'a str,
    host: &'a str,
    explicit_port: bool,
    path_shape: &'a str,
    existing_query_shape: &'a str,
    audience_append_key: &'a str,
    redirects_allowed: bool,
    alternate_origins_allowed: bool,
}

#[derive(Serialize)]
struct ObservationIdentityPayload<'a> {
    schema_version: u32,
    endpoint_profile_identity: &'a str,
    runner_environment: &'a str,
    runner_os: &'a str,
    runner_architecture: &'a str,
    runner_version: &'a str,
    protected_launcher_capability_identity: &'a str,
    request_scheme: &'a str,
    request_host: &'a str,
    request_path_shape: &'a str,
    request_query_shape: &'a str,
}

pub(crate) fn github_actions_oidc_request_endpoint_profile_v1()
-> Result<GithubActionsOidcRequestEndpointProfileV1, GithubOidcEndpointError> {
    let mut profile = GithubActionsOidcRequestEndpointProfileV1 {
        schema_version: 1,
        identity: String::new(),
        profile_id: PROFILE_ID.into(),
        scheme: REQUEST_SCHEME.into(),
        host: REQUEST_HOST.into(),
        explicit_port: false,
        path_shape: PATH_SHAPE.into(),
        existing_query_shape: QUERY_SHAPE.into(),
        audience_append_key: AUDIENCE_KEY.into(),
        redirects_allowed: false,
        alternate_origins_allowed: false,
    };
    profile.identity = profile_identity(&profile)?;
    Ok(profile)
}

pub(crate) fn resolve_github_actions_oidc_endpoint_observation_v1(
    profile: &GithubActionsOidcRequestEndpointProfileV1,
    input: &GithubActionsOidcEndpointObservationInputV1,
) -> Result<ResolvedGithubActionsOidcEndpointObservationV1, GithubOidcEndpointError> {
    verify_github_actions_oidc_request_endpoint_profile_v1(profile)?;
    if input.schema_version != 1 {
        return Err(error(
            "secret_delivery_oidc_endpoint_observation_version_unsupported",
            "OIDC request endpoint observation version is unsupported",
        ));
    }
    if input.runner_environment != "self-hosted"
        || input.runner_os != "linux"
        || input.runner_architecture != "x64"
    {
        return Err(error(
            "secret_delivery_oidc_endpoint_target_unsupported",
            "OIDC request endpoint observation is outside the protected Linux/X64 target",
        ));
    }
    let runner_version = Version::parse(&input.runner_version).map_err(|_| {
        error(
            "secret_delivery_oidc_endpoint_runner_version_invalid",
            "GitHub Actions Runner version is not canonical semantic version",
        )
    })?;
    if runner_version.to_string() != input.runner_version {
        return Err(error(
            "secret_delivery_oidc_endpoint_runner_version_invalid",
            "GitHub Actions Runner version is not canonical semantic version",
        ));
    }
    validate_sha256_identity(&input.protected_launcher_capability_identity)?;
    validate_request_url(profile, &input.request_url)?;

    let mut resolved = ResolvedGithubActionsOidcEndpointObservationV1 {
        schema_version: 1,
        identity: String::new(),
        endpoint_profile_identity: profile.identity.clone(),
        runner_environment: input.runner_environment.clone(),
        runner_os: input.runner_os.clone(),
        runner_architecture: input.runner_architecture.clone(),
        runner_version: input.runner_version.clone(),
        protected_launcher_capability_identity: input
            .protected_launcher_capability_identity
            .clone(),
        request_scheme: profile.scheme.clone(),
        request_host: profile.host.clone(),
        request_path_shape: profile.path_shape.clone(),
        request_query_shape: profile.existing_query_shape.clone(),
    };
    resolved.identity = observation_identity(&resolved)?;
    Ok(resolved)
}

pub(crate) fn verify_github_actions_oidc_request_endpoint_profile_v1(
    profile: &GithubActionsOidcRequestEndpointProfileV1,
) -> Result<(), GithubOidcEndpointError> {
    let expected = github_actions_oidc_request_endpoint_profile_v1()?;
    if profile != &expected {
        return Err(error(
            "secret_delivery_oidc_endpoint_profile_invalid",
            "OIDC request endpoint profile does not match its canonical semantics",
        ));
    }
    Ok(())
}

pub(crate) fn verify_github_actions_oidc_endpoint_observation_v1(
    profile: &GithubActionsOidcRequestEndpointProfileV1,
    retained_input: &GithubActionsOidcEndpointObservationInputV1,
    observation: &ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<(), GithubOidcEndpointError> {
    let expected = resolve_github_actions_oidc_endpoint_observation_v1(profile, retained_input)?;
    if observation != &expected {
        return Err(error(
            "secret_delivery_oidc_endpoint_observation_identity_mismatch",
            "OIDC request endpoint observation does not match retained input",
        ));
    }
    Ok(())
}

fn validate_request_url(
    profile: &GithubActionsOidcRequestEndpointProfileV1,
    request_url: &str,
) -> Result<(), GithubOidcEndpointError> {
    if request_url.is_empty()
        || !request_url.is_ascii()
        || request_url.bytes().any(|byte| byte.is_ascii_control())
        || request_url.contains('#')
        || request_url.contains('%')
    {
        return Err(invalid_endpoint());
    }
    let prefix = format!("{}://", profile.scheme);
    let remainder = request_url
        .strip_prefix(&prefix)
        .ok_or_else(invalid_endpoint)?;
    let (authority, path_and_query) = remainder.split_once('/').ok_or_else(invalid_endpoint)?;
    if authority != profile.host || authority.contains('@') || authority.contains(':') {
        return Err(invalid_endpoint());
    }
    let (path, query) = path_and_query
        .split_once('?')
        .ok_or_else(invalid_endpoint)?;
    if query.contains('?') || query != profile.existing_query_shape {
        return Err(invalid_endpoint());
    }
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() != 5
        || !is_canonical_positive_decimal(segments[0])
        || !segments[1].is_empty()
        || segments[2] != "idtoken"
        || !is_canonical_uuid(segments[3])
        || !is_canonical_uuid(segments[4])
    {
        return Err(invalid_endpoint());
    }
    Ok(())
}

fn is_canonical_positive_decimal(value: &str) -> bool {
    value
        .parse::<u64>()
        .is_ok_and(|parsed| parsed > 0 && value == parsed.to_string())
}

fn is_canonical_uuid(value: &str) -> bool {
    if value.len() != 36 {
        return false;
    }
    for (index, byte) in value.bytes().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return false;
            }
        } else if !(byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
            return false;
        }
    }
    matches!(value.as_bytes()[14], b'1'..=b'5')
        && matches!(value.as_bytes()[19], b'8' | b'9' | b'a' | b'b')
}

fn validate_sha256_identity(value: &str) -> Result<(), GithubOidcEndpointError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(invalid_capability_identity());
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid_capability_identity());
    }
    Ok(())
}

fn profile_identity(
    profile: &GithubActionsOidcRequestEndpointProfileV1,
) -> Result<String, GithubOidcEndpointError> {
    domain_identity(
        PROFILE_DOMAIN,
        &ProfileIdentityPayload {
            schema_version: profile.schema_version,
            profile_id: &profile.profile_id,
            scheme: &profile.scheme,
            host: &profile.host,
            explicit_port: profile.explicit_port,
            path_shape: &profile.path_shape,
            existing_query_shape: &profile.existing_query_shape,
            audience_append_key: &profile.audience_append_key,
            redirects_allowed: profile.redirects_allowed,
            alternate_origins_allowed: profile.alternate_origins_allowed,
        },
    )
}

fn observation_identity(
    observation: &ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<String, GithubOidcEndpointError> {
    domain_identity(
        OBSERVATION_DOMAIN,
        &ObservationIdentityPayload {
            schema_version: observation.schema_version,
            endpoint_profile_identity: &observation.endpoint_profile_identity,
            runner_environment: &observation.runner_environment,
            runner_os: &observation.runner_os,
            runner_architecture: &observation.runner_architecture,
            runner_version: &observation.runner_version,
            protected_launcher_capability_identity: &observation
                .protected_launcher_capability_identity,
            request_scheme: &observation.request_scheme,
            request_host: &observation.request_host,
            request_path_shape: &observation.request_path_shape,
            request_query_shape: &observation.request_query_shape,
        },
    )
}

fn domain_identity<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<String, GithubOidcEndpointError> {
    let canonical = serde_jcs::to_vec(value).map_err(|details| {
        error(
            "secret_delivery_oidc_endpoint_identity_canonicalization_failed",
            details.to_string(),
        )
    })?;
    let mut bytes = Vec::with_capacity(domain.len() + canonical.len());
    bytes.extend_from_slice(domain);
    bytes.extend_from_slice(&canonical);
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn invalid_endpoint() -> GithubOidcEndpointError {
    error(
        "secret_delivery_oidc_request_endpoint_invalid",
        "GitHub Actions OIDC request endpoint is outside the retained protected-runner profile",
    )
}

fn invalid_capability_identity() -> GithubOidcEndpointError {
    error(
        "secret_delivery_oidc_endpoint_capability_identity_invalid",
        "protected launcher capability identity is not canonical SHA-256",
    )
}

fn error(code: &'static str, message: impl Into<String>) -> GithubOidcEndpointError {
    GithubOidcEndpointError {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://run-actions-1-azure-eastus.actions.githubusercontent.com/158853568//idtoken/123e4567-e89b-12d3-a456-426614174000/987e6543-e21b-32d3-b456-426614174111?api-version=2.0";

    fn identity(character: char) -> String {
        format!("sha256:{}", character.to_string().repeat(64))
    }

    fn input() -> GithubActionsOidcEndpointObservationInputV1 {
        GithubActionsOidcEndpointObservationInputV1 {
            schema_version: 1,
            request_url: URL.into(),
            runner_environment: "self-hosted".into(),
            runner_os: "linux".into(),
            runner_architecture: "x64".into(),
            runner_version: "2.337.0".into(),
            protected_launcher_capability_identity: identity('a'),
        }
    }

    #[test]
    fn exact_protected_x64_endpoint_shape_resolves_and_reverifies() {
        let profile = github_actions_oidc_request_endpoint_profile_v1().expect("profile");
        let retained = input();
        let resolved = resolve_github_actions_oidc_endpoint_observation_v1(&profile, &retained)
            .expect("observation");
        assert_eq!(resolved.request_path_shape, PATH_SHAPE);
        assert_eq!(resolved.request_query_shape, QUERY_SHAPE);
        assert!(!format!("{resolved:?}").contains("123e4567"));
        verify_github_actions_oidc_endpoint_observation_v1(&profile, &retained, &resolved)
            .expect("semantic re-verification");
    }

    #[test]
    fn endpoint_substitutions_refuse() {
        let profile = github_actions_oidc_request_endpoint_profile_v1().expect("profile");
        let substitutions = [
            URL.replacen("https://", "http://", 1),
            URL.replacen(REQUEST_HOST, "token.actions.githubusercontent.com", 1),
            URL.replacen(REQUEST_HOST, &format!("{REQUEST_HOST}:443"), 1),
            URL.replacen("/158853568//idtoken/", "/158853568/idtoken/", 1),
            URL.replacen("/158853568//idtoken/", "/0//idtoken/", 1),
            URL.replacen("/158853568//idtoken/", "/0158853568//idtoken/", 1),
            URL.replacen("123e4567", "123E4567", 1),
            URL.replacen("api-version=2.0", "api-version=2.1", 1),
            format!("{URL}&audience=forged"),
            format!("{URL}&audience=first&audience=second"),
            format!("{URL}&scope=forged"),
            format!("{URL}&api-version=2.0"),
            format!("{URL}#fragment"),
            URL.replacen("https://", "https://caller@", 1),
            URL.replacen("idtoken", "id%74oken", 1),
        ];
        for request_url in substitutions {
            let mut changed = input();
            changed.request_url = request_url;
            assert_eq!(
                resolve_github_actions_oidc_endpoint_observation_v1(&profile, &changed)
                    .expect_err("substitution must refuse")
                    .code,
                "secret_delivery_oidc_request_endpoint_invalid"
            );
        }
    }

    #[test]
    fn canonical_runner_version_forms_align_with_observation_resolution() {
        let profile = github_actions_oidc_request_endpoint_profile_v1().expect("profile");
        for runner_version in [
            "2.337.0",
            "2.337.0-rc.1",
            "2.337.0+build.7",
            "2.337.0-rc.1+build.7",
        ] {
            let mut retained = input();
            retained.runner_version = runner_version.into();
            resolve_github_actions_oidc_endpoint_observation_v1(&profile, &retained)
                .expect("canonical runner version");
        }
        for runner_version in ["02.337.0", "2.337", "2.337.0-01", "v2.337.0"] {
            let mut retained = input();
            retained.runner_version = runner_version.into();
            assert_eq!(
                resolve_github_actions_oidc_endpoint_observation_v1(&profile, &retained)
                    .expect_err("noncanonical runner version must refuse")
                    .code,
                "secret_delivery_oidc_endpoint_runner_version_invalid"
            );
        }
    }

    #[test]
    fn target_capability_and_profile_substitutions_refuse() {
        let profile = github_actions_oidc_request_endpoint_profile_v1().expect("profile");
        for (field, value) in [
            ("environment", "github-hosted"),
            ("os", "macos"),
            ("architecture", "arm64"),
        ] {
            let mut changed = input();
            match field {
                "environment" => changed.runner_environment = value.into(),
                "os" => changed.runner_os = value.into(),
                "architecture" => changed.runner_architecture = value.into(),
                _ => unreachable!(),
            }
            assert_eq!(
                resolve_github_actions_oidc_endpoint_observation_v1(&profile, &changed)
                    .expect_err("target substitution must refuse")
                    .code,
                "secret_delivery_oidc_endpoint_target_unsupported"
            );
        }

        let mut changed = input();
        changed.protected_launcher_capability_identity = identity('A');
        assert_eq!(
            resolve_github_actions_oidc_endpoint_observation_v1(&profile, &changed)
                .expect_err("invalid capability identity")
                .code,
            "secret_delivery_oidc_endpoint_capability_identity_invalid"
        );

        let mut forged_profile = profile.clone();
        forged_profile.host = "alternate.actions.githubusercontent.com".into();
        forged_profile.identity = profile_identity(&forged_profile).expect("forged identity");
        assert_eq!(
            resolve_github_actions_oidc_endpoint_observation_v1(&forged_profile, &input())
                .expect_err("self-consistent profile forgery")
                .code,
            "secret_delivery_oidc_endpoint_profile_invalid"
        );

        let retained = input();
        let mut forged_observation =
            resolve_github_actions_oidc_endpoint_observation_v1(&profile, &retained)
                .expect("observation");
        forged_observation.runner_version = "2.338.0".into();
        forged_observation.identity =
            observation_identity(&forged_observation).expect("forged observation identity");
        assert_eq!(
            verify_github_actions_oidc_endpoint_observation_v1(
                &profile,
                &retained,
                &forged_observation,
            )
            .expect_err("self-consistent observation forgery")
            .code,
            "secret_delivery_oidc_endpoint_observation_identity_mismatch"
        );
    }
}
