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
use crate::openid::verifier::{
    Provider, RequestObjectRequest, RequestObjectResponse, RequestObjectType,
};
use crate::openid::provider::StateStore;
use crate::openid::{Error, Result};
use crate::w3c_vc::proof::Type;

use super::state::State;

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

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use credibil_infosec::jose::jws;
    use insta::assert_yaml_snapshot as assert_snapshot;
    use crate::test_utils::verifier::{Provider, VERIFIER_ID};
    use crate::core::Kind;
    use crate::dif_exch::PresentationDefinition;
    use crate::openid::verifier::{ClientIdScheme, RequestObject, ResponseType, Verifier};
    use crate::{test_utils, verify_key};

    use super::*;
    use super::super::state::Expire;

    #[tokio::test]
    async fn request_jwt() {
        test_utils::init_tracer();

        let provider = Provider::new();
        let state_key = "ABCDEF123456";
        let nonce = "1234567890";

        let req_obj = RequestObject {
            response_type: ResponseType::VpToken,
            client_id: format!("{VERIFIER_ID}/post"),
            state: Some(state_key.to_string()),
            nonce: nonce.to_string(),
            response_mode: Some("direct_post".into()),
            response_uri: Some(format!("{VERIFIER_ID}/post")),
            presentation_definition: Kind::Object(PresentationDefinition::default()),
            client_id_scheme: Some(ClientIdScheme::RedirectUri),
            client_metadata: Verifier::default(),

            // TODO: populate missing RequestObject attributes
            redirect_uri: None,
            scope: None,
        };

        let state = State {
            expires_at: Utc::now() + Expire::Request.duration(),
            request_object: req_obj,
        };
        StateStore::put(&provider, &state_key, &state, state.expires_at)
            .await
            .expect("state exists");

        let request = RequestObjectRequest {
            client_id: VERIFIER_ID.to_string(),
            id: state_key.to_string(),
        };
        let response = request_object(provider.clone(), &request).await.expect("response is valid");

        let RequestObjectType::Jwt(jwt_enc) = &response.request_object else {
            panic!("no JWT found in response");
        };

        let jwt: jws::Jwt<RequestObject> =
            jws::decode(&jwt_enc, verify_key!(&provider)).await.expect("jwt is valid");
        assert_snapshot!("response", jwt);

        // request state should not exist
        assert!(StateStore::get::<State>(&provider, state_key).await.is_ok());
    }
}
