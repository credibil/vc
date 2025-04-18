//! # Response Endpoint
//!
//! This endpoint is where the Wallet **redirects** to when returning an [RFC6749](https://www.rfc-editor.org/rfc/rfc6749.html).
//! Authorization Response when both Wallet and Verifier interact on the same
//! device. That is, during a 'same-device flow'.
//!
//! The Wallet only returns a VP Token if the corresponding Authorization
//! Request contained a `presentation_definition` parameter, a
//! `presentation_definition_uri` parameter, or a `scope` parameter representing
//! a Presentation Definition.
//!
//! The VP Token can be returned in the Authorization Response or the Token
//! Response depending on the Response Type used.
//!
//! If the Authorization Request's Response Type value is "`vp_token`", the VP
//! Token is returned in the Authorization Response. When the Response Type
//! value is "`vp_token id_token`" and the scope parameter contains "openid",
//! the VP Token is returned in the Authorization Response alongside a
//! Self-Issued ID Token as defined in [SIOPv2](https://openid.net/specs/openid-connect-self-issued-v2-1_0.html).
//!
//! If the Response Type value is "code" (Authorization Code Grant Type), the VP
//! Token is provided in the Token Response.

use crate::format::sd_jwt;
use crate::oid4vp::endpoint::{Body, Handler, NoHeaders, Request, Response};
use crate::oid4vp::provider::{Provider, StateStore};
use crate::oid4vp::state::State;
use crate::oid4vp::types::{AuthorzationResponse, RedirectResponse, RequestedFormat};
use crate::oid4vp::{Error, Result};

/// Endpoint for the Wallet to respond Verifier's Authorization Request.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
async fn response(
    _verifier: &str, provider: &impl Provider, request: AuthorzationResponse,
) -> Result<RedirectResponse> {
    // TODO: handle case where Wallet returns error instead of submission
    verify(provider, &request).await?;

    // clear state
    let Some(state_key) = &request.state else {
        return Err(Error::InvalidRequest("client state not found".to_string()));
    };
    StateStore::purge(provider, state_key)
        .await
        .map_err(|e| Error::ServerError(format!("issue purging state: {e}")))?;

    Ok(RedirectResponse {
        // TODO: add response to state using `response_code` so Wallet can fetch full response
        // TODO: align redirct_uri to spec
        // redirect_uri: Some(format!("http://localhost:3000/cb#response_code={}", "1234")),
        redirect_uri: Some("http://localhost:3000/cb".to_string()),
        response_code: None,
    })
}

impl Handler for Request<AuthorzationResponse, NoHeaders> {
    type Response = RedirectResponse;

    fn handle(
        self, verifier: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<impl Into<Response<Self::Response>>>> + Send {
        response(verifier, provider, self.body)
    }
}

impl Body for AuthorzationResponse {}

// TODO: validate  Verifiable Presentation by format
// Check integrity, authenticity, and holder binding of each Presentation
// in the VP Token according to the rules for the Presentation's format.

// Verfiy the `vp_token` and presentation submission against the `dcql_query`
// in the request.
async fn verify(provider: &impl Provider, request: &AuthorzationResponse) -> Result<()> {
    // get state by client state key
    let Some(state_key) = &request.state else {
        return Err(Error::InvalidRequest("client state not found".to_string()));
    };
    let Ok(state) = StateStore::get::<State>(provider, state_key).await else {
        return Err(Error::InvalidRequest("state not found".to_string()));
    };

    let request_object = &state.request_object;
    let dcql_query = &request_object.dcql_query;

    // verify presentation matches query:
    //  - verify request has been fulfilled for each credential requested:
    //  - check VC format matches a requested format
    //  - verify query constraints have been met
    //  - verify VC is valid (hasn't expired, been revoked, etc)

    // check nonce matches
    for (query_id, presentations) in &request.vp_token {
        let Some(query) = dcql_query.credentials.iter().find(|q| q.id == *query_id) else {
            return Err(Error::InvalidRequest(format!("query not found: {query_id}")));
        };

        for vp in presentations {
            match query.format {
                RequestedFormat::DcSdJwt => {
                    sd_jwt::verify(vp, provider).await.map_err(|e| {
                        Error::InvalidRequest(format!("failed to verify sd-jwt presentation: {e}"))
                    })?;
                    // sd_jwt::verify(
                    //     &request_object.client_id,
                    //     &query.id,
                    //     &query.format,
                    //     &presentations,
                    // )
                    // .await
                }
                _ => {
                    return Err(Error::InvalidRequest(format!(
                        "unsupported format: {:?}",
                        query.format
                    )));
                }
            }
        }

        // if nonce != request_object.nonce {
        //     return Err(Error::InvalidRequest("nonce does not match".to_string()));
        // }

        println!("vps: {presentations:?}");
    }

    // let dcql_query = &request_object.dcql_query;

    // FIXME: look up credential status using status.id
    // if let Some(_status) = &vc.credential_status {
    //     // FIXME: look up credential status using status.id
    // }

    // FIXME: perform Verifier policy checks
    // Checks based on the set of trust requirements such as trust frameworks
    // it belongs to (i.e., revocation checks), if applicable.

    Ok(())
}
