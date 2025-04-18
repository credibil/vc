//! #! Querying credentials

use std::collections::HashMap;

use anyhow::Result;
use credibil_did::SignerExt;

use super::ClientIdentifier;
use crate::format::sd_jwt::SdJwtVpBuilder;
use crate::oid4vp::types::{QueryResult, RequestedFormat};

/// Generate a Verifiable Presentation (VP) token.
///
/// # Errors
///
/// Returns an error when building a presentation from a `QueryResult` fails.
pub async fn generate(
    client_id: &ClientIdentifier, results: &[QueryResult<'_>], signer: &impl SignerExt,
) -> Result<HashMap<String, Vec<String>>> {
    let mut token = HashMap::<String, Vec<String>>::new();

    // create an entry for each credential query
    for result in results {
        let mut presentations = vec![];

        // create presentation for each query result
        match result.query.format {
            RequestedFormat::DcSdJwt => {
                for matched in &result.matches {
                    let vp = SdJwtVpBuilder::new()
                        .client_id(client_id.to_string())
                        .matched(matched)
                        .signer(signer)
                        .build()
                        .await?;
                    presentations.push(vp);
                }
            }
            RequestedFormat::MsoMdoc => {
                continue;
            }
            RequestedFormat::JwtVcJson | RequestedFormat::JwtVcJsonLd | RequestedFormat::LdpVc => {
                todo!()
            }
        }

        token.insert(result.query.id.clone(), presentations);
    }

    Ok(token)
}
