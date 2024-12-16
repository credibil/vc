//! This module has types for the application state. The application is divided
//! into sub-apps, so the shape of the state depends on whether the application
//! is managing credentials, responding to an offer of issuance, or responding
//! to a request for presentation. The underlying application state is
//! translated into a view model for the shell to render.

mod credential;
pub mod issuance;
pub mod presentation;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use vercre_holder::credential::Credential;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[typeshare]
#[allow(clippy::module_name_repetitions)]
pub enum SubApp {
    #[default]
    Splash,
    Credential,
    Issuance,
    Presentation,
}

/// Application state
#[derive(Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct AppState {
    /// The sub-app currently active
    pub sub_app: SubApp,
    /// Credentials stored in the wallet
    pub credential: Vec<Credential>,
    /// State of issuance flow (if active)
    pub issuance: issuance::IssuanceState,
    /// State of presentation flow (if active)
    pub presentation: presentation::PresentationState,
    /// Error information
    pub error: Option<String>,
}
