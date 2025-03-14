pub mod keystore;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use credibil_did::{DidResolver, Document};
use credibil_infosec::{self, Algorithm, PublicKey, Receiver, SharedSecret, Signer};
use credibil_vc::oid4vp::provider::{Metadata, Provider, StateStore};
use credibil_vc::oid4vp::types::{Verifier, Wallet};
pub use keystore::Keystore;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

pub const VERIFIER_ID: &str = "http://localhost:8080";

#[derive(Default, Clone, Debug)]
pub struct ProviderImpl {
    verifiers: Arc<Mutex<HashMap<String, Verifier>>>,
    state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl ProviderImpl {
    #[must_use]
    pub fn new() -> Self {
        let json = include_bytes!("../data/verifier.json");
        let verifier: Verifier = serde_json::from_slice(json).expect("should serialize");

        Self {
            verifiers: Arc::new(Mutex::new(HashMap::from([(
                verifier.oauth.client_id.clone(),
                verifier,
            )]))),
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Provider for ProviderImpl {}

impl Metadata for ProviderImpl {
    async fn verifier(&self, verifier_id: &str) -> Result<Verifier> {
        let Some(verifier) = self.verifiers.lock().expect("should lock").get(verifier_id).cloned()
        else {
            return Err(anyhow!("verifier not found for verifier_id: {verifier_id}"));
        };
        Ok(verifier)
    }

    async fn register(&self, verifier: &Verifier) -> Result<Verifier> {
        let mut verifier = verifier.clone();
        verifier.oauth.client_id = Uuid::new_v4().to_string();

        self.verifiers
            .lock()
            .expect("should lock")
            .insert(verifier.oauth.client_id.to_string(), verifier.clone());

        Ok(verifier)
    }

    async fn wallet(&self, _wallet_id: &str) -> Result<Wallet> {
        unimplemented!("WalletMetadata")
    }
}

impl StateStore for ProviderImpl {
    async fn put(&self, key: &str, state: impl Serialize, _dt: DateTime<Utc>) -> Result<()> {
        let state = serde_json::to_vec(&state)?;
        self.state.lock().expect("should lock").insert(key.to_string(), state);
        Ok(())
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
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
    async fn resolve(&self, _url: &str) -> anyhow::Result<Document> {
        serde_json::from_slice(include_bytes!("../data/did-web.json"))
            .map_err(|e| anyhow!("issue deserializing document: {e}"))
    }
}

impl Signer for ProviderImpl {
    async fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        Keystore::try_sign(msg)
    }

    async fn verifying_key(&self) -> Result<Vec<u8>> {
        Keystore::public_key()
    }

    fn algorithm(&self) -> Algorithm {
        Keystore::algorithm()
    }

    async fn verification_method(&self) -> Result<String> {
        Ok(Keystore::verification_method())
    }
}

impl Receiver for ProviderImpl {
    fn key_id(&self) -> String {
        todo!()
    }

    async fn shared_secret(&self, _sender_public: PublicKey) -> Result<SharedSecret> {
        todo!()
    }
}
