//! # Core Utilities for Credibil VC

// // generic member access API on the error trait
// // https://github.com/rust-lang/rust/issues/99301
// #![feature(error_generic_member_access)]

pub mod generate;
pub mod pkce;
pub mod strings;
pub mod urlencode;

use anyhow::{Result, anyhow};
use credibil_did::{DidResolver, PublicKeyJwk, Resource};
use serde::{Deserialize, Serialize};

/// `Kind` allows serde to serialize/deserialize a string or an object.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Kind<T> {
    /// Simple string value
    String(String),

    /// Complex object value
    Object(T),
}

impl<T> Default for Kind<T> {
    fn default() -> Self {
        Self::String(String::new())
    }
}

impl<T> From<String> for Kind<T> {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl<T> Kind<T> {
    /// Returns `true` if the quota is a single object.
    pub const fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            Self::Object(_) => None,
        }
    }

    /// Returns `true` if the quota contains an array of objects.
    pub const fn as_object(&self) -> Option<&T> {
        match self {
            Self::String(_) => None,
            Self::Object(o) => Some(o),
        }
    }
}

/// `OneMany` allows serde to serialize/deserialize a single object or a set of
/// objects.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OneMany<T> {
    /// Single object
    One(T),

    /// Set of objects
    Many(Vec<T>),
}

impl<T: Default> Default for OneMany<T> {
    fn default() -> Self {
        Self::One(T::default())
    }
}

impl<T> From<T> for OneMany<T> {
    fn from(value: T) -> Self {
        Self::One(value)
    }
}

impl<T: Clone + Default + PartialEq> OneMany<T> {
    /// Returns `true` if the quota is a single object.
    pub const fn as_one(&self) -> Option<&T> {
        match self {
            Self::One(o) => Some(o),
            Self::Many(_) => None,
        }
    }

    /// Returns `true` if the quota contains an array of objects.
    pub const fn as_many(&self) -> Option<&[T]> {
        match self {
            Self::One(_) => None,
            Self::Many(m) => Some(m.as_slice()),
        }
    }

    /// Adds an object to the quota. If the quota is a single object, it is
    /// converted to a set of objects.
    pub fn add(&mut self, item: T) {
        match self {
            Self::One(one) => {
                *self = Self::Many(vec![one.clone(), item]);
            }
            Self::Many(many) => {
                many.push(item);
            }
        }
    }

    /// Returns the length of the quota.
    pub fn len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::Many(many) => many.len(),
        }
    }

    /// Returns `true` if the quota is an empty `Many`.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::One(_) => false,
            Self::Many(many) => many.is_empty(),
        }
    }
}

/// Retrieve the JWK specified by the provided DID URL.
///
/// # Errors
///
/// TODO: Document errors
pub async fn did_jwk<R>(did_url: &str, resolver: &R) -> Result<PublicKeyJwk>
where
    R: DidResolver + Send + Sync,
{
    let deref = credibil_did::dereference(did_url, None, resolver.clone())
        .await
        .map_err(|e| anyhow!("issue dereferencing DID URL: {e}"))?;
    let Some(Resource::VerificationMethod(vm)) = deref.content_stream else {
        return Err(anyhow!("Verification method not found"));
    };
    vm.method_type.jwk().map_err(|e| anyhow!("JWK not found: {e}"))
}
