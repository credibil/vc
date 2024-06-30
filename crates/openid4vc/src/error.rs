//! # `OpenID` Errors
//!
//! This module defines errors for `OpenID` for Verifiable Credential Issuance
//! and Verifiable Presentations.

use std::fmt::Debug;

// use anyhow::Error;
// use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Internal error codes for `OpenID` for Verifiable Credential Issuance
#[derive(Error, Debug)]
pub enum Err {
    /// The request is missing a required parameter, includes an unsupported
    /// parameter value, repeats a parameter, includes multiple credentials,
    /// utilizes more than one mechanism for authenticating the client, or is
    /// otherwise malformed.
    #[error(r#"{{"error": "invalid_request", "error_description": "{0}"}}"#)]
    InvalidRequest(String),

    /// Client authentication failed (e.g., unknown client, no client
    /// authentication included, or unsupported authentication method).
    ///
    /// The client tried to send a Token Request with a Pre-Authorized Code
    /// without Client ID but the Authorization Server does not support
    /// anonymous access.
    ///
    /// For Verifiable Presentations:
    ///
    /// `client_metadata` or `client_metadata_uri` is set, but the Wallet
    /// recognizes Client Identifier and already knows metadata associated
    /// with it.
    ///
    /// Verifier's pre-registered metadata has been found based on the Client
    /// Identifier, but `client_metadata` parameter is also set.
    #[error(r#"{{"error": "invalid_client", "error_description": "{0}"}}"#)]
    InvalidClient(String),

    /// The provided authorization grant (e.g., authorization code,
    /// pre-authorized_code) or refresh token is invalid, expired, revoked,
    /// does not match the redirection URI used in the authorization
    /// request, or was issued to another client.
    ///
    /// The Authorization Server expects a PIN in the pre-authorized flow but
    /// the client provides the wrong PIN.
    #[error(r#"{{"error": "invalid_grant", "error_description": "{0}"}}"#)]
    InvalidGrant(String),

    /// The client is not authorized to request an authorization code using this
    /// method.
    #[error(r#"{{"error": "unauthorized_client", "error_description": "{0}"}}"#)]
    UnauthorizedClient(String),

    /// The authorization grant type is not supported by the authorization
    /// server.
    #[error(r#"{{"error": "unsupported_grant_type", "error_description": "{0}"}}"#)]
    UnsupportedGrantType(String),

    /// The requested scope is invalid, unknown, malformed, or exceeds the scope
    /// granted.
    #[error(r#"{{"error": "invalid_scope", "error_description": "{0}"}}"#)]
    InvalidScope(String),

    /// The resource owner or authorization server denied the request.
    #[error(r#"{{"error": "access_denied", "error_description": "{0}"}}"#)]
    AccessDenied(String),

    /// The authorization server does not support obtaining an authorization
    /// code using this method.
    #[error(r#"{{"error": "unsupported_response_type", "error_description": "{0}"}}"#)]
    UnsupportedResponseType(String),

    /// The authorization server encountered an unexpected condition that
    /// prevented it from fulfilling the request.
    #[error(r#"{{"error": "server_error", "error_description": "{0}"}}"#)]
    ServerError(#[from] anyhow::Error),

    /// The authorization server is unable to handle the request due to
    /// temporary overloading or maintenance.
    #[error(r#"{{"error": "temporarily_unavailable", "error_description": "{0}"}}"#)]
    TemporarilyUnavailable(String),

    /// ------------------------------
    /// Verifiable Credential Issuance
    /// ------------------------------

    /// Token Endpoint:

    /// Returned if the Authorization Server is waiting for an End-User interaction
    /// or downstream process to complete. The Wallet SHOULD repeat the access token
    /// request to the token endpoint (a process known as polling). Before each new
    /// request, the Wallet MUST wait at least the number of seconds specified by the
    /// interval claim of the Credential Offer or the authorization response, or 5
    /// seconds if none was provided, and respect any increase in the polling interval
    /// required by the "`slow_down`" error.
    #[error(r#"{{"error": "authorization_pending", "error_description": "{0}"}}"#)]
    AuthorizationPending(String),

    /// A variant of `authorization_pending` error code, the authorization request is
    /// still pending and polling should continue, but the interval MUST be increased
    /// by 5 seconds for this and all subsequent requests.
    #[error(r#"{{"error": "slow_down", "error_description": "{0}"}}"#)]
    SlowDown(String),

    /// Credential Endpoint:

    /// The Credential Request is missing a required parameter, includes an unsupported
    /// parameter or parameter value, repeats the same parameter, or is otherwise
    /// malformed.
    #[error(r#"{{"error": "invalid_credential_request", "error_description": "{0}"}}"#)]
    InvalidCredentialRequest(String),

    /// Requested credential type is not supported.
    #[error(r#"{{"error": "unsupported_credential_type", "error_description": "{0}"}}"#)]
    UnsupportedCredentialType(String),

    /// Requested credential format is not supported.
    #[error(r#"{{"error": "unsupported_credential_format", "error_description": "{0}"}}"#)]
    UnsupportedCredentialFormat(String),

    /// Credential Request did not contain a proof, or proof was invalid, i.e. it was
    /// not bound to a Credential Issuer provided `c_nonce`. The error response contains
    /// new `c_nonce` as well as `c_nonce_expires_in` values to be used by the Wallet
    /// when creating another proof of possession of key material.
    #[allow(missing_docs)]
    #[error(r#"{{"error": "invalid_proof", "error_description": "{hint}", "c_nonce": "{c_nonce}", "c_nonce_expires_in": {c_nonce_expires_in}}}"#)]
    InvalidProof { hint: String, c_nonce: String, c_nonce_expires_in: i64 },

    /// This error occurs when the encryption parameters in the Credential Request are
    /// either invalid or missing. In the latter case, it indicates that the Credential
    /// Issuer requires the Credential Response to be sent encrypted, but the Credential
    /// Request does not contain the necessary encryption parameters.
    #[error(r#"{{"error": "invalid_encryption_parameters", "error_description": "{0}"}}"#)]
    InvalidEncryptionParameters(String),

    /// Deferred Issuance Endpoint:

    /// The Credential issuance is still pending. The error response SHOULD also contain
    /// the interval member, determining the minimum amount of time in seconds that the
    /// Wallet needs to wait before providing a new request to the Deferred Credential
    /// Endpoint. If interval member is missing or its value is not provided, the Wallet
    /// MUST use 5 as the default value.
    #[error(r#"{{"error": "issuance_pending", "error_description": "{0}"}}"#)]
    IssuancePending(String),

    /// The Deferred Credential Request contains an invalid `transaction_id`. This error
    /// occurs when the `transaction_id` was not issued by the respective Credential
    /// Issuer or it was already used to obtain the Credential.
    #[error(r#"{{"error": "invalid_transaction_id", "error_description": "{0}"}}"#)]
    InvalidTransactionId(String),

    /// ------------------------------
    /// Verifiable Presentation
    /// ------------------------------

    /// The Wallet does not support any of the formats requested by the
    /// Verifier, such as those included in the `vp_formats` registration
    /// parameter.
    #[error(r#"{{"error": "vp_formats_not_supported", "error_description": "{0}"}}"#)]
    VpFormatsNotSupported(String),

    /// The Presentation Definition URL cannot be reached.
    #[error(r#"{{"error": "invalid_presentation_definition_uri", "error_description": "{0}"}}"#)]
    InvalidPresentationDefinitionUri(String),

    /// The Presentation Definition URL can be reached, but the specified
    /// `presentation_definition` cannot be found at the URL.
    #[error(
        r#"{{"error": "invalid_presentation_definition_reference", "error_description": "{0}"}}"#
    )]
    InvalidPresentationDefinitionReference(String),
}

impl Err {
    /// Transfrom error to `OpenID` compatible json format.
    #[must_use]
    pub fn to_json(self) -> serde_json::Value {
        self.into()
    }

    /// Transfrom error to `OpenID` compatible query string format.
    /// Does not include `c_nonce` as this is not required for in query
    /// string responses.
    #[must_use]
    pub fn to_querystring(self) -> String {
        let value: serde_json::Value = self.into();
        serde_qs::to_string(&value).unwrap_or_default()
    }

    /// Returns the `c_nonce` and `c_nonce_expires_in` values for `Err::InvalidProof` errors.
    #[must_use]
    pub fn c_nonce(&self) -> Option<(String, i64)> {
        if let Self::InvalidProof {
            c_nonce,
            c_nonce_expires_in,
            ..
        } = &self
        {
            return Some((c_nonce.clone(), *c_nonce_expires_in));
        };

        None
    }
}

impl From<Err> for serde_json::Value {
    fn from(err: Err) -> Self {
        serde_json::from_str(&err.to_string()).unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    // use std::env;

    use serde_json::json;

    use super::*;

    // Test that error details are retuned as json.
    #[test]
    fn err_json() {
        let err: Err = Err::InvalidRequest("bad request".into());

        assert_eq!(
            err.to_json(),
            json!({"error":"invalid_request", "error_description": "bad request"})
        );
    }

    // Test that the error details are returned as an http query string.
    #[test]
    fn err_querystring() {
        let res: crate::Result<()> = Err(Err::InvalidRequest("Invalid request description".into()));
        let err = res.expect_err("expected error");

        assert_eq!(
            err.to_querystring(),
            "error=invalid_request&error_description=Invalid+request+description"
        );
    }

    //     // Test hint and client state are returned in the external response.
    //     #[test]
    //     fn err_state() {
    //         let res: Result<()> = Err(Err::InvalidRequest).state("client-state").hint("Some hint");
    //         let err = res.expect_err("expected error");

    //         assert_eq!(
    //             err.to_json(),
    //             json!({
    //                 "error": "invalid_request",
    //                 "error_description": "Some hint",
    //                 "state": "client-state"
    //             })
    //         );
    //     }

    // Test an InvalidProof error returns c_nonce and c_nonce_expires_in values
    // in the external response.
    #[test]
    fn proof_err() {
        let err: Err = Err::InvalidProof {
            hint: "".into(),
            c_nonce: "c_nonce".into(),
            c_nonce_expires_in: 10,
        }
        .into();

        assert_eq!(err.c_nonce(), Some(("c_nonce".into(), 10)));
        assert_eq!(
            err.to_json(),
            json!({
                "error": "invalid_proof",
                "error_description": "",
                "c_nonce": "c_nonce",
                "c_nonce_expires_in": 10,
            })
        );
    }
}
