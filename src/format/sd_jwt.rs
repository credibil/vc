//! # IETF SD-JWT-based Credential Format
//!
//! This module provides the implementation of SD-JWT-based Verifiable
//! Credentials (SD-JWT VC).
//!
//! Encompasses data formats as well as validation and processing rules to
//! express Verifiable Credentials with JSON payloads with and without
//! selective disclosure based on the SD-JWT [I-D.ietf-oauth-sd-jwt-vc] format.
//!
//! [I-D.ietf-oauth-sd-jwt-vc]: https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html

mod issue;
mod present;
mod store;
mod verify;

use anyhow::Result;
use base64ct::{Base64UrlUnpadded, Encoding};
use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Utc};
use credibil_infosec::PublicKeyJwk;
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub use self::issue::SdJwtVcBuilder;
pub use self::present::SdJwtVpBuilder;
pub use self::store::to_queryable;
pub use self::verify::verify;

/// Claims that can be included in the payload of SD-JWT VCs.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SdJwtClaims {
    /// Digests of selective disclosure claims. Each digest is a hash (using
    /// `_sd_alg` hashing algorithm) of the base64url-encoded Disclosure.
    #[serde(rename = "_sd")]
    pub sd: Vec<String>,

    /// Algorithm used to generate `_sd` digests. A default of `sha-256` is
    /// used when not set.
    #[serde(rename = "_sd_alg")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sd_alg: Option<String>,

    /// The type of Verifiable Credential represented.
    /// For example, `https://credentials.example.com/identity_credential`
    pub vct: String,

    /// VCT integrity metadata.
    #[serde(rename = "vct#integrity")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vct_integrity: Option<String>,

    /// The Issuer (as a URI) of the Verifiable Credential.
    pub iss: String,

    /// The time of issuance of the Verifiable Credential.
    #[serde(with = "ts_seconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<DateTime<Utc>>,

    /// The time before which the Verifiable Credential must not be accepted
    /// before validating.
    #[serde(with = "ts_seconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<DateTime<Utc>>,

    /// The expiry time of the Verifiable Credential after which it is no longer
    /// valid.
    #[serde(with = "ts_seconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<DateTime<Utc>>,

    /// The identifier of the Subject of the Verifiable Credential. The Issuer
    /// MAY use it to provide the Subject identifier known by the Issuer. There
    /// is no requirement for a binding to exist between sub and cnf claims.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    /// Contains a public key associated with the key binding (provided by the
    /// Wallet via proof-of-possession of key material) in order to provide
    /// confirmation of cryptographic Key Binding.
    ///
    /// The Key Binding JWT in the SD-JWT presentation must be secured by the
    /// key identified in this claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cnf: Option<KeyBinding>,

    /// The information on how to read the status of the Verifiable Credential.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Key Binding JWT is used in SD-JWT presentations when requested by the
/// Verifier.
///
/// A Key Binding JWT is "tied to" an SD-JWT when its payload is signed using
/// the key included in the SD-JWT payload, and the KB-JWT contains a hash
/// of the SD-JWT in its `sd_hash` claim.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KbJwtClaims {
    /// The value of nonce from the Authorization Request.
    pub nonce: String,

    /// The Client Identifier, except for requests over the DC API where it
    /// MUST be the Origin prefixed with origin.
    pub aud: String,

    /// The time of issuance of the Key Binding JWT.
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,

    /// The base64url-encoded hash value over the Issuer-signed JWT and the
    /// selected Disclosures.
    pub sd_hash: String,
}

/// The type of binding between the SD-JWT and the public key.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyBinding {
    /// The public key is bound to the SD-JWT using a JWK.
    Jwk(PublicKeyJwk),
}

/// A claim disclosure.
pub struct Disclosure {
    /// The claim name.
    pub name: String,

    /// The disclosure value.
    pub value: Value,

    salt: String,
}

impl Disclosure {
    /// Create a new disclosure.
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
            salt: Base64UrlUnpadded::encode_string(&rng().random::<[u8; 16]>()),
        }
    }

    /// `Base64Url` encode the disclosure as JSON array of the form:
    /// `["<b64 Salt>","<Claim Name>","<Claim Value>"]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the encoding fails.
    pub fn encoded(&self) -> Result<String> {
        let sd_json = serde_json::to_vec(&json!([self.salt, self.name, self.value]))?;
        Ok(Base64UrlUnpadded::encode_string(&sd_json))
    }

    /// Generate the disclosure digest. Each digest is a base64url-encoded hash
    /// (using `_sd_alg` hashing algorithm) of the encoded Disclosure.
    ///
    /// # Errors
    ///
    /// Returns an error if the encoding fails.
    pub fn hashed(&self) -> Result<String> {
        Ok(Base64UrlUnpadded::encode_string(Sha256::digest(&self.encoded()?).as_slice()))
    }
}

/// JWT `typ` headers options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum JwtType {
    /// JWT `typ` for SD-JWT credentials.
    #[serde(rename = "dc+sd-jwt")]
    #[default]
    SdJwt,

    /// JWT `typ` for Key Binding JWT.
    #[serde(rename = "kb+jwt")]
    KbJwt,
}

impl From<JwtType> for String {
    fn from(t: JwtType) -> Self {
        From::from(&t)
    }
}

impl From<&JwtType> for String {
    fn from(t: &JwtType) -> Self {
        match t {
            JwtType::SdJwt => "dc+sd-jwt".to_string(),
            JwtType::KbJwt => "kb+jwt".to_string(),
        }
    }
}
