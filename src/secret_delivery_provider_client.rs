//! Network-disabled provider request and response models for secret delivery.
//!
//! This module derives exact Google operation targets from semantically verified Core truth. It
//! can consume a semantically reverified V2 transaction only to retain a network-disabled transport
//! posture and then take the one-use GitHub request capability. It cannot open a socket, contact a
//! provider, materialize a recipient environment, or publish evidence.

#![allow(dead_code)]

use std::fmt;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
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
use crate::secret_delivery_transaction::{
    SecretDeliveryTransactionCandidateRealization,
    SemanticallyVerifiedSecretDeliveryTransactionCandidate,
    secret_delivery_transaction_candidate_identity,
};
use crate::secret_delivery_transaction_binding::{
    SecretDeliveryTransactionBindingError, VerifiedSecretDeliveryTransactionBindingV2,
};
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
        let value = value.into();
        if value.is_empty() {
            return Err(error(
                "secret_delivery_provider_value_empty",
                "protected provider values cannot be empty",
            ));
        }
        Ok(Self(value))
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
        self.0.fill(0);
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

pub(crate) struct ServiceAccountAccessTokenV1 {
    token: ProtectedProviderValue,
    pub expires_at_unix_seconds: u64,
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
redacted_debug!(
    ServiceAccountAccessTokenV1,
    "token",
    expires_at_unix_seconds
);
redacted_debug!(SecretManagerPayloadV1, "value", crc32c);

/// One consumed V2 binding coupled to its exact provider operation plan. This is intentionally
/// opaque: later transport code cannot substitute another binding, candidate, or operation.
pub(crate) struct ConsumedSecretDeliveryProviderCapabilityV1 {
    binding: ota_authority_protocol::ProtectedLauncherSecretDeliveryTransactionBindingV2,
    plan: SecretDeliveryProviderClientPlanV1,
}

impl fmt::Debug for ConsumedSecretDeliveryProviderCapabilityV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ConsumedSecretDeliveryProviderCapabilityV1([PROTECTED])")
    }
}

/// The acquired GitHub request capability is retained only after V2 consumption. It has no
/// accessor until a later explicitly authorized network-call slice owns request dispatch.
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
    configuration: UreqConfig,
    maximum_response_body_bytes: usize,
}

impl fmt::Debug for PreparedSecretDeliveryProviderTransportV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PreparedSecretDeliveryProviderTransportV1([PROTECTED])")
    }
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
    let plan = derive_secret_delivery_provider_client_plan_v1(candidate)?;
    if plan.transaction_candidate_identity != binding.secret_transaction_candidate_identity {
        return Err(error(
            "secret_delivery_provider_transport_candidate_mismatch",
            "consumed V2 binding does not match the provider operation plan",
        ));
    }
    if binding.projection_identity
        != endpoint_input.protected_launcher_capability_projection_identity
        || retained_runner_version != endpoint_input.runner_version
    {
        return Err(error(
            "secret_delivery_provider_transport_observation_mismatch",
            "OIDC endpoint observation does not match the consumed V2 transaction",
        ));
    }
    if plan.operations.len() != 1 {
        return Err(error(
            "secret_delivery_provider_transport_operation_ambiguous",
            "the initial provider transport requires exactly one selected operation",
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
    let bearer = take_github_oidc_bearer_v1(&endpoint_input.request_url)?;

    let configuration = fixed_transport_configuration_v1();
    verify_fixed_transport_configuration_v1(&configuration, MAX_SECRET_RESPONSE_BYTES)?;
    let capability = ConsumedSecretDeliveryProviderCapabilityV1 { binding, plan };
    let oidc = RetainedGithubOidcRequestCapabilityV1 {
        endpoint_profile: endpoint_profile.clone(),
        endpoint_input,
        endpoint_observation,
        operation: capability.plan.operations[0].clone(),
        bearer,
    };
    Ok(PreparedSecretDeliveryProviderTransportV1 {
        capability,
        oidc,
        configuration,
        maximum_response_body_bytes: MAX_SECRET_RESPONSE_BYTES,
    })
}

fn take_github_oidc_bearer_v1(
    expected_request_url: &str,
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
                "GitHub OIDC request URL is unavailable after V2 consumption",
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
                "GitHub OIDC bearer is unavailable after V2 consumption",
            )
        })?
        .into_string()
        .map_err(|_| {
            error(
                "secret_delivery_provider_transport_oidc_input_invalid",
                "GitHub OIDC bearer is not valid UTF-8",
            )
        })?;
    if request_url.is_empty() || request_url != expected_request_url {
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
        "V2 transaction consumption refused before provider transport preparation",
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

fn operation_plan(
    realization: &SecretDeliveryTransactionCandidateRealization,
) -> Result<SecretDeliveryProviderOperationPlanV1, SecretDeliveryProviderClientError> {
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
    validate_google_tuple(&tuple).map_err(|_| {
        error(
            "secret_delivery_provider_tuple_invalid",
            "provider operation tuple is not provider-canonical",
        )
    })?;
    let expected_oidc_audience = format!(
        "https://iam.googleapis.com/{}",
        realization.workload_identity_provider
    );
    if realization.oidc_audience != expected_oidc_audience {
        return Err(error(
            "secret_delivery_provider_tuple_invalid",
            "OIDC audience does not match the protected WIF provider",
        ));
    }
    let secret_version_resource = format!(
        "{}/versions/{}",
        realization.secret_resource, realization.secret_version
    );
    Ok(SecretDeliveryProviderOperationPlanV1 {
        realization_identity: realization.realization_identity.clone(),
        invocation_binding_identity: realization.invocation_binding_identity.clone(),
        oidc_issuer: realization.oidc_issuer.clone(),
        oidc_audience: realization.oidc_audience.clone(),
        sts_audience: format!(
            "//iam.googleapis.com/{}",
            realization.workload_identity_provider
        ),
        sts_url: STS_URL.into(),
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
    bearer: ProtectedProviderValue,
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
            retained_input.request_url,
            percent_encode(operation.oidc_audience.as_bytes())
        ),
        media_type: None,
        authorization: Some(bearer_authorization(bearer)?),
        body: Vec::new(),
    })
}

pub(crate) fn parse_github_oidc_response_v1(
    bytes: &[u8],
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        value: String,
    }
    let response: Response = parse_bounded_json(bytes, MAX_OIDC_RESPONSE_BYTES)?;
    validate_compact_jwt(&response.value)?;
    ProtectedProviderValue::new(response.value.into_bytes())
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
    federated_token: StsAccessTokenV1,
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
        authorization: Some(bearer_authorization(federated_token.token)?),
        body,
    })
}

pub(crate) fn parse_service_account_token_response_v1(
    bytes: &[u8],
    observed_at_unix_seconds: u64,
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
    let expires_at_value = OffsetDateTime::parse(&response.expire_time, &Rfc3339)
        .map_err(|_| invalid_iam_response())?;
    if !response.expire_time.ends_with('Z')
        || expires_at_value
            .format(&Rfc3339)
            .map_err(|_| invalid_iam_response())?
            != response.expire_time
    {
        return Err(invalid_iam_response());
    }
    let expires_at =
        u64::try_from(expires_at_value.unix_timestamp()).map_err(|_| invalid_iam_response())?;
    if response.access_token.is_empty()
        || response.access_token.len() > MAX_TOKEN_BYTES
        || expires_at <= observed_at_unix_seconds
        || expires_at - observed_at_unix_seconds > 600
    {
        return Err(invalid_iam_response());
    }
    Ok(ServiceAccountAccessTokenV1 {
        token: ProtectedProviderValue::new(response.access_token.into_bytes())?,
        expires_at_unix_seconds: expires_at,
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
        || value.split('.').any(|segment| {
            let Ok(decoded) = URL_SAFE_NO_PAD.decode(segment) else {
                return true;
            };
            decoded.is_empty() || URL_SAFE_NO_PAD.encode(decoded) != segment
        })
    {
        return Err(error(
            "secret_delivery_provider_oidc_response_invalid",
            "GitHub OIDC response does not contain one canonical compact JWT",
        ));
    }
    Ok(())
}

fn bearer_authorization(
    token: ProtectedProviderValue,
) -> Result<ProtectedProviderValue, SecretDeliveryProviderClientError> {
    validate_bearer(&token)?;
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
    use crate::secret_provider_profile::SecretDeliveryTargetPosture;

    fn identity(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn candidate() -> SemanticallyVerifiedSecretDeliveryTransactionCandidate {
        let realization = SecretDeliveryTransactionCandidateRealization {
            realization_identity: identity('5'),
            requirement_identity: identity('4'),
            provider_binding_identity: identity('9'),
            provider_binding_source_identity: identity('a'),
            profile_semantic_identity: identity('b'),
            implementation_subject_identity: identity('c'),
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
            oidc_claims: Default::default(),
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
        let oidc = build_github_oidc_request_v1(
            &profile,
            &endpoint_input,
            &endpoint,
            ProtectedProviderValue::new(b"runner-bearer".to_vec()).unwrap(),
            operation,
        )
        .unwrap();
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
        let request = build_service_account_token_request_v1(&operation, sts).unwrap();
        assert_eq!(request.media_type, Some(JSON_MEDIA_TYPE));
        assert_eq!(
            request.body_for_test(),
            br#"{"scope":["https://www.googleapis.com/auth/cloud-platform"],"lifetime":"600s"}"#
        );
        assert_eq!(
            request.authorization_for_test(),
            Some(b"Bearer federated".as_slice())
        );

        let expiry = OffsetDateTime::parse("2026-09-15T12:10:00Z", &Rfc3339)
            .unwrap()
            .unix_timestamp() as u64;
        let valid = br#"{"accessToken":"service-account","expireTime":"2026-09-15T12:10:00Z"}"#;
        let token = parse_service_account_token_response_v1(valid, expiry - 600).unwrap();
        assert_eq!(token.expires_at_unix_seconds, expiry);
        assert!(parse_service_account_token_response_v1(valid, expiry - 601).is_err());
        assert!(
            parse_service_account_token_response_v1(
                br#"{"accessToken":"x","expireTime":"2026-09-15T12:10:00Z","delegates":[]}"#,
                expiry - 600
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

        let bearer = take_github_oidc_bearer_v1("https://runner.invalid/id-token").unwrap();
        assert!(!format!("{bearer:?}").contains("runner-bearer"));
        assert!(take_github_oidc_bearer_v1("https://runner.invalid/id-token").is_err());

        reset_github_oidc_capability_owner_for_test();
        unsafe {
            std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_URL, "https://runner.invalid/other");
            std::env::set_var(ACTIONS_ID_TOKEN_REQUEST_TOKEN, "runner-bearer");
        }
        assert!(take_github_oidc_bearer_v1("https://runner.invalid/id-token").is_err());
        assert!(take_github_oidc_bearer_v1("https://runner.invalid/id-token").is_err());

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
}
