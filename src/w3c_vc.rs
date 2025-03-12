//! # Verifiable Credentials
//!
//! This module encompasses the family of W3C Recommendations for Verifiable
//! Credentials, as outlined below.
//!
//! The recommendations provide a mechanism to express credentials on the Web in
//! a way that is cryptographically secure, privacy respecting, and
//! machine-verifiable.

pub mod jose;
pub mod proof;
pub mod vc;
pub mod vp;

use anyhow::anyhow;
use chrono::Utc;
use credibil_infosec::Signer;
use credibil_infosec::jose::jws;

use crate::core::types::LangString;
use crate::core::{Kind, OneMany};
use crate::w3c_vc::jose::VcClaims;
use crate::w3c_vc::vc::{CredentialStatus, CredentialSubject, VerifiableCredential};

/// Generate an ISO mDL `mso_mdoc` format credential.
#[derive(Debug)]
pub struct W3cVcBuilder<S> {
    // subject_id: Option<String>,
    // name: Option<String>,
    // vc_type: Option<String>,
    // status: Option<String>,
    vc: VerifiableCredential,

    signer: S,
}

/// Builder has no signer.
#[doc(hidden)]
pub struct NoSigner;
/// Builder state has a signer.
#[doc(hidden)]
pub struct HasSigner<'a, S: Signer>(pub &'a S);

impl W3cVcBuilder<NoSigner> {
    pub fn new() -> Self {
        let mut vc = VerifiableCredential::default();
        vc.type_ = OneMany::One("VerifiableCredential".to_string());
        Self { vc, signer: NoSigner }
    }
}

impl W3cVcBuilder<NoSigner> {
    /// Set the credential Signer.
    pub fn signer<S: Signer>(self, signer: &'_ S) -> W3cVcBuilder<HasSigner<'_, S>> {
        W3cVcBuilder {
            vc: self.vc,
            signer: HasSigner(signer),
        }
    }
}

impl<S> W3cVcBuilder<S> {
    /// Sets the `id` property
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.vc.id = Some(id.into());
        self
    }

    /// Sets the `type_` property
    #[must_use]
    pub fn add_type(mut self, type_: impl Into<String>) -> Self {
        self.vc.type_.add(type_.into());
        self
    }

    /// Sets the `name` property
    #[must_use]
    pub fn add_name(mut self, name: Option<LangString>) -> Self {
        self.vc.name = name;
        self
    }

    /// Sets the `description` property
    #[must_use]
    pub fn add_description(mut self, description: Option<LangString>) -> Self {
        self.vc.description = description;
        self
    }

    /// Sets the `issuer` property
    #[must_use]
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.vc.issuer = Kind::String(issuer.into());
        self
    }

    /// Adds one or more `credential_subject` properties.
    #[must_use]
    pub fn add_subject(mut self, subj: CredentialSubject) -> Self {
        let one_set = match self.vc.credential_subject {
            OneMany::One(one) => {
                if one == CredentialSubject::default() {
                    OneMany::One(subj)
                } else {
                    OneMany::Many(vec![one, subj])
                }
            }
            OneMany::Many(mut set) => {
                set.push(subj);
                OneMany::Many(set)
            }
        };

        self.vc.credential_subject = one_set;
        self
    }

    /// Sets the `credential_status` property.
    #[must_use]
    pub fn status(mut self, status: Option<OneMany<CredentialStatus>>) -> Self {
        self.vc.credential_status = status;
        self
    }
}

impl<S: Signer> W3cVcBuilder<HasSigner<'_, S>> {
    /// Build the ISO mDL credential, returning a base64url-encoded,
    /// CBOR-encoded, ISO mDL.
    ///
    /// # Errors
    /// TODO: Document errors
    pub async fn build(self) -> anyhow::Result<String> {
        let claims = VcClaims::from_vc(self.vc, Utc::now());

        jws::encode(&claims, self.signer.0)
            .await
            .map_err(|e| anyhow!("issue generating `jwt_vc_json` credential: {e}"))
    }
}

// TODO: move this macro to a more appropriate location (its own crate perhaps).
// N.B. the current dependency tree is a little complex, so this is a temporary
// solution that avoids cyclic dependencies.

/// Generate a closure to resolve public key material required by `Jws::decode`.
///
/// # Example
///
/// ```rust,ignore
/// use credibil_infosec::{verify_key, KeyOps};
///
/// let resolver = KeyOps::resolver(&provider, &request.credential_issuer)?;
/// let jwt = jws::decode(proof_jwt, verify_key!(resolver)).await?;
/// ...
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! verify_key {
    ($resolver:expr) => {{
        // create local reference before moving into closure
        let resolver = $resolver;
        move |kid: String| {
            let local_resolver = resolver.clone();
            async move {
                let resp = credibil_did::dereference(&kid, None, local_resolver)
                    .await
                    .map_err(|e| anyhow::anyhow!("issue dereferencing DID: {e}"))?;
                let Some(credibil_did::Resource::VerificationMethod(vm)) = resp.content_stream
                else {
                    return Err(anyhow::anyhow!("Verification method not found"));
                };
                vm.method_type.jwk().map_err(|e| anyhow::anyhow!("JWK not found: {e}"))
            }
        }
    }};
}
