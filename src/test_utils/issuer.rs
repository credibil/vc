use chrono::{DateTime, Utc};
use credibil_did::{DidResolver, Document};
use credibil_infosec::{Algorithm, PublicKey, Receiver, SharedSecret, Signer};
use serde::de::DeserializeOwned;
use serde::Serialize;

use super::store::keystore::IssuerKeystore;
use super::store::{issuance, resolver, state};
use crate::openid::issuer::{Client, Dataset, Issuer, Metadata, Server, Subject};
use crate::openid::provider::{Result, StateStore};
use crate::status::issuer::Status;

pub const CREDENTIAL_ISSUER: &str = "http://credibil.io";
pub const CLIENT_ID: &str = "96bfb9cb-0513-7d64-5532-bed74c48f9ab";
pub const NORMAL_USER: &str = "normal_user";
pub const PENDING_USER: &str = "pending_user";
pub const REDIRECT_URI: &str = "http://localhost:3000/callback";

#[derive(Default, Clone, Debug)]
pub struct Provider {
    pub client: issuance::ClientStore,
    pub issuer: issuance::IssuerStore,
    pub server: issuance::ServerStore,
    pub subject: issuance::DatasetStore,
    pub state: state::Store,
}

impl Provider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: issuance::ClientStore::new(),
            issuer: issuance::IssuerStore::new(),
            server: issuance::ServerStore::new(),
            subject: issuance::DatasetStore::new(),
            state: state::Store::new(),
        }
    }
}

impl crate::openid::issuer::Provider for Provider {}

impl Metadata for Provider {
    async fn client(&self, client_id: &str) -> Result<Client> {
        self.client.get(client_id)
    }

    async fn register(&self, client: &Client) -> Result<Client> {
        self.client.add(client)
    }

    async fn issuer(&self, issuer_id: &str) -> Result<Issuer> {
        self.issuer.get(issuer_id)
    }

    async fn server(&self, server_id: &str, _issuer_id: Option<&str>) -> Result<Server> {
        self.server.get(server_id)
    }
}

impl Subject for Provider {
    /// Authorize issuance of the specified credential for the holder.
    async fn authorize(
        &self, subject_id: &str, credential_configuration_id: &str,
    ) -> Result<Vec<String>> {
        self.subject.authorize(subject_id, credential_configuration_id)
    }

    async fn dataset(&self, subject_id: &str, credential_identifier: &str) -> Result<Dataset> {
        self.subject.dataset(subject_id, credential_identifier)
    }
}

impl StateStore for Provider {
    async fn put(&self, key: &str, state: impl Serialize, dt: DateTime<Utc>) -> Result<()> {
        self.state.put(key, state, dt)
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        self.state.get(key)
    }

    async fn purge(&self, key: &str) -> Result<()> {
        self.state.purge(key)
    }
}

impl DidResolver for Provider {
    async fn resolve(&self, url: &str) -> anyhow::Result<Document> {
        resolver::resolve_did(url).await
    }
}

impl Signer for Provider {
    async fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        IssuerKeystore::try_sign(msg)
    }

    async fn verifying_key(&self) -> Result<Vec<u8>> {
        IssuerKeystore::public_key()
    }

    fn algorithm(&self) -> Algorithm {
        IssuerKeystore::algorithm()
    }

    async fn verification_method(&self) -> Result<String> {
        Ok(IssuerKeystore::verification_method())
    }
}

impl Receiver for Provider {
    fn key_id(&self) -> String {
        todo!()
    }

    async fn shared_secret(&self, _sender_public: PublicKey) -> Result<SharedSecret> {
        todo!()
    }
}

impl Status for Provider {}
