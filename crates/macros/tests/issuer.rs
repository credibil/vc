use vercre_macros::create_offer_request;
use vercre_openid::issuer::{CreateOfferRequest, SendType};

#[test]
fn create_offer() {
    const CREDENTIAL_ISSUER: &str = "http://vercre.io";
    let subject_id = "normal_user";

    let request = create_offer_request!({
        "credential_issuer": CREDENTIAL_ISSUER,
        "credential_configuration_ids": ["EmployeeID_JWT"],
        "subject_id": subject_id,
        "pre-authorize": true,
        "tx_code_required": true,
        "send_type": SendType::ByVal,
    });

    println!("{:?}", request);
}
