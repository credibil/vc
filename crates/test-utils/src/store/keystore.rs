use anyhow::anyhow;
use base64ct::{Base64UrlUnpadded, Encoding};
use ed25519_dalek::{SecretKey, Signer, SigningKey};
use vercre_datasec::jose::jwa::Algorithm;
use vercre_datasec::Document;
use vercre_openid::provider::Result;

#[derive(Default, Clone, Debug)]
pub struct IssuerKeystore;

const ISSUER_DID: &str = "did:web:demo.credibil.io";
const ISSUER_VERIFY_KEY: &str = "key-0";
const ISSUER_SECRET: &str = "cCxmHfFfIJvP74oNKjAuRC3zYoDMo0pFsAs19yKMowY";

impl IssuerKeystore {
    pub fn algorithm(&self) -> Algorithm {
        Algorithm::EdDSA
    }

    pub fn verification_method(&self) -> String {
        format!("{ISSUER_DID}#{ISSUER_VERIFY_KEY}")
    }

    pub fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        // let decoded = Base64UrlUnpadded::decode_vec(SECRET_KEY)?;
        // let signing_key: SigningKey<Secp256k1> = SigningKey::from_slice(&decoded)?;
        // Ok(signing_key.sign(msg).to_vec())

        let decoded = Base64UrlUnpadded::decode_vec(ISSUER_SECRET)?;
        let secret_key: SecretKey =
            decoded.try_into().map_err(|_| anyhow!("Invalid secret key"))?;
        let signing_key: SigningKey = SigningKey::from_bytes(&secret_key);
        Ok(signing_key.sign(msg).to_bytes().to_vec())
    }
}

#[derive(Default, Clone, Debug)]
pub struct VerifierKeystore;

const VERIFIER_DID: &str = "did:web:demo.credibil.io";
const VERIFIER_VERIFY_KEY: &str = "key-0";
const VERIFIER_SECRET: &str = "cCxmHfFfIJvP74oNKjAuRC3zYoDMo0pFsAs19yKMowY";

impl VerifierKeystore {
    pub fn algorithm(&self) -> Algorithm {
        Algorithm::EdDSA
    }

    pub fn verification_method(&self) -> String {
        format!("{VERIFIER_DID}#{VERIFIER_VERIFY_KEY}")
    }

    pub fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let decoded = Base64UrlUnpadded::decode_vec(VERIFIER_SECRET)?;
        let secret_key: SecretKey =
            decoded.try_into().map_err(|_| anyhow!("Invalid secret key"))?;
        let signing_key: SigningKey = SigningKey::from_bytes(&secret_key);
        Ok(signing_key.sign(msg).to_bytes().to_vec())
    }
}

const HOLDER_DID: &str = "did:key:z6Mkj8Jr1rg3YjVWWhg7ahEYJibqhjBgZt1pDCbT4Lv7D4HX";
const HOLDER_VERIFY_KEY: &str = "z6Mkj8Jr1rg3YjVWWhg7ahEYJibqhjBgZt1pDCbT4Lv7D4HX";
const HOLDER_SECRET: &str = "8rmFFiUcTjjrL5mgBzWykaH39D64VD0mbDHwILvsu30";

#[derive(Default, Clone, Debug)]
pub struct HolderKeystore;

impl HolderKeystore {
    pub fn algorithm() -> Algorithm {
        Algorithm::EdDSA
    }

    pub fn verification_method() -> String {
        format!("{HOLDER_DID}#{HOLDER_VERIFY_KEY}")
    }

    pub fn try_sign(msg: &[u8]) -> Result<Vec<u8>> {
        let decoded = Base64UrlUnpadded::decode_vec(HOLDER_SECRET)?;
        let bytes: [u8; 32] = decoded.as_slice().try_into().expect("should convert ");
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&bytes);
        let signature: ed25519_dalek::Signature = signing_key.sign(msg);
        Ok(signature.to_vec())
    }
}

/// Dereference DID URL to public key. For example,  did:web:demo.credibil.io#key-0.
///
/// did:web:demo.credibil.io -> did:web:demo.credibil.io/.well-known/did.json
/// did:web:demo.credibil.io:entity:supplier -> did:web:demo.credibil.io/entity/supplier/did.json
pub async fn get_did(_did_url: &str) -> Result<Document> {
    serde_json::from_slice(include_bytes!("did.json"))
        .map_err(|e| anyhow!("issue deserializing document: {e}"))
}
