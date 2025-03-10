//! Pre-Authorized Code Flow Tests

mod utils;

use std::collections::HashMap;
use std::sync::LazyLock;

use credibil_infosec::jose::JwsBuilder;
use credibil_vc::OneMany;
use credibil_vc::oid4vci::endpoint;
use credibil_vc::oid4vci::proof::{self, Payload, Type, Verify};
use credibil_vc::oid4vci::types::{
    AuthorizationDetail, CreateOfferRequest, Credential, CredentialOfferRequest, CredentialRequest,
    NonceRequest, NotificationEvent, NotificationRequest, ProofClaims, ResponseType,
    TokenGrantType, TokenRequest,
};
use http::header::{AUTHORIZATION, HeaderMap};
use insta::assert_yaml_snapshot as assert_snapshot;
use utils::issuer::{CREDENTIAL_ISSUER as ALICE_ISSUER, NORMAL_USER, ProviderImpl};
use utils::wallet::{self, Keyring};

static BOB_KEYRING: LazyLock<Keyring> = LazyLock::new(wallet::keyring);

// Should return a credential when using the pre-authorized code flow and the
// credential offer to the Wallet is made by value.
#[tokio::test]
async fn offer_val() {
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob
    // --------------------------------------------------
    let request = CreateOfferRequest::builder()
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

    let request = TokenRequest::builder()
        // .client_id(BOB_CLIENT)
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
        .payload(
            ProofClaims::new()
                // .client_id(BOB_CLIENT)
                .credential_issuer(ALICE_ISSUER)
                .nonce(nonce.c_nonce),
        )
        .add_signer(&*BOB_KEYRING)
        .build()
        .await
        .expect("builds JWS");
    let jwt = jws.encode().expect("encodes JWS");

    // --------------------------------------------------
    // Bob requests a credential
    // --------------------------------------------------
    let details = &token.authorization_details.expect("should have authorization details");
    let request = CredentialRequest::builder()
        .credential_identifier(&details[0].credential_identifiers[0])
        .with_proof(jwt)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.access_token.parse().unwrap());
    let request = endpoint::Request {
        body: request,
        headers: Some(headers),
    };

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
        ".credentialSubject" => insta::sorted_redaction(),
        ".credentialSubject.id" => "[id]"
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
    let request = CreateOfferRequest::builder()
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

// Should return two credential datasets for a single credential
// configuration id.
#[tokio::test]
async fn two_datasets() {
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob
    // --------------------------------------------------
    let request = CreateOfferRequest::builder()
        .subject_id(NORMAL_USER)
        .with_credential("Developer_JWT")
        .build();
    let response =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should create offer");

    // --------------------------------------------------
    // Bob receives the offer and requests a token
    // --------------------------------------------------
    let offer = response.offer_type.as_object().expect("should have offer").clone();
    let grants = offer.grants.expect("should have grant");
    let pre_auth_grant = grants.pre_authorized_code.expect("should have pre-authorized code grant");

    let request = TokenRequest::builder()
        .grant_type(TokenGrantType::PreAuthorizedCode {
            pre_authorized_code: pre_auth_grant.pre_authorized_code,
            tx_code: response.tx_code.clone(),
        })
        .build();
    let token =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should return token");

    // --------------------------------------------------
    // Bob receives the token and prepares 2 credential requests
    // --------------------------------------------------
    let details = &token.authorization_details.expect("should have authorization details");
    let expected = HashMap::from([
        ("OpenSourceDeveloper", vec!["A. Mazing", "Hacker"]),
        ("PHLDeveloper", vec!["A. Developer", "Lead"]),
    ]);

    for identifier in &details[0].credential_identifiers {
        let nonce = endpoint::handle(ALICE_ISSUER, NonceRequest, &provider)
            .await
            .expect("should return nonce");

        // proof of possession of key material
        let jws = JwsBuilder::new()
            .jwt_type(Type::Openid4VciProofJwt)
            .payload(ProofClaims::new().credential_issuer(ALICE_ISSUER).nonce(nonce.c_nonce))
            .add_signer(&*BOB_KEYRING)
            .build()
            .await
            .expect("builds JWS");
        let jwt = jws.encode().expect("encodes JWS");

        // --------------------------------------------------
        // Bob requests a credential
        // --------------------------------------------------
        let request =
            CredentialRequest::builder().credential_identifier(identifier).with_proof(jwt).build();

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, token.access_token.parse().unwrap());
        let request = endpoint::Request {
            body: request,
            headers: Some(headers),
        };

        let response = endpoint::handle(ALICE_ISSUER, request, &provider)
            .await
            .expect("should return credential");

        // --------------------------------------------------
        // Bob extracts and verifies the received credential
        // --------------------------------------------------
        let ResponseType::Credentials { credentials, .. } = &response.response else {
            panic!("expected single credential");
        };
        let Credential { credential } = credentials.first().expect("should have credential");

        // verify the credential proof
        let Ok(Payload::Vc { vc, .. }) =
            proof::verify(Verify::Vc(credential), provider.clone()).await
        else {
            panic!("should be valid VC");
        };

        // validate the credential subject
        let OneMany::One(subject) = vc.credential_subject else {
            panic!("should have single subject");
        };
        assert_eq!(subject.claims["name"], expected[identifier.as_str()][0]);
        assert_eq!(subject.claims["role"], expected[identifier.as_str()][1]);
    }
}

// Should return a single credential when two are offered and only one is
// requested in the token request.
#[tokio::test]
async fn reduce_credentials() {
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob with 2 credentials
    // --------------------------------------------------
    let request = CreateOfferRequest::builder()
        .subject_id(NORMAL_USER)
        .with_credential("Developer_JWT")
        .with_credential("EmployeeID_JWT")
        .build();
    let response =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should create offer");

    let offer = response.offer_type.as_object().expect("should have offer");
    assert_eq!(offer.credential_configuration_ids.len(), 2);

    // --------------------------------------------------
    // Bob receives the offer and requests a token for 1 of the offered
    // credentials
    // --------------------------------------------------
    let offer = response.offer_type.as_object().expect("should have offer").clone();
    let grants = offer.grants.expect("should have grant");
    let pre_auth_grant = grants.pre_authorized_code.expect("should have pre-authorized code grant");

    let request = TokenRequest::builder()
        // .client_id(BOB_CLIENT)
        .grant_type(TokenGrantType::PreAuthorizedCode {
            pre_authorized_code: pre_auth_grant.pre_authorized_code,
            tx_code: response.tx_code.clone(),
        })
        .with_authorization_detail(
            AuthorizationDetail::builder().configuration_id("EmployeeID_JWT").build(),
        )
        .build();
    let token =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should return token");

    // --------------------------------------------------
    // Bob receives the token and prepares a credential request
    // --------------------------------------------------
    let details = &token.authorization_details.expect("should have authorization details");
    assert_eq!(details[0].credential_identifiers.len(), 1);

    let identifier = &details[0].credential_identifiers[0];

    let nonce =
        endpoint::handle(ALICE_ISSUER, NonceRequest, &provider).await.expect("should return nonce");

    // proof of possession of key material
    let jws = JwsBuilder::new()
        .jwt_type(Type::Openid4VciProofJwt)
        .payload(ProofClaims::new().credential_issuer(ALICE_ISSUER).nonce(nonce.c_nonce))
        .add_signer(&*BOB_KEYRING)
        .build()
        .await
        .expect("builds JWS");
    let jwt = jws.encode().expect("encodes JWS");

    // --------------------------------------------------
    // Bob requests the credential
    // --------------------------------------------------
    let request =
        CredentialRequest::builder().credential_identifier(identifier).with_proof(jwt).build();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.access_token.parse().unwrap());
    let request = endpoint::Request {
        body: request,
        headers: Some(headers),
    };

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

    // validate the credential subject
    let OneMany::One(subject) = vc.credential_subject else {
        panic!("should have single subject");
    };
    assert_eq!(subject.claims["given_name"], "Normal");
    assert_eq!(subject.claims["family_name"], "Person");
}

// Should return fewer claims when requested in token request.
#[tokio::test]
async fn reduce_claims() {
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob
    // --------------------------------------------------
    let request = CreateOfferRequest::builder()
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

    let request = TokenRequest::builder()
        .grant_type(TokenGrantType::PreAuthorizedCode {
            pre_authorized_code: pre_auth_grant.pre_authorized_code,
            tx_code: response.tx_code.clone(),
        })
        .with_authorization_detail(
            AuthorizationDetail::builder()
                .configuration_id("EmployeeID_JWT")
                .with_claim(&vec!["credentialSubject", "given_name"])
                .with_claim(&vec!["credentialSubject", "family_name"])
                .build(),
        )
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
        .payload(ProofClaims::new().credential_issuer(ALICE_ISSUER).nonce(nonce.c_nonce))
        .add_signer(&*BOB_KEYRING)
        .build()
        .await
        .expect("builds JWS");
    let jwt = jws.encode().expect("encodes JWS");

    // --------------------------------------------------
    // Bob requests a credential
    // --------------------------------------------------
    let details = &token.authorization_details.expect("should have authorization details");
    let request = CredentialRequest::builder()
        .credential_identifier(&details[0].credential_identifiers[0])
        .with_proof(jwt)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.access_token.parse().unwrap());
    let request = endpoint::Request {
        body: request,
        headers: Some(headers),
    };

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

    assert_snapshot!("reduce_claims", vc, {
        ".validFrom" => "[validFrom]",
        ".credentialSubject" => insta::sorted_redaction(),
        ".credentialSubject.id" => "[id]"
    });
}

// Should handle an acceptance notication from the wallet.
#[tokio::test]
async fn notify_accepted() {
    utils::init_tracer();
    let provider = ProviderImpl::new();

    // --------------------------------------------------
    // Alice creates a credential offer for Bob
    // --------------------------------------------------
    let request = CreateOfferRequest::builder()
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

    let request = TokenRequest::builder()
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
        .payload(ProofClaims::new().credential_issuer(ALICE_ISSUER).nonce(nonce.c_nonce))
        .add_signer(&*BOB_KEYRING)
        .build()
        .await
        .expect("builds JWS");
    let jwt = jws.encode().expect("encodes JWS");

    // --------------------------------------------------
    // Bob requests a credential
    // --------------------------------------------------
    let details = &token.authorization_details.expect("should have authorization details");
    let request = CredentialRequest::builder()
        .credential_identifier(&details[0].credential_identifiers[0])
        .with_proof(jwt)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.access_token.parse().unwrap());
    let request = endpoint::Request {
        body: request,
        headers: Some(headers),
    };

    let response =
        endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("should return credential");

    // --------------------------------------------------
    // Bob send a notication advising the credential was accepted
    // --------------------------------------------------
    let Some(notification_id) = response.notification_id else {
        panic!("should have notification id");
    };

    let request = NotificationRequest::builder()
        .notification_id(notification_id)
        .event(NotificationEvent::CredentialAccepted)
        .event_description("Credential accepted")
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, token.access_token.parse().unwrap());
    let request = endpoint::Request {
        body: request,
        headers: Some(headers),
    };

    endpoint::handle(ALICE_ISSUER, request, &provider).await.expect("response is ok");
}
