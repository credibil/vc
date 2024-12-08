//! View model for the issuance sub-app

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use vercre_holder::issuance::IssuanceState;
use vercre_holder::TxCode;

use crate::view::credential::CredentialDisplay;

/// Issuance flow viewable state
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[typeshare]
#[allow(clippy::module_name_repetitions)]
pub struct IssuanceView {
    /// Credentials on offer
    pub credentials: HashMap<String, CredentialDisplay>,
    /// PIN
    pub pin: Option<String>,
    /// PIN schema
    pub pin_schema: Option<PinSchema>,
}

/// Convert the underlying issuance flow state to a view model of the same
impl From<IssuanceState> for IssuanceView {
    fn from(state: IssuanceState) -> Self {
        let mut creds: HashMap<String, CredentialDisplay> = HashMap::new();
        let on_offer = state.offered().unwrap_or_default();
        let issuer = state.issuer.clone().unwrap_or_default();
        for (id, offered) in &on_offer {
            let mut cred: CredentialDisplay = offered.into();
            cred.issuer = Some(issuer.credential_issuer.clone());
            creds.insert(id.clone(), cred);
        }
        let schema = state.offer.as_ref().and_then(|offer| {
            offer.grants.as_ref().and_then(|grants| {
                grants.pre_authorized_code.as_ref().and_then(|pre_auth| {
                    pre_auth.tx_code.as_ref().map(|tx_code| tx_code.clone().into())
                })
            })
        });
        Self {
            credentials: creds,
            pin: state.pin,
            pin_schema: schema,
        }
    }
}

/// Types of PIN characters
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[typeshare]
pub enum PinInputMode {
    /// Only digits
    #[default]
    Numeric,
    /// Any characters
    Text,
}

/// Criteria for PIN
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[typeshare]
pub struct PinSchema {
    /// Input mode for the PIN
    pub input_mode: PinInputMode,

    /// Specifies the length of the PIN. This helps the Wallet to render
    /// the input screen and improve the user experience.
    pub length: i32,

    /// Guidance for the Holder of the Wallet on how to obtain the Transaction
    /// Code,
    pub description: Option<String>,
}

impl From<TxCode> for PinSchema {
    fn from(tx_code: TxCode) -> Self {
        let mut input_mode: PinInputMode = PinInputMode::Numeric;
        if let Some(mode) = tx_code.input_mode {
            if mode == "text" {
                input_mode = PinInputMode::Text;
            }
        }
        Self {
            input_mode,
            length: tx_code.length.unwrap_or(6),
            description: tx_code.description.clone(),
        }
    }
}
