//! Pre-Authorized Code Flow

mod utils;

use chrono::Utc;
use credibil_infosec::jose::JwsBuilder;
use credibil_vc::oid4vci::client::{
    CreateOfferRequestBuilder, CredentialRequestBuilder, TokenRequestBuilder,
};
use credibil_vc::oid4vci::endpoint;
use credibil_vc::oid4vci::proof::{self, Payload, Type, Verify};
use credibil_vc::oid4vci::types::{
    Credential, CredentialOfferRequest, NonceRequest, ProofClaims, ResponseType, TokenGrantType,
};
use insta::assert_yaml_snapshot as assert_snapshot;
use test_issuer::{
    CLIENT_ID as BOB_CLIENT, CREDENTIAL_ISSUER as ALICE_ISSUER, NORMAL_USER, ProviderImpl,
};

// Should return a credential when using the pre-authorized code flow and the
// credential offer to the Wallet is made by value.
#[tokio::test]
async fn offer_val() {
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob
    // --------------------------------------------------
    let request = CreateOfferRequestBuilder::new()
        .subject_id(NORMAL_USER)
        .with_credential("EmployeeID_JWT")
        .build();
    let response =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should create offer");

    // --------------------------------------------------
    // Bob receives the offer and requests a token
    // --------------------------------------------------
    let offer = response.offer_type.as_object().expect("should have offer").clone();
    let grants = offer.grants.expect("should have grant");
    let pre_auth_grant = grants.pre_authorized_code.expect("should have pre-authorized code grant");

    let request = TokenRequestBuilder::new()
        .client_id(BOB_CLIENT)
        .grant_type(TokenGrantType::PreAuthorizedCode {
            pre_authorized_code: pre_auth_grant.pre_authorized_code,
            tx_code: response.tx_code.clone(),
        })
        .build();
    let token =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should return token");

    // --------------------------------------------------
    // Bob receives the token and prepares a proof for a credential request
    // --------------------------------------------------
    let nonce =
        endpoint::handle(ALICE_ISSUER, NonceRequest, &provider).await.expect("should return nonce");

    // proof of possession of key material
    let jws = JwsBuilder::new()
        .jwt_type(Type::Openid4VciProofJwt)
        .payload(ProofClaims {
            iss: Some(BOB_CLIENT.to_string()),
            aud: ALICE_ISSUER.to_string(),
            iat: Utc::now().timestamp(),
            nonce: Some(nonce.c_nonce),
        })
        .add_signer(&test_holder::ProviderImpl)
        .build()
        .await
        .expect("builds JWS");
    let jwt = jws.encode().expect("encodes JWS");

    // --------------------------------------------------
    // Bob requests a credential
    // --------------------------------------------------
    let details = &token.authorization_details.expect("should have authorization details");
    let request = CredentialRequestBuilder::new()
        .credential_identifier(&details[0].credential_identifiers[0])
        .with_proof(jwt)
        .access_token(token.access_token)
        .build();
    let response =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should return credential");

    // --------------------------------------------------
    // Bob extracts and verifies the received credential
    // --------------------------------------------------
    let ResponseType::Credentials { credentials, .. } = &response.response else {
        panic!("expected single credential");
    };
    let Credential { credential } = credentials.first().expect("should have credential");

    // verify the credential proof
    let Ok(Payload::Vc { vc, .. }) = proof::verify(Verify::Vc(credential), provider.clone()).await
    else {
        panic!("should be valid VC");
    };

    assert_snapshot!("offer_val", vc, {
        ".validFrom" => "[validFrom]",
        ".credentialSubject" => insta::sorted_redaction()
    });
}

// Should return a credential when using the pre-authorized code flow and the
// credential offer to the Wallet is made by reference.
#[tokio::test]
async fn offer_ref() {
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob
    // --------------------------------------------------
    let request = CreateOfferRequestBuilder::new()
        .subject_id(NORMAL_USER)
        .with_credential("EmployeeID_JWT")
        .by_ref(true)
        .build();
    let create_offer =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should create offer");

    // --------------------------------------------------
    // Bob receives the offer URI and fetches the offer
    // --------------------------------------------------
    let uri = create_offer.offer_type.as_uri().expect("should have offer");
    let path = format!("{ALICE_ISSUER}/credential_offer/");
    let Some(id) = uri.strip_prefix(&path) else {
        panic!("should have prefix");
    };
    let request = CredentialOfferRequest { id: id.to_string() };
    let response =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should fetch offer");

    // validate offer
    let offer = response.credential_offer;
    assert_eq!(offer.credential_configuration_ids, vec!["EmployeeID_JWT".to_string()]);

    let grants = offer.grants.expect("should have grant");
    let pre_auth_grant = grants.pre_authorized_code.expect("should have pre-authorized code grant");
    assert_eq!(pre_auth_grant.pre_authorized_code.len(), 43);
}
