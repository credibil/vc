#![allow(dead_code)]

mod block_store;
#[path = "../../kms/mod.rs"]
mod kms;
mod store;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use blockstore::InMemoryBlockstore;
use credibil_did::{DidResolver, Document};
use credibil_infosec::Signer;
use credibil_infosec::jose::jwa::Algorithm;
use credibil_vc::BlockStore;
use credibil_vc::oid4vci::provider::Subject;
use credibil_vc::oid4vci::types::Dataset;
use credibil_vc::status::issuer::Status;
use futures::executor::block_on;
use kms::Keyring;
use store::DatasetStore;

pub const CREDENTIAL_ISSUER: &str = "http://credibil.io";
pub const NORMAL_USER: &str = "normal_user";
pub const PENDING_USER: &str = "pending_user";

#[derive(Clone, Debug)]
pub struct ProviderImpl {
    keyring: Keyring,
    subject: DatasetStore,
    state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    blockstore: Arc<InMemoryBlockstore<64>>,
}

impl ProviderImpl {
    #[must_use]
    pub fn new() -> Self {
        let provider = Self {
            subject: DatasetStore::new(),
            state: Arc::new(Mutex::new(HashMap::new())),
            keyring: Keyring::new(),
            blockstore: Arc::new(InMemoryBlockstore::<64>::new()),
        };

        block_on(async {
            // let localhost = "http://localhost:8080";
            let credibil = "http://credibil.io";

            // Issuer
            let issuer_data = include_bytes!("../data/issuer.json");
            // BlockStore::put(&provider, "owner", "ISSUER", localhost, issuer_data).await.unwrap();
            BlockStore::put(&provider, "owner", "ISSUER", credibil, issuer_data).await.unwrap();

            // Server
            let server_data = include_bytes!("../data/server.json");
            // BlockStore::put(&provider, "owner", "SERVER", localhost, server_data).await.unwrap();
            BlockStore::put(&provider, "owner", "SERVER", credibil, server_data).await.unwrap();

            // Client
            let client_data = include_bytes!("../data/client.json");
            let client_id = "96bfb9cb-0513-7d64-5532-bed74c48f9ab";
            BlockStore::put(&provider, "owner", "CLIENT", client_id, client_data).await.unwrap();
            // BlockStore::put(&provider, "owner", "CLIENT", localhost, client_data).await.unwrap();
        });

        provider
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
