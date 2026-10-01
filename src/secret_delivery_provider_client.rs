//! Provider request and response models for secret delivery.
//!
//! This module derives exact Google operation targets from semantically verified Core truth. It
//! retains the historical network-disabled V2 preparation path. Only the feature-gated V4
//! dispatch owner can make one GitHub Actions OIDC request. Its JWT retains the exact consumed
//! V4 context. Only the separately signed STS workflow may consume it for one fixed Google STS
//! exchange, dropping even an accepted token before terminal refusal. No later provider operation,
//! recipient materialization, selected work, or positive delivery evidence is enabled.

#![allow(dead_code)]

use std::fmt;
#[cfg(feature = "secret-delivery-pressure")]
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
#[cfg(feature = "secret-delivery-pressure")]
use std::time::Instant;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
#[cfg(feature = "secret-delivery-pressure")]
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
#[cfg(feature = "secret-delivery-pressure")]
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use ureq::config::Config as UreqConfig;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

use crate::secret_delivery_oidc_endpoint::{
    GithubActionsOidcEndpointObservationInputV1, GithubActionsOidcRequestEndpointProfileV1,
    ResolvedGithubActionsOidcEndpointObservationV1,
    verify_github_actions_oidc_endpoint_observation_v1,
};
#[cfg(feature = "secret-delivery-pressure")]
use crate::secret_delivery_oidc_endpoint::{
    ProtectedGithubOidcRequestUrlV1, github_actions_oidc_request_endpoint_profile_v1,
    resolve_github_actions_oidc_endpoint_observation_v1,
};
use crate::secret_delivery_transaction::{
    SecretDeliveryTransactionCandidateRealization,
    SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    secret_delivery_transaction_candidate_identity,
};
#[cfg(feature = "secret-delivery-pressure")]
use crate::secret_delivery_transaction_binding::ConsumedSecretDeliveryTransactionBindingV4;
use crate::secret_delivery_transaction_binding::{
    SecretDeliveryTransactionBindingError, VerifiedSecretDeliveryTransactionBindingV2,
    VerifiedSecretDeliveryTransactionBindingV4,
};
use crate::secret_delivery_transport_dependencies::{
    SecretDeliveryTransportDependencyFeatureGraphV1, SecretDeliveryTransportDependencyRecordV1,
    embedded_transport_dependency_expectation_v1,
};
#[cfg(feature = "secret-delivery-pressure")]
use crate::secret_provider_profile::GithubOidcClaim;
use crate::secret_provider_profile::{
    GithubOidcClaimValue, SecretDeliveryArchitecture, SecretDeliveryExecutionMode,
    SecretDeliveryInvocationBindingInput, SecretDeliveryOperatingSystem,
    SecretDeliveryRecipientBoundary, SecretDeliveryRuntime, validate_google_tuple,
};

const PLAN_DOMAIN: &[u8] = b"ota.secret-delivery-provider-client-plan.v1\0";
const CLOUD_PLATFORM_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";
const TOKEN_EXCHANGE_GRANT: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
const ACCESS_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:access_token";
const JWT_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:jwt";
const STS_URL: &str = "https://sts.googleapis.com/v1/token";
const IAM_CREDENTIALS_ORIGIN: &str = "https://iamcredentials.googleapis.com";
const SECRET_MANAGER_ORIGIN: &str = "https://secretmanager.googleapis.com";
const FORM_MEDIA_TYPE: &str = "application/x-www-form-urlencoded";
const JSON_MEDIA_TYPE: &str = "application/json";
const MAX_OIDC_RESPONSE_BYTES: usize = 65_536;
const MAX_TOKEN_RESPONSE_BYTES: usize = 16_384;
const MAX_SECRET_RESPONSE_BYTES: usize = 96 * 1024;
const MAX_TOKEN_BYTES: usize = 12_288;
const MAX_SECRET_BYTES: usize = 65_536;
const MAX_PROVIDER_RESPONSE_HEADER_BYTES: usize = 16 * 1024;
const PROVIDER_TIMEOUT_GLOBAL: Duration = Duration::from_secs(30);
const PROVIDER_TIMEOUT_RESOLVE: Duration = Duration::from_secs(5);
const PROVIDER_TIMEOUT_CONNECT: Duration = Duration::from_secs(10);
const PROVIDER_TIMEOUT_SEND: Duration = Duration::from_secs(5);
const PROVIDER_TIMEOUT_RECEIVE: Duration = Duration::from_secs(10);
const ACTIONS_ID_TOKEN_REQUEST_URL: &str = "ACTIONS_ID_TOKEN_REQUEST_URL";
const ACTIONS_ID_TOKEN_REQUEST_TOKEN: &str = "ACTIONS_ID_TOKEN_REQUEST_TOKEN";

static GITHUB_OIDC_CAPABILITY_OWNER: OnceLock<Mutex<bool>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryProviderClientError {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for SecretDeliveryProviderClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SecretDeliveryProviderClientError {}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryProviderClientPlanV1 {
    pub schema_version: u32,
    pub identity: String,
    pub transaction_candidate_identity: String,
    pub operations: Vec<SecretDeliveryProviderOperationPlanV1>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SecretDeliveryProviderOperationPlanV1 {
    pub realization_identity: String,
    pub invocation_binding_identity: String,
    pub transport_dependency_record_identity: String,
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub sts_audience: String,
    pub sts_url: String,
    pub service_account_token_url: String,
    pub secret_version_url: String,
    pub secret_version_resource: String,
}

#[derive(Serialize)]
struct PlanIdentityPayload<'a> {
    schema_version: u32,
    transaction_candidate_identity: &'a str,
    operations: Vec<OperationIdentityPayload<'a>>,
}

#[derive(Serialize)]
struct OperationIdentityPayload<'a> {
    realization_identity: &'a str,
    invocation_binding_identity: &'a str,
    transport_dependency_record_identity: &'a str,
    oidc_issuer: &'a str,
    oidc_audience: &'a str,
    sts_audience: &'a str,
    sts_url: &'a str,
    service_account_token_url: &'a str,
    secret_version_url: &'a str,
    secret_version_resource: &'a str,
}

impl fmt::Debug for SecretDeliveryProviderClientPlanV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretDeliveryProviderClientPlanV1([PROTECTED])")
    }
}

/// Protected in-memory bytes whose debug representation never includes the value.
pub(crate) struct ProtectedProviderValue(Vec<u8>);

impl ProtectedProviderValue {
    pub(crate) fn new(
        value: impl Into<Vec<u8>>,
    ) -> Result<Self, SecretDeliveryProviderClientError> {
        let protected = Self(value.into());
        if protected.0.is_empty() {
            return Err(error(
                "secret_delivery_provider_value_empty",
                "protected provider values cannot be empty",
            ));
        }
        Ok(protected)
    }

    fn as_utf8(&self) -> Result<&str, SecretDeliveryProviderClientError> {
        std::str::from_utf8(&self.0).map_err(|_| {
            error(
                "secret_delivery_provider_value_invalid",
                "protected provider value is not valid UTF-8",
            )
        })
    }

    #[cfg(test)]
    fn bytes_for_test(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for ProtectedProviderValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProtectedProviderValue([REDACTED])")
    }
}

impl Drop for ProtectedProviderValue {
    fn drop(&mut self) {
        #[cfg(test)]
        if self.0 == b"synthetic-federated-token" {
            // Observe disposal of the fake response's unique token without retaining its bytes.
            STS_TOKEN_DROPS.with(|count| count.set(count.get() + 1));
        }
        #[cfg(test)]
        if self.0 == b"synthetic-iam-token" {
            IAM_TOKEN_DROPS.with(|count| count.set(count.get() + 1));
        }
        self.0.fill(0);
    }
}

#[cfg(feature = "secret-delivery-pressure")]
struct ProtectedResponseBuffer(Vec<u8>);

#[cfg(feature = "secret-delivery-pressure")]
impl Drop for ProtectedResponseBuffer {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

#[cfg(feature = "secret-delivery-pressure")]
pub(crate) struct RetainedUnadmittedGithubOidcJwtV1(ProtectedProviderValue);

#[cfg(feature = "secret-delivery-pressure")]
impl fmt::Debug for RetainedUnadmittedGithubOidcJwtV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RetainedUnadmittedGithubOidcJwtV1([REDACTED])")
    }
}

#[cfg(feature = "secret-delivery-pressure")]
struct RetainedGithubOidcTransactionV4 {
    prepared: PreparedSecretDeliveryProviderTransportV4,
    jwt: RetainedUnadmittedGithubOidcJwtV1,
    binding_identity: String,
    first_dispatch_at: OffsetDateTime,
    first_dispatch_instant: Instant,
}

#[cfg(feature = "secret-delivery-pressure")]
impl fmt::Debug for RetainedGithubOidcTransactionV4 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RetainedGithubOidcTransactionV4([PROTECTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderHttpMethod {
    Get,
    Post,
}

/// Exact request bytes retained only in memory. Authorization and body are redacted from Debug.
pub(crate) struct ProtectedProviderRequestV1 {
    pub method: ProviderHttpMethod,
    pub url: String,
    pub media_type: Option<&'static str>,
    authorization: Option<ProtectedProviderValue>,
    body: Vec<u8>,
}

impl fmt::Debug for ProtectedProviderRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProtectedProviderRequestV1")
            .field("method", &self.method)
            .field("url", &"[PROTECTED]")
            .field("media_type", &self.media_type)
            .field(
                "authorization",
                &self.authorization.as_ref().map(|_| "[REDACTED]"),
            )
            .field("body", &"[REDACTED]")
            .finish()
    }
}

impl Drop for ProtectedProviderRequestV1 {
    fn drop(&mut self) {
        // Zero is valid UTF-8, so the String invariant remains intact until deallocation.
        unsafe { self.url.as_bytes_mut() }.fill(0);
        self.body.fill(0);
    }
}

impl ProtectedProviderRequestV1 {
    #[cfg(test)]
    fn authorization_for_test(&self) -> Option<&[u8]> {
        self.authorization
            .as_ref()
            .map(ProtectedProviderValue::bytes_for_test)
    }

    #[cfg(test)]
    fn body_for_test(&self) -> &[u8] {
        &self.body
    }
}

pub(crate) struct StsAccessTokenV1 {
    token: ProtectedProviderValue,
    pub expires_in_seconds: u64,
}

#[cfg(test)]
thread_local! {
    static STS_TOKEN_DROPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static IAM_TOKEN_DROPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static PROVIDER_TRUTH_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) struct ServiceAccountAccessTokenV1 {
    token: ProtectedProviderValue,
    pub expires_at: OffsetDateTime,
}

pub(crate) struct SecretManagerPayloadV1 {
    value: ProtectedProviderValue,
    pub crc32c: u32,
}

macro_rules! redacted_debug {
    ($type:ty, $secret:literal, $field:ident) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($type))
                    .field($secret, &"[REDACTED]")
                    .field(stringify!($field), &self.$field)
                    .finish()
            }
        }
    };
}

redacted_debug!(StsAccessTokenV1, "token", expires_in_seconds);
redacted_debug!(ServiceAccountAccessTokenV1, "token", expires_at);
redacted_debug!(SecretManagerPayloadV1, "value", crc32c);

/// One consumed V2 binding coupled to its exact provider operation plan. This is intentionally
/// opaque: later transport code cannot substitute another binding, candidate, or operation.
pub(crate) struct ConsumedSecretDeliveryProviderCapabilityV1 {
    binding: ota_authority_protocol::ProtectedLauncherSecretDeliveryTransactionBindingV2,
    plan: SecretDeliveryProviderClientPlanV1,
    transport_dependency_record_identity: String,
}

impl fmt::Debug for ConsumedSecretDeliveryProviderCapabilityV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ConsumedSecretDeliveryProviderCapabilityV1([PROTECTED])")
    }
}

pub(crate) struct ConsumedSecretDeliveryProviderCapabilityV4 {
    binding: ota_authority_protocol::ProtectedLauncherSecretDeliveryTransactionBindingV4,
    runner_version: String,
    candidate: SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    plan: SecretDeliveryProviderClientPlanV1,
    transport_dependency_record_identity: String,
    transport_dependency_feature_graph: SecretDeliveryTransportDependencyFeatureGraphV1,
    transport_dependency_record: SecretDeliveryTransportDependencyRecordV1,
}

impl fmt::Debug for ConsumedSecretDeliveryProviderCapabilityV4 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ConsumedSecretDeliveryProviderCapabilityV4([PROTECTED])")
    }
}

/// The acquired GitHub request capability is retained only after verified transaction
/// consumption; only the feature-gated V4 owner can dispatch it.
struct RetainedGithubOidcRequestCapabilityV1 {
    endpoint_profile: GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
    operation: SecretDeliveryProviderOperationPlanV1,
    bearer: ProtectedProviderValue,
}

impl fmt::Debug for RetainedGithubOidcRequestCapabilityV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RetainedGithubOidcRequestCapabilityV1([PROTECTED])")
    }
}

/// Fixed, network-disabled transport posture. This contains no Agent, connector, socket, or
/// dispatch API. A later authorization must consume this exact configuration before any request.
pub(crate) struct PreparedSecretDeliveryProviderTransportV1 {
    capability: ConsumedSecretDeliveryProviderCapabilityV1,
    oidc: RetainedGithubOidcRequestCapabilityV1,
    transport_dependency_record_identity: String,
    configuration: UreqConfig,
    maximum_response_body_bytes: usize,
}

impl fmt::Debug for PreparedSecretDeliveryProviderTransportV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PreparedSecretDeliveryProviderTransportV1([PROTECTED])")
    }
}

pub(crate) struct PreparedSecretDeliveryProviderTransportV4 {
    capability: ConsumedSecretDeliveryProviderCapabilityV4,
    oidc: RetainedGithubOidcRequestCapabilityV1,
    transport_dependency_record_identity: String,
    configuration: UreqConfig,
    maximum_response_body_bytes: usize,
}

impl fmt::Debug for PreparedSecretDeliveryProviderTransportV4 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PreparedSecretDeliveryProviderTransportV4([PROTECTED])")
    }
}

impl PreparedSecretDeliveryProviderTransportV4 {
    fn verify_signed_transport_expectation(&self) -> Result<(), SecretDeliveryProviderClientError> {
        let (expected_graph, expected_record) = embedded_transport_dependency_expectation_v1()
            .map_err(|_| {
                error(
                    "secret_delivery_provider_transport_dependencies_invalid",
                    "embedded transport dependency expectation is invalid",
                )
            })?;
        if self.capability.transport_dependency_feature_graph != expected_graph
            || self.capability.transport_dependency_record != expected_record
            || self.capability.transport_dependency_record_identity
                != expected_record.record_identity
            || self.transport_dependency_record_identity != expected_record.record_identity
            || self.capability.binding.transport_dependency_record_identity
                != expected_record.record_identity
            || self.capability.plan.operations.len() != 1
            || self.capability.plan.operations[0].transport_dependency_record_identity
                != expected_record.record_identity
            || self
                .capability
                .candidate
                .candidate()
                .transport_dependency_record_identity
                != expected_record.record_identity
            || self
                .capability
                .binding
                .secret_transaction_candidate_identity
                != self.capability.candidate.candidate().identity
            || self.capability.binding.projection_identity
                != self
                    .oidc
                    .endpoint_input
                    .protected_launcher_capability_projection_identity
            || self.capability.runner_version != self.oidc.endpoint_input.runner_version
            || self.capability.binding.schema_version != 4
            || self.capability.binding.protected_snapshot_schema_version != 2
        {
            return Err(error(
                "secret_delivery_provider_transport_dependencies_mismatch",
                "prepared transport does not retain the complete signed dependency expectation",
            ));
        }
        verify_secret_delivery_provider_client_plan_v1(
            &self.capability.plan,
            &self.capability.candidate,
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn signed_transport_expectation_for_test(
        &self,
    ) -> (
        &SecretDeliveryTransportDependencyFeatureGraphV1,
        &SecretDeliveryTransportDependencyRecordV1,
    ) {
        (
            &self.capability.transport_dependency_feature_graph,
            &self.capability.transport_dependency_record,
        )
    }

    #[cfg(all(test, feature = "secret-delivery-pressure"))]
    pub(crate) fn invalidate_binding_for_test(&mut self) {
        self.capability.binding.transport_dependency_record_identity = "invalid".into();
    }
}

#[cfg(feature = "secret-delivery-pressure")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GithubOidcDispatchOutcomeV1 {
    NotAttempted,
    ResponseReceived,
    ClaimsRefused,
    TransportRefused,
}

#[cfg(feature = "secret-delivery-pressure")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GithubOidcDispatchAttemptStateV1 {
    core_invocations: u8,
    outcome: GithubOidcDispatchOutcomeV1,
}

#[cfg(feature = "secret-delivery-pressure")]
pub(crate) struct GithubOidcDispatchTerminalV1 {
    attempt: GithubOidcDispatchAttemptStateV1,
    result: Result<RetainedGithubOidcTransactionV4, SecretDeliveryProviderClientError>,
}

#[cfg(feature = "secret-delivery-pressure")]
impl fmt::Debug for GithubOidcDispatchTerminalV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GithubOidcDispatchTerminalV1")
            .field("attempt", &self.attempt)
            .field("result", &self.result.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

#[cfg(feature = "secret-delivery-pressure")]
impl GithubOidcDispatchTerminalV1 {
    pub(crate) fn public_posture(&self) -> (u8, &'static str) {
        let outcome = match self.attempt.outcome {
            GithubOidcDispatchOutcomeV1::NotAttempted => "not_attempted",
            GithubOidcDispatchOutcomeV1::ResponseReceived => "response_received",
            GithubOidcDispatchOutcomeV1::ClaimsRefused => "claims_refused",
            GithubOidcDispatchOutcomeV1::TransportRefused => "transport_refused",
        };
        (self.attempt.core_invocations, outcome)
    }
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
impl GithubOidcDispatchTerminalV1 {
    pub(crate) fn refused_without_invocation_for_test(&self) -> bool {
        self.attempt.core_invocations == 0
            && self.attempt.outcome == GithubOidcDispatchOutcomeV1::NotAttempted
            && self.result.is_err()
    }
}

#[cfg(feature = "secret-delivery-pressure")]
impl GithubOidcDispatchAttemptStateV1 {
    fn new() -> Self {
        Self {
            core_invocations: 0,
            outcome: GithubOidcDispatchOutcomeV1::NotAttempted,
        }
    }

    fn invoke_once(&mut self) -> Result<(), SecretDeliveryProviderClientError> {
        if self.core_invocations != 0 {
            return Err(oidc_dispatch_refused());
        }
        self.core_invocations = 1;
        self.outcome = GithubOidcDispatchOutcomeV1::TransportRefused;
        Ok(())
    }

    fn claims_refused(&mut self) {
        self.outcome = GithubOidcDispatchOutcomeV1::ClaimsRefused;
    }
}

#[cfg(feature = "secret-delivery-pressure")]
fn oidc_dispatch_refused() -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_github_oidc_dispatch_refused",
        "GitHub OIDC dispatch refused without retaining transport details",
    )
}

#[cfg(feature = "secret-delivery-pressure")]
fn verify_oidc_dispatch_preflight(
    prepared: &PreparedSecretDeliveryProviderTransportV4,
) -> Result<(), SecretDeliveryProviderClientError> {
    let now = u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
        .map_err(|_| oidc_dispatch_refused())?;
    verify_oidc_dispatch_preflight_at(prepared, now)
}

#[cfg(feature = "secret-delivery-pressure")]
fn verify_oidc_dispatch_preflight_at(
    prepared: &PreparedSecretDeliveryProviderTransportV4,
    now: u64,
) -> Result<(), SecretDeliveryProviderClientError> {
    verify_oidc_dispatch_authority_v4(prepared)?;
    if prepared.capability.binding.expires_at_unix_seconds <= now {
        return Err(oidc_dispatch_refused());
    }
    Ok(())
}

#[cfg(feature = "secret-delivery-pressure")]
fn verify_oidc_dispatch_authority_v4(
    prepared: &PreparedSecretDeliveryProviderTransportV4,
) -> Result<(), SecretDeliveryProviderClientError> {
    prepared.verify_signed_transport_expectation()?;
    ota_authority_protocol::validate_protected_launcher_secret_delivery_transaction_binding_v4(
        &prepared.capability.binding,
    )
    .map_err(|_| oidc_dispatch_refused())?;
    verify_fixed_transport_configuration_v1(
        &prepared.configuration,
        prepared.maximum_response_body_bytes,
    )?;
    verify_github_actions_oidc_endpoint_observation_v1(
        &prepared.oidc.endpoint_profile,
        &prepared.oidc.endpoint_input,
        &prepared.oidc.endpoint_observation,
    )
    .map_err(|_| oidc_dispatch_refused())?;
    if prepared.oidc.operation != prepared.capability.plan.operations[0] {
        return Err(oidc_dispatch_refused());
    }
    Ok(())
}

#[cfg(feature = "secret-delivery-pressure")]
fn verify_exact_oidc_request(
    request: &ProtectedProviderRequestV1,
    endpoint_input: &GithubActionsOidcEndpointObservationInputV1,
    operation: &SecretDeliveryProviderOperationPlanV1,
    bearer: &ProtectedProviderValue,
) -> Result<(), SecretDeliveryProviderClientError> {
    let expected_url = ProtectedGithubOidcRequestUrlV1::new(
        format!(
            "{}&audience={}",
            endpoint_input
                .request_url
                .as_str()
                .map_err(|_| oidc_dispatch_refused())?,
            percent_encode(operation.oidc_audience.as_bytes())
        )
        .into_bytes(),
    )
    .map_err(|_| oidc_dispatch_refused())?;
    let authorization = request
        .authorization
        .as_ref()
        .ok_or_else(oidc_dispatch_refused)?;
    let authorization = authorization
        .as_utf8()
        .map_err(|_| oidc_dispatch_refused())?;
    if request.method != ProviderHttpMethod::Get
        || request.url.as_bytes() != expected_url.as_bytes()
        || request.media_type.is_some()
        || !request.body.is_empty()
        || !authorization.starts_with("Bearer ")
        || authorization[7..].is_empty()
        || authorization.as_bytes()[7..] != bearer.0
        || authorization[7..]
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(oidc_dispatch_refused());
    }
    Ok(())
}

#[cfg(feature = "secret-delivery-pressure")]
fn is_oidc_json_content_type(value: &str) -> bool {
    let mut parts = value.split(';');
    if !parts
        .next()
        .is_some_and(|essence| essence.trim().eq_ignore_ascii_case(JSON_MEDIA_TYPE))
    {
        return false;
    }
    let charset = parts.next();
    if parts.next().is_some() {
        return false;
    }
    charset.is_none_or(|value| value.trim().eq_ignore_ascii_case("charset=utf-8"))
}

#[cfg(feature = "secret-delivery-pressure")]
fn verify_oidc_response_head(
    status: ureq::http::StatusCode,
    headers: &ureq::http::HeaderMap,
) -> Result<(), SecretDeliveryProviderClientError> {
    // Keep the injected response seam bounded too; ureq separately bounds the wire header block.
    let header_bytes = headers
        .iter()
        .try_fold(0usize, |total, (name, value)| {
            total
                .checked_add(name.as_str().len())?
                .checked_add(value.as_bytes().len())?
                .checked_add(4)
        })
        .ok_or_else(oidc_dispatch_refused)?;
    if status != ureq::http::StatusCode::OK
        || header_bytes > MAX_PROVIDER_RESPONSE_HEADER_BYTES
        || headers
            .get_all(ureq::http::header::CONTENT_TYPE)
            .iter()
            .count()
            != 1
        || !headers
            .get(ureq::http::header::CONTENT_TYPE)
            .and_then(|header| header.to_str().ok())
            .is_some_and(is_oidc_json_content_type)
    {
        return Err(oidc_dispatch_refused());
    }
    Ok(())
}

/// V4-only one-shot owner for the feature-gated protected GitHub OIDC workflow route.
#[cfg(feature = "secret-delivery-pressure")]
pub(crate) fn dispatch_github_oidc_v4(
    prepared: PreparedSecretDeliveryProviderTransportV4,
) -> GithubOidcDispatchTerminalV1 {
    let mut attempt = GithubOidcDispatchAttemptStateV1::new();
    let result = dispatch_github_oidc_v4_inner(prepared, &mut attempt);
    GithubOidcDispatchTerminalV1 { attempt, result }
}

#[cfg(feature = "secret-delivery-pressure")]
fn dispatch_github_oidc_v4_inner(
    prepared: PreparedSecretDeliveryProviderTransportV4,
    attempt: &mut GithubOidcDispatchAttemptStateV1,
) -> Result<RetainedGithubOidcTransactionV4, SecretDeliveryProviderClientError> {
    verify_oidc_dispatch_preflight(&prepared)?;
    let agent = ureq::Agent::new_with_config(prepared.configuration.clone());
    let request = build_github_oidc_request_v1(
        &prepared.oidc.endpoint_profile,
        &prepared.oidc.endpoint_input,
        &prepared.oidc.endpoint_observation,
        &prepared.oidc.bearer,
        &prepared.oidc.operation,
    )?;
    let authorization = request
        .authorization
        .as_ref()
        .ok_or_else(oidc_dispatch_refused)?
        .as_utf8()
        .map_err(|_| oidc_dispatch_refused())?;
    let http_request = ureq::http::Request::get(request.url.as_str())
        .header(ureq::http::header::AUTHORIZATION, authorization)
        .body(())
        .map_err(|_| oidc_dispatch_refused())?;
    let built_uri =
        ProtectedGithubOidcRequestUrlV1::new(http_request.uri().to_string().into_bytes())
            .map_err(|_| oidc_dispatch_refused())?;
    if http_request.method() != ureq::http::Method::GET
        || built_uri.as_bytes() != request.url.as_bytes()
        || http_request.headers().len() != 1
        || http_request
            .headers()
            .get(ureq::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            != Some(authorization)
    {
        return Err(oidc_dispatch_refused());
    }
    verify_oidc_dispatch_preflight(&prepared)?;
    verify_exact_oidc_request(
        &request,
        &prepared.oidc.endpoint_input,
        &prepared.oidc.operation,
        &prepared.oidc.bearer,
    )?;
    let first_dispatch_at = OffsetDateTime::now_utc();
    let first_dispatch_instant = Instant::now();
    attempt.invoke_once()?;
    let mut response = agent
        .run(http_request)
        .map_err(|_| oidc_dispatch_refused())?;
    verify_oidc_response_head(response.status(), response.headers())?;
    let mut body = ProtectedResponseBuffer(Vec::new());
    response
        .body_mut()
        .with_config()
        .limit((MAX_OIDC_RESPONSE_BYTES + 1) as u64)
        .reader()
        .read_to_end(&mut body.0)
        .map_err(|_| oidc_dispatch_refused())?;
    if body.0.len() > MAX_OIDC_RESPONSE_BYTES {
        return Err(oidc_dispatch_refused());
    }
    let jwt = parse_github_oidc_response_v1(&body.0)?;
    let jwt = retain_reconciled_github_oidc_jwt_v4(jwt, &prepared, attempt)?;
    Ok(RetainedGithubOidcTransactionV4 {
        binding_identity: prepared.capability.binding.identity.clone(),
        prepared,
        jwt,
        first_dispatch_at,
        first_dispatch_instant,
    })
}

#[cfg(feature = "secret-delivery-pressure")]
fn retain_reconciled_github_oidc_jwt_v4(
    jwt: ProtectedProviderValue,
    prepared: &PreparedSecretDeliveryProviderTransportV4,
    attempt: &mut GithubOidcDispatchAttemptStateV1,
) -> Result<RetainedUnadmittedGithubOidcJwtV1, SecretDeliveryProviderClientError> {
    let now = OffsetDateTime::now_utc()
        .unix_timestamp()
        .try_into()
        .map_err(|_| oidc_dispatch_refused())?;
    retain_reconciled_github_oidc_jwt_v4_at(jwt, prepared, attempt, now)
}

#[cfg(feature = "secret-delivery-pressure")]
fn retain_reconciled_github_oidc_jwt_v4_at(
    jwt: ProtectedProviderValue,
    prepared: &PreparedSecretDeliveryProviderTransportV4,
    attempt: &mut GithubOidcDispatchAttemptStateV1,
    now: u64,
) -> Result<RetainedUnadmittedGithubOidcJwtV1, SecretDeliveryProviderClientError> {
    // A response arrived; any remaining refusal is local claim reconciliation, not transport.
    attempt.claims_refused();
    reconcile_github_oidc_jwt_claims_v4_at(&jwt, prepared, now)?;
    attempt.outcome = GithubOidcDispatchOutcomeV1::ResponseReceived;
    Ok(RetainedUnadmittedGithubOidcJwtV1(jwt))
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
pub(crate) fn reconcile_github_oidc_response_v4_for_test(
    prepared: PreparedSecretDeliveryProviderTransportV4,
    body: &[u8],
    now: u64,
) -> GithubOidcDispatchTerminalV1 {
    let mut attempt = GithubOidcDispatchAttemptStateV1::new();
    let result = (|| {
        attempt.invoke_once()?;
        let jwt = parse_github_oidc_response_v1(body)?;
        let jwt = retain_reconciled_github_oidc_jwt_v4_at(jwt, &prepared, &mut attempt, now)?;
        Ok(RetainedGithubOidcTransactionV4 {
            binding_identity: prepared.capability.binding.identity.clone(),
            prepared,
            jwt,
            first_dispatch_at: OffsetDateTime::from_unix_timestamp(
                i64::try_from(now).map_err(|_| oidc_dispatch_refused())?,
            )
            .map_err(|_| oidc_dispatch_refused())?,
            first_dispatch_instant: Instant::now(),
        })
    })();
    GithubOidcDispatchTerminalV1 { attempt, result }
}

#[cfg(feature = "secret-delivery-pressure")]
struct GoogleStsTransportResponseV1 {
    status: ureq::http::StatusCode,
    headers: ureq::http::HeaderMap,
    body: ProtectedResponseBuffer,
}

#[cfg(feature = "secret-delivery-pressure")]
#[derive(Debug)]
struct GoogleStsExchangeTerminalV1 {
    core_invocations: u8,
    result: Result<(), SecretDeliveryProviderClientError>,
}

#[cfg(feature = "secret-delivery-pressure")]
impl GoogleStsExchangeTerminalV1 {
    fn public_posture(&self) -> (u8, &'static str) {
        (
            self.core_invocations,
            match (self.core_invocations, self.result.is_ok()) {
                (0, _) => "not_attempted",
                (_, true) => "response_accepted",
                (_, false) => "response_refused",
            },
        )
    }
}

#[cfg(feature = "secret-delivery-pressure")]
fn selected_sts_checkpoint(
    prepared: &PreparedSecretDeliveryProviderTransportV4,
) -> Result<bool, SecretDeliveryProviderClientError> {
    prepared.verify_signed_transport_expectation()?;
    let operation = &prepared.capability.plan.operations[0];
    let realizations: Vec<_> = prepared
        .capability
        .candidate
        .candidate()
        .realizations
        .iter()
        .filter(|realization| {
            realization.realization_identity == operation.realization_identity
                && realization.invocation_binding_identity == operation.invocation_binding_identity
        })
        .collect();
    let [realization] = realizations.as_slice() else {
        return Err(sts_exchange_refused());
    };
    match realization
        .oidc_claims
        .get(&GithubOidcClaim::WorkflowRef)
        .map(String::as_str)
    {
        Some(crate::secret_delivery_transaction_binding::LIVE_GOOGLE_STS_WORKFLOW_REFERENCE_V1) => {
            Ok(true)
        }
        Some(
            crate::secret_delivery_transaction_binding::LIVE_GITHUB_OIDC_WORKFLOW_REFERENCE_V1,
        ) => Ok(false),
        _ => Err(sts_exchange_refused()),
    }
}

/// Route selection comes from the retained signed candidate, never ambient workflow inputs.
#[cfg(feature = "secret-delivery-pressure")]
pub(crate) fn dispatch_secret_delivery_checkpoint_v4(
    prepared: PreparedSecretDeliveryProviderTransportV4,
) -> (u8, &'static str, u8, &'static str) {
    dispatch_secret_delivery_checkpoint_v4_with_transport(
        prepared,
        dispatch_github_oidc_v4,
        exchange_google_sts_v4,
    )
}

#[cfg(feature = "secret-delivery-pressure")]
fn dispatch_secret_delivery_checkpoint_v4_with_transport(
    prepared: PreparedSecretDeliveryProviderTransportV4,
    github: impl FnOnce(PreparedSecretDeliveryProviderTransportV4) -> GithubOidcDispatchTerminalV1,
    sts: impl FnOnce(RetainedGithubOidcTransactionV4) -> GoogleStsExchangeTerminalV1,
) -> (u8, &'static str, u8, &'static str) {
    let Ok(sts_selected) = selected_sts_checkpoint(&prepared) else {
        return (0, "not_attempted", 0, "not_attempted");
    };
    let terminal = github(prepared);
    let (github_calls, github_outcome) = terminal.public_posture();
    if !sts_selected {
        return (github_calls, github_outcome, 0, "not_selected");
    }
    let (sts_calls, sts_outcome) = match terminal.result {
        Ok(owner) => sts(owner).public_posture(),
        Err(_) => (0, "not_attempted"),
    };
    (github_calls, github_outcome, sts_calls, sts_outcome)
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
pub(crate) fn checkpoint_without_sts_dispatch_for_test(
    prepared: PreparedSecretDeliveryProviderTransportV4,
    body: &[u8],
    now: u64,
) -> (u8, &'static str, u8, &'static str) {
    dispatch_secret_delivery_checkpoint_v4_with_transport(
        prepared,
        |prepared| reconcile_github_oidc_response_v4_for_test(prepared, body, now),
        |_| panic!("this route must not reach STS"),
    )
}

#[cfg(feature = "secret-delivery-pressure")]
fn sts_exchange_refused() -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_google_sts_exchange_refused",
        "Google STS exchange refused without retaining transport details",
    )
}

// The production route and network-disabled tests share exact request construction and bounded
// response reads. The callback seam is private; no caller can supply an endpoint or trust root.
#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_sts_v4_with_transport(
    owner: RetainedGithubOidcTransactionV4,
    transport: impl FnOnce(
        &ProtectedProviderRequestV1,
    )
        -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError>,
) -> GoogleStsExchangeTerminalV1 {
    exchange_google_sts_v4_with_transport_at(owner, transport, || {
        u64::try_from(OffsetDateTime::now_utc().unix_timestamp())
            .map_err(|_| sts_exchange_refused())
    })
}

#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_sts_v4_with_transport_at(
    owner: RetainedGithubOidcTransactionV4,
    transport: impl FnOnce(
        &ProtectedProviderRequestV1,
    )
        -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError>,
    clock: impl Fn() -> Result<u64, SecretDeliveryProviderClientError>,
) -> GoogleStsExchangeTerminalV1 {
    let terminal = exchange_google_sts_checkpoint_v4_with_transport_at(
        owner,
        StsContinuationV1::Terminal,
        transport,
        || {
            Ok(ProviderClockObservationV1 {
                wall: OffsetDateTime::from_unix_timestamp(
                    i64::try_from(clock()?).map_err(|_| sts_exchange_refused())?,
                )
                .map_err(|_| sts_exchange_refused())?,
                elapsed: Duration::ZERO,
            })
        },
    );
    GoogleStsExchangeTerminalV1 {
        core_invocations: terminal.core_invocations,
        result: terminal.result.map(drop),
    }
}

#[cfg(feature = "secret-delivery-pressure")]
enum StsContinuationV1 {
    Terminal,
    Iam,
}

#[cfg(feature = "secret-delivery-pressure")]
#[derive(Clone, Copy)]
struct ProviderClockObservationV1 {
    wall: OffsetDateTime,
    elapsed: Duration,
}

#[cfg(feature = "secret-delivery-pressure")]
impl ProviderClockObservationV1 {
    fn read(first_dispatch_instant: Instant) -> Self {
        Self {
            wall: OffsetDateTime::now_utc(),
            elapsed: first_dispatch_instant.elapsed(),
        }
    }
}

#[cfg(feature = "secret-delivery-pressure")]
struct RetainedGoogleStsTransactionV4 {
    oidc: RetainedGithubOidcTransactionV4,
    token: StsAccessTokenV1,
    token_expires_at: OffsetDateTime,
    token_expires_elapsed: Duration,
    last_clock: ProviderClockObservationV1,
}

#[cfg(feature = "secret-delivery-pressure")]
struct GoogleStsContinuationTerminalV1 {
    core_invocations: u8,
    result: Result<Option<RetainedGoogleStsTransactionV4>, SecretDeliveryProviderClientError>,
}

#[cfg(feature = "secret-delivery-pressure")]
fn selected_iam_checkpoint(
    owner: &RetainedGithubOidcTransactionV4,
) -> Result<(), SecretDeliveryProviderClientError> {
    owner
        .prepared
        .verify_signed_transport_expectation()
        .map_err(|_| iam_exchange_refused())?;
    let realization = &owner.prepared.capability.candidate.candidate().realizations;
    if realization.len() != 1
        || realization[0]
            .oidc_claims
            .get(&GithubOidcClaim::WorkflowRef)
            .map(String::as_str)
            != Some(
                crate::secret_delivery_transaction_binding::LIVE_GOOGLE_IAM_WORKFLOW_REFERENCE_V1,
            )
        || owner.prepared.capability.binding.identity != owner.binding_identity
    {
        return Err(iam_exchange_refused());
    }
    Ok(())
}

#[cfg(feature = "secret-delivery-pressure")]
fn verify_iam_transaction_clock_v4(
    owner: &RetainedGithubOidcTransactionV4,
    clock: ProviderClockObservationV1,
    previous: ProviderClockObservationV1,
    jwt_freshness: &GithubOidcJwtFreshnessV1,
) -> Result<(), SecretDeliveryProviderClientError> {
    let deadline = owner
        .first_dispatch_at
        .checked_add(time::Duration::seconds(600))
        .ok_or_else(iam_exchange_refused)?;
    if clock.wall < previous.wall
        || clock.elapsed < previous.elapsed
        || clock.wall < owner.first_dispatch_at
        || clock.wall >= deadline
        || clock.elapsed >= Duration::from_secs(600)
    {
        return Err(iam_exchange_refused());
    }
    let now = u64::try_from(clock.wall.unix_timestamp()).map_err(|_| iam_exchange_refused())?;
    if owner.prepared.capability.binding.expires_at_unix_seconds <= now {
        return Err(iam_exchange_refused());
    }
    jwt_freshness
        .validate_at(now)
        .map_err(|_| iam_exchange_refused())
}

#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_sts_checkpoint_v4_with_transport_at(
    owner: RetainedGithubOidcTransactionV4,
    continuation: StsContinuationV1,
    transport: impl FnOnce(
        &ProtectedProviderRequestV1,
    )
        -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError>,
    mut clock: impl FnMut() -> Result<ProviderClockObservationV1, SecretDeliveryProviderClientError>,
) -> GoogleStsContinuationTerminalV1 {
    let mut core_invocations = 0;
    let result = (|| {
        let mut previous = ProviderClockObservationV1 {
            wall: owner.first_dispatch_at,
            elapsed: Duration::ZERO,
        };
        let mut verify = || {
            match continuation {
                StsContinuationV1::Iam => selected_iam_checkpoint(&owner)?,
                StsContinuationV1::Terminal if !selected_sts_checkpoint(&owner.prepared)? => {
                    return Err(sts_exchange_refused());
                }
                StsContinuationV1::Terminal => {}
            }
            verify_oidc_dispatch_authority_v4(&owner.prepared)
                .map_err(|_| sts_exchange_refused())?;
            let jwt_freshness = reconcile_github_oidc_jwt_context_v4(&owner.jwt.0, &owner.prepared)
                .map_err(|_| sts_exchange_refused())?;
            // Expensive signed truth and JWT context checks finish before the temporal sample.
            let observed = clock().map_err(|_| match continuation {
                StsContinuationV1::Terminal => sts_exchange_refused(),
                StsContinuationV1::Iam => iam_exchange_refused(),
            })?;
            match continuation {
                StsContinuationV1::Iam => {
                    verify_iam_transaction_clock_v4(&owner, observed, previous, &jwt_freshness)?
                }
                StsContinuationV1::Terminal => {
                    let now = u64::try_from(observed.wall.unix_timestamp())
                        .map_err(|_| sts_exchange_refused())?;
                    if owner.prepared.capability.binding.expires_at_unix_seconds <= now {
                        return Err(sts_exchange_refused());
                    }
                    jwt_freshness
                        .validate_at(now)
                        .map_err(|_| sts_exchange_refused())?;
                }
            }
            previous = observed;
            Ok(observed)
        };
        verify()?;
        let operation = &owner.prepared.capability.plan.operations[0];
        let request = build_google_sts_request_v1(operation, &owner.jwt.0)
            .map_err(|_| sts_exchange_refused())?;
        let dispatch_clock = verify()?;
        core_invocations = 1;
        let response = transport(&request).map_err(|_| sts_exchange_refused())?;
        verify_oidc_response_head(response.status, &response.headers)
            .map_err(|_| sts_exchange_refused())?;
        if response.body.0.len() > MAX_TOKEN_RESPONSE_BYTES {
            return Err(sts_exchange_refused());
        }
        let token =
            parse_google_sts_response_v1(&response.body.0).map_err(|_| sts_exchange_refused())?;
        let response_clock = verify()?;
        match continuation {
            StsContinuationV1::Terminal => {
                drop(token);
                Ok(None)
            }
            StsContinuationV1::Iam => {
                let token_expires_at = dispatch_clock
                    .wall
                    .checked_add(time::Duration::seconds(
                        i64::try_from(token.expires_in_seconds)
                            .map_err(|_| iam_exchange_refused())?,
                    ))
                    .ok_or_else(iam_exchange_refused)?;
                let token_expires_elapsed = dispatch_clock
                    .elapsed
                    .checked_add(Duration::from_secs(token.expires_in_seconds))
                    .ok_or_else(iam_exchange_refused)?;
                if response_clock.wall >= token_expires_at
                    || response_clock.elapsed >= token_expires_elapsed
                {
                    return Err(iam_exchange_refused());
                }
                Ok(Some(RetainedGoogleStsTransactionV4 {
                    oidc: owner,
                    token,
                    token_expires_at,
                    token_expires_elapsed,
                    last_clock: response_clock,
                }))
            }
        }
    })();
    GoogleStsContinuationTerminalV1 {
        core_invocations,
        result,
    }
}

#[cfg(feature = "secret-delivery-pressure")]
fn iam_exchange_refused() -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_google_iam_exchange_refused",
        "Google IAM exchange refused without retaining transport details",
    )
}

#[cfg(feature = "secret-delivery-pressure")]
#[derive(Debug)]
struct GoogleIamExchangeTerminalV1 {
    core_invocations: u8,
    result: Result<(), SecretDeliveryProviderClientError>,
}

#[cfg(feature = "secret-delivery-pressure")]
fn build_iam_http_request<'a>(
    request: &'a ProtectedProviderRequestV1,
    operation: &SecretDeliveryProviderOperationPlanV1,
    token: &StsAccessTokenV1,
) -> Result<ureq::http::Request<&'a [u8]>, SecretDeliveryProviderClientError> {
    let expected = build_service_account_token_request_v1(operation, token)
        .map_err(|_| iam_exchange_refused())?;
    if request.method != expected.method
        || request.url != expected.url
        || request.media_type != expected.media_type
        || request.body != expected.body
        || request
            .authorization
            .as_ref()
            .map(|value| value.0.as_slice())
            != expected
                .authorization
                .as_ref()
                .map(|value| value.0.as_slice())
    {
        return Err(iam_exchange_refused());
    }
    let authorization = request
        .authorization
        .as_ref()
        .ok_or_else(iam_exchange_refused)?
        .as_utf8()
        .map_err(|_| iam_exchange_refused())?;
    let http = ureq::http::Request::post(request.url.as_str())
        .header(ureq::http::header::CONTENT_TYPE, JSON_MEDIA_TYPE)
        .header(ureq::http::header::AUTHORIZATION, authorization)
        .body(request.body.as_slice())
        .map_err(|_| iam_exchange_refused())?;
    if http.method() != ureq::http::Method::POST
        || http.uri().to_string() != request.url
        || http.headers().len() != 2
        || http
            .headers()
            .get(ureq::http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            != Some(JSON_MEDIA_TYPE)
        || http
            .headers()
            .get(ureq::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            != Some(authorization)
        || *http.body() != request.body.as_slice()
    {
        return Err(iam_exchange_refused());
    }
    Ok(http)
}

// No production transport is attached to this seam in the network-disabled batch.
#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_iam_v4_with_transport_at(
    owner: RetainedGoogleStsTransactionV4,
    transport: impl FnOnce(
        &ProtectedProviderRequestV1,
    )
        -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError>,
    mut clock: impl FnMut() -> Result<ProviderClockObservationV1, SecretDeliveryProviderClientError>,
) -> GoogleIamExchangeTerminalV1 {
    let mut core_invocations = 0;
    let result = (|| {
        let mut previous = owner.last_clock;
        let mut verify = || {
            selected_iam_checkpoint(&owner.oidc)?;
            verify_oidc_dispatch_authority_v4(&owner.oidc.prepared)
                .map_err(|_| iam_exchange_refused())?;
            let jwt_freshness =
                reconcile_github_oidc_jwt_context_v4(&owner.oidc.jwt.0, &owner.oidc.prepared)
                    .map_err(|_| iam_exchange_refused())?;
            let observed = clock().map_err(|_| iam_exchange_refused())?;
            verify_iam_transaction_clock_v4(&owner.oidc, observed, previous, &jwt_freshness)?;
            if observed.wall >= owner.token_expires_at
                || observed.elapsed >= owner.token_expires_elapsed
            {
                return Err(iam_exchange_refused());
            }
            previous = observed;
            Ok(observed)
        };
        verify()?;
        let operation = &owner.oidc.prepared.capability.plan.operations[0];
        let request = build_service_account_token_request_v1(operation, &owner.token)
            .map_err(|_| iam_exchange_refused())?;
        build_iam_http_request(&request, operation, &owner.token)?;
        verify()?;
        core_invocations = 1;
        let response = transport(&request).map_err(|_| iam_exchange_refused())?;
        verify_oidc_response_head(response.status, &response.headers)
            .map_err(|_| iam_exchange_refused())?;
        let observed = verify()?;
        let token = parse_service_account_token_response_v1(&response.body.0, observed.wall)
            .map_err(|_| iam_exchange_refused())?;
        if verify()?.wall >= token.expires_at {
            return Err(iam_exchange_refused());
        }
        drop(token);
        Ok(())
    })();
    GoogleIamExchangeTerminalV1 {
        core_invocations,
        result,
    }
}

#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_sts_for_iam_v4_with_transport(
    owner: RetainedGithubOidcTransactionV4,
    transport: impl FnOnce(
        &ProtectedProviderRequestV1,
    )
        -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError>,
) -> GoogleStsContinuationTerminalV1 {
    let started = owner.first_dispatch_instant;
    exchange_google_sts_checkpoint_v4_with_transport_at(
        owner,
        StsContinuationV1::Iam,
        transport,
        || Ok(ProviderClockObservationV1::read(started)),
    )
}

#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_iam_v4_with_transport(
    owner: RetainedGoogleStsTransactionV4,
    transport: impl FnOnce(
        &ProtectedProviderRequestV1,
    )
        -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError>,
) -> GoogleIamExchangeTerminalV1 {
    let started = owner.oidc.first_dispatch_instant;
    exchange_google_iam_v4_with_transport_at(owner, transport, || {
        Ok(ProviderClockObservationV1::read(started))
    })
}

#[cfg(feature = "secret-delivery-pressure")]
fn build_sts_http_request(
    request: &ProtectedProviderRequestV1,
) -> Result<ureq::http::Request<&[u8]>, SecretDeliveryProviderClientError> {
    if request.method != ProviderHttpMethod::Post
        || request.url != STS_URL
        || request.media_type != Some(FORM_MEDIA_TYPE)
        || request.authorization.is_some()
    {
        return Err(sts_exchange_refused());
    }
    let http = ureq::http::Request::post(STS_URL)
        .header(ureq::http::header::CONTENT_TYPE, FORM_MEDIA_TYPE)
        .body(request.body.as_slice())
        .map_err(|_| sts_exchange_refused())?;
    if http.method() != ureq::http::Method::POST
        || http.uri().to_string() != STS_URL
        || http.headers().len() != 1
        || http
            .headers()
            .get(ureq::http::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            != Some(FORM_MEDIA_TYPE)
        || *http.body() != request.body.as_slice()
    {
        return Err(sts_exchange_refused());
    }
    Ok(http)
}

#[cfg(feature = "secret-delivery-pressure")]
fn read_sts_transport_response(
    status: ureq::http::StatusCode,
    headers: ureq::http::HeaderMap,
    reader: impl Read,
) -> Result<GoogleStsTransportResponseV1, SecretDeliveryProviderClientError> {
    verify_oidc_response_head(status, &headers).map_err(|_| sts_exchange_refused())?;
    let mut body = ProtectedResponseBuffer(Vec::new());
    reader
        .take((MAX_TOKEN_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut body.0)
        .map_err(|_| sts_exchange_refused())?;
    if body.0.len() > MAX_TOKEN_RESPONSE_BYTES {
        return Err(sts_exchange_refused());
    }
    Ok(GoogleStsTransportResponseV1 {
        status,
        headers,
        body,
    })
}

#[cfg(feature = "secret-delivery-pressure")]
fn exchange_google_sts_v4(owner: RetainedGithubOidcTransactionV4) -> GoogleStsExchangeTerminalV1 {
    let configuration = owner.prepared.configuration.clone();
    exchange_google_sts_v4_with_transport(owner, |request| {
        verify_fixed_transport_configuration_v1(&configuration, MAX_SECRET_RESPONSE_BYTES)
            .map_err(|_| sts_exchange_refused())?;
        let http_request = build_sts_http_request(request)?;
        let agent = ureq::Agent::new_with_config(configuration);
        let mut response = agent
            .run(http_request)
            .map_err(|_| sts_exchange_refused())?;
        let status = response.status();
        let headers = response.headers().clone();
        read_sts_transport_response(
            status,
            headers,
            response
                .body_mut()
                .with_config()
                .limit((MAX_TOKEN_RESPONSE_BYTES + 1) as u64)
                .reader(),
        )
    })
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
#[derive(Clone, Copy, serde::Deserialize)]
pub(crate) enum GoogleStsFaultForTestV1 {
    None,
    ExpiredBinding,
    ExpiredJwt,
    SubstitutedOperation,
    SubstitutedTransportRecord,
    SubstitutedJwt,
    TransportFailure,
    Redirect,
    DuplicateContentType,
    OversizedResponse,
    MalformedResponse,
    InvalidTokenType,
    ResponseExpired,
    ResponseJwtExpired,
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
pub(crate) fn exchange_google_sts_from_oidc_response_v4_for_test(
    prepared: PreparedSecretDeliveryProviderTransportV4,
    oidc_body: &[u8],
    now: u64,
    fault: GoogleStsFaultForTestV1,
) -> (u8, bool, u8, String) {
    let terminal = reconcile_github_oidc_response_v4_for_test(prepared, oidc_body, now);
    let mut owner = terminal.result.expect("locally reconciled V4 JWT");
    let expected_audience = owner.prepared.capability.plan.operations[0]
        .sts_audience
        .clone();
    match fault {
        GoogleStsFaultForTestV1::ExpiredBinding => {
            owner.prepared.capability.binding.expires_at_unix_seconds = now;
            owner.prepared.capability.binding.identity = ota_authority_protocol::protected_launcher_secret_delivery_transaction_binding_v4_identity(
                &owner.prepared.capability.binding,
            ).expect("rederived expired binding identity");
        }
        GoogleStsFaultForTestV1::ExpiredJwt
        | GoogleStsFaultForTestV1::SubstitutedJwt
        | GoogleStsFaultForTestV1::ResponseJwtExpired => {
            let jwt = owner.jwt.0.as_utf8().expect("JWT");
            let segments: Vec<_> = jwt.split('.').collect();
            let mut payload: serde_json::Value =
                serde_json::from_slice(&URL_SAFE_NO_PAD.decode(segments[1]).expect("JWT payload"))
                    .expect("JWT claims");
            if matches!(fault, GoogleStsFaultForTestV1::ExpiredJwt) {
                payload["exp"] = serde_json::json!(now);
            } else if matches!(fault, GoogleStsFaultForTestV1::ResponseJwtExpired) {
                payload["exp"] = serde_json::json!(now + 1);
            } else {
                payload["run_id"] = serde_json::json!("substituted-run");
            }
            owner.jwt.0 = ProtectedProviderValue::new(
                format!(
                    "{}.{}.{}",
                    segments[0],
                    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).expect("mutated claims")),
                    segments[2],
                )
                .into_bytes(),
            )
            .expect("mutated JWT");
        }
        GoogleStsFaultForTestV1::SubstitutedOperation => {
            owner.prepared.capability.plan.operations[0]
                .sts_audience
                .push_str("-substituted");
            owner.prepared.oidc.operation = owner.prepared.capability.plan.operations[0].clone();
            owner.prepared.capability.plan.identity =
                plan_identity(&owner.prepared.capability.plan)
                    .expect("rederived substituted plan identity");
        }
        GoogleStsFaultForTestV1::SubstitutedTransportRecord => {
            owner
                .prepared
                .capability
                .transport_dependency_record_identity = "f".repeat(64);
        }
        _ => {}
    }
    let expected_jwt = owner.jwt.0.as_utf8().expect("JWT").to_owned();
    if matches!(fault, GoogleStsFaultForTestV1::ResponseJwtExpired) {
        // Independently establish that only JWT freshness can refuse after response receipt.
        verify_oidc_dispatch_preflight_at(&owner.prepared, now + 2)
            .expect("V4 binding and authority remain fresh at response time");
        assert!(reconcile_github_oidc_jwt_claims_v4_at(&owner.jwt.0, &owner.prepared, now).is_ok());
        assert!(
            reconcile_github_oidc_jwt_claims_v4_at(&owner.jwt.0, &owner.prepared, now + 2).is_err()
        );
    }
    let mut fake_calls = 0;
    let drops_before = STS_TOKEN_DROPS.with(std::cell::Cell::get);
    let checks = std::cell::Cell::new(0);
    let terminal = exchange_google_sts_v4_with_transport_at(
        owner,
        |request| {
            fake_calls += 1;
            let http = build_sts_http_request(request).expect("production HTTP request");
            assert_eq!(http.method(), ureq::http::Method::POST);
            assert_eq!(http.uri().to_string(), STS_URL);
            assert_eq!(*http.body(), request.body.as_slice());
            assert_eq!(request.method, ProviderHttpMethod::Post);
            assert_eq!(request.url, "https://sts.googleapis.com/v1/token");
            assert_eq!(
                request.media_type,
                Some("application/x-www-form-urlencoded")
            );
            assert!(request.authorization.is_none());
            // Decode independently of the production request builder and encoder.
            let decode = |encoded: &[u8]| {
                let mut decoded = Vec::new();
                let mut index = 0;
                while index < encoded.len() {
                    match encoded[index] {
                        b'%' => {
                            let hex = std::str::from_utf8(&encoded[index + 1..index + 3])
                                .expect("form escape");
                            decoded.push(u8::from_str_radix(hex, 16).expect("hex escape"));
                            index += 3;
                        }
                        b'+' => {
                            decoded.push(b' ');
                            index += 1;
                        }
                        byte => {
                            decoded.push(byte);
                            index += 1;
                        }
                    }
                }
                String::from_utf8(decoded).expect("UTF-8 form field")
            };
            let fields: Vec<_> = request
                .body
                .split(|byte| *byte == b'&')
                .map(|field| {
                    let separator = field
                        .iter()
                        .position(|byte| *byte == b'=')
                        .expect("form pair");
                    (decode(&field[..separator]), decode(&field[separator + 1..]))
                })
                .collect();
            assert_eq!(
                fields,
                vec![
                    (
                        "grant_type".into(),
                        "urn:ietf:params:oauth:grant-type:token-exchange".into()
                    ),
                    ("audience".into(), expected_audience),
                    (
                        "scope".into(),
                        "https://www.googleapis.com/auth/cloud-platform".into()
                    ),
                    (
                        "requested_token_type".into(),
                        "urn:ietf:params:oauth:token-type:access_token".into()
                    ),
                    ("subject_token".into(), expected_jwt),
                    (
                        "subject_token_type".into(),
                        "urn:ietf:params:oauth:token-type:jwt".into()
                    ),
                ]
            );
            if matches!(fault, GoogleStsFaultForTestV1::TransportFailure) {
                return Err(error(
                    "fake_failure",
                    "synthetic-federated-token transport detail",
                ));
            }
            let mut headers = ureq::http::HeaderMap::new();
            headers.insert(
                ureq::http::header::CONTENT_TYPE,
                ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
            );
            let status = if matches!(fault, GoogleStsFaultForTestV1::Redirect) {
                ureq::http::StatusCode::FOUND
            } else {
                ureq::http::StatusCode::OK
            };
            if matches!(fault, GoogleStsFaultForTestV1::DuplicateContentType) {
                headers.append(
                    ureq::http::header::CONTENT_TYPE,
                    ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
                );
            }
            let mut body = br#"{"access_token":"synthetic-federated-token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":600}"#.to_vec();
            match fault {
                GoogleStsFaultForTestV1::OversizedResponse => {
                    body.resize(MAX_TOKEN_RESPONSE_BYTES + 1, b' ');
                }
                GoogleStsFaultForTestV1::MalformedResponse => {
                    body = b"synthetic-federated-token".to_vec();
                }
                GoogleStsFaultForTestV1::InvalidTokenType => {
                    body = br#"{"access_token":"synthetic-federated-token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Basic","expires_in":600}"#.to_vec();
                }
                _ => {}
            }
            read_sts_transport_response(status, headers, std::io::Cursor::new(body))
        },
        || {
            checks.set(checks.get() + 1);
            Ok(
                if matches!(fault, GoogleStsFaultForTestV1::ResponseExpired) && checks.get() >= 3 {
                    now + 3600
                } else if matches!(fault, GoogleStsFaultForTestV1::ResponseJwtExpired)
                    && checks.get() >= 3
                {
                    now + 2
                } else {
                    now
                },
            )
        },
    );
    assert_eq!(
        STS_TOKEN_DROPS.with(std::cell::Cell::get) - drops_before,
        usize::from(
            fake_calls == 1
                && matches!(
                    fault,
                    GoogleStsFaultForTestV1::None
                        | GoogleStsFaultForTestV1::ResponseExpired
                        | GoogleStsFaultForTestV1::ResponseJwtExpired
                )
        ),
        "every parsed STS token must be dropped before returning terminal status",
    );
    if matches!(fault, GoogleStsFaultForTestV1::ResponseJwtExpired) {
        assert_eq!(terminal.public_posture(), (1, "response_refused"));
    }
    let rendered = format!("{terminal:?}");
    assert!(!rendered.contains("synthetic-federated-token"));
    assert!(!rendered.contains("googleapis"));
    assert!(!rendered.contains("eyJ"));
    (
        terminal.core_invocations,
        terminal.result.is_ok(),
        fake_calls,
        rendered,
    )
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) enum GoogleIamFaultForTestV1 {
    None,
    RealClock,
    Fraction0,
    Fraction6,
    Fraction9,
    SubstitutedAccount,
    SubstitutedSession,
    SubstitutedGraph,
    SubstitutedRecord,
    ExpiredBinding,
    ExpiredJwt,
    ExpiredSts,
    TransactionExpired,
    MonotonicExpired,
    ClockBackward,
    MonotonicBackward,
    ClockFailure,
    ClockOverflow,
    StsResponseDelayed,
    TransportFailure,
    Redirect,
    DuplicateContentType,
    MissingContentType,
    BadContentType,
    OversizedHead,
    OversizedResponse,
    TruncatedResponse,
    ExtraField,
    DuplicateField,
    EmptyToken,
    OversizedToken,
    InvalidTimestamp,
    ExpiredToken,
    ExcessLifetime,
    InvalidFraction,
    FractionBeyondLimit,
    ReadFailure,
    ResponseBindingExpired,
    ResponseJwtExpired,
    ResponseStsExpired,
    ResponseTransactionExpired,
    ResponseClockBackward,
    PostParseExpired,
    PostParseTokenExpired,
    ValidationBindingExpired,
    ValidationJwtExpired,
    ValidationStsExpired,
    ValidationTransactionExpired,
    ValidationClockFailure,
    StsValidationBindingExpired,
    StsValidationJwtExpired,
    StsValidationTransactionExpired,
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
impl GoogleIamFaultForTestV1 {
    pub(crate) const ALL: [Self; 52] = [
        Self::None,
        Self::RealClock,
        Self::Fraction0,
        Self::Fraction6,
        Self::Fraction9,
        Self::SubstitutedAccount,
        Self::SubstitutedSession,
        Self::SubstitutedGraph,
        Self::SubstitutedRecord,
        Self::ExpiredBinding,
        Self::ExpiredJwt,
        Self::ExpiredSts,
        Self::TransactionExpired,
        Self::MonotonicExpired,
        Self::ClockBackward,
        Self::MonotonicBackward,
        Self::ClockFailure,
        Self::ClockOverflow,
        Self::StsResponseDelayed,
        Self::TransportFailure,
        Self::Redirect,
        Self::DuplicateContentType,
        Self::MissingContentType,
        Self::BadContentType,
        Self::OversizedHead,
        Self::OversizedResponse,
        Self::TruncatedResponse,
        Self::ExtraField,
        Self::DuplicateField,
        Self::EmptyToken,
        Self::OversizedToken,
        Self::InvalidTimestamp,
        Self::ExpiredToken,
        Self::ExcessLifetime,
        Self::InvalidFraction,
        Self::FractionBeyondLimit,
        Self::ReadFailure,
        Self::ResponseBindingExpired,
        Self::ResponseJwtExpired,
        Self::ResponseStsExpired,
        Self::ResponseTransactionExpired,
        Self::ResponseClockBackward,
        Self::PostParseExpired,
        Self::PostParseTokenExpired,
        Self::ValidationBindingExpired,
        Self::ValidationJwtExpired,
        Self::ValidationStsExpired,
        Self::ValidationTransactionExpired,
        Self::ValidationClockFailure,
        Self::StsValidationBindingExpired,
        Self::StsValidationJwtExpired,
        Self::StsValidationTransactionExpired,
    ];

    pub(crate) fn expected_calls(self) -> (u8, bool) {
        use GoogleIamFaultForTestV1::*;
        match self {
            None | RealClock | Fraction0 | Fraction6 | Fraction9 => (1, true),
            TransportFailure
            | Redirect
            | DuplicateContentType
            | MissingContentType
            | BadContentType
            | OversizedHead
            | OversizedResponse
            | TruncatedResponse
            | ExtraField
            | DuplicateField
            | EmptyToken
            | OversizedToken
            | InvalidTimestamp
            | ExpiredToken
            | ExcessLifetime
            | InvalidFraction
            | FractionBeyondLimit
            | ReadFailure
            | ResponseBindingExpired
            | ResponseJwtExpired
            | ResponseStsExpired
            | ResponseTransactionExpired
            | ResponseClockBackward
            | PostParseExpired
            | PostParseTokenExpired => (1, false),
            _ => (0, false),
        }
    }

    pub(crate) fn expected_sts_calls(self) -> u8 {
        u8::from(!matches!(
            self,
            Self::StsValidationBindingExpired
                | Self::StsValidationJwtExpired
                | Self::StsValidationTransactionExpired
        ))
    }
}

#[cfg(all(test, feature = "secret-delivery-pressure"))]
pub(crate) fn exchange_google_iam_from_oidc_response_v4_for_test(
    prepared: PreparedSecretDeliveryProviderTransportV4,
    oidc_body: &[u8],
    now: u64,
    fault: GoogleIamFaultForTestV1,
) -> (u8, u8, bool) {
    use GoogleIamFaultForTestV1::*;
    let mut jwt_body: serde_json::Value = serde_json::from_slice(oidc_body).unwrap();
    let jwt = jwt_body["value"].as_str().unwrap();
    let parts: Vec<_> = jwt.split('.').collect();
    let mut payload: serde_json::Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap();
    payload["exp"] = serde_json::json!(
        now + if matches!(
            fault,
            ExpiredJwt | ResponseJwtExpired | ValidationJwtExpired | StsValidationJwtExpired
        ) {
            1
        } else {
            800
        }
    );
    jwt_body["value"] = serde_json::json!(format!(
        "{}.{}.{}",
        parts[0],
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap()),
        parts[2]
    ));
    let terminal = reconcile_github_oidc_response_v4_for_test(
        prepared,
        &serde_json::to_vec(&jwt_body).unwrap(),
        now,
    );
    let oidc = terminal
        .result
        .expect("exact locally reconciled IAM fixture");
    let binding_expiry = oidc.prepared.capability.binding.expires_at_unix_seconds;
    let wall = OffsetDateTime::from_unix_timestamp(i64::try_from(now).unwrap()).unwrap();
    let sts_drops = STS_TOKEN_DROPS.with(std::cell::Cell::get);
    let iam_drops = IAM_TOKEN_DROPS.with(std::cell::Cell::get);
    let mut sts_calls = 0;
    let sts_checks = std::cell::Cell::new(0);
    let last_sts_truth = std::cell::Cell::new(PROVIDER_TRUTH_CHECKS.with(std::cell::Cell::get));
    let sts_transport = |request: &ProtectedProviderRequestV1| {
        sts_calls += 1;
        build_sts_http_request(request).unwrap();
        let lifetime = if matches!(
            fault,
            ExpiredSts | ResponseStsExpired | StsResponseDelayed | ValidationStsExpired
        ) {
            1
        } else {
            3600
        };
        let body = serde_json::to_vec(&serde_json::json!({"access_token":"synthetic-federated-token", "issued_token_type":ACCESS_TOKEN_TYPE,"token_type":"Bearer","expires_in":lifetime})).unwrap();
        let mut headers = ureq::http::HeaderMap::new();
        headers.insert(
            ureq::http::header::CONTENT_TYPE,
            ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
        );
        read_sts_transport_response(
            ureq::http::StatusCode::OK,
            headers,
            std::io::Cursor::new(body),
        )
    };
    let sts_clock = || {
        let truth_checks = PROVIDER_TRUTH_CHECKS.with(std::cell::Cell::get);
        assert!(
            truth_checks > last_sts_truth.replace(truth_checks),
            "STS clock must follow complete truth validation"
        );
        sts_checks.set(sts_checks.get() + 1);
        let delay = if matches!(fault, StsResponseDelayed) && sts_checks.get() >= 3 {
            1
        } else {
            0
        };
        let mut observed = ProviderClockObservationV1 {
            wall: wall + time::Duration::seconds(delay),
            elapsed: Duration::from_secs(delay as u64),
        };
        if sts_checks.get() >= 2 {
            match fault {
                StsValidationBindingExpired => {
                    observed.wall =
                        OffsetDateTime::from_unix_timestamp(binding_expiry as i64).unwrap()
                }
                StsValidationJwtExpired => observed.wall += time::Duration::seconds(1),
                StsValidationTransactionExpired => observed.elapsed = Duration::from_secs(600),
                _ => {}
            }
        }
        Ok(observed)
    };
    let sts_terminal = if matches!(fault, RealClock) {
        exchange_google_sts_for_iam_v4_with_transport(oidc, sts_transport)
    } else {
        exchange_google_sts_checkpoint_v4_with_transport_at(
            oidc,
            StsContinuationV1::Iam,
            sts_transport,
            sts_clock,
        )
    };
    assert_eq!(sts_terminal.core_invocations, sts_calls);
    let mut iam_calls = 0;
    let matched = match sts_terminal.result {
        Ok(Some(mut owner)) => {
            match fault {
                SubstitutedAccount => {
                    owner.oidc.prepared.capability.plan.operations[0]
                        .service_account_token_url
                        .push_str("?substituted");
                    owner.oidc.prepared.oidc.operation =
                        owner.oidc.prepared.capability.plan.operations[0].clone();
                    owner.oidc.prepared.capability.plan.identity =
                        plan_identity(&owner.oidc.prepared.capability.plan).unwrap();
                }
                SubstitutedSession => {
                    owner.oidc.prepared.capability.binding.session_identity =
                        format!("sha256:{}", "a".repeat(64));
                    owner.oidc.prepared.capability.binding.identity = ota_authority_protocol::protected_launcher_secret_delivery_transaction_binding_v4_identity(&owner.oidc.prepared.capability.binding).unwrap();
                    assert_ne!(
                        owner.oidc.binding_identity,
                        owner.oidc.prepared.capability.binding.identity
                    );
                }
                SubstitutedGraph => {
                    owner
                        .oidc
                        .prepared
                        .capability
                        .transport_dependency_feature_graph
                        .schema_version = 99
                }
                SubstitutedRecord => {
                    owner
                        .oidc
                        .prepared
                        .capability
                        .transport_dependency_record
                        .record_identity = "f".repeat(64)
                }
                ClockOverflow => {
                    owner.oidc.first_dispatch_at = time::Date::MAX
                        .with_time(time::Time::from_hms(23, 59, 59).unwrap())
                        .assume_utc()
                }
                _ => {}
            }
            let binding_expiry = owner
                .oidc
                .prepared
                .capability
                .binding
                .expires_at_unix_seconds;
            let checks = std::cell::Cell::new(0);
            let last_iam_truth =
                std::cell::Cell::new(PROVIDER_TRUTH_CHECKS.with(std::cell::Cell::get));
            let iam_transport = |request: &ProtectedProviderRequestV1| {
                iam_calls += 1;
                assert_eq!(request.method, ProviderHttpMethod::Post);
                assert!(request.url.starts_with(
                    "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/"
                ));
                assert_eq!(request.body, br#"{"scope":["https://www.googleapis.com/auth/cloud-platform"],"lifetime":"600s"}"#);
                assert_eq!(
                    request.authorization.as_ref().unwrap().bytes_for_test(),
                    b"Bearer synthetic-federated-token"
                );
                if matches!(fault, TransportFailure) {
                    return Err(error(
                        "fake_transport",
                        "synthetic-iam-token confidential detail",
                    ));
                }
                let mut headers = ureq::http::HeaderMap::new();
                headers.insert(
                    ureq::http::header::CONTENT_TYPE,
                    ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
                );
                if matches!(fault, DuplicateContentType) {
                    headers.append(
                        ureq::http::header::CONTENT_TYPE,
                        ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
                    );
                }
                match fault {
                    MissingContentType => {
                        headers.remove(ureq::http::header::CONTENT_TYPE);
                    }
                    BadContentType => {
                        headers.insert(
                            ureq::http::header::CONTENT_TYPE,
                            ureq::http::HeaderValue::from_static("text/html"),
                        );
                    }
                    OversizedHead => {
                        headers.insert(
                            "x-padding",
                            ureq::http::HeaderValue::from_str(
                                &"a".repeat(MAX_PROVIDER_RESPONSE_HEADER_BYTES + 1),
                            )
                            .unwrap(),
                        );
                    }
                    _ => {}
                }
                let status = if matches!(fault, Redirect) {
                    ureq::http::StatusCode::FOUND
                } else {
                    ureq::http::StatusCode::OK
                };
                let seconds = match fault {
                    ExpiredToken => now - 1,
                    ExcessLifetime => now + 601,
                    FractionBeyondLimit => now + 600,
                    _ => now + 60,
                };
                let base = OffsetDateTime::from_unix_timestamp(seconds as i64)
                    .unwrap()
                    .format(&Rfc3339)
                    .unwrap();
                let fraction = match fault {
                    Fraction0 => "",
                    Fraction6 => ".100000",
                    Fraction9 => ".100000000",
                    ExcessLifetime | FractionBeyondLimit => ".000000001",
                    InvalidFraction => ".1",
                    _ => ".100",
                };
                let expiry = format!("{}{fraction}Z", base.strip_suffix('Z').unwrap());
                let mut body = serde_json::to_vec(
                    &serde_json::json!({"accessToken":"synthetic-iam-token", "expireTime":expiry}),
                )
                .unwrap();
                match fault {
                        OversizedResponse => body.resize(MAX_TOKEN_RESPONSE_BYTES + 1, b' '),
                        TruncatedResponse => { body.pop(); },
                        ExtraField => body = format!(r#"{{"accessToken":"synthetic-iam-token","expireTime":"{expiry}","extra":true}}"#).into_bytes(),
                        DuplicateField => body = format!(r#"{{"accessToken":"synthetic-iam-token","accessToken":"synthetic-iam-token","expireTime":"{expiry}"}}"#).into_bytes(),
                        EmptyToken => body = serde_json::to_vec(&serde_json::json!({"accessToken":"", "expireTime":expiry})).unwrap(),
                        OversizedToken => body = serde_json::to_vec(&serde_json::json!({"accessToken":"a".repeat(MAX_TOKEN_BYTES + 1), "expireTime":expiry})).unwrap(),
                        InvalidTimestamp => body = serde_json::to_vec(&serde_json::json!({"accessToken":"synthetic-iam-token", "expireTime":"2026-02-30T12:00:00Z"})).unwrap(),
                        _ => {}
                    }
                if matches!(fault, ReadFailure) {
                    struct FailedRead;
                    impl Read for FailedRead {
                        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                            Err(std::io::Error::other("synthetic-iam-token"))
                        }
                    }
                    return read_sts_transport_response(status, headers, FailedRead);
                }
                read_sts_transport_response(status, headers, std::io::Cursor::new(body))
            };
            let iam_clock = || {
                let truth_checks = PROVIDER_TRUTH_CHECKS.with(std::cell::Cell::get);
                assert!(
                    truth_checks > last_iam_truth.replace(truth_checks),
                    "IAM clock must follow complete truth validation"
                );
                checks.set(checks.get() + 1);
                let response = checks.get() >= 3;
                if matches!(fault, ClockFailure)
                    || matches!(fault, ValidationClockFailure) && checks.get() >= 2
                {
                    return Err(error("fake_clock", "synthetic-iam-token"));
                }
                let mut observed = ProviderClockObservationV1 {
                    wall,
                    elapsed: Duration::ZERO,
                };
                match fault {
                    ExpiredBinding => {
                        observed.wall =
                            OffsetDateTime::from_unix_timestamp(binding_expiry as i64).unwrap()
                    }
                    ExpiredJwt | ExpiredSts => {
                        observed.wall += time::Duration::seconds(1);
                        observed.elapsed = Duration::from_secs(1);
                    }
                    TransactionExpired => observed.wall += time::Duration::seconds(600),
                    MonotonicExpired => observed.elapsed = Duration::from_secs(600),
                    ClockBackward => observed.wall -= time::Duration::nanoseconds(1),
                    MonotonicBackward => {
                        observed.elapsed = Duration::from_secs(u64::from(checks.get() == 1));
                    }
                    ResponseBindingExpired if response => {
                        observed.wall =
                            OffsetDateTime::from_unix_timestamp(binding_expiry as i64).unwrap()
                    }
                    ResponseJwtExpired | ResponseStsExpired if response => {
                        observed.wall += time::Duration::seconds(1);
                        observed.elapsed = Duration::from_secs(1);
                    }
                    ResponseTransactionExpired if response => {
                        observed.wall += time::Duration::seconds(600)
                    }
                    ResponseClockBackward if response => {
                        observed.wall -= time::Duration::nanoseconds(1)
                    }
                    PostParseExpired if checks.get() >= 4 => {
                        observed.wall =
                            OffsetDateTime::from_unix_timestamp(binding_expiry as i64).unwrap()
                    }
                    PostParseTokenExpired if checks.get() >= 4 => {
                        observed.wall += time::Duration::seconds(61)
                    }
                    ValidationBindingExpired if checks.get() >= 2 => {
                        observed.wall =
                            OffsetDateTime::from_unix_timestamp(binding_expiry as i64).unwrap()
                    }
                    ValidationJwtExpired | ValidationStsExpired if checks.get() >= 2 => {
                        observed.wall += time::Duration::seconds(1);
                        observed.elapsed = Duration::from_secs(1);
                    }
                    ValidationTransactionExpired if checks.get() >= 2 => {
                        observed.elapsed = Duration::from_secs(600)
                    }
                    _ => {}
                }
                Ok(observed)
            };
            let terminal = if matches!(fault, RealClock) {
                exchange_google_iam_v4_with_transport(owner, iam_transport)
            } else {
                exchange_google_iam_v4_with_transport_at(owner, iam_transport, iam_clock)
            };
            assert_eq!(terminal.core_invocations, iam_calls);
            let debug = format!("{terminal:?}");
            for prohibited in [
                "synthetic-iam-token",
                "synthetic-federated-token",
                "googleapis",
                "eyJ",
            ] {
                assert!(!debug.contains(prohibited), "{debug}");
            }
            terminal.result.is_ok()
        }
        _ => false,
    };
    assert_eq!(
        STS_TOKEN_DROPS.with(std::cell::Cell::get) - sts_drops,
        usize::from(sts_calls == 1)
    );
    assert_eq!(
        IAM_TOKEN_DROPS.with(std::cell::Cell::get) - iam_drops,
        usize::from(matched || matches!(fault, PostParseExpired | PostParseTokenExpired))
    );
    (sts_calls, iam_calls, matched)
}

struct PreparedOidcContextV1 {
    plan: SecretDeliveryProviderClientPlanV1,
    oidc: RetainedGithubOidcRequestCapabilityV1,
    transport_dependency_record_identity: String,
    configuration: UreqConfig,
}

/// Consumes the exact V2 transaction before retaining a GitHub OIDC request capability. This
/// checkpoint intentionally stops before constructing an HTTP client or dispatching a request.
pub(crate) fn prepare_secret_delivery_provider_transport_v1(
    transaction: &mut VerifiedSecretDeliveryTransactionBindingV2,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedSecretDeliveryProviderTransportV1, SecretDeliveryProviderClientError> {
    let binding = transaction
        .consume_before_provider_request()
        .map_err(provider_transport_binding_error)?;
    let runner_version = transaction.observed_runner_version().to_owned();
    prepare_after_v2_consumption_v1(
        binding,
        &runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
    )
}

pub(crate) fn prepare_secret_delivery_provider_transport_v4(
    transaction: &mut VerifiedSecretDeliveryTransactionBindingV4,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedSecretDeliveryProviderTransportV4, SecretDeliveryProviderClientError> {
    let signed_expectation = transaction
        .signed_transport_expectation()
        .map_err(provider_transport_binding_error)?;
    let binding = transaction
        .consume_before_provider_request()
        .map_err(provider_transport_binding_error)?;
    let runner_version = transaction.observed_runner_version().to_owned();
    prepare_after_v4_consumption_v1(
        binding,
        signed_expectation,
        &runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
    )
}

#[cfg(feature = "secret-delivery-pressure")]
pub(crate) fn prepare_secret_delivery_provider_transport_from_private_relay_v4(
    authority: ConsumedSecretDeliveryTransactionBindingV4,
    frame: ota_authority_protocol::CorrelatedProtectedGithubOidcPrivateFrameV1,
) -> Result<PreparedSecretDeliveryProviderTransportV4, SecretDeliveryProviderClientError> {
    let (binding, runner_version, candidate, signed_expectation) = authority.into_provider_parts();
    let request_url =
        ProtectedGithubOidcRequestUrlV1::new(frame.url_bytes().to_vec()).map_err(|_| {
            error(
                "secret_delivery_provider_transport_oidc_input_invalid",
                "relayed GitHub OIDC request URL is invalid",
            )
        })?;
    let bearer = ProtectedProviderValue::new(frame.bearer_bytes().to_vec())?;
    drop(frame);
    validate_bearer(&bearer)?;
    let endpoint_profile = github_actions_oidc_request_endpoint_profile_v1().map_err(|_| {
        error(
            "secret_delivery_provider_transport_endpoint_invalid",
            "canonical GitHub OIDC endpoint profile is invalid",
        )
    })?;
    let endpoint_input = GithubActionsOidcEndpointObservationInputV1 {
        schema_version: 1,
        request_url,
        runner_environment: "self-hosted".into(),
        runner_os: "linux".into(),
        runner_architecture: "x64".into(),
        runner_version: runner_version.clone(),
        protected_launcher_capability_projection_identity: binding.projection_identity.clone(),
    };
    let endpoint_observation =
        resolve_github_actions_oidc_endpoint_observation_v1(&endpoint_profile, &endpoint_input)
            .map_err(|_| {
                error(
                    "secret_delivery_provider_transport_endpoint_invalid",
                    "relayed GitHub OIDC endpoint is outside the retained profile",
                )
            })?;
    prepare_after_v4_consumption_with_bearer_v1(
        binding,
        signed_expectation,
        &runner_version,
        &candidate,
        &endpoint_profile,
        endpoint_input,
        endpoint_observation,
        bearer,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_secret_delivery_provider_transport_at_v4(
    transaction: &mut VerifiedSecretDeliveryTransactionBindingV4,
    verifier: &crate::protected_capability_observation::RetainedCapabilityProjectionVerifierV1,
    observed_at_unix_seconds: u64,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedSecretDeliveryProviderTransportV4, SecretDeliveryProviderClientError> {
    let signed_expectation = transaction
        .signed_transport_expectation()
        .map_err(provider_transport_binding_error)?;
    let binding = transaction
        .consume_at(verifier, observed_at_unix_seconds)
        .map_err(provider_transport_binding_error)?;
    let runner_version = transaction.observed_runner_version().to_owned();
    prepare_after_v4_consumption_v1(
        binding,
        signed_expectation,
        &runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_secret_delivery_provider_transport_at_v1(
    transaction: &mut VerifiedSecretDeliveryTransactionBindingV2,
    verifier: &crate::protected_capability_observation::RetainedCapabilityProjectionVerifierV1,
    observed_at_unix_seconds: u64,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedSecretDeliveryProviderTransportV1, SecretDeliveryProviderClientError> {
    let binding = transaction
        .consume_at(verifier, observed_at_unix_seconds)
        .map_err(provider_transport_binding_error)?;
    let runner_version = transaction.observed_runner_version().to_owned();
    prepare_after_v2_consumption_v1(
        binding,
        &runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
    )
}

fn prepare_after_v2_consumption_v1(
    binding: ota_authority_protocol::ProtectedLauncherSecretDeliveryTransactionBindingV2,
    retained_runner_version: &str,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedSecretDeliveryProviderTransportV1, SecretDeliveryProviderClientError> {
    let context = prepare_oidc_context_v1(
        &binding.secret_transaction_candidate_identity,
        &binding.projection_identity,
        retained_runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
    )?;
    let capability = ConsumedSecretDeliveryProviderCapabilityV1 {
        binding,
        plan: context.plan,
        transport_dependency_record_identity: context.transport_dependency_record_identity.clone(),
    };
    Ok(PreparedSecretDeliveryProviderTransportV1 {
        capability,
        oidc: context.oidc,
        transport_dependency_record_identity: context.transport_dependency_record_identity,
        configuration: context.configuration,
        maximum_response_body_bytes: MAX_SECRET_RESPONSE_BYTES,
    })
}

fn prepare_after_v4_consumption_v1(
    binding: ota_authority_protocol::ProtectedLauncherSecretDeliveryTransactionBindingV4,
    signed_expectation: (
        SecretDeliveryTransportDependencyFeatureGraphV1,
        SecretDeliveryTransportDependencyRecordV1,
    ),
    retained_runner_version: &str,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedSecretDeliveryProviderTransportV4, SecretDeliveryProviderClientError> {
    let bearer = take_github_oidc_bearer_v1(endpoint_input.request_url.as_bytes())?;
    prepare_after_v4_consumption_with_bearer_v1(
        binding,
        signed_expectation,
        retained_runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
        bearer,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_after_v4_consumption_with_bearer_v1(
    binding: ota_authority_protocol::ProtectedLauncherSecretDeliveryTransactionBindingV4,
    signed_expectation: (
        SecretDeliveryTransportDependencyFeatureGraphV1,
        SecretDeliveryTransportDependencyRecordV1,
    ),
    retained_runner_version: &str,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
    bearer: ProtectedProviderValue,
) -> Result<PreparedSecretDeliveryProviderTransportV4, SecretDeliveryProviderClientError> {
    let (transport_dependency_feature_graph, transport_dependency_record) = signed_expectation;
    if binding.transport_dependency_record_identity
        != candidate.candidate().transport_dependency_record_identity
        || binding.transport_dependency_record_identity
            != transport_dependency_record.record_identity
    {
        return Err(error(
            "secret_delivery_provider_transport_dependencies_mismatch",
            "V4 binding does not match the retained transaction dependency record",
        ));
    }
    let context = prepare_oidc_context_with_bearer_v1(
        &binding.secret_transaction_candidate_identity,
        &binding.projection_identity,
        retained_runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
        bearer,
    )?;
    let capability = ConsumedSecretDeliveryProviderCapabilityV4 {
        binding,
        runner_version: retained_runner_version.to_owned(),
        candidate: candidate.clone(),
        plan: context.plan,
        transport_dependency_record_identity: context.transport_dependency_record_identity.clone(),
        transport_dependency_feature_graph,
        transport_dependency_record,
    };
    let prepared = PreparedSecretDeliveryProviderTransportV4 {
        capability,
        oidc: context.oidc,
        transport_dependency_record_identity: context.transport_dependency_record_identity,
        configuration: context.configuration,
        maximum_response_body_bytes: MAX_SECRET_RESPONSE_BYTES,
    };
    prepared.verify_signed_transport_expectation()?;
    Ok(prepared)
}

#[allow(clippy::too_many_arguments)]
fn prepare_oidc_context_v1(
    binding_candidate_identity: &str,
    binding_projection_identity: &str,
    retained_runner_version: &str,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
) -> Result<PreparedOidcContextV1, SecretDeliveryProviderClientError> {
    let bearer = take_github_oidc_bearer_v1(endpoint_input.request_url.as_bytes())?;
    prepare_oidc_context_with_bearer_v1(
        binding_candidate_identity,
        binding_projection_identity,
        retained_runner_version,
        candidate,
        endpoint_profile,
        endpoint_input,
        endpoint_observation,
        bearer,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_oidc_context_with_bearer_v1(
    binding_candidate_identity: &str,
    binding_projection_identity: &str,
    retained_runner_version: &str,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    endpoint_profile: &GithubActionsOidcRequestEndpointProfileV1,
    endpoint_input: GithubActionsOidcEndpointObservationInputV1,
    endpoint_observation: ResolvedGithubActionsOidcEndpointObservationV1,
    bearer: ProtectedProviderValue,
) -> Result<PreparedOidcContextV1, SecretDeliveryProviderClientError> {
    let plan = derive_secret_delivery_provider_client_plan_v1(candidate)?;
    if plan.transaction_candidate_identity != binding_candidate_identity {
        return Err(error(
            "secret_delivery_provider_transport_candidate_mismatch",
            "consumed binding does not match the provider operation plan",
        ));
    }
    if binding_projection_identity
        != endpoint_input.protected_launcher_capability_projection_identity
        || retained_runner_version != endpoint_input.runner_version
    {
        return Err(error(
            "secret_delivery_provider_transport_observation_mismatch",
            "OIDC endpoint observation does not match the consumed transaction",
        ));
    }
    if plan.operations.len() != 1 {
        return Err(error(
            "secret_delivery_provider_transport_operation_ambiguous",
            "the initial provider transport requires exactly one selected operation",
        ));
    }
    let transport_dependency_record_identity = plan.operations[0]
        .transport_dependency_record_identity
        .clone();
    if transport_dependency_record_identity
        != candidate.candidate().transport_dependency_record_identity
    {
        return Err(error(
            "secret_delivery_provider_transport_dependencies_mismatch",
            "provider operation does not match the retained transaction dependency record",
        ));
    }
    verify_github_actions_oidc_endpoint_observation_v1(
        endpoint_profile,
        &endpoint_input,
        &endpoint_observation,
    )
    .map_err(|_| {
        error(
            "secret_delivery_provider_transport_endpoint_invalid",
            "OIDC request capability is outside the retained endpoint profile",
        )
    })?;
    let configuration = fixed_transport_configuration_v1();
    verify_fixed_transport_configuration_v1(&configuration, MAX_SECRET_RESPONSE_BYTES)?;
    let oidc = RetainedGithubOidcRequestCapabilityV1 {
        endpoint_profile: endpoint_profile.clone(),
        endpoint_input,
        endpoint_observation,
        operation: plan.operations[0].clone(),
        bearer,
    };
    Ok(PreparedOidcContextV1 {
        plan,
        oidc,
        transport_dependency_record_identity,
        configuration,
    })
}

fn take_github_oidc_bearer_v1(
    expected_request_url: &[u8],
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    let owner = GITHUB_OIDC_CAPABILITY_OWNER.get_or_init(|| Mutex::new(false));
    let mut consumed = owner
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if *consumed {
        return Err(error(
            "secret_delivery_provider_transport_oidc_input_consumed",
            "GitHub OIDC request capability has already been consumed",
        ));
    }
    *consumed = true;
    let request_url = std::env::var_os(ACTIONS_ID_TOKEN_REQUEST_URL);
    let bearer = std::env::var_os(ACTIONS_ID_TOKEN_REQUEST_TOKEN);
    let request_url = request_url
        .ok_or_else(|| {
            error(
                "secret_delivery_provider_transport_oidc_input_missing",
                "GitHub OIDC request URL is unavailable after transaction consumption",
            )
        })?
        .into_string()
        .map_err(|_| {
            error(
                "secret_delivery_provider_transport_oidc_input_invalid",
                "GitHub OIDC request URL is not valid UTF-8",
            )
        })?;
    let bearer = bearer
        .ok_or_else(|| {
            error(
                "secret_delivery_provider_transport_oidc_input_missing",
                "GitHub OIDC bearer is unavailable after transaction consumption",
            )
        })?
        .into_string()
        .map_err(|_| {
            error(
                "secret_delivery_provider_transport_oidc_input_invalid",
                "GitHub OIDC bearer is not valid UTF-8",
            )
        })?;
    if request_url.is_empty() || request_url.as_bytes() != expected_request_url {
        return Err(error(
            "secret_delivery_provider_transport_oidc_input_mismatch",
            "GitHub OIDC request URL does not match the retained endpoint observation",
        ));
    }
    let bearer = ProtectedProviderValue::new(bearer.into_bytes())?;
    validate_bearer(&bearer)?;
    Ok(bearer)
}

#[cfg(test)]
pub(crate) fn reset_github_oidc_capability_owner_for_test() {
    if let Some(owner) = GITHUB_OIDC_CAPABILITY_OWNER.get() {
        *owner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = false;
    }
}

fn provider_transport_binding_error(
    _error: SecretDeliveryTransactionBindingError,
) -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_provider_transport_binding_invalid",
        "transaction consumption refused before provider transport preparation",
    )
}

fn fixed_transport_configuration_v1() -> UreqConfig {
    UreqConfig::builder()
        .https_only(true)
        .proxy(None)
        .max_redirects(0)
        .max_response_header_size(MAX_PROVIDER_RESPONSE_HEADER_BYTES)
        .timeout_global(Some(PROVIDER_TIMEOUT_GLOBAL))
        .timeout_resolve(Some(PROVIDER_TIMEOUT_RESOLVE))
        .timeout_connect(Some(PROVIDER_TIMEOUT_CONNECT))
        .timeout_send_request(Some(PROVIDER_TIMEOUT_SEND))
        .timeout_send_body(Some(PROVIDER_TIMEOUT_SEND))
        .timeout_recv_response(Some(PROVIDER_TIMEOUT_RECEIVE))
        .timeout_recv_body(Some(PROVIDER_TIMEOUT_RECEIVE))
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                .root_certs(RootCerts::WebPki)
                .client_cert(None)
                .use_sni(true)
                .disable_verification(false)
                .build(),
        )
        .build()
}

fn verify_fixed_transport_configuration_v1(
    configuration: &UreqConfig,
    maximum_response_body_bytes: usize,
) -> Result<(), SecretDeliveryProviderClientError> {
    let tls = configuration.tls_config();
    let timeouts = configuration.timeouts();
    if !configuration.https_only()
        || configuration.proxy().is_some()
        || configuration.max_redirects() != 0
        || configuration.max_response_header_size() != MAX_PROVIDER_RESPONSE_HEADER_BYTES
        || timeouts.global != Some(PROVIDER_TIMEOUT_GLOBAL)
        || timeouts.resolve != Some(PROVIDER_TIMEOUT_RESOLVE)
        || timeouts.connect != Some(PROVIDER_TIMEOUT_CONNECT)
        || timeouts.send_request != Some(PROVIDER_TIMEOUT_SEND)
        || timeouts.send_body != Some(PROVIDER_TIMEOUT_SEND)
        || timeouts.recv_response != Some(PROVIDER_TIMEOUT_RECEIVE)
        || timeouts.recv_body != Some(PROVIDER_TIMEOUT_RECEIVE)
        || maximum_response_body_bytes != MAX_SECRET_RESPONSE_BYTES
        || tls.provider() != TlsProvider::Rustls
        || !matches!(tls.root_certs(), RootCerts::WebPki)
        || tls.client_cert().is_some()
        || !tls.use_sni()
        || tls.disable_verification()
    {
        return Err(error(
            "secret_delivery_provider_transport_posture_invalid",
            "provider transport posture is not the fixed direct-TLS configuration",
        ));
    }
    Ok(())
}

pub(crate) fn derive_secret_delivery_provider_client_plan_v1(
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
) -> Result<SecretDeliveryProviderClientPlanV1, SecretDeliveryProviderClientError> {
    let candidate = candidate.candidate();
    if candidate.identity
        != secret_delivery_transaction_candidate_identity(candidate).map_err(|_| {
            error(
                "secret_delivery_provider_candidate_invalid",
                "provider plan candidate identity is invalid",
            )
        })?
        || candidate.realizations.is_empty()
    {
        return Err(error(
            "secret_delivery_provider_candidate_invalid",
            "provider plans require one semantically valid non-empty candidate",
        ));
    }

    let mut operations = candidate
        .realizations
        .iter()
        .map(operation_plan)
        .collect::<Result<Vec<_>, _>>()?;
    operations.sort_by(|left, right| left.realization_identity.cmp(&right.realization_identity));
    if operations
        .windows(2)
        .any(|pair| pair[0].realization_identity == pair[1].realization_identity)
    {
        return Err(error(
            "secret_delivery_provider_realization_duplicate",
            "provider plans cannot contain duplicate realization identities",
        ));
    }

    let mut plan = SecretDeliveryProviderClientPlanV1 {
        schema_version: 1,
        identity: String::new(),
        transaction_candidate_identity: candidate.identity.clone(),
        operations,
    };
    plan.identity = plan_identity(&plan)?;
    Ok(plan)
}

pub(crate) fn verify_secret_delivery_provider_client_plan_v1(
    plan: &SecretDeliveryProviderClientPlanV1,
    candidate: &SemanticallyVerifiedSecretDeliveryTransactionCandidate,
) -> Result<(), SecretDeliveryProviderClientError> {
    if plan != &derive_secret_delivery_provider_client_plan_v1(candidate)? {
        return Err(error(
            "secret_delivery_provider_plan_reconciliation_failed",
            "provider client plan does not match retained candidate truth",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct GoogleStsOperationTargetV1 {
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub workload_identity_pool: String,
    pub workload_identity_provider: String,
    pub sts_audience: String,
    pub sts_url: String,
}

/// Pure target derivation shared by runtime planning and offline inspection. This produces no
/// candidate, request, transport or execution authority.
pub(crate) fn derive_google_sts_operation_target_v1(
    tuple: &SecretDeliveryInvocationBindingInput,
) -> Result<GoogleStsOperationTargetV1, SecretDeliveryProviderClientError> {
    validate_google_tuple(tuple).map_err(|_| {
        error(
            "secret_delivery_provider_tuple_invalid",
            "provider operation tuple is not provider-canonical",
        )
    })?;
    if tuple.oidc_issuer != "https://token.actions.githubusercontent.com"
        || tuple.oidc_audience
            != format!(
                "https://iam.googleapis.com/{}",
                tuple.workload_identity_provider
            )
    {
        return Err(error(
            "secret_delivery_provider_tuple_invalid",
            "OIDC issuer or audience does not match the protected WIF provider",
        ));
    }
    Ok(GoogleStsOperationTargetV1 {
        oidc_issuer: tuple.oidc_issuer.clone(),
        oidc_audience: tuple.oidc_audience.clone(),
        workload_identity_pool: tuple.workload_identity_pool.clone(),
        workload_identity_provider: tuple.workload_identity_provider.clone(),
        sts_audience: format!("//iam.googleapis.com/{}", tuple.workload_identity_provider),
        sts_url: STS_URL.into(),
    })
}

fn operation_plan(
    realization: &SecretDeliveryTransactionCandidateRealization,
) -> Result<SecretDeliveryProviderOperationPlanV1, SecretDeliveryProviderClientError> {
    let expected_transport_dependency_record_identity = crate::secret_delivery_transport_dependencies::embedded_transport_dependency_record_identity_v1()
        .map_err(|_| {
            error(
                "secret_delivery_provider_transport_dependencies_invalid",
                "embedded transport dependency record is invalid",
            )
        })?;
    if realization.transport_dependency_record_identity
        != expected_transport_dependency_record_identity
    {
        return Err(error(
            "secret_delivery_provider_transport_dependencies_mismatch",
            "provider operation does not match the active transport dependency record",
        ));
    }
    let target = &realization.target;
    if target.operating_system != SecretDeliveryOperatingSystem::Linux
        || target.architecture != SecretDeliveryArchitecture::X86_64
        || target.runtime != SecretDeliveryRuntime::GithubActions
        || target.execution_mode != SecretDeliveryExecutionMode::Native
        || target.recipient_boundary
            != SecretDeliveryRecipientBoundary::TransientSelectedProcessTree
        || realization.oidc_issuer != "https://token.actions.githubusercontent.com"
    {
        return Err(error(
            "secret_delivery_provider_target_unsupported",
            "provider plan target is outside the initial Linux/X86_64 GitHub Actions profile",
        ));
    }
    let tuple = SecretDeliveryInvocationBindingInput {
        schema_version: 1,
        profile_semantic_identity: realization.profile_semantic_identity.clone(),
        implementation_subject_identity: realization.implementation_subject_identity.clone(),
        transport_dependency_record_identity: realization
            .transport_dependency_record_identity
            .clone(),
        requirement_identity: realization.requirement_identity.clone(),
        provider_binding_identity: realization.provider_binding_identity.clone(),
        provider_binding_source_identity: realization.provider_binding_source_identity.clone(),
        oidc_issuer: realization.oidc_issuer.clone(),
        oidc_audience: realization.oidc_audience.clone(),
        oidc_claims: realization
            .oidc_claims
            .iter()
            .map(|(claim, expected_value)| GithubOidcClaimValue {
                claim: *claim,
                expected_value: expected_value.clone(),
            })
            .collect(),
        workload_identity_pool: realization.workload_identity_pool.clone(),
        workload_identity_provider: realization.workload_identity_provider.clone(),
        service_account: realization.service_account.clone(),
        google_project: realization.google_project.clone(),
        secret_resource: realization.secret_resource.clone(),
        secret_version: realization.secret_version,
    };
    let target = derive_google_sts_operation_target_v1(&tuple)?;
    let secret_version_resource = format!(
        "{}/versions/{}",
        realization.secret_resource, realization.secret_version
    );
    Ok(SecretDeliveryProviderOperationPlanV1 {
        realization_identity: realization.realization_identity.clone(),
        invocation_binding_identity: realization.invocation_binding_identity.clone(),
        transport_dependency_record_identity: realization
            .transport_dependency_record_identity
            .clone(),
        oidc_issuer: target.oidc_issuer,
        oidc_audience: target.oidc_audience,
        sts_audience: target.sts_audience,
        sts_url: target.sts_url,
        service_account_token_url: format!(
            "{IAM_CREDENTIALS_ORIGIN}/v1/projects/-/serviceAccounts/{}:generateAccessToken",
            realization.service_account
        ),
        secret_version_url: format!("{SECRET_MANAGER_ORIGIN}/v1/{secret_version_resource}:access"),
        secret_version_resource,
    })
}

pub(crate) fn build_github_oidc_request_v1(
    profile: &GithubActionsOidcRequestEndpointProfileV1,
    retained_input: &GithubActionsOidcEndpointObservationInputV1,
    observation: &ResolvedGithubActionsOidcEndpointObservationV1,
    bearer: &ProtectedProviderValue,
    operation: &SecretDeliveryProviderOperationPlanV1,
) -> Result<ProtectedProviderRequestV1, SecretDeliveryProviderClientError> {
    verify_github_actions_oidc_endpoint_observation_v1(profile, retained_input, observation)
        .map_err(|_| {
            error(
                "secret_delivery_provider_oidc_request_invalid",
                "OIDC request inputs are outside the retained endpoint profile",
            )
        })?;
    Ok(ProtectedProviderRequestV1 {
        method: ProviderHttpMethod::Get,
        url: format!(
            "{}&audience={}",
            retained_input.request_url.as_str().map_err(|_| {
                error(
                    "secret_delivery_provider_oidc_request_invalid",
                    "OIDC request URL is invalid",
                )
            })?,
            percent_encode(operation.oidc_audience.as_bytes())
        ),
        media_type: None,
        authorization: Some(bearer_authorization_borrowed(bearer)?),
        body: Vec::new(),
    })
}

pub(crate) fn parse_github_oidc_response_v1(
    bytes: &[u8],
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response<'a> {
        value: &'a str,
    }
    let response: Response<'_> = parse_bounded_json(bytes, MAX_OIDC_RESPONSE_BYTES)?;
    validate_compact_jwt(response.value)?;
    ProtectedProviderValue::new(response.value.as_bytes().to_vec())
}

pub(crate) fn build_google_sts_request_v1(
    operation: &SecretDeliveryProviderOperationPlanV1,
    subject_token: &ProtectedProviderValue,
) -> Result<ProtectedProviderRequestV1, SecretDeliveryProviderClientError> {
    let subject_token = subject_token.as_utf8()?;
    validate_compact_jwt(subject_token)?;
    let fields = [
        ("grant_type", TOKEN_EXCHANGE_GRANT),
        ("audience", operation.sts_audience.as_str()),
        ("scope", CLOUD_PLATFORM_SCOPE),
        ("requested_token_type", ACCESS_TOKEN_TYPE),
        ("subject_token", subject_token),
        ("subject_token_type", JWT_TOKEN_TYPE),
    ];
    Ok(ProtectedProviderRequestV1 {
        method: ProviderHttpMethod::Post,
        url: operation.sts_url.clone(),
        media_type: Some(FORM_MEDIA_TYPE),
        authorization: None,
        body: encode_form(&fields),
    })
}

pub(crate) fn parse_google_sts_response_v1(
    bytes: &[u8],
) -> Result<StsAccessTokenV1, SecretDeliveryProviderClientError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        access_token: String,
        issued_token_type: String,
        token_type: String,
        expires_in: u64,
    }
    let response: Response = parse_bounded_json(bytes, MAX_TOKEN_RESPONSE_BYTES)?;
    if response.access_token.is_empty()
        || response.access_token.len() > MAX_TOKEN_BYTES
        || response.issued_token_type != ACCESS_TOKEN_TYPE
        || response.token_type != "Bearer"
        || response.expires_in == 0
        || response.expires_in > 3600
    {
        return Err(error(
            "secret_delivery_provider_sts_response_invalid",
            "Google STS response is outside the admitted access-token profile",
        ));
    }
    Ok(StsAccessTokenV1 {
        token: ProtectedProviderValue::new(response.access_token.into_bytes())?,
        expires_in_seconds: response.expires_in,
    })
}

pub(crate) fn build_service_account_token_request_v1(
    operation: &SecretDeliveryProviderOperationPlanV1,
    federated_token: &StsAccessTokenV1,
) -> Result<ProtectedProviderRequestV1, SecretDeliveryProviderClientError> {
    #[derive(Serialize)]
    struct Body<'a> {
        scope: [&'a str; 1],
        lifetime: &'a str,
    }
    let body = serde_json::to_vec(&Body {
        scope: [CLOUD_PLATFORM_SCOPE],
        lifetime: "600s",
    })
    .map_err(|_| invalid_request_serialization())?;
    Ok(ProtectedProviderRequestV1 {
        method: ProviderHttpMethod::Post,
        url: operation.service_account_token_url.clone(),
        media_type: Some(JSON_MEDIA_TYPE),
        authorization: Some(bearer_authorization_borrowed(&federated_token.token)?),
        body,
    })
}

pub(crate) fn parse_service_account_token_response_v1(
    bytes: &[u8],
    observed_at: OffsetDateTime,
) -> Result<ServiceAccountAccessTokenV1, SecretDeliveryProviderClientError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        #[serde(rename = "accessToken")]
        access_token: String,
        #[serde(rename = "expireTime")]
        expire_time: String,
    }
    let response: Response = parse_bounded_json(bytes, MAX_TOKEN_RESPONSE_BYTES)?;
    let timestamp = response.expire_time.as_bytes();
    let fraction_digits = match timestamp.len() {
        20 => 0,
        24 => 3,
        27 => 6,
        30 => 9,
        _ => return Err(invalid_iam_response()),
    };
    // Google's canonical timestamps retain 3/6/9 fractional digits, including trailing zeros.
    if timestamp[4] != b'-'
        || timestamp[7] != b'-'
        || timestamp[10] != b'T'
        || timestamp[13] != b':'
        || timestamp[16] != b':'
        || timestamp[timestamp.len() - 1] != b'Z'
        || &timestamp[17..19] == b"60"
        || !timestamp[..19]
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7 | 10 | 13 | 16) || byte.is_ascii_digit())
        || (fraction_digits != 0
            && (timestamp[19] != b'.'
                || !timestamp[20..20 + fraction_digits]
                    .iter()
                    .all(u8::is_ascii_digit)))
    {
        return Err(invalid_iam_response());
    }
    let expires_at = OffsetDateTime::parse(&response.expire_time, &Rfc3339)
        .map_err(|_| invalid_iam_response())?;
    let latest_expiry = observed_at
        .checked_add(time::Duration::seconds(600))
        .ok_or_else(invalid_iam_response)?;
    if response.access_token.is_empty()
        || response.access_token.len() > MAX_TOKEN_BYTES
        || observed_at.unix_timestamp() < 0
        || expires_at <= observed_at
        || expires_at > latest_expiry
    {
        return Err(invalid_iam_response());
    }
    Ok(ServiceAccountAccessTokenV1 {
        token: ProtectedProviderValue::new(response.access_token.into_bytes())?,
        expires_at,
    })
}

pub(crate) fn build_secret_manager_access_request_v1(
    operation: &SecretDeliveryProviderOperationPlanV1,
    access_token: ServiceAccountAccessTokenV1,
) -> Result<ProtectedProviderRequestV1, SecretDeliveryProviderClientError> {
    Ok(ProtectedProviderRequestV1 {
        method: ProviderHttpMethod::Get,
        url: operation.secret_version_url.clone(),
        media_type: None,
        authorization: Some(bearer_authorization(access_token.token)?),
        body: Vec::new(),
    })
}

pub(crate) fn parse_secret_manager_access_response_v1(
    bytes: &[u8],
    operation: &SecretDeliveryProviderOperationPlanV1,
) -> Result<SecretManagerPayloadV1, SecretDeliveryProviderClientError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Payload {
        data: String,
        #[serde(rename = "dataCrc32c")]
        data_crc32c: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        name: String,
        payload: Payload,
    }
    let response: Response = parse_bounded_json(bytes, MAX_SECRET_RESPONSE_BYTES)?;
    if response.name != operation.secret_version_resource
        || (response.payload.data_crc32c.starts_with('0') && response.payload.data_crc32c != "0")
    {
        return Err(invalid_secret_response());
    }
    let value = STANDARD
        .decode(&response.payload.data)
        .map_err(|_| invalid_secret_response())?;
    if value.is_empty()
        || value.len() > MAX_SECRET_BYTES
        || STANDARD.encode(&value) != response.payload.data
    {
        return Err(invalid_secret_response());
    }
    let expected_crc = response
        .payload
        .data_crc32c
        .parse::<u32>()
        .map_err(|_| invalid_secret_response())?;
    if expected_crc.to_string() != response.payload.data_crc32c {
        return Err(invalid_secret_response());
    }
    let actual_crc = crc32c(&value);
    if actual_crc != expected_crc {
        return Err(error(
            "secret_delivery_provider_secret_checksum_mismatch",
            "Secret Manager payload CRC32C does not match the returned bytes",
        ));
    }
    Ok(SecretManagerPayloadV1 {
        value: ProtectedProviderValue::new(value)?,
        crc32c: actual_crc,
    })
}

fn plan_identity(
    plan: &SecretDeliveryProviderClientPlanV1,
) -> Result<String, SecretDeliveryProviderClientError> {
    if plan.schema_version != 1 {
        return Err(error(
            "secret_delivery_provider_plan_version_unsupported",
            "provider client plan version is unsupported",
        ));
    }
    let canonical = serde_jcs::to_vec(&PlanIdentityPayload {
        schema_version: plan.schema_version,
        transaction_candidate_identity: &plan.transaction_candidate_identity,
        operations: plan
            .operations
            .iter()
            .map(|operation| OperationIdentityPayload {
                realization_identity: &operation.realization_identity,
                invocation_binding_identity: &operation.invocation_binding_identity,
                transport_dependency_record_identity: &operation
                    .transport_dependency_record_identity,
                oidc_issuer: &operation.oidc_issuer,
                oidc_audience: &operation.oidc_audience,
                sts_audience: &operation.sts_audience,
                sts_url: &operation.sts_url,
                service_account_token_url: &operation.service_account_token_url,
                secret_version_url: &operation.secret_version_url,
                secret_version_resource: &operation.secret_version_resource,
            })
            .collect(),
    })
    .map_err(|_| invalid_request_serialization())?;
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest([PLAN_DOMAIN, canonical.as_slice()].concat())
    ))
}

fn parse_bounded_json<'a, T: Deserialize<'a>>(
    bytes: &'a [u8],
    maximum: usize,
) -> Result<T, SecretDeliveryProviderClientError> {
    if bytes.is_empty() || bytes.len() > maximum {
        return Err(error(
            "secret_delivery_provider_response_size_invalid",
            "provider response is empty or exceeds its bounded size",
        ));
    }
    serde_json::from_slice(bytes).map_err(|_| {
        error(
            "secret_delivery_provider_response_invalid",
            "provider response is malformed or contains unsupported fields",
        )
    })
}

fn validate_compact_jwt(value: &str) -> Result<(), SecretDeliveryProviderClientError> {
    if value.len() > MAX_OIDC_RESPONSE_BYTES
        || value.bytes().any(|byte| byte.is_ascii_whitespace())
        || value.split('.').count() != 3
        || value
            .split('.')
            .any(|segment| !canonical_base64url_segment(segment))
    {
        return Err(error(
            "secret_delivery_provider_oidc_response_invalid",
            "GitHub OIDC response does not contain one canonical compact JWT",
        ));
    }
    Ok(())
}

#[cfg(feature = "secret-delivery-pressure")]
fn reconcile_github_oidc_jwt_claims_v4_at(
    jwt: &ProtectedProviderValue,
    prepared: &PreparedSecretDeliveryProviderTransportV4,
    now: u64,
) -> Result<(), SecretDeliveryProviderClientError> {
    reconcile_github_oidc_jwt_context_v4(jwt, prepared)?.validate_at(now)
}

#[cfg(feature = "secret-delivery-pressure")]
fn reconcile_github_oidc_jwt_context_v4(
    jwt: &ProtectedProviderValue,
    prepared: &PreparedSecretDeliveryProviderTransportV4,
) -> Result<GithubOidcJwtFreshnessV1, SecretDeliveryProviderClientError> {
    prepared.verify_signed_transport_expectation()?;
    let operation = prepared
        .capability
        .plan
        .operations
        .first()
        .ok_or_else(oidc_dispatch_refused)?;
    let realizations: Vec<_> = prepared
        .capability
        .candidate
        .candidate()
        .realizations
        .iter()
        .filter(|realization| {
            realization.realization_identity == operation.realization_identity
                && realization.invocation_binding_identity == operation.invocation_binding_identity
        })
        .collect();
    let [realization] = realizations.as_slice() else {
        return Err(oidc_dispatch_refused());
    };
    let freshness = reconcile_github_oidc_jwt_context_v1(
        jwt,
        operation,
        realization,
        &prepared.oidc.endpoint_input.runner_environment,
    )?;
    #[cfg(test)]
    PROVIDER_TRUTH_CHECKS.with(|count| count.set(count.get() + 1));
    Ok(freshness)
}

#[cfg(feature = "secret-delivery-pressure")]
fn reconcile_github_oidc_jwt_claims_at_v1(
    jwt: &ProtectedProviderValue,
    operation: &SecretDeliveryProviderOperationPlanV1,
    realization: &SecretDeliveryTransactionCandidateRealization,
    expected_runner_environment: &str,
    now_unix_seconds: u64,
) -> Result<(), SecretDeliveryProviderClientError> {
    reconcile_github_oidc_jwt_context_v1(jwt, operation, realization, expected_runner_environment)?
        .validate_at(now_unix_seconds)
}

#[cfg(feature = "secret-delivery-pressure")]
fn reconcile_github_oidc_jwt_context_v1(
    jwt: &ProtectedProviderValue,
    operation: &SecretDeliveryProviderOperationPlanV1,
    realization: &SecretDeliveryTransactionCandidateRealization,
    expected_runner_environment: &str,
) -> Result<GithubOidcJwtFreshnessV1, SecretDeliveryProviderClientError> {
    if operation.oidc_issuer != realization.oidc_issuer
        || operation.oidc_audience != realization.oidc_audience
        || expected_runner_environment != "self-hosted"
    {
        return Err(oidc_dispatch_refused());
    }

    let payload = decode_compact_jwt_payload(jwt.as_utf8()?)?;
    let claims = parse_unique_jwt_claims(&payload.0)?;
    require_string_claim(claims.iss, &operation.oidc_issuer)?;
    require_string_claim(claims.aud, &operation.oidc_audience)?;
    require_string_claim(claims.runner_environment, expected_runner_environment)?;

    for claim in [
        GithubOidcClaim::Subject,
        GithubOidcClaim::RepositoryId,
        GithubOidcClaim::RepositoryOwnerId,
        GithubOidcClaim::WorkflowRef,
        GithubOidcClaim::WorkflowSha,
        GithubOidcClaim::Ref,
        GithubOidcClaim::Sha,
        GithubOidcClaim::ActorId,
        GithubOidcClaim::EventName,
        GithubOidcClaim::RunId,
        GithubOidcClaim::RunAttempt,
    ] {
        let expected = realization
            .oidc_claims
            .get(&claim)
            .ok_or_else(oidc_dispatch_refused)?;
        require_string_claim(claims.github_claim(claim), expected)?;
    }

    let workflow_ref = realization
        .oidc_claims
        .get(&GithubOidcClaim::WorkflowRef)
        .ok_or_else(oidc_dispatch_refused)?;
    let reference = realization
        .oidc_claims
        .get(&GithubOidcClaim::Ref)
        .ok_or_else(oidc_dispatch_refused)?;
    let (owner, repository) = parse_bound_workflow_repository(workflow_ref, reference)?;
    let repository_identity = format!("{owner}/{repository}");
    require_string_claim(claims.repository, &repository_identity)?;
    require_string_claim(claims.repository_owner, owner)?;
    Ok(GithubOidcJwtFreshnessV1 {
        nbf: claims.nbf.ok_or_else(oidc_dispatch_refused)?,
        issued_at: claims.issued_at.ok_or_else(oidc_dispatch_refused)?,
        expires_at: claims.expires_at.ok_or_else(oidc_dispatch_refused)?,
    })
}

#[cfg(feature = "secret-delivery-pressure")]
fn decode_compact_jwt_payload(
    jwt: &str,
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    validate_compact_jwt(jwt)?;
    let payload = jwt.split('.').nth(1).ok_or_else(oidc_dispatch_refused)?;
    let decoded = ProtectedProviderValue::new(
        URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| oidc_dispatch_refused())?,
    )?;
    let canonical = ProtectedProviderValue::new(URL_SAFE_NO_PAD.encode(&decoded.0).into_bytes())?;
    if canonical.0.as_slice() != payload.as_bytes() {
        return Err(oidc_dispatch_refused());
    }
    Ok(decoded)
}

#[cfg(feature = "secret-delivery-pressure")]
struct UniqueJwtClaims<'a> {
    iss: Option<&'a str>,
    aud: Option<&'a str>,
    runner_environment: Option<&'a str>,
    subject: Option<&'a str>,
    repository: Option<&'a str>,
    repository_owner: Option<&'a str>,
    repository_id: Option<&'a str>,
    repository_owner_id: Option<&'a str>,
    workflow_ref: Option<&'a str>,
    workflow_sha: Option<&'a str>,
    reference: Option<&'a str>,
    sha: Option<&'a str>,
    actor_id: Option<&'a str>,
    event_name: Option<&'a str>,
    run_id: Option<&'a str>,
    run_attempt: Option<&'a str>,
    nbf: Option<u64>,
    issued_at: Option<u64>,
    expires_at: Option<u64>,
}

#[cfg(feature = "secret-delivery-pressure")]
impl UniqueJwtClaims<'_> {
    fn github_claim(&self, claim: GithubOidcClaim) -> Option<&str> {
        match claim {
            GithubOidcClaim::Subject => self.subject,
            GithubOidcClaim::RepositoryId => self.repository_id,
            GithubOidcClaim::RepositoryOwnerId => self.repository_owner_id,
            GithubOidcClaim::WorkflowRef => self.workflow_ref,
            GithubOidcClaim::WorkflowSha => self.workflow_sha,
            GithubOidcClaim::Ref => self.reference,
            GithubOidcClaim::Sha => self.sha,
            GithubOidcClaim::ActorId => self.actor_id,
            GithubOidcClaim::EventName => self.event_name,
            GithubOidcClaim::RunId => self.run_id,
            GithubOidcClaim::RunAttempt => self.run_attempt,
        }
    }
}

#[cfg(feature = "secret-delivery-pressure")]
impl<'de> Deserialize<'de> for UniqueJwtClaims<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ObjectVisitor;

        impl<'de> Visitor<'de> for ObjectVisitor {
            type Value = UniqueJwtClaims<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object with unique member names")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut claims = UniqueJwtClaims {
                    iss: None,
                    aud: None,
                    runner_environment: None,
                    subject: None,
                    repository: None,
                    repository_owner: None,
                    repository_id: None,
                    repository_owner_id: None,
                    workflow_ref: None,
                    workflow_sha: None,
                    reference: None,
                    sha: None,
                    actor_id: None,
                    event_name: None,
                    run_id: None,
                    run_attempt: None,
                    nbf: None,
                    issued_at: None,
                    expires_at: None,
                };
                let mut seen = Vec::new();
                while let Some(key) = map.next_key::<&'de str>()? {
                    if seen.contains(&key) {
                        return Err(de::Error::custom("duplicate JSON member"));
                    }
                    seen.push(key);
                    match key {
                        "iss" => claims.iss = Some(map.next_value()?),
                        "aud" => claims.aud = Some(map.next_value()?),
                        "runner_environment" => claims.runner_environment = Some(map.next_value()?),
                        "sub" => claims.subject = Some(map.next_value()?),
                        "repository" => claims.repository = Some(map.next_value()?),
                        "repository_owner" => claims.repository_owner = Some(map.next_value()?),
                        "repository_id" => claims.repository_id = Some(map.next_value()?),
                        "repository_owner_id" => {
                            claims.repository_owner_id = Some(map.next_value()?)
                        }
                        "workflow_ref" => claims.workflow_ref = Some(map.next_value()?),
                        "workflow_sha" => claims.workflow_sha = Some(map.next_value()?),
                        "ref" => claims.reference = Some(map.next_value()?),
                        "sha" => claims.sha = Some(map.next_value()?),
                        "actor_id" => claims.actor_id = Some(map.next_value()?),
                        "event_name" => claims.event_name = Some(map.next_value()?),
                        "run_id" => claims.run_id = Some(map.next_value()?),
                        "run_attempt" => claims.run_attempt = Some(map.next_value()?),
                        "nbf" => claims.nbf = Some(map.next_value()?),
                        "iat" => claims.issued_at = Some(map.next_value()?),
                        "exp" => claims.expires_at = Some(map.next_value()?),
                        _ => {
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                Ok(claims)
            }
        }

        deserializer.deserialize_map(ObjectVisitor)
    }
}

#[cfg(feature = "secret-delivery-pressure")]
fn parse_unique_jwt_claims(
    payload: &[u8],
) -> Result<UniqueJwtClaims<'_>, SecretDeliveryProviderClientError> {
    serde_json::from_slice::<UniqueJwtClaims<'_>>(payload).map_err(|_| oidc_dispatch_refused())
}

#[cfg(feature = "secret-delivery-pressure")]
fn require_string_claim<'a>(
    value: Option<&'a str>,
    expected: &str,
) -> Result<&'a str, SecretDeliveryProviderClientError> {
    let value = value.ok_or_else(oidc_dispatch_refused)?;
    if value != expected {
        return Err(oidc_dispatch_refused());
    }
    Ok(value)
}

#[cfg(feature = "secret-delivery-pressure")]
fn parse_bound_workflow_repository<'a>(
    workflow_ref: &'a str,
    reference: &str,
) -> Result<(&'a str, &'a str), SecretDeliveryProviderClientError> {
    let suffix = format!("@{reference}");
    let prefix = workflow_ref
        .strip_suffix(&suffix)
        .ok_or_else(oidc_dispatch_refused)?;
    let mut parts = prefix.split('/');
    let (Some(owner), Some(repository), Some(github), Some(workflows), Some(file), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(oidc_dispatch_refused());
    };
    if owner.is_empty()
        || repository.is_empty()
        || file.is_empty()
        || github != ".github"
        || workflows != "workflows"
        || owner.contains('%')
        || repository.contains('%')
        || file.contains('%')
    {
        return Err(oidc_dispatch_refused());
    }
    let workflow_path = format!("{owner}/{repository}/.github/workflows/{file}");
    if workflow_path != prefix {
        return Err(oidc_dispatch_refused());
    }
    Ok((owner, repository))
}

#[cfg(feature = "secret-delivery-pressure")]
struct GithubOidcJwtFreshnessV1 {
    nbf: u64,
    issued_at: u64,
    expires_at: u64,
}

#[cfg(feature = "secret-delivery-pressure")]
impl GithubOidcJwtFreshnessV1 {
    fn validate_at(&self, now_unix_seconds: u64) -> Result<(), SecretDeliveryProviderClientError> {
        let Self {
            nbf,
            issued_at,
            expires_at,
        } = *self;
        let latest_issued_at = now_unix_seconds
            .checked_add(60)
            .ok_or_else(oidc_dispatch_refused)?;
        if nbf > now_unix_seconds
            || expires_at <= now_unix_seconds
            || issued_at > latest_issued_at
            || issued_at >= expires_at
            || nbf >= expires_at
            || expires_at
                .checked_sub(issued_at)
                .ok_or_else(oidc_dispatch_refused)?
                > 900
            || expires_at
                .checked_sub(nbf)
                .ok_or_else(oidc_dispatch_refused)?
                > 900
        {
            return Err(oidc_dispatch_refused());
        }
        Ok(())
    }
}

fn canonical_base64url_segment(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    if bytes.len() < 2 || bytes.len() % 4 == 1 {
        return false;
    }
    let mut last = 0;
    for &byte in bytes {
        last = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return false,
        };
    }
    match bytes.len() % 4 {
        2 => last & 0b1111 == 0,
        3 => last & 0b11 == 0,
        _ => true,
    }
}

fn bearer_authorization(
    token: ProtectedProviderValue,
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    validate_bearer(&token)?;
    let mut header = b"Bearer ".to_vec();
    header.extend_from_slice(&token.0);
    ProtectedProviderValue::new(header)
}

fn bearer_authorization_borrowed(
    token: &ProtectedProviderValue,
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    validate_bearer(token)?;
    let mut header = b"Bearer ".to_vec();
    header.extend_from_slice(&token.0);
    ProtectedProviderValue::new(header)
}

fn validate_bearer(
    token: &ProtectedProviderValue,
) -> Result<(), SecretDeliveryProviderClientError> {
    let token_text = token.as_utf8()?;
    if token_text
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(error(
            "secret_delivery_provider_bearer_invalid",
            "provider bearer token contains unsupported bytes",
        ));
    }
    Ok(())
}

fn encode_form(fields: &[(&str, &str)]) -> Vec<u8> {
    fields
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                percent_encode(key.as_bytes()),
                percent_encode(value.as_bytes())
            )
        })
        .collect::<Vec<_>>()
        .join("&")
        .into_bytes()
}

fn percent_encode(bytes: &[u8]) -> String {
    let mut encoded = String::new();
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn crc32c(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn invalid_request_serialization() -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_provider_request_serialization_failed",
        "provider request could not be serialized canonically",
    )
}

fn invalid_iam_response() -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_provider_iam_response_invalid",
        "IAM Credentials response is outside the admitted token profile",
    )
}

fn invalid_secret_response() -> SecretDeliveryProviderClientError {
    error(
        "secret_delivery_provider_secret_response_invalid",
        "Secret Manager response is outside the exact numeric-version profile",
    )
}

fn error(code: &'static str, message: impl Into<String>) -> SecretDeliveryProviderClientError {
    SecretDeliveryProviderClientError {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_delivery_oidc_endpoint::{
        GithubActionsOidcEndpointObservationInputV1,
        github_actions_oidc_request_endpoint_profile_v1,
        resolve_github_actions_oidc_endpoint_observation_v1,
    };
    use crate::secret_delivery_transaction::{
        SecretDeliveryTransactionCandidate, SecretDeliveryTransactionCandidateRealization,
        secret_delivery_transaction_candidate_identity,
    };
    use crate::secret_delivery_transaction_binding::tests::verified_candidate;
    use crate::secret_provider_profile::{GithubOidcClaim, SecretDeliveryTargetPosture};

    fn identity(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn candidate() -> SemanticallyVerifiedSecretDeliveryTransactionCandidate {
        let transport_dependency_record_identity = crate::secret_delivery_transport_dependencies::embedded_transport_dependency_record_identity_v1()
            .expect("transport dependencies");
        let realization = SecretDeliveryTransactionCandidateRealization {
            realization_identity: identity('5'),
            requirement_identity: identity('4'),
            provider_binding_identity: identity('9'),
            provider_binding_source_identity: identity('a'),
            profile_semantic_identity: identity('b'),
            implementation_subject_identity: identity('c'),
            transport_dependency_record_identity: transport_dependency_record_identity.clone(),
            invocation_binding_identity: identity('d'),
            target: SecretDeliveryTargetPosture {
                operating_system: SecretDeliveryOperatingSystem::Linux,
                architecture: SecretDeliveryArchitecture::X86_64,
                runtime: SecretDeliveryRuntime::GithubActions,
                execution_mode: SecretDeliveryExecutionMode::Native,
                recipient_boundary: SecretDeliveryRecipientBoundary::TransientSelectedProcessTree,
            },
            oidc_issuer: "https://token.actions.githubusercontent.com".into(),
            oidc_audience: "https://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github".into(),
            oidc_claims: [
                (
                    GithubOidcClaim::Subject,
                    "repo:ota-run/ota:ref:refs/heads/1.6.29-implementation".into(),
                ),
                (GithubOidcClaim::RepositoryId, "1001".into()),
                (GithubOidcClaim::RepositoryOwnerId, "1000".into()),
                (
                    GithubOidcClaim::WorkflowRef,
                    "ota-run/ota/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@refs/heads/1.6.29-implementation".into(),
                ),
                (GithubOidcClaim::WorkflowSha, "b".repeat(40)),
                (GithubOidcClaim::Ref, "refs/heads/1.6.29-implementation".into()),
                (GithubOidcClaim::Sha, "b".repeat(40)),
                (GithubOidcClaim::ActorId, "1002".into()),
                (GithubOidcClaim::EventName, "workflow_dispatch".into()),
                (GithubOidcClaim::RunId, "1003".into()),
                (GithubOidcClaim::RunAttempt, "1".into()),
            ]
            .into_iter()
            .collect(),
            workload_identity_pool:
                "projects/123/locations/global/workloadIdentityPools/ota-pool".into(),
            workload_identity_provider:
                "projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github"
                    .into(),
            service_account: "ota-pressure@ota-pressure.iam.gserviceaccount.com".into(),
            google_project: "ota-pressure".into(),
            secret_resource: "projects/ota-pressure/secrets/CAEP_API-Key_1".into(),
            secret_version: 7,
        };
        let mut candidate = SecretDeliveryTransactionCandidate {
            schema_version: 1,
            identity: String::new(),
            evaluation_identity: identity('1'),
            dry_run_plan_identity: identity('2'),
            contract_snapshot_identity: identity('3'),
            selected_requirement_identities: vec![identity('4')],
            realization_identities: vec![identity('5')],
            policy_decision_identity: identity('6'),
            selected_invocation_identity: identity('7'),
            execution_graph_identity: identity('8'),
            transport_dependency_record_identity,
            realizations: vec![realization],
        };
        candidate.identity = secret_delivery_transaction_candidate_identity(&candidate).unwrap();
        verified_candidate(&candidate)
    }

    fn operation() -> SecretDeliveryProviderOperationPlanV1 {
        derive_secret_delivery_provider_client_plan_v1(&candidate())
            .unwrap()
            .operations
            .remove(0)
    }

    fn jwt() -> ProtectedProviderValue {
        ProtectedProviderValue::new(b"eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ4In0.c2ln".to_vec()).unwrap()
    }

    #[cfg(feature = "secret-delivery-pressure")]
    fn claim_jwt(payload: serde_json::Value) -> ProtectedProviderValue {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        ProtectedProviderValue::new(format!("{header}.{payload}.c2ln").into_bytes()).unwrap()
    }

    #[cfg(feature = "secret-delivery-pressure")]
    fn expected_claim_payload() -> serde_json::Value {
        serde_json::json!({
            "iss": "https://token.actions.githubusercontent.com",
            "aud": "https://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github",
            "sub": "repo:ota-run/ota:ref:refs/heads/1.6.29-implementation",
            "repository": "ota-run/ota",
            "repository_owner": "ota-run",
            "repository_id": "1001",
            "repository_owner_id": "1000",
            "workflow_ref": "ota-run/ota/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@refs/heads/1.6.29-implementation",
            "workflow_sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "ref": "refs/heads/1.6.29-implementation",
            "sha": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "actor_id": "1002",
            "event_name": "workflow_dispatch",
            "run_id": "1003",
            "run_attempt": "1",
            "runner_environment": "self-hosted",
            "nbf": 900,
            "iat": 950,
            "exp": 1100
        })
    }

    #[cfg(feature = "secret-delivery-pressure")]
    fn reconcile_test_jwt(
        jwt: &ProtectedProviderValue,
        now: u64,
    ) -> Result<(), SecretDeliveryProviderClientError> {
        let candidate = candidate();
        let operation = derive_secret_delivery_provider_client_plan_v1(&candidate)
            .unwrap()
            .operations
            .remove(0);
        reconcile_github_oidc_jwt_claims_at_v1(
            jwt,
            &operation,
            &candidate.candidate().realizations[0],
            "self-hosted",
            now,
        )
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn github_oidc_claim_reconciliation_accepts_only_the_exact_unadmitted_context() {
        let jwt = claim_jwt(expected_claim_payload());
        assert!(reconcile_test_jwt(&jwt, 1_000).is_ok());
        assert!(!format!("{jwt:?}").contains("ota-run"));

        for (claim, value) in [
            ("iss", serde_json::json!("https://other.example")),
            ("aud", serde_json::json!("https://other.example/audience")),
            (
                "sub",
                serde_json::json!("repo:other/repository:ref:refs/heads/main"),
            ),
            ("repository", serde_json::json!("other/repository")),
            ("repository_owner", serde_json::json!("other")),
            ("repository_id", serde_json::json!("0")),
            ("repository_owner_id", serde_json::json!("0")),
            (
                "workflow_ref",
                serde_json::json!("other/repository/.github/workflows/x.yml@refs/heads/main"),
            ),
            ("workflow_sha", serde_json::json!("a".repeat(40))),
            ("ref", serde_json::json!("refs/heads/main")),
            ("sha", serde_json::json!("a".repeat(40))),
            ("actor_id", serde_json::json!("0")),
            ("event_name", serde_json::json!("push")),
            ("run_id", serde_json::json!("0")),
            ("run_attempt", serde_json::json!("2")),
            ("runner_environment", serde_json::json!("github-hosted")),
        ] {
            let mut payload = expected_claim_payload();
            payload[claim] = value;
            assert!(
                reconcile_test_jwt(&claim_jwt(payload), 1_000).is_err(),
                "{claim}"
            );
        }
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn github_oidc_claim_reconciliation_refuses_malformed_duplicate_and_invalid_time_claims() {
        let mut missing = expected_claim_payload();
        missing.as_object_mut().unwrap().remove("repository_id");
        assert!(reconcile_test_jwt(&claim_jwt(missing), 1_000).is_err());

        let mut wrong_type = expected_claim_payload();
        wrong_type["repository_id"] = serde_json::json!(1001);
        assert!(reconcile_test_jwt(&claim_jwt(wrong_type), 1_000).is_err());

        let mut audience_array = expected_claim_payload();
        audience_array["aud"] = serde_json::json!([
            "https://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ota-pool/providers/github"
        ]);
        assert!(reconcile_test_jwt(&claim_jwt(audience_array), 1_000).is_err());

        let canonical_payload = serde_json::to_string(&expected_claim_payload()).unwrap();
        let duplicate_payload = format!(
            r#"{},"iss":"https://token.actions.githubusercontent.com"}}"#,
            canonical_payload.strip_suffix('}').unwrap()
        );
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let duplicate = ProtectedProviderValue::new(
            format!(
                "{header}.{}.c2ln",
                URL_SAFE_NO_PAD.encode(duplicate_payload)
            )
            .into_bytes(),
        )
        .unwrap();
        assert!(reconcile_test_jwt(&duplicate, 1_000).is_err());

        for (name, value) in [("nbf", 1_001), ("iat", 1_061), ("exp", 1_000)] {
            let mut payload = expected_claim_payload();
            payload[name] = serde_json::json!(value);
            assert!(
                reconcile_test_jwt(&claim_jwt(payload), 1_000).is_err(),
                "{name}"
            );
        }

        let mut overlong = expected_claim_payload();
        overlong["exp"] = serde_json::json!(1_851);
        assert!(reconcile_test_jwt(&claim_jwt(overlong), 1_000).is_err());

        let mut nbf_window_overlong = expected_claim_payload();
        nbf_window_overlong["nbf"] = serde_json::json!(199);
        assert!(reconcile_test_jwt(&claim_jwt(nbf_window_overlong), 1_000).is_err());

        let mut impossible_order = expected_claim_payload();
        impossible_order["iat"] = serde_json::json!(1_100);
        impossible_order["exp"] = serde_json::json!(1_100);
        assert!(reconcile_test_jwt(&claim_jwt(impossible_order), 1_000).is_err());

        assert!(reconcile_test_jwt(&claim_jwt(expected_claim_payload()), u64::MAX).is_err());

        let mut nbf_before_iat = expected_claim_payload();
        nbf_before_iat["nbf"] = serde_json::json!(900);
        nbf_before_iat["iat"] = serde_json::json!(950);
        nbf_before_iat["exp"] = serde_json::json!(1_100);
        assert!(reconcile_test_jwt(&claim_jwt(nbf_before_iat), 1_000).is_ok());
    }

    #[test]
    fn exact_plan_and_request_bytes_are_stable_and_redacted() {
        let candidate = candidate();
        let plan = derive_secret_delivery_provider_client_plan_v1(&candidate).unwrap();
        verify_secret_delivery_provider_client_plan_v1(&plan, &candidate).unwrap();
        let plan_debug = format!("{plan:?}");
        assert!(!plan_debug.contains("sha256:"));
        assert!(!plan_debug.contains("iam.googleapis.com"));
        assert!(!plan_debug.contains("secretmanager"));
        let operation = &plan.operations[0];
        assert_eq!(operation.sts_url, STS_URL);
        assert_eq!(
            operation.secret_version_resource,
            "projects/ota-pressure/secrets/CAEP_API-Key_1/versions/7"
        );

        let profile = github_actions_oidc_request_endpoint_profile_v1().unwrap();
        let endpoint_input = GithubActionsOidcEndpointObservationInputV1 {
            schema_version: 1,
            request_url: "https://run-actions-1-azure-eastus.actions.githubusercontent.com/1//idtoken/123e4567-e89b-12d3-a456-426614174000/123e4567-e89b-12d3-a456-426614174001?api-version=2.0".into(),
            runner_environment: "self-hosted".into(),
            runner_os: "linux".into(),
            runner_architecture: "x64".into(),
            runner_version: "2.337.0".into(),
            protected_launcher_capability_projection_identity: identity('e'),
        };
        let endpoint =
            resolve_github_actions_oidc_endpoint_observation_v1(&profile, &endpoint_input).unwrap();
        let bearer = ProtectedProviderValue::new(b"runner-bearer".to_vec()).unwrap();
        let oidc =
            build_github_oidc_request_v1(&profile, &endpoint_input, &endpoint, &bearer, operation)
                .unwrap();
        #[cfg(feature = "secret-delivery-pressure")]
        {
            verify_exact_oidc_request(&oidc, &endpoint_input, operation, &bearer).unwrap();
            let substituted = ProtectedProviderValue::new(b"other-bearer".to_vec()).unwrap();
            assert!(
                verify_exact_oidc_request(&oidc, &endpoint_input, operation, &substituted).is_err()
            );
        }
        assert!(oidc.url.ends_with("audience=https%3A%2F%2Fiam.googleapis.com%2Fprojects%2F123%2Flocations%2Fglobal%2FworkloadIdentityPools%2Fota-pool%2Fproviders%2Fgithub"));
        assert_eq!(
            oidc.authorization_for_test(),
            Some(b"Bearer runner-bearer".as_slice())
        );
        assert!(!format!("{oidc:?}").contains("runner-bearer"));

        let sts = build_google_sts_request_v1(operation, &jwt()).unwrap();
        let body = std::str::from_utf8(sts.body_for_test()).unwrap();
        assert_eq!(body.split('&').count(), 6);
        assert!(body.starts_with("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&audience=%2F%2Fiam.googleapis.com"));
        assert!(!format!("{sts:?}").contains("eyJ"));
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn sts_production_reader_stops_at_the_bound_and_refuses_before_reading_bad_heads() {
        let mut headers = ureq::http::HeaderMap::new();
        headers.insert(
            ureq::http::header::CONTENT_TYPE,
            ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
        );
        let bytes = vec![b'x'; MAX_TOKEN_RESPONSE_BYTES * 2];
        let mut reader = std::io::Cursor::new(&bytes);
        assert!(
            read_sts_transport_response(ureq::http::StatusCode::OK, headers.clone(), &mut reader)
                .is_err()
        );
        assert_eq!(reader.position(), (MAX_TOKEN_RESPONSE_BYTES + 1) as u64);
        let mut reader = std::io::Cursor::new(&bytes);
        assert!(
            read_sts_transport_response(
                ureq::http::StatusCode::FOUND,
                headers.clone(),
                &mut reader
            )
            .is_err()
        );
        assert_eq!(reader.position(), 0);
        headers.append(
            ureq::http::header::CONTENT_TYPE,
            ureq::http::HeaderValue::from_static(JSON_MEDIA_TYPE),
        );
        assert!(
            read_sts_transport_response(ureq::http::StatusCode::OK, headers, &mut reader).is_err()
        );
        assert_eq!(reader.position(), 0);
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn sts_production_request_refuses_endpoint_method_header_and_credential_substitution() {
        let plan = derive_secret_delivery_provider_client_plan_v1(&candidate()).unwrap();
        for fault in 0..4 {
            let mut request = build_google_sts_request_v1(&plan.operations[0], &jwt()).unwrap();
            match fault {
                0 => request.url = "https://caller.invalid/token".into(),
                1 => request.method = ProviderHttpMethod::Get,
                2 => request.media_type = Some(JSON_MEDIA_TYPE),
                _ => request.authorization = Some(jwt()),
            }
            assert!(build_sts_http_request(&request).is_err());
        }
    }

    #[test]
    fn closed_token_responses_reject_unknown_type_and_lifetime_substitution() {
        let oidc = br#"{"value":"eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ4In0.c2ln"}"#;
        assert_eq!(
            parse_github_oidc_response_v1(oidc)
                .unwrap()
                .bytes_for_test(),
            jwt().bytes_for_test()
        );
        assert!(parse_github_oidc_response_v1(br#"{"value":"a.b.c","extra":true}"#).is_err());

        let valid = br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600}"#;
        assert_eq!(
            parse_google_sts_response_v1(valid)
                .unwrap()
                .expires_in_seconds,
            3600
        );
        for invalid in [
            br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:refresh_token","token_type":"Bearer","expires_in":3600}"#.as_slice(),
            br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"bearer","expires_in":3600}"#,
            br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3601}"#,
            br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600,"access_boundary_session_key":"x"}"#,
            br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600,"refresh_token":"x"}"#,
        ] {
            assert!(parse_google_sts_response_v1(invalid).is_err());
        }
    }

    #[test]
    fn iam_request_and_response_are_exact_and_bounded() {
        let operation = operation();
        let sts = parse_google_sts_response_v1(br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":600}"#).unwrap();
        let request = build_service_account_token_request_v1(&operation, &sts).unwrap();
        assert_eq!(request.media_type, Some(JSON_MEDIA_TYPE));
        assert_eq!(
            request.body_for_test(),
            br#"{"scope":["https://www.googleapis.com/auth/cloud-platform"],"lifetime":"600s"}"#
        );
        assert_eq!(
            request.authorization_for_test(),
            Some(b"Bearer federated".as_slice())
        );

        let expiry = OffsetDateTime::parse("2026-09-15T12:10:00Z", &Rfc3339).unwrap();
        let valid = br#"{"accessToken":"service-account","expireTime":"2026-09-15T12:10:00Z"}"#;
        let token =
            parse_service_account_token_response_v1(valid, expiry - time::Duration::seconds(600))
                .unwrap();
        assert_eq!(token.expires_at, expiry);
        assert!(
            parse_service_account_token_response_v1(valid, expiry - time::Duration::seconds(601))
                .is_err()
        );
        assert!(
            parse_service_account_token_response_v1(
                br#"{"accessToken":"x","expireTime":"2026-09-15T12:10:00Z","delegates":[]}"#,
                expiry - time::Duration::seconds(600)
            )
            .is_err()
        );
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn iam_http_request_rejects_each_material_request_mutation() {
        let operation = operation();
        let sts = parse_google_sts_response_v1(br#"{"access_token":"federated","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":600}"#).unwrap();
        let request = build_service_account_token_request_v1(&operation, &sts).unwrap();
        let http = build_iam_http_request(&request, &operation, &sts).unwrap();
        assert_eq!(http.headers().len(), 2);
        assert_eq!(http.uri().to_string(), operation.service_account_token_url);
        for field in 0..11 {
            let mut request = build_service_account_token_request_v1(&operation, &sts).unwrap();
            match field {
                0 => request.method = ProviderHttpMethod::Get,
                1 => request.url = request.url.replace("iamcredentials.googleapis.com", "example.invalid"),
                2 => request.url = request.url.replace(":generateAccessToken", ":signBlob"),
                3 => request.url.push_str("?account=substitute"),
                4 => request.url = request.url.replace("serviceAccounts/", "serviceAccounts/substitute"),
                5 => request.authorization = None,
                6 => request.authorization = Some(ProtectedProviderValue::new(b"Bearer substitute".to_vec()).unwrap()),
                7 => request.media_type = Some(FORM_MEDIA_TYPE),
                8 => request.body = br#"{"scope":["substitute"],"lifetime":"600s"}"#.to_vec(),
                9 => request.body = br#"{"scope":["https://www.googleapis.com/auth/cloud-platform"],"lifetime":"601s"}"#.to_vec(),
                10 => request.body = br#"{"scope":["https://www.googleapis.com/auth/cloud-platform"],"lifetime":"600s","delegates":[]}"#.to_vec(),
                _ => unreachable!(),
            }
            assert!(
                build_iam_http_request(&request, &operation, &sts).is_err(),
                "mutation {field}"
            );
        }
    }

    #[test]
    fn iam_expiry_accepts_google_fractional_widths_without_truncation() {
        let observed = OffsetDateTime::parse("2026-09-15T12:00:00Z", &Rfc3339).unwrap();
        for suffix in [
            "Z",
            ".000Z",
            ".100Z",
            ".000000Z",
            ".100000Z",
            ".000000000Z",
            ".100000000Z",
            ".123456789Z",
        ] {
            let expiry = format!("2026-09-15T12:00:01{suffix}");
            let body = serde_json::to_vec(
                &serde_json::json!({"accessToken":"synthetic-iam-token", "expireTime":expiry}),
            )
            .unwrap();
            let token = parse_service_account_token_response_v1(&body, observed).unwrap();
            assert_eq!(
                token.expires_at,
                OffsetDateTime::parse(&expiry, &Rfc3339).unwrap()
            );
        }
        for expiry in [
            "2026-09-15T12:00:00Z",
            "2026-09-15T12:10:00.000000001Z",
            "2026-09-15T12:00:01.1Z",
            "2026-09-15T12:00:01.1000Z",
            "2026-09-15T12:00:01+00:00",
            "2026-09-15t12:00:01Z",
            "2026-02-30T12:00:01Z",
            "2026-09-15T12:00:60Z",
            "2026-09-15T12:00:01z",
        ] {
            let body = serde_json::to_vec(
                &serde_json::json!({"accessToken":"synthetic-iam-token", "expireTime":expiry}),
            )
            .unwrap();
            assert!(
                parse_service_account_token_response_v1(&body, observed).is_err(),
                "{expiry}"
            );
        }
        let body = br#"{"accessToken":"synthetic-iam-token","expireTime":"2026-09-15T12:00:00.000000001Z"}"#;
        assert!(parse_service_account_token_response_v1(body, observed).is_ok());
        assert!(
            parse_service_account_token_response_v1(
                body,
                observed + time::Duration::nanoseconds(1)
            )
            .is_err()
        );
        let body =
            br#"{"accessToken":"synthetic-iam-token","expireTime":"2026-09-15T12:10:00.100Z"}"#;
        assert!(
            parse_service_account_token_response_v1(
                body,
                observed + time::Duration::milliseconds(100)
            )
            .is_ok()
        );
        assert!(
            parse_service_account_token_response_v1(
                body,
                observed + time::Duration::milliseconds(99)
            )
            .is_err()
        );
    }

    #[test]
    fn secret_response_requires_exact_resource_canonical_base64_and_crc32c() {
        assert_eq!(crc32c(b"123456789"), 0xe306_9283);
        let operation = operation();
        let value = b"synthetic-canary";
        let encoded = STANDARD.encode(value);
        let response = format!(
            "{{\"name\":\"{}\",\"payload\":{{\"data\":\"{}\",\"dataCrc32c\":\"{}\"}}}}",
            operation.secret_version_resource,
            encoded,
            crc32c(value)
        );
        let parsed =
            parse_secret_manager_access_response_v1(response.as_bytes(), &operation).unwrap();
        assert_eq!(parsed.value.bytes_for_test(), value);

        for invalid in [
            response.replace("versions/7", "versions/latest"),
            response.replace(&encoded, encoded.trim_end_matches('=')),
            response.replace(&crc32c(value).to_string(), "1"),
            response.replace(
                &format!("\"{}\"", crc32c(value)),
                &format!("\"+{}\"", crc32c(value)),
            ),
            response.replace("}}", ",\"extra\":true}}"),
        ] {
            assert!(
                parse_secret_manager_access_response_v1(invalid.as_bytes(), &operation).is_err()
            );
        }
    }

    #[test]
    fn target_and_tuple_substitution_refuse_before_request_construction() {
        let original = candidate();
        let refuse = |mutate: fn(&mut SecretDeliveryTransactionCandidateRealization)| {
            let mut forged = original.candidate().clone();
            mutate(&mut forged.realizations[0]);
            forged.identity = secret_delivery_transaction_candidate_identity(&forged).unwrap();
            assert!(
                derive_secret_delivery_provider_client_plan_v1(&verified_candidate(&forged))
                    .is_err()
            );
        };

        refuse(|value| value.target.operating_system = SecretDeliveryOperatingSystem::Macos);
        refuse(|value| value.target.architecture = SecretDeliveryArchitecture::Aarch64);
        refuse(|value| value.target.runtime = SecretDeliveryRuntime::Local);
        refuse(|value| value.target.execution_mode = SecretDeliveryExecutionMode::Container);
        refuse(|value| {
            value.target.recipient_boundary = SecretDeliveryRecipientBoundary::PersistentRuntime
        });
        refuse(|value| value.oidc_issuer = "https://issuer.example".into());
        refuse(|value| value.oidc_audience = "https://example.invalid".into());
        refuse(|value| value.workload_identity_pool.push_str("-other"));
        refuse(|value| value.workload_identity_provider.push_str("-other"));
        refuse(|value| value.service_account = "other@example.iam.gserviceaccount.com".into());
        refuse(|value| value.google_project = "other-project".into());
        refuse(|value| value.secret_resource.push_str("/other"));
        refuse(|value| value.secret_version = 0);
    }

    #[test]
    fn fixed_transport_posture_ignores_ambient_proxy_and_trust_inputs() {
        let _environment = crate::test_support::env_mutex_lock();
        let variables = [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "NETRC",
        ];
        let previous = variables
            .iter()
            .map(|name| (*name, std::env::var_os(name)))
            .collect::<Vec<_>>();
        unsafe {
            for name in variables {
                std::env::set_var(name, "https://caller.invalid/credential");
            }
        }

        let configuration = fixed_transport_configuration_v1();
        verify_fixed_transport_configuration_v1(&configuration, MAX_SECRET_RESPONSE_BYTES).unwrap();
        assert!(configuration.proxy().is_none());
        assert!(configuration.https_only());
        assert_eq!(configuration.max_redirects(), 0);
        assert_eq!(
            configuration.max_response_header_size(),
            MAX_PROVIDER_RESPONSE_HEADER_BYTES
        );
        assert_eq!(
            configuration.timeouts().global,
            Some(PROVIDER_TIMEOUT_GLOBAL)
        );
        assert_eq!(
            configuration.timeouts().resolve,
            Some(PROVIDER_TIMEOUT_RESOLVE)
        );
        assert_eq!(
            configuration.timeouts().connect,
            Some(PROVIDER_TIMEOUT_CONNECT)
        );
        assert_eq!(
            configuration.timeouts().send_request,
            Some(PROVIDER_TIMEOUT_SEND)
        );
        assert_eq!(
            configuration.timeouts().send_body,
            Some(PROVIDER_TIMEOUT_SEND)
        );
        assert_eq!(
            configuration.timeouts().recv_response,
            Some(PROVIDER_TIMEOUT_RECEIVE)
        );
        assert_eq!(
            configuration.timeouts().recv_body,
            Some(PROVIDER_TIMEOUT_RECEIVE)
        );
        assert!(matches!(
            configuration.tls_config().root_certs(),
            RootCerts::WebPki
        ));

        unsafe {
            for (name, value) in previous {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }

    #[test]
    fn retained_bearer_validation_and_debug_remain_redacted() {
        let bearer = ProtectedProviderValue::new(b"runner-bearer".to_vec()).unwrap();
        validate_bearer(&bearer).unwrap();
        assert!(!format!("{bearer:?}").contains("runner-bearer"));
        assert!(
            validate_bearer(&ProtectedProviderValue::new(b"runner\n bearer".to_vec()).unwrap())
                .is_err()
        );
    }

    #[test]
    fn oidc_input_owner_requires_the_retained_url_and_consumes_ambient_capability() {
        let _environment = crate::test_support::env_mutex_lock();
        reset_github_oidc_capability_owner_for_test();
        let previous_url = std::env::var_os(ACTIONS_ID_TOKEN_REQUEST_URL);
        let previous_token = std::env::var_os(ACTIONS_ID_TOKEN_REQUEST_TOKEN);
        unsafe {
            std::env::set_var(
                ACTIONS_ID_TOKEN_REQUEST_URL,
                "https://runner.invalid/id-token",
            );
            std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_TOKEN, "runner-bearer");
        }

        let bearer = take_github_oidc_bearer_v1(b"https://runner.invalid/id-token").unwrap();
        assert!(!format!("{bearer:?}").contains("runner-bearer"));
        assert!(take_github_oidc_bearer_v1(b"https://runner.invalid/id-token").is_err());

        reset_github_oidc_capability_owner_for_test();
        unsafe {
            std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_URL, "https://runner.invalid/other");
            std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_TOKEN, "runner-bearer");
        }
        assert!(take_github_oidc_bearer_v1(b"https://runner.invalid/id-token").is_err());
        assert!(take_github_oidc_bearer_v1(b"https://runner.invalid/id-token").is_err());

        unsafe {
            match previous_url {
                Some(value) => std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_URL, value),
                None => std::env::remove_var(ACTIONS_ID_TOKEN_REQUEST_URL),
            }
            match previous_token {
                Some(value) => std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_TOKEN, value),
                None => std::env::remove_var(ACTIONS_ID_TOKEN_REQUEST_TOKEN),
            }
        }
        reset_github_oidc_capability_owner_for_test();
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn oidc_dispatch_state_and_media_type_are_closed() {
        let mut state = GithubOidcDispatchAttemptStateV1::new();
        assert_eq!(state.core_invocations, 0);
        assert_eq!(state.outcome, GithubOidcDispatchOutcomeV1::NotAttempted);
        state.invoke_once().unwrap();
        assert_eq!(state.core_invocations, 1);
        assert_eq!(state.outcome, GithubOidcDispatchOutcomeV1::TransportRefused);
        assert!(state.invoke_once().is_err());
        assert_eq!(state.core_invocations, 1);
        state.claims_refused();
        assert_eq!(state.outcome, GithubOidcDispatchOutcomeV1::ClaimsRefused);

        for accepted in ["application/json", "Application/JSON; charset=UTF-8"] {
            assert!(is_oidc_json_content_type(accepted));
        }
        for refused in [
            "text/json",
            "application/json; charset=latin1",
            "application/json; charset=utf-8; boundary=x",
            "application/json;",
            "",
        ] {
            assert!(!is_oidc_json_content_type(refused));
        }

        let mut headers = ureq::http::HeaderMap::new();
        headers.insert(
            ureq::http::header::CONTENT_TYPE,
            ureq::http::HeaderValue::from_static("application/json"),
        );
        assert!(verify_oidc_response_head(ureq::http::StatusCode::OK, &headers).is_ok());
        assert!(verify_oidc_response_head(ureq::http::StatusCode::FOUND, &headers).is_err());
        headers.append(
            ureq::http::header::CONTENT_TYPE,
            ureq::http::HeaderValue::from_static("text/plain"),
        );
        assert!(verify_oidc_response_head(ureq::http::StatusCode::OK, &headers).is_err());

        let terminal = GithubOidcDispatchTerminalV1 {
            attempt: state,
            result: Err(oidc_dispatch_refused()),
        };
        assert_eq!(terminal.attempt.core_invocations, 1);
        assert_eq!(
            terminal.attempt.outcome,
            GithubOidcDispatchOutcomeV1::ClaimsRefused
        );
        assert_eq!(terminal.public_posture(), (1, "claims_refused"));
        assert!(!format!("{terminal:?}").contains("Bearer"));
    }

    #[cfg(feature = "secret-delivery-pressure")]
    #[test]
    fn oidc_response_owner_refuses_oversize_and_unprotected_json_shapes() {
        let valid = br#"{"value":"eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ4In0.c2ln"}"#;
        let retained =
            RetainedUnadmittedGithubOidcJwtV1(parse_github_oidc_response_v1(valid).unwrap());
        assert!(!format!("{retained:?}").contains("eyJ"));
        assert!(parse_github_oidc_response_v1(&vec![b'a'; MAX_OIDC_RESPONSE_BYTES + 1]).is_err());
        assert!(parse_github_oidc_response_v1(br#"{"value":"a.b.c","extra":1}"#).is_err());
        assert!(parse_github_oidc_response_v1(br#"{"value":"a\\u002eb.c"}"#).is_err());
        assert!(canonical_base64url_segment("AA"));
        assert!(canonical_base64url_segment("AAA"));
        assert!(canonical_base64url_segment("AAAA"));
        for noncanonical in ["", "A", "AB", "AAB", "AAAAA", "AA=", "AA+", "AA/"] {
            assert!(!canonical_base64url_segment(noncanonical));
        }
    }
}
