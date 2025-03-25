pub mod kms;
pub mod store;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use credibil_did::{DidResolver, Document};
use credibil_infosec::Signer;
use credibil_infosec::jose::jwa::Algorithm;
use credibil_vc::oid4vci::provider::{Metadata, Provider, StateStore, Subject};
use credibil_vc::oid4vci::types::{Client, Dataset, Issuer, Server};
use credibil_vc::status::issuer::Status;
use serde::{Deserialize, Serialize};

use crate::provider::kms::Keyring;
use crate::provider::store::{ClientStore, DatasetStore, IssuerStore, ServerStore};

#[derive(Clone, Debug)]
pub struct ProviderImpl {
    client: ClientStore,
    issuer: IssuerStore,
    server: ServerStore,
    keyring: Keyring,
    subject: DatasetStore,
    state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl ProviderImpl {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: ClientStore::new(),
            issuer: IssuerStore::new(),
            server: ServerStore::new(),
            subject: DatasetStore::new(),
            state: Arc::new(Mutex::new(HashMap::new())),
            keyring: Keyring::new(),
        }
    }
}

impl Provider for ProviderImpl {}

impl Metadata for ProviderImpl {
    async fn client(&self, client_id: &str) -> Result<Client> {
        self.client.get(client_id)
    }

    async fn register(&self, client: &Client) -> Result<Client> {
        self.client.add(client)
    }

    async fn issuer(&self, credential_issuer: &str) -> Result<Issuer> {
        self.issuer.get(credential_issuer)
    }

    async fn server(&self, issuer: &str) -> Result<Server> {
        self.server.get(issuer)
    }
}

impl Subject for ProviderImpl {
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

impl StateStore for ProviderImpl {
    async fn put(&self, key: &str, state: impl Serialize, _dt: DateTime<Utc>) -> Result<()> {
        let state = serde_json::to_vec(&state)?;
        self.state.lock().expect("should lock").insert(key.to_string(), state);
        Ok(())
    }

    async fn get<T: for<'a> Deserialize<'a>>(&self, key: &str) -> Result<T> {
        let Some(state) = self.state.lock().expect("should lock").get(key).cloned() else {
            return Err(anyhow!("state not found for key: {key}"));
        };
        Ok(serde_json::from_slice(&state)?)
    }

    async fn purge(&self, key: &str) -> Result<()> {
        self.state.lock().expect("should lock").remove(key);
        Ok(())
    }
}

impl DidResolver for ProviderImpl {
    async fn resolve(&self, url: &str) -> anyhow::Result<Document> {
        self.keyring.resolve(url).await
    }
}

impl Signer for ProviderImpl {
    async fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        self.keyring.try_sign(msg).await
    }

    async fn verifying_key(&self) -> Result<Vec<u8>> {
        self.keyring.verifying_key().await
    }

    fn algorithm(&self) -> Algorithm {
        self.keyring.algorithm()
    }

    async fn verification_method(&self) -> Result<String> {
        self.keyring.verification_method().await
    }
}

impl Status for ProviderImpl {}
