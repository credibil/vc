mod test_provider;

use std::sync::LazyLock;

use insta::assert_yaml_snapshot as assert_snapshot;
use providers::issuance::{CREDENTIAL_ISSUER, NORMAL_USER};
use providers::wallet::CLIENT_ID;
use test_provider::TestProvider;
use vercre_holder::issuance::OfferRequest;
use vercre_holder::Endpoint;
use vercre_issuer::create_offer::CreateOfferRequest;

static PROVIDER: LazyLock<TestProvider> = LazyLock::new(|| TestProvider::new());

fn sample_offer_request() -> CreateOfferRequest {
    CreateOfferRequest {
        credential_issuer: CREDENTIAL_ISSUER.into(),
        credential_configuration_ids: vec!["EmployeeID_JWT".into()],
        holder_id: Some(NORMAL_USER.into()),
        pre_authorize: true,
        tx_code_required: true,
        callback_id: Some("1234".into()),
        callback_id: Some("1234".into()),
    }
}

#[tokio::test]
async fn e2e_test() {
    // Use the issuance service endpoint to create a sample offer so that we can get a valid
    // pre-auhorized code.
    let offer = vercre_issuer::Endpoint::new(PROVIDER.clone())
        .create_offer(&sample_offer_request())
        .await
        .expect("should get offer");

    // Initiate the pre-authorized code flow
    let offer_req = OfferRequest {
        client_id: CLIENT_ID.into(),
        offer: offer.credential_offer.expect("should have offer"),
    };
    let issuance =
        Endpoint::new(PROVIDER.clone()).offer(&offer_req).await.expect("should process offer");
    assert_snapshot!("issuance", issuance, {
        ".id" => "[id]",
        ".offer" => insta::sorted_redaction(),
        ".offer.grants[\"urn:ietf:params:oauth:grant-type:pre-authorized_code\"][\"pre-authorized_code\"]" => "[pre-authorized_code]",
        ".offered.EmployeeID_JWT.credential_definition.credentialSubject" => insta::sorted_redaction(),
    });

    // Accept offer
}
