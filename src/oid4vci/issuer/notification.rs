// TODO: implement Notification endpoint

//! # Notification Endpoint
//!
//! This endpoint is used by the Wallet to notify the Credential Issuer of
//! certain events for issued Credentials. These events enable the Credential
//! Issuer to take subsequent actions after issuance.
//!
//! The Credential Issuer needs to return one or
//! more `notification_id` parameters in the Credential Response or the Batch
//! Credential Response for the Wallet to be able to use this Endpoint. Support
//! for this endpoint is OPTIONAL. The Issuer cannot assume that a notification
//! will be sent for every issued credential since the use of this Endpoint is
//! not mandatory for the Wallet.
//!
//! The notification from the Wallet is idempotent. When the Credential Issuer
//! receives multiple identical calls from the Wallet for the same
//! `notification_id`, it returns success. Due to the network errors, there are
//! no guarantees that a Credential Issuer will receive a notification within a
//! certain time period or at all.

use tracing::instrument;

use crate::oid4vci::endpoint::Handler;
use crate::oid4vci::provider::{Provider, StateStore};
use crate::oid4vci::state::State;
use crate::oid4vci::types::{NotificationRequest, NotificationResponse};
use crate::oid4vci::{Error, Result};

/// Notification request handler.
///
/// # Errors
///
/// Returns an `OpenID4VP` error if the request is invalid or if the provider is
/// not available.
#[instrument(level = "debug", skip(provider))]
pub async fn notification(
    credential_issuer: &str, provider: &impl Provider, request: NotificationRequest,
) -> Result<NotificationResponse> {
    tracing::debug!("notification");

    // verify access token
    let _ = StateStore::get::<State>(provider, &request.access_token)
        .await
        .map_err(|_| Error::AccessDenied("invalid access token".to_string()))?;

    let Ok(_state) = StateStore::get::<State>(provider, &request.notification_id).await else {
        return Err(Error::AccessDenied("invalid notification id".to_string()));
    };

    tracing::info!("notification: {:#?}, {:#?}", request.event, request.event_description,);

    Ok(NotificationResponse)
}

impl Handler for NotificationRequest {
    type Response = NotificationResponse;

    fn handle(
        self, credential_issuer: &str, provider: &impl Provider,
    ) -> impl Future<Output = Result<Self::Response>> + Send {
        notification(credential_issuer, provider, self)
    }
}
