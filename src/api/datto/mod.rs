pub mod devices;
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
}
