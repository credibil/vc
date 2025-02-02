//! Tests for issuer-initiated pre-authorized issuance flow where the holder
//! decides to only accept a subset of the credentials on offer and a subset of
//! claims within the credential.
mod provider;

use std::collections::HashMap;

use insta::assert_yaml_snapshot;
use test_utils::issuer::{self, CLIENT_ID, CREDENTIAL_ISSUER, NORMAL_USER};
use vercre_holder::issuance::{
    AuthorizationSpec, IssuanceFlow, NotAccepted, PreAuthorized, WithOffer, WithoutToken,
};
use vercre_holder::jose::JwsBuilder;
use vercre_holder::provider::{Issuer, MetadataRequest};
use vercre_holder::Claim;
use vercre_issuer::{CredentialResponseType, OfferType, SendType};
use vercre_macros::create_offer_request;
use vercre_w3c_vc::proof::{Payload, Type, Verify};

use crate::provider as holder;

// Test end-to-end pre-authorized issuance flow (issuer-initiated), with
// acceptance of a subset of credentials on offer and a subset of claims within
// a credential.
#[tokio::test]
async fn preauth_narrow() {
    // Use the issuance service endpoint to create a sample offer that we can
    // use to start the flow. This is test set-up only - wallets do not ask an
    // issuer for an offer. Usually this code is internal to an issuer service.
    let request = create_offer_request!({
        "credential_issuer": CREDENTIAL_ISSUER,
        "credential_configuration_ids": ["EmployeeID_JWT", "Developer_JWT"],
        "subject_id": NORMAL_USER,
        "grant_types": ["urn:ietf:params:oauth:grant-type:pre-authorized_code"],
        "tx_code_required": false, // We will forgo the use of a PIN for this test.
        "send_type": SendType::ByVal,
    });

    let issuer_provider = issuer::Provider::new();
    let offer_resp = vercre_issuer::create_offer(issuer_provider.clone(), request)
        .await
        .expect("should get offer");
    let OfferType::Object(offer) = offer_resp.offer_type else {
        panic!("expected CredentialOfferType::Object");
    };

    let provider = holder::Provider::new(Some(issuer_provider), None);

    //--------------------------------------------------------------------------
    // Get issuer metadata to flow state.
    //--------------------------------------------------------------------------
    let metadata_request = MetadataRequest {
        credential_issuer: offer.credential_issuer.clone(),
        languages: None,
    };
    let issuer_metadata =
        provider.metadata(metadata_request).await.expect("should get issuer metadata");

    //--------------------------------------------------------------------------
    // Initiate flow state with the offer and metadata.
    //--------------------------------------------------------------------------

    let pre_auth_code_grant = offer.pre_authorized_code().expect("should get pre-authorized code");
    let state = IssuanceFlow::<WithOffer, PreAuthorized, NotAccepted, WithoutToken>::new(
        CLIENT_ID,
        NORMAL_USER,
        issuer_metadata.credential_issuer,
        offer,
        pre_auth_code_grant,
    );
    let offered = state.offered();

    //--------------------------------------------------------------------------
    // Present the offer to the holder for them to choose what to accept. (No
    // PIN required for this test.)
    //--------------------------------------------------------------------------
    insta::assert_yaml_snapshot!("offered", offered, {
        "." => insta::sorted_redaction(),
        ".**.credentialSubject" => insta::sorted_redaction(),
        ".**.credentialSubject.address" => insta::sorted_redaction(),
    });

    // Accept only the Developer credential on offer and only the proficiency
    // claim.
    let accept_spec = Some(vec![AuthorizationSpec {
        credential_configuration_id: "Developer_JWT".into(),
        claims: Some(HashMap::from([("proficiency".to_string(), Claim::default())])),
    }]);
    let state = state.accept(&accept_spec, None);

    //--------------------------------------------------------------------------
    // Request an access token from the issuer.
    //--------------------------------------------------------------------------
    let token_request = state.token_request();
    let token_response = provider.token(token_request).await.expect("should get token response");
    let mut state = state.token(token_response.clone());

    //--------------------------------------------------------------------------
    // Make credential requests.
    //--------------------------------------------------------------------------
    // For this test we are going to accept all credentials we have a token for
    // and as specified in the `accept` step above. (There is just one
    // credential to be retrieved in this case but we use a loop to demonstate
    // the pattern for multiple credentials.) We
    // are making the request by credential identifier.
    let Some(authorized) = &token_response.authorization_details else {
        panic!("no authorization details in token response");
    };
    let mut identifiers = vec![];
    for auth in authorized {
        for id in auth.credential_identifiers.iter() {
            identifiers.push(id.clone());
        }
    }
    let jws_claims = state.proof();
    let jws = JwsBuilder::new()
        .jwt_type(Type::Openid4VciProofJwt)
        .payload(jws_claims)
        .add_signer(&provider)
        .build()
        .await
        .expect("should build jws");
    let jwt = jws.encode().expect("should encode proof claims");
    let credential_requests = state.credential_requests(&identifiers, &jwt).clone();
    for request in credential_requests {
        let credential_response =
            provider.credential(request.1).await.expect("should get credentials");
        // A credential response could contain a single credential, multiple
        // credentials or a deferred transaction ID. Any credential issued also
        // needs its proof verified by using a DID resolver.
        match credential_response.response {
            CredentialResponseType::Credential(vc_kind) => {
                // Single credential in response.
                let Payload::Vc { vc, issued_at } =
                    vercre_w3c_vc::proof::verify(Verify::Vc(&vc_kind), provider.clone())
                        .await
                        .expect("should parse credential")
                else {
                    panic!("expected Payload::Vc");
                };
                state
                    .add_credential(&vc, &vc_kind, &issued_at, &request.0, None, None)
                    .expect("should add credential");
            }
            CredentialResponseType::Credentials(creds) => {
                // Multiple credentials in response.
                for vc_kind in creds {
                    let Payload::Vc { vc, issued_at } =
                        vercre_w3c_vc::proof::verify(Verify::Vc(&vc_kind), provider.clone())
                            .await
                            .expect("should parse credential")
                    else {
                        panic!("expected Payload::Vc");
                    };
                    state
                        .add_credential(&vc, &vc_kind, &issued_at, &request.0, None, None)
                        .expect("should add credential");
                }
            }
            CredentialResponseType::TransactionId(tx_id) => {
                // Deferred transaction ID.
                state.add_deferred(&tx_id, &request.0);
            }
        }
    }

    // The flow is complete and the credential on the issuance state could now
    // be saved to the wallet using the `CredentialStorer` provider trait. For
    // the test we check snapshot of the state's credentials.
    assert_yaml_snapshot!("credentials", state.credentials(), {
        "[].type" => insta::sorted_redaction(),
        "[].subject_claims[]" => insta::sorted_redaction(),
        "[].subject_claims[].claims" => insta::sorted_redaction(),
        "[].subject_claims[].claims.address" => insta::sorted_redaction(),
        "[].claim_definitions" => insta::sorted_redaction(),
        "[].claim_definitions.address" => insta::sorted_redaction(),
        "[].issued" => "[issued]",
        "[].issuance_date" => "[issuance_date]",
    });
}
