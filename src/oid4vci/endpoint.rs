//! # Endpoint
//!
//! `Endpoint` provides the entry point for DWN messages. Messages are routed
//! to the appropriate handler for processing, returning a reply that can be
//! serialized to a JSON object.

use std::fmt::Debug;

use http::HeaderMap;

use crate::invalid;
use crate::oid4vci::Result;
use crate::oid4vci::provider::Provider;

/// Handle incoming messages.
///
/// # Errors
///
/// This method can fail for a number of reasons related to the imcoming
/// message's viability. Expected failues include invalid authorization,
/// insufficient permissions, and invalid message content.
///
/// Implementers should look to the Error type and description for more
/// information on the reason for failure.
pub async fn handle<B, H, U>(
    issuer: &str, request: impl Into<Request<B, H>>, provider: &impl Provider,
) -> Result<U>
where
    B: Body,
    H: Headers,
    Request<B, H>: Handler<Response = U>,
{
    let request: Request<B, H> = request.into();
    request.validate(issuer, provider).await?;
    request.handle(issuer, provider).await
}

/// A request to process.
#[derive(Clone, Debug)]
pub struct Request<B, H>
where
    B: Body,
    H: Headers,
{
    /// The request to process.
    pub body: B,

    /// Optional headers associated with this request.
    pub headers: Option<HeaderMap>,

    /// Optional headers associated with this request.
    pub headers2: Option<H>,
}

impl<B, H> From<B> for Request<B, H>
where
    B: Body,
    H: Headers,
{
    fn from(body: B) -> Self {
        Self {
            body,
            headers: None,
            headers2: None,
        }
    }
}

/// Empty request headers implementation
#[derive(Clone, Debug)]
pub struct NoHeaders;
impl Headers for NoHeaders {}

/// Methods common to all messages.
///
/// The primary role of this trait is to provide a common interface for
/// messages so they can be handled by [`handle`] method.
pub trait Handler: Clone + Debug + Send + Sync {
    /// The inner reply type specific to the implementing message.
    type Response;

    /// Routes the message to the concrete handler used to process the message.
    fn handle(
        self, issuer: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<Self::Response>> + Send;

    /// Perform initial validation of the message.
    ///
    /// Validation undertaken here is common to all messages, with message-
    /// specific validation performed by the message's handler.
    fn validate(
        &self, issuer: &str, _provider: &impl Provider,
    ) -> impl Future<Output = Result<()>> + Send {
        async {
            // if !tenant_gate.active(issuer)? {
            //     return Err(Error::Unauthorized("tenant not active"));
            // }
            // `credential_issuer` required
            if issuer.is_empty() {
                return Err(invalid!("no `credential_issuer` specified"));
            }

            // // validate the message schema during development
            // #[cfg(debug_assertions)]
            // schema::validate(self)?;

            // // authenticate the requestor
            // if let Some(authzn) = self.authorization() {
            //     if let Err(e) = authzn.verify(provider.clone()).await {
            //         return Err(unauthorized!("failed to authenticate: {e}"));
            //     }
            // }

            Ok(())
        }
    }
}

pub(crate) use seal::{Body, Headers};
pub(crate) mod seal {
    use std::fmt::Debug;

    /// The `Body` trait is used to restrict the types able to be a Request
    /// body. It is implemented by all `xxxRequest` types.
    pub trait Body: Clone + Debug + Send + Sync {}

    /// The `Headers` trait is used to restrict the types able to be a Request
    /// headers. It is implemented by handlers expecting headers.
    pub trait Headers: Clone + Debug + Send + Sync {}
}
