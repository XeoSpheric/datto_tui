pub mod types;

use crate::config::DattoConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use types::{DevicesResponse, SitesResponse, TokenResponse};

#[derive(Clone, Debug)]
pub struct DattoClient {
    client: Client,
    config: DattoConfig,
    access_token: Option<String>,
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

    pub async fn get_devices(
        &self,
        site_uid: &str,
        page: i32,
        max: i32,
    ) -> Result<DevicesResponse> {
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
    pub async fn get_site_variables(&self, site_uid: &str) -> Result<Vec<types::SiteVariable>> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!("{}/api/v2/site/{}/variables", self.config.api_url, site_uid);

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

        let resp_json = response
            .json::<types::SiteVariablesResponse>()
            .await
            .context("Failed to parse JSON")?;

        Ok(resp_json.variables)
    }
    pub async fn create_site_variable(
        &self,
        site_uid: &str,
        req: types::CreateVariableRequest,
    ) -> Result<types::SiteVariable> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!("{}/api/v2/site/{}/variable", self.config.api_url, site_uid);

        let response = self
            .client
            .put(&url)
            .bearer_auth(access_token)
            .json(&req)
            .send()
            .await
            .context("Failed to send create variable request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        // DEBUG LOG
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "CREATE VARIABLE RESPONSE Status: {}", status).unwrap();
                writeln!(f, "CREATE VARIABLE RESPONSE Body: {}", text).unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        // Handle empty response by returning a dummy variable
        // The app refetches variables on create anyway, so the ID doesn't matter much *locally* for the list update
        // as long as we don't error out.
        if text.trim().is_empty() || text == "null" {
            Ok(types::SiteVariable {
                id: 0,
                name: req.name,
                value: req.value,
                masked: req.masked,
            })
        } else {
            let variable = serde_json::from_str::<types::SiteVariable>(&text)
                .context("Failed to parse response")?;
            Ok(variable)
        }
    }

    pub async fn update_site_variable(
        &self,
        site_uid: &str,
        variable_id: i32,
        req: types::UpdateVariableRequest,
    ) -> Result<types::SiteVariable> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!(
            "{}/api/v2/site/{}/variable/{}",
            self.config.api_url, site_uid, variable_id
        );

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&req)
            .send()
            .await
            .context("Failed to send update variable request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        // DEBUG LOG
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "UPDATE VARIABLE RESPONSE Status: {}", status).unwrap();
                writeln!(f, "UPDATE VARIABLE RESPONSE Body: {}", text).unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        // Handle empty response by constructing the variable locally
        if text.trim().is_empty() || text == "null" {
            Ok(types::SiteVariable {
                id: variable_id,
                name: req.name,   // Use the new name from request
                value: req.value, // Use the new value from request
                masked: false, // Defaulting to false as we don't know the original state here easily
            })
        } else {
            let variable = serde_json::from_str::<types::SiteVariable>(&text)
                .context("Failed to parse response")?;
            Ok(variable)
        }
    }
    pub async fn update_site(
        &self,
        site_uid: &str,
        req: types::UpdateSiteRequest,
    ) -> Result<types::Site> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!("{}/api/v2/site/{}", self.config.api_url, site_uid);

        // DEBUG LOG
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "API UPDATE SITE: URL={}", url).unwrap();
                writeln!(f, "Payload: {:?}", req).unwrap();
            });

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&req)
            .send()
            .await
            .context("Failed to send update site request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        // DEBUG LOG RESPONSE
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "API RESPONSE Status: {}", status).unwrap();
                writeln!(f, "API RESPONSE Body: {}", text).unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let site =
            serde_json::from_str::<types::Site>(&text).context("Failed to parse response")?;
        Ok(site)
    }
}
