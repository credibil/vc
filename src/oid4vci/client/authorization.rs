//! # Authorization

use crate::oauth::{CodeChallengeMethod, ResponseType};
use crate::oid4vci::types::{
    AuthorizationCredential, AuthorizationDetail, AuthorizationDetailType, AuthorizationRequest,
    ClaimsDescription, Format, RequestObject,
};

impl AuthorizationRequest {
    /// Create a new `AuthorizationRequestBuilder`.
    #[must_use]
    pub fn builder() -> AuthorizationRequestBuilder {
        AuthorizationRequestBuilder::new()
    }
}

/// Build an [`AuthorizationRequest`].
#[derive(Default, Debug)]
pub struct AuthorizationRequestBuilder {
    credential_issuer: Option<String>,
    response_type: ResponseType,
    client_id: String,
    redirect_uri: Option<String>,
    state: Option<String>,
    code_challenge: String,
    authorization_details: Option<Vec<AuthorizationDetail>>,
    scope: Option<String>,
    resource: Option<String>,
    subject_id: Option<String>,
    wallet_issuer: Option<String>,
    user_hint: Option<String>,
    issuer_state: Option<String>,
}

impl AuthorizationRequestBuilder {
    /// Create a new `AuthorizationRequestBuilder` with sensible defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the URL of the Credential Issuer .
    #[must_use]
    pub fn credential_issuer(mut self, credential_issuer: impl Into<String>) -> Self {
        self.credential_issuer = Some(credential_issuer.into());
        self
    }

    /// Specify the response type for the authorization request.
    #[must_use]
    pub const fn response_type(mut self, response_type: ResponseType) -> Self {
        self.response_type = response_type;
        self
    }

    /// Specify the Wallet's Client ID.
    #[must_use]
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = client_id.into();
        self
    }

    /// Specify the client's redirection endpoint as previously established
    /// during client registration.
    #[must_use]
    pub fn redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    /// Specify the client state. This is used by the client to maintain state
    /// between the request and callback response.
    #[must_use]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Specify the PKCE code challenge. This is used to prevent authorization
    /// code interception attacks and mitigate the need for client secrets.
    #[must_use]
    pub fn code_challenge(mut self, code_challenge: impl Into<String>) -> Self {
        self.code_challenge = code_challenge.into();
        self
    }

    /// Authorization Details may used to request credentials.
    #[must_use]
    pub fn with_authorization_detail(mut self, authorization_detail: AuthorizationDetail) -> Self {
        self.authorization_details.get_or_insert_with(Vec::new).push(authorization_detail);
        self
    }

    /// Specify an OAuth 2.0 scope value may be used to request a credential.
    ///
    /// The scope value is mapped to a credential type as defined in
    /// `credential_configurations_supported` protoery of the Issuer's metadata.
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Specify the resource to use. This may be the Issuer's identifier to
    /// so the Authorization Server can differentiate between Issuers or the
    /// target resource to which access is being requested. MUST be an
    /// absolute URI.
    #[must_use]
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Specify the (previously authenticated) Holder for the Issuer to use
    /// when authorizing credential issuance.
    #[must_use]
    pub fn subject_id(mut self, subject_id: impl Into<String>) -> Self {
        self.subject_id = Some(subject_id.into());
        self
    }

    /// Specify the Wallet's `OpenID` Connect issuer URL.
    ///
    /// This is useful when the Issuer needs to use the [SIOPv2] discovery
    /// process to determine the Wallet's capabilities and endpoints. This is
    /// recommended for Dynamic Credential Requests.
    ///
    /// [SIOPv2]: (https://openid.net/specs/openid-connect-self-issued-v2-1_0.html)
    #[must_use]
    pub fn wallet_issuer(mut self, wallet_issuer: impl Into<String>) -> Self {
        self.wallet_issuer = Some(wallet_issuer.into());
        self
    }

    /// Specify a user hint that may be used in subsequent callbacks to the
    /// Wallet in order to optimize the user's experience.
    #[must_use]
    pub fn user_hint(mut self, user_hint: impl Into<String>) -> Self {
        self.user_hint = Some(user_hint.into());
        self
    }

    /// Specify Issuer state identifier as provided earlier by the Issuer. This
    /// value typically comes from a Credential Offer made to the Wallet.
    #[must_use]
    pub fn issuer_state(mut self, issuer_state: impl Into<String>) -> Self {
        self.issuer_state = Some(issuer_state.into());
        self
    }

    /// Build the Create Offer request.
    #[must_use]
    pub fn build(self) -> AuthorizationRequest {
        AuthorizationRequest::Object(RequestObject {
            credential_issuer: self.credential_issuer.unwrap_or_default(),
            response_type: self.response_type,
            client_id: self.client_id,
            redirect_uri: self.redirect_uri,
            state: self.state,
            code_challenge: self.code_challenge,
            code_challenge_method: CodeChallengeMethod::S256,
            authorization_details: self.authorization_details,
            scope: self.scope,
            resource: self.resource,
            subject_id: self.subject_id.unwrap_or_default(),
            wallet_issuer: self.wallet_issuer,
            user_hint: self.user_hint,
            issuer_state: self.issuer_state,
        })
    }
}

impl AuthorizationDetail {
    /// Create a new `AuthorizationDetailBuilder`.
    #[must_use]
    pub fn builder() -> AuthorizationDetailBuilder {
        AuthorizationDetailBuilder::new()
    }
}

/// Build an [`AuthorizationDetail`].
#[derive(Default, Debug)]
pub struct AuthorizationDetailBuilder {
    credential_configuration_id: String,
    claims: Option<Vec<ClaimsDescription>>,
    format: Format,
}

impl AuthorizationDetailBuilder {
    /// Create a new `AuthorizationDetailBuilder` with sensible defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the credential configuration ID.
    #[must_use]
    pub fn configuration_id(mut self, configuration_id: impl Into<String>) -> Self {
        self.credential_configuration_id = configuration_id.into();
        self
    }

    /// Specify the claims to include in the credential.
    #[must_use]
    pub fn claims(mut self, claims: Vec<ClaimsDescription>) -> Self {
        self.claims = Some(claims);
        self
    }

    /// Specify the format of the credential.
    #[must_use]
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Build the `AuthorizationDetail`.
    #[must_use]
    pub fn build(self) -> AuthorizationDetail {
        AuthorizationDetail {
            type_: AuthorizationDetailType::OpenIdCredential,
            credential: AuthorizationCredential::ConfigurationId {
                credential_configuration_id: self.credential_configuration_id,
            },
            locations: None,
            claims: self.claims,
        }
    }
}
