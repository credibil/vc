//! # JOSE Proofs
//!
//! JSON Object Signing and Encryption ([JOSE]) proofs are a form of enveloping
//! proofs of Credentials based on JWT [RFC7519], JWS [RFC7515], and JWK
//! [RFC7517].
//!
//! The Securing Verifiable Credentials using JOSE and COSE [VC-JOSE-COSE]
//! recommendation defines a "bridge" between these and the Verifiable
//! Credentials Data Model v2.0, specifying the suitable header claims, media
//! types, etc.
//!
//! In the case of JOSE, the Credential is the "payload". This is preceded by a
//! suitable header whose details are specified by Securing Verifiable
//! Credentials using JOSE and COSE for the usage of JWT. These are encoded,
//! concatenated, and signed, to be transferred in a compact form by one entity
//! to an other (e.g., sent by the holder to the verifier). All the intricate
//! details on signatures, encryption keys, etc., are defined by the IETF
//! specifications; see Example 6 for a specific case.
//!
//! ## Note
//!
//! If the JWT is only a JWE, iss, exp and aud MUST be omitted in the JWT Claims
//! Set of the JWE, and the processing rules as per JARM Section 2.4 related to
//! these claims do not apply. [OpenID4VP] JWT - JWE
//!
//! ```json
//! {
//!   "vp_token": "eyJI...",
//!   "presentation_submission": {...}
//! }
//! ```
//!
//! [JOSE]: https://datatracker.ietf.org/wg/jose/about
//! [RFC7519]: https://www.rfc-editor.org/rfc/rfc7519
//! [RFC7517]: https://www.rfc-editor.org/rfc/rfc7517
//! [VC-JOSE-COSE]: https://w3c.github.io/vc-jose-cose
//! [OpenID4VP]: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html

use std::fmt::Debug;
use std::str;

use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::super::model::{VerifiableCredential, VerifiablePresentation};
use crate::core::{Kind, OneMany};

/// Claims used for Verifiable Credential issuance when format is
/// "`jwt_vc_json`".
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct VcClaims {
    /// The Holder ID the Credential is intended for. Typically, the DID of the
    /// Holder from the Credential's `credentialSubject.id` property.
    ///
    /// For example, "did:example:ebfeb1f712ebc6f1c276e12ec21".
    pub sub: String,

    /// The `issuer` property of the Credential.
    ///
    /// For example, "did:example:123456789abcdefghi#keys-1".
    pub iss: String,

    /// The Credential's issuance date, encoded as a UNIX timestamp.
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,

    /// The `id` property of the Credential.
    pub jti: String,

    /// The expiration time of the signature, encoded as a UNIX timestamp. This
    /// is NOT the same as the Credential `validUntil`property.
    #[serde(with = "ts_seconds_option")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub exp: Option<DateTime<Utc>>,

    /// The Credential.
    pub vc: VerifiableCredential,
}

impl VcClaims {
    /// Create Verifiable Credential JWT payload from a W3C Verifiable
    /// Credential.
    #[must_use]
    pub fn from_vc(vc: VerifiableCredential, issued_at: DateTime<Utc>) -> Self {
        let subject = match &vc.credential_subject {
            OneMany::One(sub) => sub,
            OneMany::Many(subs) => &subs[0],
        };

        let issuer_id = match &vc.issuer {
            Kind::String(id) => id,
            Kind::Object(issuer) => &issuer.id,
        };

        Self {
            // TODO: find better way to set sub (shouldn't need to be in vc)
            sub: subject.id.clone().unwrap_or_default(),
            iss: issuer_id.clone(),
            iat: issued_at,
            jti: vc.id.clone().unwrap_or_default(),
            exp: vc.valid_until,
            vc,
        }
    }
}

/// To sign, or sign and encrypt the Authorization Response, implementations MAY
/// use JWT Secured Authorization Response Mode for OAuth 2.0
/// ([JARM](https://openid.net/specs/oauth-v2-jarm-final.html)).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct VpClaims {
    /// The `holder` property of the Presentation.
    /// For example, "did:example:123456789abcdefghi".
    pub iss: String,

    /// The `id` property of the Presentation.
    ///
    /// For example, "urn:uuid:3978344f-8596-4c3a-a978-8fcaba3903c5".
    pub jti: String,

    /// The `client_id` value from the Verifier's Authorization Request.
    pub aud: String,

    /// The `nonce` value from the Verifier's Authorization Request.
    pub nonce: String,

    /// The time the Presentation was created, encoded as a UNIX timestamp
    /// ([RFC7519](https://www.rfc-editor.org/rfc/rfc7519) `NumericDate`).
    #[serde(with = "ts_seconds")]
    pub nbf: DateTime<Utc>,

    /// The time the Presentation was created, encoded as a UNIX timestamp
    /// ([RFC7519](https://www.rfc-editor.org/rfc/rfc7519) `NumericDate`).
    #[serde(with = "ts_seconds")]
    pub iat: DateTime<Utc>,

    /// The time the Presentation will expire, encoded as a UNIX timestamp
    /// ([RFC7519](https://www.rfc-editor.org/rfc/rfc7519) `NumericDate`).
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,

    /// The Verifiable Presentation.
    pub vp: VerifiablePresentation,
}

impl From<VerifiablePresentation> for VpClaims {
    fn from(vp: VerifiablePresentation) -> Self {
        Self {
            iss: vp.holder.clone().unwrap_or_default(),
            jti: vp.id.clone().unwrap_or_default(),
            nbf: Utc::now(),
            iat: Utc::now(),

            // TODO: configure `exp` time
            exp: Utc::now()
                .checked_add_signed(TimeDelta::try_hours(1).unwrap_or_default())
                .unwrap_or_default(),
            vp,

            ..Self::default()
        }
    }
}
