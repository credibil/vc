use anyhow::{anyhow, Error, Result};
use futures::StreamExt;
use tauri::async_runtime::{block_on, spawn};
use tauri::Manager;
use vercre_wallet::signer::{SignerRequest, SignerResponse};

use crate::iroh::{Doc, DocType};
use crate::stronghold::Stronghold;
use crate::{error, IrohState};

const KEY_VAULT: &str = "docaaacaopj7u7mkmrbxv536p2j4ihk3t3qn36oycl27po2orshfl2srd3bafk62aofuwwwu5zb5ocvzj5v3rtqt6siglyuhoxhqtu4fxravvoteajcnb2hi4dthixs65ltmuys2mjomrsxe4bonfzg62bonzsxi53pojvs4lydaac2cyt22erablaraaa5ciqbfiaqj7ya6cbpuaaaaaaaaaaaahjce";

// initialise the Stronghold key store
pub fn init(handle: &tauri::AppHandle) -> Result<()> {
    // FIXME: get passphrase from user and salt from file(?)
    let passphrase = b"pass-phrase";
    let salt = b"randomsalt";
    let password = argon2::hash_raw(passphrase, salt, &argon2::Config::default())?;

    let doc = block_on(async {
        let state = handle.state::<IrohState>();
        let doc: Doc = state.node.lock().await.join_doc(DocType::KeyVault, KEY_VAULT).await?;

        let mut stream = doc.updates().await;
        spawn(async move {
            while let Some(_) = stream.next().await {
                println!("should process event");
            }
        });

        Ok::<Doc, Error>(doc)
    })?;

    // open/initialize Stronghold snapshot
    let values = block_on(async { doc.entries().await.expect("should find an entry") });
    if values.len() > 0 {
        // TODO: move this to iroh module
        let snapshot = values.get(0).expect("should have a value").clone();
        let stronghold = Stronghold::new(password, Some(snapshot))?;
        handle.manage(stronghold);
    }

    Ok(())
}

pub fn request<R>(
    op: &SignerRequest, app_handle: &tauri::AppHandle<R>,
) -> Result<SignerResponse, error::Error>
where
    R: tauri::Runtime,
{
    let stronghold = app_handle.state::<Stronghold>();

    match op {
        SignerRequest::Sign(msg) => {
            let signed = stronghold.sign(msg.clone()).unwrap();
            Ok(SignerResponse::Signature(signed))
        }
        SignerRequest::Verification => {
            // FIXME: implement
            let alg = String::from("EdDSA"); // String::from("ES256K");
            let Ok(kid) = stronghold.verifiction() else {
                return Err(error::Error::Other(anyhow!("verification failed")));
            };
            Ok(SignerResponse::Verification { alg, kid })
        }
    }
}

#[cfg(test)]
mod test {
    use assert_let_bind::assert_let;
    use lazy_static::lazy_static;
    use serde_json::json;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    use super::*;

    #[tokio::test]
    async fn sign() {
        // set up store
        let app = create_app(mock_builder());

        let msg = String::from("hello world");
        let req = SignerRequest::Sign(msg.into_bytes());
        let resp = request(&req, app.app_handle()).expect("should be ok");

        // // check counts match
        assert_let!(SignerResponse::Signature(res), resp);
        println!("res: {:?}", res);
        // let vals = serde_json::from_slice::<Vec<Value>>(&res).expect("should deserialize");
        // assert_eq!(count, vals.len());
    }

    fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
        builder
            // .plugin(tauri_plugin_store::Builder::<R>::default().build())
            .build(mock_context(noop_assets()))
            .expect("failed to build app")
    }

    lazy_static! {
        static ref ENTRIES: serde_json::Value = json!({
                "id": "https://credibil.io/credentials/3732",
        });
    }
}
