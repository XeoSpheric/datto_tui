use super::DattoClient;
use crate::api::datto::types::DevicesResponse;
use anyhow::{Context, Result};

pub(crate) trait DevicesApi {
    async fn get_devices(&self, site_uid: &str, page: i32, max: i32) -> Result<DevicesResponse>;
}

impl DevicesApi for DattoClient {
    async fn get_devices(&self, site_uid: &str, page: i32, max: i32) -> Result<DevicesResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!(
            "{}/api/v2/site/{}/devices?page={}&max={}",
            self.config.api_url, site_uid, page, max
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let text = response
            .text()
            .await
            .context("Failed to get response text")?;

        let devices_response = serde_json::from_str(&text).context("Failed to parse JSON")?;
        Ok(devices_response)
    }
}
