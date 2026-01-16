pub mod types;

use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use types::{SitesResponse, TokenResponse};

#[derive(Clone, Debug)]
pub struct RmmClient {
    client: Client,
    config: Config,
    access_token: Option<String>,
}

impl RmmClient {
    pub fn new(config: Config) -> Result<Self> {
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

    pub async fn get_sites(
        &self,
        page: i32,
        max: i32,
        site_name: Option<String>,
    ) -> Result<SitesResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let mut url = format!(
            "{}/api/v2/account/sites?page={}&max={}",
            self.config.api_url, page, max
        );

        if let Some(name) = site_name {
            url.push_str(&format!("&siteName={}", name));
        }

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

        let sites_response = response
            .json::<SitesResponse>()
            .await
            .context("Failed to parse JSON")?;
        Ok(sites_response)
    }
}
