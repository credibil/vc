//! # Metadata Endpoint
//!
//! This endpoint is used to make Verifier metadata available to the Wallet.
//!
//! As the Verifier is a client to the Wallet's Authorization Server, this
//! endpoint returns Client metadata as defined in [RFC7591](https://www.rfc-editor.org/rfc/rfc7591).

use crate::oid4vp::endpoint::{Body, Handler, NoHeaders, Request, Response};
use crate::oid4vp::provider::{Metadata, Provider};
use crate::oid4vp::types::{IssuerRequest, IssuerResponse};
use crate::oid4vp::{Error, Result};

/// Endpoint for Wallets to request Verifier (Client) metadata.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
async fn metadata(
    _verifier: &str, provider: &impl Provider, request: IssuerRequest,
) -> Result<IssuerResponse> {
    Ok(IssuerResponse {
        client: Metadata::verifier(provider, &request.client_id)
            .await
            .map_err(|e| Error::ServerError(format!("issue getting metadata: {e}")))?,
    })
}

impl Handler for Request<IssuerRequest, NoHeaders> {
    type Response = IssuerResponse;

    fn handle(
        self, verifier: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<impl Into<Response<Self::Response>>>> + Send {
        metadata(verifier, provider, self.body)
    }
}

impl Body for IssuerRequest {}
