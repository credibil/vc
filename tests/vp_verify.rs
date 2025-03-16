//! Tests for the Verifier API

mod utils;

// use credibil_vc::oid4vp::endpoint;
// use credibil_vc::oid4vp::types::CreateRequestRequest;
// use insta::assert_yaml_snapshot as assert_snapshot;
// use serde_json::json;
use utils::verifier::ProviderImpl;
// use utils::wallet::{self, Keyring};

#[tokio::test]
async fn same_device() {
    utils::init_tracer();
    let _provider = ProviderImpl::new();

    // // create offer to 'send' to the app
    // let body = json!({
    //     "purpose": "To verify employment",
    //     "input_descriptors": [{
    //         "id": "employment",
    //         "constraints": {
    //             "fields": [{
    //                 "path":["$.type"],
    //                 "filter": {
    //                     "type": "string",
    //                     "const": "EmployeeIDCredential"
    //                 }
    //             }]
    //         }
    //     }],
    //     "device_flow": "SameDevice"
    // });

    // let mut request: CreateRequestRequest =
    //     serde_json::from_value::<CreateRequestRequest>(body).expect("should deserialize");
    // request.client_id = "http://localhost:8080".to_string();

    // let response = endpoint::handle("http://localhost:8080", request, &provider).await.expect("ok");

    // assert_eq!(response.request_uri, None);
    // assert_let!(Some(req_obj), &response.request_object);

    // assert!(req_obj.presentation_definition.is_object());

    // // compare response with saved state
    // let state_key = req_obj.state.as_ref().expect("has state");
    // let state = StateStore::get::<State>(&provider, state_key).await.expect("state exists");

    // assert_eq!(req_obj.nonce, state.request_object.nonce);
    // assert_snapshot!("sd-response", response, {
    //     ".request_object.presentation_definition"  => "[presentation_definition]",
    //     ".request_object.client_metadata" => "[client_metadata]",
    //     ".request_object.state" => "[state]",
    //     ".request_object.nonce" => "[nonce]",
    // });
}
