use super::RocketCyberClient;
use crate::api::rocket_cyber::types;
use anyhow::{Context, Result};

pub(crate) trait IncidentsApi {
    async fn get_incidents(&self) -> Result<Vec<types::Incident>>;
}

impl IncidentsApi for RocketCyberClient {
    async fn get_incidents(&self) -> Result<Vec<types::Incident>> {
        let url = format!("/v3/{}/incidents?pageSize=100", self.config.api_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.config.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("RocketCyber API failed: {} - {}", status, text);
        }

        let parsed: types::IncidentsResponse =
            response.json().await.context("Failed to parse response")?;
        Ok(parsed.data)
    }
}
