//! # Create Offer Handler
//!
//! The `create_offer` handler generates and returns a Credential Offer for
//! use in invoking a credential issuance flow with a wallet.
//!
//! See <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-credential-offer-endpoint>

use std::vec;

use chrono::Utc;

use crate::core::generate;
use crate::oauth::GrantType;
use crate::oid4vci::endpoint::{Body, Handler, NoHeaders, Request, Response};
use crate::oid4vci::provider::{Metadata, Provider, StateStore, Subject};
use crate::oid4vci::state::{Expire, Offer, Stage, State};
use crate::oid4vci::types::{
    AuthorizationCodeGrant, AuthorizationCredential, AuthorizationDetail, AuthorizationDetailType,
    AuthorizedDetail, CreateOfferRequest, CreateOfferResponse, CredentialOffer, Grants, Issuer,
    OfferType, PreAuthorizedCodeGrant, SendType, Server, TxCode,
};
use crate::oid4vci::{Error, Result};
use crate::{invalid, server};

#[derive(Debug, Default)]
struct Context {
    issuer: Issuer,
    server: Server,
}

/// Credential Offer request handler generates and returns a Credential Offer.
async fn create_offer(
    issuer: &str, provider: &impl Provider, request: CreateOfferRequest,
) -> Result<Response<CreateOfferResponse>> {
    let iss = Metadata::issuer(provider, issuer)
        .await
        .map_err(|e| server!("issue getting issuer metadata: {e}"))?;

    // TODO: determine how to select correct server?
    // select `authorization_server`, if specified
    let server = Metadata::server(provider, issuer, None)
        .await
        .map_err(|e| server!("issue getting server metadata: {e}"))?;

    let ctx = Context { issuer: iss, server };

    request.verify(&ctx)?;

    let grant_types = request.grant_types.clone().unwrap_or_default();
    let offer = request.create_offer(&ctx);
    let tx_code = if request.tx_code_required && grant_types.contains(&GrantType::PreAuthorizedCode)
    {
        Some(generate::tx_code())
    } else {
        None
    };

    // save offer details to state
    if grant_types.contains(&GrantType::PreAuthorizedCode)
        || grant_types.contains(&GrantType::AuthorizationCode)
    {
        let auth_items = if grant_types.contains(&GrantType::PreAuthorizedCode) {
            Some(authorize(provider, &request).await?)
        } else {
            None
        };
        let state_key = state_key(offer.grants.as_ref())?;

        let state = State {
            expires_at: Utc::now() + Expire::Authorized.duration(),
            subject_id: request.subject_id.clone(),
            stage: Stage::Offered(Offer {
                details: auth_items,
                tx_code: tx_code.clone(),
            }),
        };
        StateStore::put(provider, &state_key, &state, state.expires_at)
            .await
            .map_err(|e| server!("issue saving state: {e}"))?;
    }

    // respond with Offer object or uri?
    if request.send_type == SendType::ByVal {
        return Ok(Response {
            status: 201,
            headers: None,
            body: CreateOfferResponse {
                offer_type: OfferType::Object(offer.clone()),
                tx_code: tx_code.clone(),
            },
        });
    }

    let uri_token = generate::uri_token();

    // save offer to state
    let state = State {
        expires_at: Utc::now() + Expire::Authorized.duration(),
        subject_id: request.subject_id,
        stage: Stage::Pending(offer),
    };
    StateStore::put(provider, &uri_token, &state, state.expires_at)
        .await
        .map_err(|e| server!("issue saving state: {e}"))?;

    Ok(Response {
        status: 201,
        headers: None,
        body: CreateOfferResponse {
            offer_type: OfferType::Uri(format!("{issuer}/credential_offer/{uri_token}",)),
            tx_code,
        },
    })
}

impl Handler for Request<CreateOfferRequest, NoHeaders> {
    type Response = CreateOfferResponse;

    fn handle(
        self, issuer: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<impl Into<Response<Self::Response>>>> + Send {
        create_offer(issuer, provider, self.body)
    }
}

impl Body for CreateOfferRequest {}

impl CreateOfferRequest {
    fn verify(&self, ctx: &Context) -> Result<()> {
        tracing::debug!("create_offer::verify");

        // credentials required
        if self.credential_configuration_ids.is_empty() {
            return Err(invalid!("no credentials requested"));
        }

        // are requested credential(s) is supported
        for cred_id in &self.credential_configuration_ids {
            if !ctx.issuer.credential_configurations_supported.contains_key(cred_id) {
                return Err(Error::UnsupportedCredentialType(
                    "requested credential is unsupported".to_string(),
                ));
            }
        }

        // TODO: check requested `grant_types` are supported by OAuth Client
        if let Some(grant_types) = &self.grant_types {
            // check requested `grant_types` are supported by OAuth Server
            if let Some(supported_grants) = &ctx.server.oauth.grant_types_supported {
                for gt in grant_types {
                    if !supported_grants.contains(gt) {
                        return Err(Error::UnsupportedGrantType(
                            "unsupported grant type".to_string(),
                        ));
                    }
                }
            }

            // subject_id is required for pre-authorized offers
            if grant_types.contains(&GrantType::PreAuthorizedCode) && self.subject_id.is_none() {
                return Err(invalid!("`subject_id` is required for pre-authorization"));
            }
        }

        Ok(())
    }

    /// Create `CredentialOffer`
    fn create_offer(&self, ctx: &Context) -> CredentialOffer {
        let auth_code = generate::auth_code();
        let grant_types = self.grant_types.clone().unwrap_or_default();

        // TODO: determine how to select correct server?
        // select `authorization_server`, if specified
        let authorization_server =
            ctx.issuer.authorization_servers.as_ref().map(|servers| servers[0].clone());

        let mut grants = Grants {
            authorization_code: None,
            pre_authorized_code: None,
        };

        if grant_types.contains(&GrantType::PreAuthorizedCode) {
            let tx_code_def = if self.tx_code_required {
                Some(TxCode {
                    input_mode: Some("numeric".to_string()),
                    length: Some(6),
                    description: Some("Please provide the one-time code received".to_string()),
                })
            } else {
                None
            };

            grants.pre_authorized_code = Some(PreAuthorizedCodeGrant {
                pre_authorized_code: auth_code.clone(),
                tx_code: tx_code_def,
                authorization_server: authorization_server.clone(),
            });
        }

        if grant_types.contains(&GrantType::AuthorizationCode) {
            grants.authorization_code = Some(AuthorizationCodeGrant {
                // issuer_state: Some(gen::issuer_state()),
                issuer_state: Some(auth_code),
                authorization_server,
            });
        }

        let grants = if grants.authorization_code.is_some() || grants.pre_authorized_code.is_some()
        {
            Some(grants)
        } else {
            None
        };

        CredentialOffer {
            credential_configuration_ids: self.credential_configuration_ids.clone(),
            grants,
        }
    }
}

/// Authorize requested credentials for the subject.
async fn authorize(
    provider: &impl Provider, request: &CreateOfferRequest,
) -> Result<Vec<AuthorizedDetail>> {
    // skip authorization if not pre-authorized

    let mut authorized = vec![];
    let subject_id = request.subject_id.clone().unwrap_or_default();

    for config_id in request.credential_configuration_ids.clone() {
        let identifiers = Subject::authorize(provider, &subject_id, &config_id)
            .await
            .map_err(|e| server!("issue authorizing holder: {e}"))?;

        authorized.push(AuthorizedDetail {
            authorization_detail: AuthorizationDetail {
                type_: AuthorizationDetailType::OpenIdCredential,
                credential: AuthorizationCredential::ConfigurationId {
                    credential_configuration_id: config_id.clone(),
                },
                claims: None,
                locations: None,
            },
            credential_identifiers: identifiers,
        });
    }

    Ok(authorized)
}

/// Extract `pre_authorized_code` or `issuer_state` from `CredentialOffer` to
/// use as state key.
pub fn state_key(grants: Option<&Grants>) -> Result<String> {
    // get pre-authorized code as state key
    let Some(grants) = grants else {
        return Err(server!("no grants"));
    };

    if let Some(pre_auth_code) = &grants.pre_authorized_code {
        return Ok(pre_auth_code.pre_authorized_code.clone());
    }

    if let Some(authorization_code) = &grants.authorization_code {
        let Some(issuer_state) = &authorization_code.issuer_state else {
            return Err(server!("no issuer_state"));
        };
        return Ok(issuer_state.clone());
    }

    Err(server!("no state key"))
}
