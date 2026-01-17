pub mod types;

use crate::config::RocketCyberConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RocketCyberClient {
    client: Client,
    config: RocketCyberConfig,
}

impl RocketCyberClient {
    pub fn new(config: RocketCyberConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, config })
    }

    pub async fn get_incidents(&self) -> Result<Vec<types::Incident>> {
        let url = format!("{}/incidents?pageSize=100", self.config.api_url);

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
