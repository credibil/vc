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

use tracing::instrument;
use crate::openid::issuer::{Metadata, MetadataRequest, MetadataResponse, Provider};
use crate::openid::{Error, Result};

// use crate::shell;

/// Metadata request handler.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
#[instrument(level = "debug", skip(provider))]
pub async fn metadata(
    provider: impl Provider, request: MetadataRequest,
) -> Result<MetadataResponse> {
    process(&provider, request).await
}

async fn process(provider: &impl Provider, request: MetadataRequest) -> Result<MetadataResponse> {
    tracing::debug!("metadata::process");

    // TODO: add languages to request
    let credential_issuer = Metadata::issuer(provider, &request.credential_issuer)
        .await
        .map_err(|e| Error::ServerError(format!("issue getting metadata: {e}")))?;
    Ok(MetadataResponse { credential_issuer })
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot as assert_snapshot;
    use crate::test_utils::issuer::{Provider, CREDENTIAL_ISSUER};
    use crate::snapshot;
    use crate::test_utils;

    use super::*;

    #[tokio::test]
    async fn metadata_ok() {
        test_utils::init_tracer();
        snapshot!("");

        let provider = Provider::new();

        let request = MetadataRequest {
            credential_issuer: CREDENTIAL_ISSUER.to_string(),
            languages: None,
        };
        let response = metadata(provider, request).await.expect("response is ok");
        assert_snapshot!("metadata:metadata_ok:response", response, {
            ".scopes_supported" => insta::sorted_redaction(),
            ".credential_configurations_supported" => insta::sorted_redaction(),
            ".**.credentialSubject" => insta::sorted_redaction(),
            ".**.credentialSubject.address" => insta::sorted_redaction(),
            ".**[\"org.iso.18013.5.1.mDL\"].claims" => insta::sorted_redaction(),
            ".**[\"org.iso.18013.5.1.mDL\"].claims[\"org.iso.18013.5.1\"]" => insta::sorted_redaction()
        });
    }
}
