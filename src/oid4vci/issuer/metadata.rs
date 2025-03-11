//! # Metadata Endpoint
//!
//! The Credential Issuer Metadata contains information on the Credential
//! Issuer's technical capabilities, supported Credentials, and
//! (internationalized) display information.
//!
//! The Credential Issuer's configuration can be retrieved using the Credential
//! Issuer Identifier.
//!
//! Credential Issuers publishing metadata MUST make a JSON document available
//! at the path formed by concatenating the string
//! `/.well-known/openid-credential-issuer` to the Credential Issuer Identifier.
//! If the Credential Issuer value contains a path component, any terminating /
//! MUST be removed before appending `/.well-known/openid-credential-issuer`.
//!
//! The language(s) in HTTP Accept-Language and Content-Language Headers MUST use the values defined in [RFC3066](https://www.rfc-editor.org/rfc/rfc3066).
//!
//! Below is a non-normative example of a Credential Issuer Metadata request:
//!
//! ```http
//! GET /.well-known/openid-credential-issuer HTTP/1.1
//!     Host: server.example.com
//!     Accept-Language: fr-ch, fr;q=0.9, en;q=0.8, de;q=0.7, *;q=0.5
//! ```

use crate::oid4vci::Result;
use crate::oid4vci::endpoint::{Body, Handler, Headers, Request};
use crate::oid4vci::provider::{Metadata, Provider};
use crate::oid4vci::types::{MetadataHeaders, MetadataRequest, MetadataResponse};
use crate::server;

/// Metadata request handler.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
async fn metadata(
    issuer: &str, provider: &impl Provider, _: Request<MetadataRequest, MetadataHeaders>,
) -> Result<MetadataResponse> {
    // FIXME: use language header in request
    let credential_issuer = Metadata::issuer(provider, issuer)
        .await
        .map_err(|e| server!("issue getting metadata: {e}"))?;
    Ok(MetadataResponse { credential_issuer })
}

impl Handler for Request<MetadataRequest, MetadataHeaders> {
    type Response = MetadataResponse;

    fn handle(
        self, issuer: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        metadata(issuer, provider, self)
    }
}

impl Body for MetadataRequest {}
impl Headers for MetadataHeaders {}
