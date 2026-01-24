use super::DattoClient;
use crate::api::datto::types::{self, SitesResponse, UpdateSiteRequest};
use anyhow::{Context, Result};

pub(crate) trait SitesApi {
    async fn get_sites(
        &self,
        page: i32,
        max: i32,
        site_name: Option<String>,
    ) -> Result<SitesResponse>;

    async fn update_site(&self, site_uid: &str, req: UpdateSiteRequest) -> Result<types::Site>;

    async fn get_site(&self, site_uid: &str) -> Result<types::Site>;
    async fn get_site_open_alerts(&self, site_uid: &str, page: i32, max: i32) -> Result<types::OpenAlertsResponse>;
}

impl SitesApi for DattoClient {
    async fn get_sites(
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

    async fn update_site(
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

    async fn get_site(&self, site_uid: &str) -> Result<types::Site> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!("{}/api/v2/site/{}", self.config.api_url, site_uid);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to send get site request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let site = response.json::<types::Site>().await.context("Failed to parse site response")?;
        Ok(site)
    }

    async fn get_site_open_alerts(&self, site_uid: &str, page: i32, max: i32) -> Result<types::OpenAlertsResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!("{}/api/v2/site/{}/alerts/open?page={}&max={}", self.config.api_url, site_uid, page, max);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to send site alerts request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let alerts_response = response
            .json::<types::OpenAlertsResponse>()
            .await
            .context("Failed to parse site alerts response")?;
        Ok(alerts_response)
    }
}
