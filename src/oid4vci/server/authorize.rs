//! # Authorization Endpoint
//!
//! The Authorization Endpoint is used by Wallets to request access to the
//! Credential Endpoint, that is, to request issuance of a Credential. The
//! endpoint is used in the same manner as defined in [RFC6749].
//!
//! Wallets can request authorization for issuance of a Credential using
//! `authorization_details` (as defined in [RFC9396]) or `scope` parameters (or
//! both).
//!
//! ## Authorization Requests
//!
//! - One (and only one) of `credential_configuration_id` or `format` is
//!   REQUIRED.
//! - `credential_definition` is OPTIONAL.

//! ## Example
//!
//! with `credential_configuration_id`:
//!
//! ```json
//! "authorization_details":[
//!    {
//!       "type": "openid_credential",
//!       "credential_configuration_id": "UniversityDegreeCredential"
//!    }
//! ]
//! ```
//!
//! with `format`:
//!
//! ```json
//! "authorization_details":[
//!     {
//!         "type": "openid_credential",
//!         "format": " dc+sd-jwt",
//!         "vct": "SD_JWT_VC_example_in_OpenID4VCI"
//!     }
//! ]
//! ```
//!
//! **VC Signed as a JWT, Not Using JSON-LD**
//!
//! - `credential_definition` is OPTIONAL.
//!   - `type` is OPTIONAL.
//!   - `credentialSubject` is OPTIONAL.
//!
//! ```json
// ! "authorization_details":[
// !     {
// !         "type": "openid_credential",
// !         "credential_configuration_id": "UniversityDegreeCredential",
// !         "credential_definition": {
// !             "credentialSubject": {
// !                 "given_name": {},
// !                 "family_name": {},
// !                 "degree": {}
// !             }
// !         }
// !     }
// ! ]
//! ```
//! 
//! [RFC6749]: (https://www.rfc-editor.org/rfc/rfc6749.html)
//! [RFC9396]: (https://www.rfc-editor.org/rfc/rfc9396)

use std::collections::HashMap;
use std::fmt::Debug;

use chrono::Utc;
use tracing::instrument;

use crate::core::generate;
use crate::oauth::GrantType;
use crate::oid4vci::endpoint::{Body, NoHeaders, Handler, Request};
use crate::oid4vci::provider::{Metadata, Provider, StateStore, Subject};
use crate::oid4vci::state::{Authorization, Expire, Stage, State};
use crate::oid4vci::types::{
    AuthorizationCredential, AuthorizationDetail, AuthorizationDetailType, AuthorizationRequest,
    AuthorizationResponse, AuthorizedDetail, Issuer, RequestObject,
};
use crate::oid4vci::{Error, Result};
use crate::{invalid, server};

/// Authorization request handler.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
#[instrument(level = "debug", skip(provider))]
async fn authorize(
    issuer: &str, provider: &impl Provider, request: AuthorizationRequest,
) -> Result<AuthorizationResponse> {
    // request object or URI (Pushed Authorization Request)
    let mut is_par = false;
    let request = match request {
        AuthorizationRequest::Object(request) => request,
        AuthorizationRequest::Uri(uri) => {
            is_par = true;
            let state: State = StateStore::get(provider, &uri.request_uri)
                .await
                .map_err(|e| server!("state issue: {e}"))?;
            let Stage::PushedAuthorization(par) = &state.stage else {
                return Err(invalid!("invalid state"));
            };

            if par.expires_at < Utc::now() {
                return Err(invalid!("`request_uri` has expired"));
            }
            par.request.clone()
        }
    };

    // get issuer metadata
    let Ok(issuer) = Metadata::issuer(provider, issuer).await else {
        return Err(invalid!("invalid `credential_issuer`"));
    };

    let mut ctx = Context {
        issuer,
        is_par,
        ..Context::default()
    };
    ctx.verify(provider, &request).await?;
    ctx.process(provider, request).await
}

impl Handler for Request<AuthorizationRequest, NoHeaders> {
    type Response = AuthorizationResponse;

    fn handle(
        self, issuer: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        authorize(issuer, provider, self.body)
    }
}

impl Body for AuthorizationRequest {}

#[derive(Debug, Default)]
pub struct Context {
    pub issuer: Issuer,
    pub auth_dets: HashMap<String, AuthorizationDetail>,
    pub is_par: bool,
}

impl Context {
    pub async fn verify(
        &mut self, provider: &impl Provider, request: &RequestObject,
    ) -> Result<()> {
        tracing::debug!("authorize::verify");

        // client and server metadata
        let Ok(client) = Metadata::client(provider, &request.client_id).await else {
            return Err(Error::InvalidClient("invalid `client_id`".to_string()));
        };
        // TODO: support authorization issuers
        let Ok(server) = Metadata::server(provider, &self.issuer.credential_issuer, None).await
        else {
            return Err(invalid!("invalid `credential_issuer`"));
        };

        // If the server requires pushed authorization requests, the request
        // must be a PAR.
        if let Some(must_be_par) = server.oauth.require_pushed_authorization_requests {
            if must_be_par && !self.is_par {
                return Err(invalid!("pushed authorization request is required"));
            }
        }

        // Requested `response_type` must be supported by the authorization server.
        if !server.oauth.response_types_supported.contains(&request.response_type) {
            return Err(Error::UnsupportedResponseType(
                "`response_type` not supported by server".to_string(),
            ));
        }

        // Client and server must support the same scopes.
        if let Some(client_scope) = &client.oauth.scope {
            if let Some(server_scopes) = &server.oauth.scopes_supported {
                let scopes: Vec<&str> = client_scope.split_whitespace().collect();
                if !scopes.iter().all(|s| server_scopes.contains(&(*s).to_string())) {
                    return Err(invalid!("client scope not supported"));
                }
            } else {
                return Err(invalid!("server supported scopes not set"));
            }
        } else {
            return Err(invalid!("client scope not set"));
        }

        // 'authorization_code' grant_type allowed (client and server)?
        let client_grant_types = client.oauth.grant_types.unwrap_or_default();
        if !client_grant_types.contains(&GrantType::AuthorizationCode) {
            return Err(Error::UnauthorizedClient(
                "authorization_code grant not supported for client".to_string(),
            ));
        }
        let server_grant_types = server.oauth.grant_types_supported.unwrap_or_default();
        if !server_grant_types.contains(&GrantType::AuthorizationCode) {
            return Err(invalid!("authorization_code grant not supported by server"));
        }

        // is holder identified (authenticated)?
        if request.subject_id.is_empty() {
            return Err(invalid!("missing holder subject"));
        }

        // does offer `subject_id`  match request `subject_id`?
        if let Some(issuer_state) = &request.issuer_state {
            let state: State = StateStore::get(provider, issuer_state)
                .await
                .map_err(|e| server!("issue getting state: {e}"))?;

            if state.is_expired() {
                return Err(invalid!("issuer state expired"));
            }

            if state.subject_id.as_ref() != Some(&request.subject_id) {
                return Err(invalid!("request `subject_id` does not match offer"));
            }
        }

        // has a credential been requested?
        if request.authorization_details.is_none() && request.scope.is_none() {
            return Err(invalid!("no credentials requested"));
        }

        // verify authorization_details
        if let Some(authorization_details) = &request.authorization_details {
            self.verify_authorization_details(authorization_details.clone())?;
        }
        // verify scope
        if let Some(scope) = &request.scope {
            self.verify_scope(scope)?;
        }

        // redirect_uri
        let Some(redirect_uri) = &request.redirect_uri else {
            return Err(invalid!("no `redirect_uri` specified"));
        };
        let Some(redirect_uris) = client.oauth.redirect_uris else {
            return Err(server!("`redirect_uri`s not set for client"));
        };
        if !redirect_uris.contains(redirect_uri) {
            return Err(invalid!("`redirect_uri` is not registered"));
        }

        // response_type
        if !client.oauth.response_types.unwrap_or_default().contains(&request.response_type) {
            return Err(Error::UnsupportedResponseType(
                "`response_type` not supported for client".to_string(),
            ));
        }
        if !server.oauth.response_types_supported.contains(&request.response_type) {
            return Err(Error::UnsupportedResponseType(
                "`response_type` not supported by server".to_string(),
            ));
        }

        // code_challenge
        // N.B. while optional in the spec, we require it
        let challenge_methods = server.oauth.code_challenge_methods_supported.unwrap_or_default();
        if !challenge_methods.contains(&request.code_challenge_method) {
            return Err(invalid!("unsupported `code_challenge_method`"));
        }
        if request.code_challenge.len() < 43 || request.code_challenge.len() > 128 {
            return Err(invalid!("code_challenge must be between 43 and 128 characters"));
        }

        Ok(())
    }

    // Verify Credentials requested in `authorization_details` are supported.
    // N.B. has side effect of saving valid `authorization_detail` objects into
    // context for later use.
    fn verify_authorization_details(
        &mut self, authorization_details: Vec<AuthorizationDetail>,
    ) -> Result<()> {
        // check each credential requested is supported by the issuer
        for mut detail in authorization_details {
            if detail.type_ != AuthorizationDetailType::OpenIdCredential {
                return Err(Error::InvalidAuthorizationDetails(
                    "invalid authorization_details type".to_string(),
                ));
            }

            // verify requested claims
            let config_id = match &detail.credential {
                AuthorizationCredential::ConfigurationId {
                    credential_configuration_id,
                } => credential_configuration_id,
                AuthorizationCredential::Format(fmt) => {
                    let config_id = self
                        .issuer
                        .credential_configuration_id(fmt)
                        .map_err(|e| server!("issue getting `credential_configuration_id`: {e}"))?;

                    detail.credential = AuthorizationCredential::ConfigurationId {
                        credential_configuration_id: config_id.clone(),
                    };

                    config_id
                }
            };

            // check claims are supported and include all mandatory claims
            if let Some(requested) = &detail.claims {
                let Some(config) = self.issuer.credential_configurations_supported.get(config_id)
                else {
                    return Err(Error::InvalidAuthorizationDetails(
                        "invalid credential_configuration_id".to_string(),
                    ));
                };
                config.verify_claims(requested).map_err(|e| invalid!("{e}"))?;
            }

            self.auth_dets.insert(config_id.clone(), detail.clone());
        }

        Ok(())
    }

    // Verify Credentials requested in `scope` are supported.
    // N.B. has side effect of saving valid scope items into context for later use.
    fn verify_scope(&mut self, scope: &str) -> Result<()> {
        if let Some(scope_item) = scope.split_whitespace().next() {
            // find supported configuration with the requested scope
            let mut found = false;
            for (config_id, cred_cfg) in &self.issuer.credential_configurations_supported {
                // `authorization_details` credential request takes precedence `scope` request
                if self.auth_dets.contains_key(config_id) {
                    continue;
                }

                // save scope item + credential_configuration_id
                if cred_cfg.scope == Some(scope_item.to_string()) {
                    let detail = AuthorizationDetail {
                        type_: AuthorizationDetailType::OpenIdCredential,
                        credential: AuthorizationCredential::ConfigurationId {
                            credential_configuration_id: config_id.clone(),
                        },
                        claims: None,
                        locations: None,
                    };

                    self.auth_dets.insert(config_id.clone(), detail);
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(Error::InvalidScope("invalid scope".to_string()));
            }
        }

        Ok(())
    }

    // Authorize Wallet request:
    // - check which requested `credential_configuration_id`s the holder is
    //   authorized for
    // - save related auth_dets/scope items in state
    async fn process(
        &self, provider: &impl Provider, request: RequestObject,
    ) -> Result<AuthorizationResponse> {
        tracing::debug!("authorize::process");

        // authorization_detail
        let mut details = vec![];

        for (config_id, mut auth_det) in self.auth_dets.clone() {
            let identifiers =
                Subject::authorize(provider, &request.subject_id, &config_id)
                    .await
                    .map_err(|e| Error::AccessDenied(format!("issue authorizing subject: {e}")))?;

            auth_det.credential = AuthorizationCredential::ConfigurationId {
                credential_configuration_id: config_id.clone(),
            };

            details.push(AuthorizedDetail {
                authorization_detail: auth_det.clone(),
                credential_identifiers: identifiers.clone(),
            });
        }

        // return an error if holder is not authorized for any requested credentials
        if details.is_empty() {
            return Err(Error::AccessDenied(
                "holder is not authorized for requested credentials".to_string(),
            ));
        }

        // save authorization state
        let state = State {
            expires_at: Utc::now() + Expire::Authorized.duration(),
            subject_id: Some(request.subject_id),
            stage: Stage::Authorized(Authorization {
                code_challenge: request.code_challenge,
                code_challenge_method: request.code_challenge_method,
                details,
                client_id: request.client_id,
                redirect_uri: request.redirect_uri.clone(),
            }),
        };

        let code = generate::auth_code();
        StateStore::put(provider, &code, &state, state.expires_at)
            .await
            .map_err(|e| server!("issue saving authorization state: {e}"))?;

        // remove offer state
        if let Some(issuer_state) = &request.issuer_state {
            StateStore::purge(provider, issuer_state)
                .await
                .map_err(|e| server!("issue purging offer state: {e}"))?;
        }

        Ok(AuthorizationResponse {
            code,
            state: request.state,
            redirect_uri: request.redirect_uri.unwrap_or_default(),
        })
    }
}
