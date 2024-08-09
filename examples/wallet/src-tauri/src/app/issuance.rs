//! Application state implementation for issuance operations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vercre_holder::{CredentialConfiguration, CredentialOffer, OfferRequest, PinRequest, IssuanceStatus, TxCode};

use super::{AppState, SubApp};
use crate::provider::Provider;
use crate::CLIENT_ID;

/// Application state for the issuance sub-app.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct IssuanceState {
    /// Issuance flow identifier to pass to the vercre-holder crate for state management.
    pub id: String,
    /// Status of the issuance flow.
    pub status: IssuanceStatus,
    /// Issuer of the credential(s)
    pub issuer: String,
    /// Description of the credential(s) offered, keyed by credential configuration ID.
    pub offered: HashMap<String, CredentialConfiguration>,
    /// Description of the type of PIN needed to accept the offer.
    pub tx_code: Option<TxCode>,
    /// PIN set by the holder.
    pub pin: Option<String>,
}

impl AppState {
    /// Process a credential issuance offer.
    pub async fn offer(&mut self, encoded_offer: &str, provider: Provider) -> anyhow::Result<()> {
        let offer_str = urlencoding::decode(encoded_offer)?;
        let offer = serde_json::from_str::<CredentialOffer>(&offer_str)?;
        let request = OfferRequest {
            client_id: CLIENT_ID.into(),
            offer,
        };
        let res = vercre_holder::offer(provider, &request).await?;
        self.issuance = IssuanceState {
            id: res.issuance_id,
            status: res.status,
            issuer: res.issuer,
            offered: res.offered,
            tx_code: res.tx_code,
            pin: None,
        };
        self.sub_app = SubApp::Issuance;
        Ok(())
    }

    /// Accept a credential issuance offer.
    pub async fn accept(&mut self, provider: Provider) -> anyhow::Result<()> {
        let status = vercre_holder::accept(provider, self.issuance.id.clone()).await?;
        self.issuance.status = status;
        Ok(())
    }

    /// Set a PIN
    pub async fn pin(&mut self, provider: Provider, pin: &str) -> anyhow::Result<()> {
        let request = PinRequest {
            id: self.issuance.id.clone(),
            pin: pin.into(),
        };
        let status = vercre_holder::pin(provider, &request).await?;
        self.issuance.status = status;
        self.issuance.pin = Some(pin.into());
        Ok(())
    }

    /// Get the credentials for the accepted issuance offer.
    pub async fn get_credentials(&mut self, provider: Provider) -> anyhow::Result<()> {
        let status = vercre_holder::get_credentials(provider, self.issuance.id.clone()).await?;
        self.issuance.status = status;
        self.sub_app = SubApp::Credential;
        Ok(())
    }
}
