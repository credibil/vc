//! An API for the issuance of Verifiable Credentials based on the
//! [OpenID for Verifiable Credential Issuance](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html)
//! specification.
//!
//! # OpenID for Verifiable Credential Issuance
//!
//! This library implements an OAuth protected API for the issuance of Verifiable
//! Credentials as specified by [OpenID for Verifiable Credential Issuance](https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html).
//!
//! Verifiable Credentials are similar to identity assertions, like ID Tokens in
//! OpenID Connect [OpenID.Core](https://openid.net/specs/openid-connect-core-1_0.html),
//! in that they allow a Credential Issuer to assert End-User claims. A Verifiable
//! Credential follows a pre-defined schema (the Credential type) and MAY be bound to a
//! certain holder, e.g., through Cryptographic Holder Binding. Verifiable Credentials
//! can be securely presented for the End-User to the RP, without involvement of the
//! Credential Issuer.
//!
//! Access to this API is authorized using OAuth 2.0 [RFC6749](https://www.rfc-editor.org/rfc/rfc6749.html),
//! i.e., Wallets use OAuth 2.0 to obtain authorization to receive Verifiable
//! Credentials. This way the issuance process can benefit from the proven security,
//! simplicity, and flexibility of OAuth 2.0, use existing OAuth 2.0 deployments, and
//! OpenID Connect OPs can be extended to become Credential Issuers.
//!
//! # Design
//!
//! **Endpoints**
//!
//! The library is architected around the OpenID4VCI endpoints, each with its own
//! `XxxRequest` and `XxxResponse` types. The types serialize to and from JSON, in
//! accordance with the specification.
//!
//! The endpoints are designed to be used with Rust-based HTTP servers, such as
//! [axum](https://docs.rs/axum/latest/axum/).
//!
//! Endpoints can be combined to implement both the OpenID4VCI Authorization Code Flow
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
//! etc.. See [`vercre_core`](https://docs.rs/vercre-core/latest/vercre_core/).
//!
//! # Example
//!
//! The following example demonstrates how a single endpoint might be surfaced.
//!
//! A number of elements have been excluded for brevity. A more complete example can be
//! found in the `examples` directory.
//!  
//! ```no_run
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

pub mod endpoint;
mod state;
// #[doc(no_inline)]
pub use vercre_core::vci::{
    AuthorizationCodeGrant, AuthorizationDetail, AuthorizationRequest, AuthorizationResponse,
    BatchCredentialRequest, BatchCredentialResponse, CredentialOffer, CredentialRequest,
    CredentialResponse, DeferredCredentialRequest, DeferredCredentialResponse, Grants,
    InvokeRequest, InvokeResponse, MetadataRequest, MetadataResponse, PreAuthorizedCodeGrant,
    Proof, ProofClaims, RegistrationRequest, RegistrationResponse, TokenRequest, TokenResponse,
};
pub use crate::endpoint::Endpoint;