use anyhow::Result;
use base64ct::{Base64UrlUnpadded, Encoding};
use iota_stronghold::procedures::{
    Ed25519Sign, GenerateKey, KeyType, PublicKey, StrongholdProcedure,
};
use iota_stronghold::{Client, KeyProvider, Location, Resource};

const CLIENT: &[u8] = b"signing_client";
const VAULT: &[u8] = b"signing_key_vault";
const SIGNING_KEY: &[u8] = b"signing_key";

pub struct Stronghold {
    key_location: Location,
    client: Client,
}

impl Stronghold {
    /// Create new Stronghold instance.
    /// The method will attempt to load a Stronghold snapshot from the given path,
    /// or create a new one if it does not exist.
    ///
    /// When creating a new snapshot, a signing key will be generated and saved to
    /// the vault.
    ///
    /// The snapshot is encrypted using the password provided.
    pub fn new(password: Vec<u8>, snapshot: Option<Vec<u8>>) -> Result<Self> {
        let stronghold = iota_stronghold::Stronghold::default();

        let keyprovider = KeyProvider::try_from(password)?;
        let key_location = Location::generic(VAULT, SIGNING_KEY);
        let snapshot_path = iota_stronghold::SnapshotPath::from_path(
            "/Users/andrewweston/Library/Application Support/io.credibil.wallet/stronghold.bin",
        );

        let client = {
            if let Some(snap_bytes) = snapshot {
                let source = Resource::Memory(snap_bytes);
                stronghold.load_client_from_snapshot(CLIENT, &keyprovider, &source)?
            } else {
                let client = stronghold.create_client(CLIENT)?;

                // generate signing key
                let proc = StrongholdProcedure::GenerateKey(GenerateKey {
                    ty: KeyType::Ed25519,
                    output: key_location.clone(),
                });
                let _ = client.execute_procedure(proc)?;

                // save snapshot (+ client and vault)
                let target = Resource::File(snapshot_path);
                let output = stronghold.commit_with_keyprovider(&target, &keyprovider)?;
                client
            }
        };

        Ok(Self { key_location, client })
    }

    /// Sign message using the snapshot's signing key.
    pub(super) fn sign(&self, msg: Vec<u8>) -> Result<Vec<u8>> {
        let proc = StrongholdProcedure::Ed25519Sign(Ed25519Sign {
            msg,
            private_key: self.key_location.clone(),
        });
        let output = self.client.execute_procedure(proc)?;
        Ok(output.into())
    }

    /// Get the signing key's public key from the snapshot.
    pub(super) fn verifiction(&self) -> Result<String> {
        // get public key
        let proc = StrongholdProcedure::PublicKey(PublicKey {
            ty: KeyType::Ed25519,
            private_key: self.key_location.clone(),
        });
        let output = self.client.execute_procedure(proc)?;

        // convert to did:jwk
        let x_bytes: Vec<u8> = output.into();

        let jwk = serde_json::json!({
            "kty": "OKP",
            "crv": "X25519",
            "use": "enc",
            "x": Base64UrlUnpadded::encode_string(&x_bytes),
        });
        let jwk_str = jwk.to_string();
        let jwk_b64 = Base64UrlUnpadded::encode_string(jwk_str.as_bytes());

        Ok(format!("did:jwk:{jwk_b64}#0"))
    }
}
