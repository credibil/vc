use chrono::{DateTime, Utc};
use datasec::jose::jwk::PublicKeyJwk;
use datasec::{Algorithm, DataSec, Decryptor, Encryptor, Signer, Verifier};
use openid::issuer::{
    Claims, Client, ClientMetadata, Issuer, IssuerMetadata, Result, Server, ServerMetadata,
    StateManager, Subject,
};

use crate::store::keystore::IssuerKeystore;
use crate::store::{issuance, state};

pub const CREDENTIAL_ISSUER: &str = "http://vercre.io";
pub const CLIENT_ID: &str = "96bfb9cb-0513-7d64-5532-bed74c48f9ab";
pub const NORMAL_USER: &str = "normal_user";
pub const PENDING_USER: &str = "pending_user";

#[derive(Default, Clone, Debug)]
pub struct Provider {
    pub client: issuance::ClientStore,
    pub issuer: issuance::IssuerStore,
    pub server: issuance::ServerStore,
    pub subject: issuance::SubjectStore,
    pub state: state::Store,
}

impl Provider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: issuance::ClientStore::new(),
            issuer: issuance::IssuerStore::new(),
            server: issuance::ServerStore::new(),
            subject: issuance::SubjectStore::new(),
            state: state::Store::new(),
        }
    }
}

impl openid::issuer::Provider for Provider {}

impl ClientMetadata for Provider {
    async fn metadata(&self, client_id: &str) -> Result<Client> {
        self.client.get(client_id)
    }

    async fn register(&self, client: &Client) -> Result<Client> {
        self.client.add(client)
    }
}

impl IssuerMetadata for Provider {
    async fn metadata(&self, issuer_id: &str) -> Result<Issuer> {
        self.issuer.get(issuer_id)
    }
}

impl ServerMetadata for Provider {
    async fn metadata(&self, server_id: &str) -> Result<Server> {
        self.server.get(server_id)
    }
}

impl Subject for Provider {
    /// Authorize issuance of the specified credential for the holder.
    async fn authorize(&self, holder_subject: &str, credential_identifier: &str) -> Result<bool> {
        self.subject.authorize(holder_subject, credential_identifier)
    }

    async fn claims(&self, holder_subject: &str, credential_identifier: &str) -> Result<Claims> {
        self.subject.claims(holder_subject, credential_identifier)
    }
}

impl StateManager for Provider {
    async fn put(&self, key: &str, state: Vec<u8>, dt: DateTime<Utc>) -> Result<()> {
        self.state.put(key, state, dt)
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        self.state.get(key)
    }

    async fn purge(&self, key: &str) -> Result<()> {
        self.state.purge(key)
    }
}

impl DataSec for Provider {
    fn signer(&self, _identifier: &str) -> impl Signer {
        self.clone()
    }

    fn verifier(&self, _identifier: &str) -> impl Verifier {
        self.clone()
    }

    fn encryptor(&self, _identifier: &str) -> impl Encryptor {
        self.clone()
    }

    fn decryptor(&self, _identifier: &str) -> impl Decryptor {
        self.clone()
    }
}

impl Signer for Provider {
    fn algorithm(&self) -> Algorithm {
        IssuerKeystore::algorithm()
    }

    fn verification_method(&self) -> String {
        IssuerKeystore::verification_method()
    }

    async fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        IssuerKeystore::try_sign(msg)
    }
}

impl Verifier for Provider {
    async fn deref_jwk(&self, did_url: &str) -> Result<PublicKeyJwk> {
        crate::store::keystore::deref_jwk(did_url).await
    }
}

impl Encryptor for Provider {
    async fn encrypt(&self, _plaintext: &[u8], _recipient_public_key: &[u8]) -> Result<Vec<u8>> {
        // crate::store::keystore::encrypt(plaintext, recipient_public_key)
        todo!()
    }

    fn public_key(&self) -> Vec<u8> {
        // IssuerKeystore::public_key()
        todo!()
    }
}

impl Decryptor for Provider {
    async fn decrypt(&self, _ciphertext: &[u8], _sender_public_key: &[u8]) -> Result<Vec<u8>> {
        // IssuerKeystore::decrypt(ciphertext)
        todo!()
    }
}
