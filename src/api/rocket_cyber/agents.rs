use super::RocketCyberClient;
use crate::api::rocket_cyber::types;
use anyhow::{Context, Result};

pub(crate) trait AgentsApi {
    async fn get_agents(&self, hostname: &str) -> Result<Vec<types::Agent>>;
}

impl AgentsApi for RocketCyberClient {
    async fn get_agents(&self, hostname: &str) -> Result<Vec<types::Agent>> {
        let base_url = self.config.api_url.trim_end_matches('/').trim_end_matches("/v3");
        let url = format!("{}/v3/agents", base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.config.api_key)
            .header("Content-Type", "application/json")
            .query(&[("hostname", hostname)])
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let text = response.text().await.context("Failed to get response text")?;

        // Debug Log
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(
                    f,
                    "RocketCyber Agents Search: hostname={} | URL: {} | Status: {} | Response: {}",
                    hostname, url, status, text
                )
                .unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("RocketCyber API failed: {} - {}", status, text);
        }

        let parsed: types::AgentsResponse =
            serde_json::from_str(&text).context("Failed to parse response")?;
        Ok(parsed.data)
    }
}
