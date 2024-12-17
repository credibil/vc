use http::header::{ACCEPT, CONTENT_TYPE};
use tauri_plugin_http::reqwest;
use vercre_holder::provider::Verifier;
use vercre_holder::{RequestObjectResponse, ResponseRequest, ResponseResponse};

use super::Provider;

impl Verifier for Provider {
    /// Get a request object. If an error is returned, the wallet will cancel
    /// the presentation flow.
    async fn request_object(&self, req: &str) -> anyhow::Result<RequestObjectResponse> {
        let client = reqwest::Client::new();
        let result = client.get(req).header(ACCEPT, "application/json").send().await?;
        let response = match result.json::<RequestObjectResponse>().await {
            Ok(response) => response,
            Err(e) => {
                log::error!("Error getting request object: {}", e);
                return Err(e.into());
            }
        };
        Ok(response)
    }

    /// Send the presentation to the verifier.
    async fn present(
        &self, uri: Option<&str>, presentation: &ResponseRequest,
    ) -> anyhow::Result<ResponseResponse> {
        let client = reqwest::Client::new();
        let Some(presentation_url) = uri else {
            return Err(anyhow::anyhow!("No URI provided"));
        };
        let form = presentation.form_encode()?;
        let result = client
            .post(presentation_url)
            .header(CONTENT_TYPE, "multipart/form-data")
            .header(ACCEPT, "application/json")
            .form(&form)
            .send()
            .await?;
        let response = match result.json::<ResponseResponse>().await {
            Ok(response) => response,
            Err(e) => {
                log::error!("Error sending presentation: {}", e);
                return Err(e.into());
            }
        };
        Ok(response)
    }
}
