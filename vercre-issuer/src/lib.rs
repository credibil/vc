//! An API for the issuance of Verifiable Credentials based on the
//! [OpenID for Verifiable Credential Issuance] specification.
//!
//! # [OpenID for Verifiable Credential Issuance]
//!
//! This library implements an OAuth protected API for the issuance of Verifiable
//! Credentials as specified by [OpenID for Verifiable Credential Issuance].
//!
//! Verifiable Credentials are similar to identity assertions, like ID Tokens in
//! [OpenID Connect], in that they allow a Credential Issuer to assert End-User claims.
//! A Verifiable Credential follows a pre-defined schema (the Credential type) and MAY
//! be bound to a certain holder, e.g., through Cryptographic Holder Binding. Verifiable
//! Credentials can be securely presented for the End-User to the RP, without
//! involvement of the Credential Issuer.
//!
//! Access to this API is authorized using OAuth 2.0 [RFC6749],
//! i.e., Wallets use OAuth 2.0 to obtain authorization to receive Verifiable
//! Credentials. This way the issuance process can benefit from the proven security,
//! simplicity, and flexibility of OAuth 2.0, use existing OAuth 2.0 deployments, and
//! [OpenID Connect] OPs can be extended to become Credential Issuers.
//!
//! # Design
//!
//! **Endpoints**
//!
//! The library is architected around the [OpenID4VCI] endpoints, each with its own
//! `XxxRequest` and `XxxResponse` types. The types serialize to and from JSON, in
//! accordance with the specification.
//!
//! The endpoints are designed to be used with Rust-based HTTP servers, such as
//! [axum](https://docs.rs/axum/latest/axum/).
//!
//! Endpoints can be combined to implement both the [OpenID4VCI] Authorization Code Flow
//! and Pre-Authorized Code Flow.
//!
//! **Running**
//!
//! Per the OAuth 2.0 specification, endpoints are exposed using HTTP. The library
//! will work with most common Rust HTTP servers with a few lines of 'wrapper' code
//! for each endpoint.
//!
//! In addition, implementors need to implement 'Provider' traits that are responsible
//! for handling externals such as  storage, authorization, external communication,
//! etc.. See [`core_utils`](https://docs.rs/core-utils/latest/core_utils/).
//!
//! # Example
//!
//! The following example demonstrates how a single endpoint might be surfaced.
//!
//! A number of elements have been excluded for brevity. A more complete example can be
//! found in the `examples` directory.
//!  
//! ```rust,ignore
//! #[tokio::main]
//! async fn main() {
//!     // `Provider` implements the `Provider` traits
//!     let endpoint = Arc::new(Endpoint::new(Provider::new()));
//!
//!     let router = Router::new()
//!         // --- other routes ---
//!         .route("/credential", post(credential))
//!         // --- other routes ---
//!         .with_state(endpoint);
//!
//!     let listener = TcpListener::bind("0.0.0.0:8080").await.expect("should bind");
//!     axum::serve(listener, router).await.expect("server should run");
//! }
//!
//! // Credential endpoint
//! async fn credential(
//!     State(endpoint): State<Arc<Endpoint<Provider>>>, TypedHeader(host): TypedHeader<Host>,
//!     TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
//!     Json(mut req): Json<CredentialRequest>,
//! ) -> AxResult<CredentialResponse> {
//!     // set credential issuer and access token from HTTP header
//!     req.credential_issuer = format!("http://{}", host);
//!     req.access_token = auth.token().to_string();
//!
//!     // call endpoint
//!     endpoint.credential(req).await.into()
//! }
//! ```
//!
//! [OpenID for Verifiable Credential Issuance]: (https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html)
//! [OpenID4VCI]: (https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html)
//! [OpenID Connect]: (https://openid.net/specs/openid-connect-core-1_0.html)
//! [RFC6749]: (https://www.rfc-editor.org/rfc/rfc6749.html)

mod authorize;
mod batch;
mod create_offer;
mod credential;
mod deferred;
mod metadata;
mod register;
mod state;
mod token;

/// Re-export provider traits and types.
pub mod provider {
    #[allow(clippy::module_name_repetitions)]
    pub use openid::endpoint::{
        Callback, Claims, ClientMetadata, IssuerMetadata, IssuerProvider, Payload, Result,
        ServerMetadata, StateManager, Subject,
    };
    pub use openid::issuance::{ClaimDefinition, GrantType, Issuer};
    pub use openid::{Client, Server};
    pub use proof::jose::jwk::PublicKeyJwk;
    pub use proof::signature::{Algorithm, Signer, Verifier};
}

pub use authorize::authorize;
pub use batch::batch;
pub use create_offer::create_offer;
pub use credential::credential;
pub use deferred::deferred;
pub use metadata::metadata;
// use openid::endpoint::{Handler, Provider, Request};
pub use openid::issuance::{
    AuthorizationCodeGrant, AuthorizationDetail, AuthorizationDetailType, AuthorizationRequest,
    AuthorizationResponse, BatchCredentialRequest, BatchCredentialResponse, CreateOfferRequest,
    CreateOfferResponse, CredentialConfiguration, CredentialOffer, CredentialOfferType,
    CredentialRequest, CredentialResponse, CredentialType, DeferredCredentialRequest,
    DeferredCredentialResponse, Grants, MetadataRequest, MetadataResponse, PreAuthorizedCodeGrant,
    ProofClaims, ProofType, RegistrationRequest, RegistrationResponse, TokenAuthorizationDetail,
    TokenRequest, TokenResponse, TxCode,
};
pub use openid::Result;
pub use register::register;
pub use token::token;

// async fn shell<'a, C, P, R, U, E, F>(
//     context: C, provider: P, request: &'a R, handler: F,
// ) -> Result<U, E>
// where
//     P: Provider,
//     R: Request + Sync,
//     F: Handler<'a, C, P, R, U, E>,
// {
//     println!("in wrapper: {:?}", request.state_key());
//     handler.handle(context, provider, request).await
// }

// #[cfg(test)]
// mod tests {
//     use openid::{Err, Result};
//     use test_utils::issuer::Provider;

//     use super::*;

//     #[tokio::test]
//     async fn test_ok() {
//         let request = TestRequest { return_ok: true };
//         let response = Endpoint::new(Provider::new()).test(&request).await;

//         assert!(response.is_ok());
//     }

//     #[tokio::test]
//     async fn test_err() {
//         let request = TestRequest { return_ok: false };
//         let response = Endpoint::new(Provider::new()).test(&request).await;

//         assert!(response.is_err());
//     }

//     struct TestResponse {}
//     // ------------------------------------------------------------------------
//     // Mock Endpoint
//     // ------------------------------------------------------------------------
//     #[derive(Clone, Debug, Default)]
//     struct TestRequest {
//         return_ok: bool,
//     }

//     impl<P> Endpoint<P>
//     where
//         P: ClientMetadata
//             + IssuerMetadata
//             + ServerMetadata
//             + Subject
//             + StateManager
//             + Signer
//             + Callback
//             + Clone
//             + Debug,
//     {
//         async fn test(&mut self, request: &TestRequest) -> Result<TestResponse> {
//             let ctx = Context {
//                 _p: std::marker::PhantomData,
//             };
//             openid::endpoint::Endpoint::handle_request(self, request, ctx).await
//         }
//     }

//     #[derive(Debug)]
//     struct Context<P> {
//         _p: std::marker::PhantomData<P>,
//     }

//     impl<P> openid::endpoint::Context for Context<P>
//     where
//         P: ClientMetadata
//             + IssuerMetadata
//             + ServerMetadata
//             + Subject
//             + StateManager
//             + Signer
//             + Callback
//             + Clone
//             + Debug,
//     {
//         type Provider = P;
//         type Request = TestRequest;
//         type Response = TestResponse;

//         fn callback_id(&self) -> Option<String> {
//             Some("callback_id".into())
//         }

//         async fn process(
//             &self, _provider: &Self::Provider, request: &Self::Request,
//         ) -> Result<Self::Response> {
//             match request.return_ok {
//                 true => Ok(TestResponse {}),
//                 false => return Err(Err::InvalidRequest("invalid request".into()).into()),
//             }
//         }
//     }
// }
