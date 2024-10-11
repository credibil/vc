//! An API to request and present Verifiable Credentials as Verifiable
//! Presentations based on the [OpenID for Verifiable Presentations](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html)
//! specification.
//!
//! # [OpenID for Verifiable Presentations]
//!
//! [OpenID for Verifiable Presentations] introduces the VP Token as a container
//! to enable End-Users to present Verifiable Presentations to Verifiers using
//! the Wallet. A VP Token contains one or more Verifiable Presentations in the
//! same or different Credential formats.
//!
//! As per the specification, this library supports the response being sent
//! using either a redirect (same-device flow) or an HTTPS POST request
//! (cross-device flow). This enables the response to be sent across devices, or
//! when the response size exceeds the redirect URL character size limitation.
//!
//! ## Same Device Flow
//!
//! The End-User presents a Credential to a Verifier interacting with the
//! End-User on the same device that the device the Wallet resides on.
//!
//! The flow utilizes simple redirects to pass Authorization Request and
//! Response between the Verifier and the Wallet. The Verifiable Presentations
//! are returned to the Verifier in the fragment part of the redirect URI, when
//! Response Mode is fragment.
//!
//! ```text
//! +--------------+   +--------------+                                    +--------------+
//! |     User     |   |   Verifier   |                                    |    Wallet    |
//! +--------------+   +--------------+                                    +--------------+
//!         |                 |                                                   |
//!         |    Interacts    |                                                   |
//!         |---------------->|                                                   |
//!         |                 |  (1) Authorization Request                        |
//!         |                 |  (Presentation Definition)                        |
//!         |                 |-------------------------------------------------->|
//!         |                 |                                                   |
//!         |                 |                                                   |
//!         |   User Authentication / Consent                                     |
//!         |                 |                                                   |
//!         |                 |  (2)   Authorization Response                     |
//!         |                 |  (VP Token with Verifiable Presentation(s))       |
//!         |                 |<--------------------------------------------------|
//! ```
//!
//! ## Cross Device Flow
//!
//! The End-User presents a Credential to a Verifier interacting with the
//! End-User on a different device as the device the Wallet resides on (or where
//! response size the redirect URL character size).
//!
//! In this flow the Verifier prepares an Authorization Request and renders it
//! as a QR Code. The User then uses the Wallet to scan the QR Code. The
//! Verifiable Presentations are sent to the Verifier in a direct HTTPS POST
//! request to a URL controlled by the Verifier. The flow uses the Response Type
//! "`vp_token`" in conjunction with the Response Mode "`direct_post`". In order
//! to keep the size of the QR Code small and be able to sign and optionally
//! encrypt the Request Object, the actual Authorization Request contains just a
//! Request URI, which the wallet uses to retrieve the actual Authorization
//! Request data.
//!
//! ```text
//! +--------------+   +--------------+                                    +--------------+
//! |     User     |   |   Verifier   |                                    |    Wallet    |
//! |              |   |  (device A)  |                                    |  (device B)  |
//! +--------------+   +--------------+                                    +--------------+
//!         |                 |                                                   |
//!         |    Interacts    |                                                   |
//!         |---------------->|                                                   |
//!         |                 |  (1) Authorization Request                        |
//!         |                 |      (Request URI)                                |
//!         |                 |-------------------------------------------------->|
//!         |                 |                                                   |
//!         |                 |  (2) Request the Request Object                   |
//!         |                 |<--------------------------------------------------|
//!         |                 |                                                   |
//!         |                 |  (2.5) Respond with the Request Object            |
//!         |                 |      (Presentation Definition)                    |
//!         |                 |-------------------------------------------------->|
//!         |                 |                                                   |
//!         |   User Authentication / Consent                                     |
//!         |                 |                                                   |
//!         |                 |  (3)   Authorization Response as HTTPS POST       |
//!         |                 |  (VP Token with Verifiable Presentation(s))       |
//!         |                 |<--------------------------------------------------|
//! ```
//!
//! ## JWT VC Presentation Profile
//!
//! The [JWT VC Presentation Profile] defines a set of requirements against
//! existing specifications to enable the interoperable presentation of
//! Verifiable Credentials (VCs) between Wallets and Verifiers.
//!
//! The `vercre-vp` library has been implemented to support the profile's
//! recommendations.
//!
//! # Design
//!
//! **Endpoints**
//!
//! The library is architected around the [OpenID4VP] endpoints, each with its
//! own `XxxRequest` and `XxxResponse` types. The types serialize to and from
//! JSON, in accordance with the specification.
//!
//! The endpoints are designed to be used with Rust-based HTTP servers, such as
//! [axum](https://docs.rs/axum/latest/axum/).
//!
//! Endpoints can be combined to implement both the [OpenID4VP] same-device and
//! cross-device flows.
//!
//! **Running**
//!
//! Per the OAuth 2.0 specification, endpoints are exposed using HTTP. The
//! library will work with most common Rust HTTP servers with a few lines of
//! 'wrapper' code for each endpoint.
//!
//! In addition, implementors need to implement 'Provider' traits that are
//! responsible for handling externals such as  storage, authorization, external
//! communication, etc.. See [`core_utils`](https://docs.rs/core-utils/latest/core_utils/).
//!
//! # Example
//!
//! The following example demonstrates how a single endpoint might be surfaced.
//!
//! A number of elements have been excluded for brevity. A more complete example
//! can be found in the `examples` directory.
//!  
//! ```rust,ignore
//! #[tokio::main]
//! async fn main() {
//!     // `Provider` implements the `Provider` traits
//!     let endpoint = Arc::new(Endpoint::new(Provider::new()));
//!
//!     let router = Router::new()
//!         // --- other routes ---
//!         .route("/request/:client_state", get(request_object))
//!         // --- other routes ---
//!         .with_state(endpoint);
//!
//!     let listener = TcpListener::bind("0.0.0.0:8080").await.expect("should bind");
//!     axum::serve(listener, router).await.expect("server should run");
//! }
//!
//! // Credential endpoint
//! async fn request_object(
//!     State(endpoint): State<Arc<Endpoint<Provider>>>, TypedHeader(host): TypedHeader<Host>,
//!     Path(client_state): Path<String>,
//! ) -> AxResult<RequestObjectResponse> {
//!     let req = RequestObjectRequest {
//!         client_id: format!("http://{}", host),
//!         state: client_state,
//!     };
//!
//!     endpoint.request_object(req).await.into()
//! }
//! ```
//!
//! [OpenID for Verifiable Presentations]: (https://openid.net/specs/openid-4-verifiable-presentations-1_0.html)
//! [OpenID4VP]: (https://openid.net/specs/openid-4-verifiable-presentations-1_0.html)
//! [JWT VC Presentation Profile]: (https://identity.foundation/jwt-vc-presentation-profile)

mod create_request;
mod metadata;
mod request_object;
mod response;
mod state;

/// Re-export types.
pub use vercre_openid::Result;

/// Re-export provider traits and types.
pub mod provider {
    pub use vercre_datasec::{Algorithm, Decryptor, Encryptor, PublicKeyJwk, SecOps, Signer};
    pub use vercre_did::{DidResolver, Document};
    pub use vercre_openid::issuer::{Client, Format, Server};
    pub use vercre_openid::verifier::VpFormat;
    #[allow(clippy::module_name_repetitions)]
    pub use vercre_openid::verifier::{Metadata, Provider, Result, StateStore, Verifier, Wallet};
}
pub use create_request::create_request;
pub use metadata::metadata;
pub use request_object::request_object;
pub use response::response;
pub use vercre_dif_exch::{Constraints, Field, Filter, FilterValue, InputDescriptor};
pub use vercre_openid::verifier::{
    ClientIdScheme, CreateRequestRequest, CreateRequestResponse, DeviceFlow, MetadataRequest,
    MetadataResponse, RequestObject, RequestObjectRequest, RequestObjectResponse, ResponseRequest,
    ResponseResponse, ResponseType,
};
