//! # Request Object Endpoint
//!
//! This endpoint is used by the Wallet to retrieve a previously created
//! Authorization Request Object.
//!
//! The Request Object is created by the Verifier when calling the `Create
//! Request` endpoint to create an Authorization Request. Instead of sending the
//! Request Object to the Wallet, the Verifier sends an Authorization Request
//! containing a `request_uri` which can be used to retrieve the saved Request
//! Object.
//!
//! Per the [JWT VC Presentation Profile], the Request Object MUST be returned
//! as an encoded JWT.
//!
//! [JWT VC Presentation Profile]: (https://identity.foundation/jwt-vc-presentation-profile)

use credibil_infosec::jose::JwsBuilder;
use tracing::instrument;

use super::state::State;
use crate::oid4vp::verifier::{
    Provider, RequestObjectRequest, RequestObjectResponse, RequestObjectType,
};
use crate::oid4vp::{Error, Result};
use crate::openid::provider::StateStore;
use crate::w3c_vc::proof::Type;

/// Endpoint for the Wallet to request the Verifier's Request Object when
/// engaged in a cross-device flow.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
#[instrument(level = "debug", skip(provider))]
pub async fn request_object(
    provider: impl Provider, request: &RequestObjectRequest,
) -> Result<RequestObjectResponse> {
    process(provider, request).await
}

async fn process(
    provider: impl Provider, request: &RequestObjectRequest,
) -> Result<RequestObjectResponse> {
    tracing::debug!("request_object::process");

    // retrieve request object from state
    let state = StateStore::get::<State>(&provider, &request.id)
        .await
        .map_err(|e| Error::ServerError(format!("issue fetching state: {e}")))?;
    let req_obj = state.request_object;

    // verify client_id (perhaps should use 'verify' method?)
    if req_obj.client_id != format!("{}/post", request.client_id) {
        return Err(Error::InvalidRequest("client ID mismatch".into()));
    }

    let jws = JwsBuilder::new()
        .jwt_type(Type::OauthAuthzReqJwt)
        .payload(&req_obj)
        .add_signer(&provider)
        .build()
        .await
        .map_err(|e| Error::ServerError(format!("issue building jwt: {e}")))?;
    let jwt_proof =
        jws.encode().map_err(|e| Error::ServerError(format!("issue encoding jwt: {e}")))?;

    Ok(RequestObjectResponse {
        request_object: RequestObjectType::Jwt(jwt_proof),
    })
}
