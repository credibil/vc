//! An API to request and present Verifiable Credentials as Verifiable Presentations
//! based on the [OpenID for Verifiable Presentations](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html)
//! specification.
//!
//! # [OpenID for Verifiable Presentations]
//!
//! [OpenID for Verifiable Presentations] introduces the VP Token as a container to enable
//! End-Users to present Verifiable Presentations to Verifiers using the Wallet.
//! A VP Token contains one or more Verifiable Presentations in the same or different
//! Credential formats.
//!
//! As per the specification, this library supports the response being sent using either
//! a redirect (same-device flow) or an HTTPS POST request (cross-device flow). This
//! enables the response to be sent across devices, or when the response size exceeds
//! the redirect URL character size limitation.
//!
//! ## Same Device Flow
//!
//! The End-User presents a Credential to a Verifier interacting with the End-User on
//! the same device that the device the Wallet resides on.
//!
//! The flow utilizes simple redirects to pass Authorization Request and Response
//! between the Verifier and the Wallet. The Verifiable Presentations are returned to
//! the Verifier in the fragment part of the redirect URI, when Response Mode is fragment.
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
//! The End-User presents a Credential to a Verifier interacting with the End-User on
//! a different device as the device the Wallet resides on (or where response size the
//! redirect URL character size).
//!
//! In this flow the Verifier prepares an Authorization Request and renders it as a
//! QR Code. The User then uses the Wallet to scan the QR Code. The Verifiable
//! Presentations are sent to the Verifier in a direct HTTPS POST request to a URL
//! controlled by the Verifier. The flow uses the Response Type "`vp_token`" in
//! conjunction with the Response Mode "`direct_post`". In order to keep the size of the
//! QR Code small and be able to sign and optionally encrypt the Request Object, the
//! actual Authorization Request contains just a Request URI, which the wallet uses to
//! retrieve the actual Authorization Request data.
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
//! The [JWT VC Presentation Profile] defines a set of requirements against existing
//! specifications to enable the interoperable presentation of Verifiable Credentials
//! (VCs) between Wallets and Verifiers.
//!
//! The `vercre-vp` library has been implemented to support the profile's
//! recommendations.
//!
//! # Design
//!
//! **Endpoints**
//!
//! The library is architected around the [OpenID4VP] endpoints, each with its own
//! `XxxRequest` and `XxxResponse` types. The types serialize to and from JSON, in
//! accordance with the specification.
//!
//! The endpoints are designed to be used with Rust-based HTTP servers, such as
//! [axum](https://docs.rs/axum/latest/axum/).
//!
//! Endpoints can be combined to implement both the [OpenID4VP] same-device and
//! cross-device flows.
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
//! [OpenID for Verifiable Presentations]: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html
//! [OpenID4VP]: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html
//! [JWT VC Presentation Profile]: https://identity.foundation/jwt-vc-presentation-profile


mod state;
pub mod invoke;
pub mod metadata;
pub mod request;
pub mod response;

use std::fmt::Debug;

use tracing::instrument;
use vercre_core::callback::{Payload, Status};
use vercre_core::error::Error;
use vercre_core::{Callback, Client, Result, Signer, StateManager};

// TODO: remove double borrow for traits (i.e. &self -> self)
// TODO: reintroduce impl Provider trait + lifetimes for Endpoint

/// Endpoint is used to surface the public Verifiable Presentation endpoints to
/// clients.
#[derive(Debug)]
pub struct Endpoint<P>
where
    P: Client + StateManager + Signer + Callback + Debug,
{
    provider: P,
}

/// Endpoint is used to provide a thread-safe way of handling endpoint requests.
/// Each request passes through a number of steps with request state required to
/// be maintained between steps.
///
/// The Endpoint also provides common top-level tracing, error handling, and client
/// callback functionality for all endpoints. The act of setting a request causes
/// the Endpoint to select the endpoint implementation of `Endpoint::call` specific
/// to the request.
impl<P> Endpoint<P>
where
    P: Client + StateManager + Signer + Callback + Clone + Debug,
{
    /// Create a new endpoint instance.
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Wrap the processing of individual requests for shared handling of callbacks,
    /// errors, etc..
    ///
    /// Each endpoint implements a request-specific `Endpoint::call` method which then
    /// calls `Endpoint::handle_request` to handle shared functionality.
    #[instrument]
    async fn handle_request<R, C, U>(&self, request: R, ctx: C) -> Result<U>
    where
        C: Context<Request = R, Response = U>,
        R: Default + Clone + Debug + Send + Sync,
    {
        if let Some(callback_id) = ctx.callback_id() {
            let pl = Payload {
                id: callback_id.clone(),
                status: Status::PresentationRequested,
                context: String::new(),
            };
            self.provider.callback(&pl).await?;
        }

        let res = match ctx.verify(&self.provider, &request).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!(target:"Endpoint::verify", ?e);
                self.try_callback(ctx, &e).await?;
                return Err(e);
            }
        };

        match res.process(&self.provider, &request).await {
            Ok(res) => Ok(res),
            Err(e) => {
                tracing::error!(target:"Endpoint::process", ?e);
                self.try_callback(ctx, &e).await?;
                Err(e)
            }
        }
    }

    /// Try to send a callback to the client. If the callback fails, log the error.
    #[instrument]
    async fn try_callback<R, C, U>(&self, ctx: C, e: &Error) -> anyhow::Result<()>
    where
        C: Context<Request = R, Response = U>,
        R: Default + Clone + Send + Sync + Debug,
    {
        if let Some(callback_id) = ctx.callback_id() {
            tracing::trace!("Endpoint::try_callback");

            let pl = Payload {
                id: callback_id.clone(),
                status: Status::Error,
                context: format!("{e}"),
            };
            return self.provider.callback(&pl).await;
        }
        Ok(())
    }
}

/// Context is implemented by every endpoint to set up a context for each
/// request.
#[allow(async_fn_in_trait)]
pub trait Context: Send + Sync + Debug {
    /// The request type for the request context.
    type Request;

    /// The response type for the request context.
    type Response;

    /// Callback ID is used to identify the initial request when sending status
    /// updates to the client.
    fn callback_id(&self) -> Option<String>;

    /// Verify the request.
    #[allow(clippy::unused_async)]
    async fn verify<P>(&self, _: &P, _: &Self::Request) -> Result<&Self>
    where
        P: Client + StateManager + Signer + Callback + Clone + Debug,
    {
        Ok(self)
    }

    /// Process the request.
    async fn process<P>(&self, provider: &P, request: &Self::Request) -> Result<Self::Response>
    where
        P: Client + StateManager + Signer + Callback + Clone + Debug;
}

#[cfg(test)]
mod tests {
    use test_utils::vp_provider::Provider;
    use vercre_core::err;
    use vercre_core::error::Err;

    use super::*;

    #[tokio::test]
    async fn test_ok() {
        let request = TestRequest { return_ok: true };
        let response = Endpoint::new(Provider::new()).test(request).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_err() {
        let request = TestRequest { return_ok: false };
        let response = Endpoint::new(Provider::new()).test(request).await;

        assert!(response.is_err());
    }

    // ------------------------------------------------------------------------
    // Mock Endpoint
    // ------------------------------------------------------------------------
    #[derive(Clone, Debug, Default)]
    pub(super) struct TestRequest {
        return_ok: bool,
    }

    pub(super) struct TestResponse {}

    impl<P> Endpoint<P>
    where
        P: Client + StateManager + Signer + Callback + Clone + Debug,
    {
        async fn test(&mut self, request: TestRequest) -> Result<TestResponse> {
            self.handle_request(request, Context::new()).await
        }
    }

    #[derive(Debug)]
    pub(super) struct Context;

    impl Context {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl super::Context for Context {
        type Request = TestRequest;
        type Response = TestResponse;

        fn callback_id(&self) -> Option<String> {
            Some("callback_id".to_string())
        }

        async fn process<P>(&self, _provider: &P, request: &Self::Request) -> Result<Self::Response>
        where
            P: Client + StateManager + Debug,
        {
            match request.return_ok {
                true => Ok(TestResponse {}),
                false => err!(Err::InvalidRequest, "invalid request"),
            }
        }
    }
}

