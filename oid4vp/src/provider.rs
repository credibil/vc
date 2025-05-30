//! # `OpenID` for Verifiable Presentations (`OpenID4VP`)

use std::future::Future;

use anyhow::{Result, anyhow};
use credibil_core::datastore::Datastore;
use credibil_identity::IdentityResolver;
pub use credibil_identity::SignerExt;
use credibil_status::StatusToken;

pub use crate::common::state::StateStore;
use crate::types::Verifier;

/// Verifier Provider trait.
pub trait Provider:
    Metadata + StateStore + SignerExt + IdentityResolver + StatusToken + Clone
{
}

/// A blanket implementation for `Provider` trait so that any type implementing
/// the required super traits is considered a `Provider`.
impl<T> Provider for T where
    T: Metadata + StateStore + SignerExt + IdentityResolver + StatusToken + Clone
{
}

/// The `Metadata` trait is used by implementers to provide `Verifier` (client)
/// metadata to the library.
pub trait Metadata: Send + Sync {
    /// Verifier (Client) metadata for the specified verifier.
    fn verifier(&self, verifier_id: &str) -> impl Future<Output = Result<Verifier>> + Send;

    // /// Wallet (Authorization Server) metadata.
    // fn wallet(&self, wallet_id: &str) -> impl Future<Output = Result<Wallet>> + Send;

    /// Used by OAuth 2.0 clients to dynamically register with the authorization
    /// server.
    fn register(&self, verifier: &Verifier) -> impl Future<Output = Result<Verifier>> + Send;
}

// const WALLET: &str = "WALLET";
const VERIFIER: &str = "VERIFIER";

impl<T: Datastore> Metadata for T {
    async fn verifier(&self, verifier_id: &str) -> Result<Verifier> {
        let Some(block) = Datastore::get(self, "owner", VERIFIER, verifier_id).await? else {
            return Err(anyhow!("could not find client"));
        };
        Ok(serde_json::from_slice(&block)?)
    }

    async fn register(&self, verifier: &Verifier) -> Result<Verifier> {
        let mut verifier = verifier.clone();
        verifier.oauth.client_id = uuid::Uuid::new_v4().to_string();

        let data = serde_json::to_vec(&verifier)?;
        Datastore::put(self, "owner", VERIFIER, &verifier.oauth.client_id, &data).await?;
        Ok(verifier)
    }

    // async fn wallet(&self, wallet_id: &str) -> Result<Wallet> {
    //     let Some(block) = Datastore::get(self, "owner", WALLET, wallet_id).await? else {
    //         return Err(anyhow!("could not find issuer"));
    //     };
    //     Ok(serde_json::from_slice(&block)?)
    // }
}
