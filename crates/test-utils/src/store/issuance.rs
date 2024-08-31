use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use serde::Deserialize;
use serde_json::{Map, Value};
use uuid::Uuid;
use vercre_openid::issuer::{Client, Dataset, Issuer, Server};
use vercre_openid::provider::Result;

// pub const NORMAL_USER: &str = "normal_user";
// pub const PENDING_USER: &str = "pending_user";
// pub const CLIENT_ID: &str = "96bfb9cb-0513-7d64-5532-bed74c48f9ab";

#[derive(Default, Clone, Debug)]
pub struct IssuerStore {
    issuers: HashMap<String, Issuer>,
}

impl IssuerStore {
    pub fn new() -> Self {
        let json = include_bytes!("issuer.json");
        let issuer: Issuer = serde_json::from_slice(json).expect("should serialize");

        Self {
            issuers: HashMap::from([
                ("http://localhost:8080".to_string(), issuer.clone()),
                (issuer.credential_issuer.clone(), issuer),
            ]),
        }
    }

    pub fn get(&self, issuer_id: &str) -> Result<Issuer> {
        let Some(issuer) = self.issuers.get(issuer_id) else {
            return Err(anyhow!("issuer not found"));
        };
        Ok(issuer.clone())
    }
}

#[derive(Default, Clone, Debug)]
pub struct ServerStore {
    servers: HashMap<String, Server>,
}

impl ServerStore {
    pub fn new() -> Self {
        let json = include_bytes!("server.json");
        let server: Server = serde_json::from_slice(json).expect("should serialize");

        Self {
            servers: HashMap::from([
                ("http://localhost:8080".to_string(), server.clone()),
                (server.oauth.issuer.clone(), server),
            ]),
        }
    }

    pub fn get(&self, server_id: &str) -> Result<Server> {
        let Some(server) = self.servers.get(server_id) else {
            return Err(anyhow!("issuer not found"));
        };
        Ok(server.clone())
    }
}

#[derive(Default, Clone, Debug)]
pub struct ClientStore {
    clients: Arc<Mutex<HashMap<String, Client>>>,
}

impl ClientStore {
    pub fn new() -> Self {
        let json = include_bytes!("client.json");
        let client: Client = serde_json::from_slice(json).expect("should serialize");

        // Local verifier client for use when running end to end tests
        let mut local = client.clone();
        local.oauth.client_id = "http://localhost:8080".into();

        Self {
            clients: Arc::new(Mutex::new(HashMap::from([
                (client.oauth.client_id.clone(), client),
                (local.oauth.client_id.clone(), local),
            ]))),
        }
    }

    pub fn get(&self, client_id: &str) -> Result<Client> {
        let Some(client) = self.clients.lock().expect("should lock").get(client_id).cloned() else {
            return Err(anyhow!("client not found for client_id: {client_id}"));
        };
        Ok(client)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn add(&self, client: &Client) -> Result<Client> {
        let mut client = client.clone();
        client.oauth.client_id = Uuid::new_v4().to_string();

        self.clients
            .lock()
            .expect("should lock")
            .insert(client.oauth.client_id.to_string(), client.clone());

        Ok(client)
    }
}

#[derive(Default, Clone, Debug, Deserialize)]
struct Credential {
    configuration_id: String,
    claims: Map<String, Value>,
    pending: bool,
}

#[derive(Default, Clone, Debug)]
pub struct DatasetStore {
    datasets: Arc<Mutex<HashMap<String, HashMap<String, Credential>>>>,
}

impl DatasetStore {
    pub fn new() -> Self {
        let json = include_bytes!("datasets.json");
        let datasets: HashMap<String, HashMap<String, Credential>> =
            serde_json::from_slice(json).expect("should serialize");

        Self {
            datasets: Arc::new(Mutex::new(datasets)),
        }
    }

    pub fn authorize(
        &self, subject_id: &str, credential_configuration_id: &str,
    ) -> Result<Option<Vec<String>>> {
        // get subject's datasets
        let subjects = self.datasets.lock().expect("should lock");
        let Some(datasets) = subjects.get(subject_id) else {
            return Err(anyhow!("no matching subject_id"));
        };

        // find identifier(s) for dataset(s) belonging to the credential
        // identified by `credential_configuration_id`
        let mut identifiers = vec![];
        for (k, v) in datasets.iter() {
            if v.configuration_id != credential_configuration_id {
                continue;
            }
            identifiers.push(k.clone());
        }

        if identifiers.is_empty() {
            Ok(None)
        } else {
            Ok(Some(identifiers))
        }
    }

    pub fn dataset(&self, subject_id: &str, credential_identifier: &str) -> Result<Dataset> {
        // get claims for the given `subject_id` and `credential_identifier`
        let mut subject =
            self.datasets.lock().expect("should lock").get(subject_id).unwrap().clone();
        let mut credential = subject.get(credential_identifier).unwrap().clone();

        // update subject's pending state to make Deferred Issuance work
        let pending = credential.pending;
        credential.pending = false;
        subject.insert(credential_identifier.to_string(), credential.clone());
        self.datasets.lock().expect("should lock").insert(subject_id.to_string(), subject);

        Ok(Dataset {
            claims: credential.claims,
            pending,
        })
    }
}
