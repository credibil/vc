//! # `OpenID` for Verifiable Presentations (`OpenID4VP`)

use std::collections::HashMap;
use std::fmt::Debug;

use credibil_infosec::Algorithm;
use credibil_infosec::jose::EncAlgorithm;
use credibil_infosec::jose::jwe::AlgAlgorithm;
use serde::{Deserialize, Serialize};

use crate::oauth::{OAuthClient, OAuthServer};

/// Request to retrieve the Verifier's client metadata.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IssuerRequest {
    /// The Verifier's Client Identifier for which the configuration is to be
    /// returned.
    #[serde(default)]
    pub client_id: String,
}

/// Response containing the Verifier's client metadata.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct IssuerResponse {
    /// The Client metadata for the specified Verifier.
    #[serde(flatten)]
    pub client: Verifier,
}

/// OAuth 2 client metadata used for registering clients of the issuance and
/// wallet authorization servers.
///
/// In the case of Issuance, the Wallet is the Client and the Issuer is the
/// Authorization Server.
///
/// In the case of Presentation, the Wallet is the Authorization Server and the
/// Verifier is the Client.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Verifier {
    /// The subset of Verifier metadata sent to the Wallet in the
    /// Authorization Request Object.
    #[serde(flatten)]
    pub client_metadata: VerifierMetadata,

    /// OAuth 2.0 Client
    #[serde(flatten)]
    pub oauth: OAuthClient,
}

/// Verifier metadata when sent directly in the `RequestObject`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct VerifierMetadata {
    /// Public keys, such as those used by the Wallet for encryption of the
    /// Authorization Response or where the Wallet will require the public key
    /// of the Verifier to generate the Verifiable Presentation.
    ///
    /// This allows the Verifier to pass ephemeral keys specific to this
    /// Authorization Request.
    pub jwks: Option<String>,

    /// An object defining the formats and proof types of Verifiable
    /// Presentations and Verifiable Credentials that a Verifier supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vp_formats: Option<HashMap<Format, VpFormat>>,

    /// The JWS `alg` algorithm for signing authorization responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_signed_response_alg: Option<Algorithm>,

    /// The JWE `alg` algorithm for encrypting authorization responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_encrypted_response_alg: Option<AlgAlgorithm>,

    /// The JWE `enc` algorithm for encrypting authorization responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_encrypted_response_enc: Option<EncAlgorithm>,
}

/// The `OpenID4VCI` specification defines commonly used [Credential Format
/// Profiles] to support.  The profiles define Credential format specific
/// parameters or claims used to support a particular format.
///
/// [Credential Format Profiles]: (https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#name-credential-format-profiles)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Format {
    /// W3C JWT JSON Verifiable Credential.
    #[serde(rename = "jwt_vc_json")]
    JwtVcJson,

    /// W3C JWT JSON Verifiable Presentation.
    #[serde(rename = "jwt_vp_json")]
    JwtVpJson,

    /// W3C JWT JSON Verifiable Presentation.
    #[serde(rename = "dc+sd-jwt")]
    DcSdJwt,
}

/// Used to define the format and proof types of Verifiable Presentations and
/// Verifiable Credentials that a Verifier supports.
///
/// Deployments can extend the formats supported, provided Issuers, Holders and
/// Verifiers all understand the new format.
/// See <https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#alternative_credential_formats>
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct VpFormat {
    /// Algorithms supported by the format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg_values_supported: Option<Vec<String>>,

    /// SD-JWT algorithms supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sd-jwt_alg_values")]
    pub sd_jwt_alg_values: Option<Vec<String>>,

    /// KB-JWT algorithms supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "kb-jwt_alg_values")]
    pub kb_jwt_alg_values: Option<Vec<String>>,
}

/// /// Client Identifier schemes that may be supported by the Wallet.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientIdentifierPrefix {
    /// The Verifier's redirect URI (or response URI when Response Mode is
    /// `direct_post`).
    RedirectUri,

    /// An Entity Identifier as defined in OpenID Federation.
    OpenidFederation,

    /// A DID URI as defined in DID Core specification.
    DecentralizedIdentifier,

    /// The `sub` claim in the Verifier attestation JWT when the Verifier
    /// authenticates using a JWT.
    VerifierAttestation,

    /// A DNS name matching a dNSName Subject Alternative Name (SAN) entry in
    /// the leaf certificate passed with the request.
    X509SanDns,

    /// The audience for a Credential Presentation. Only used with
    /// presentations over the Digital Credentials API.
    Origin,

    /// A hash of the leaf certificate passed with the request.
    X509Hash,

    /// A pre-registered client ID.
    #[default]
    Preregistered,
}

/// OAuth 2.0 Authorization Server metadata.
/// See RFC 8414 - Authorization Server Metadata
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Wallet {
    /// OAuth 2.0 Server
    #[serde(flatten)]
    pub oauth: OAuthServer,

    /// Supported JWE methods  for when the Wallet requires an encrypted
    /// Authorization Response.
    pub presentation_definition_uri_supported: bool,

    /// A list of key value pairs, where the key identifies a Credential format
    /// supported by the Wallet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vp_formats_supported: Option<HashMap<String, VpFormat>>,

    /// Client Identifier prefixes the Wallet supports. Defaults to
    /// `pre-registered`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id_prefixes_supported: Option<Vec<ClientIdentifierPrefix>>,

    /// When the Client Identifier Prefix permits signed Request Objects, the
    /// Wallet SHOULD list supported cryptographic algorithms for securing the
    /// Request Object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_object_signing_alg_values_supported: Option<Vec<Algorithm>>,

    // /// Supported JWS algorithms for JARM. The none algorithm, i.e. a plain
    // /// JWT, is forbidden. If the client doesn’t have a JWS algorithm
    // /// registered for JARM and requests a JWT-secured response_mode the
    // /// default algorithm is RS256.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub authorization_signing_alg_values_supported: Option<Vec<String>>,
    //
    /// Supported JWE algorithms for when the Wallet requires an encrypted
    /// Authorization Response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_encryption_alg_values_supported: Option<Vec<AlgAlgorithm>>,

    /// Supported JWE methods for when the Wallet requires an encrypted
    /// Authorization Response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_encryption_enc_values_supported: Option<Vec<EncAlgorithm>>,
}
