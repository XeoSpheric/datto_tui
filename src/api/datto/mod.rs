pub mod activity;
pub mod devices;
pub mod jobs;
pub mod sites;
pub mod types;
pub mod variables;

use crate::config::DattoConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use types::TokenResponse;

#[derive(Clone, Debug)]
pub struct DattoClient {
    pub(crate) client: Client,
    pub(crate) config: DattoConfig,
    pub(crate) access_token: Option<String>,
}

impl DattoClient {
    pub fn new(config: DattoConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self {
            client,
            config,
            access_token: None,
        })
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        let url = format!("{}/auth/oauth/token", self.config.api_url);

        let params = [
            ("grant_type", "password"),
            ("username", &self.config.api_key),
            ("password", &self.config.secret_key),
        ];

        let response = self
            .client
            .post(&url)
            .basic_auth("public-client", Some("public"))
            .form(&params)
            .send()
            .await
            .context("Failed to send auth request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Authentication failed: {} - {}", status, text);
        }

        let token_response = response
            .json::<TokenResponse>()
            .await
            .context("Failed to parse token")?;
        self.access_token = Some(token_response.access_token);

        Ok(())
    }

    pub async fn get_device_open_alerts(&self, device_uid: &str) -> Result<Vec<types::Alert>> {
        // Use /api/v2/ to match other endpoints pattern
        let url = format!("{}/api/v2/device/{}/alerts/open", self.config.api_url, device_uid);
        
        // Log the URL
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "Fetching Alerts URL: {}", url).unwrap();
            });

        let resp = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.as_ref().unwrap()),
            )
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch open alerts: {} - {}", status, text);
        }

        let text = resp.text().await?;
        let alerts_response: types::OpenAlertsResponse = serde_json::from_str(&text)?;
        Ok(alerts_response.alerts)
    }
}
