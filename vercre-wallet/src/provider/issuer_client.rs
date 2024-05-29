//! # Issuer Client provider
//!
//! This provider is used to allows the wallet to interact with an issuer's services that are
//! compliant with OpenID for VC Issuance. While the specification is oriented towards HTTP, the
//! trait allows the wallet (and issuance services) to be transport layer agnostic.
use std::future::Future;

use vercre_core::vci::{MetadataRequest, MetadataResponse};

use crate::Result;

/// `IssuerClient` is a provider that implements the wallet side of the OpenID for VC Issuance
/// interactions with an issuance service.
pub trait IssuerClient {
    /// Get issuer metadata. If an error is returned, the wallet will cancel the issuance flow.
    fn get_metadata(
        &self, req: &MetadataRequest,
    ) -> impl Future<Output = Result<MetadataResponse>> + Send;
}
