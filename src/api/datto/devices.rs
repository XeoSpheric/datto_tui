use super::DattoClient;
use crate::api::datto::types::{DevicesResponse, SoftwareResponse, Udf};
use anyhow::{Context, Result};

pub(crate) trait DevicesApi {
    async fn get_devices(&self, site_uid: &str, page: i32, max: i32) -> Result<DevicesResponse>;
    async fn search_devices(&self, hostname: &str) -> Result<DevicesResponse>;
    async fn update_device_udf(&self, device_uid: &str, udf: &Udf) -> Result<()>;
    async fn move_device(&self, device_uid: &str, site_uid: &str) -> Result<()>;
    async fn update_device_warranty(&self, device_uid: &str, date: Option<String>) -> Result<()>;
    async fn get_device_software(&self, device_uid: &str, page: i32, max: i32) -> Result<SoftwareResponse>;
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

    async fn search_devices(&self, hostname: &str) -> Result<DevicesResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!("{}/api/v2/account/devices", self.config.api_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .query(&[("hostname", hostname), ("max", "5")])
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        let text = response
            .text()
            .await
            .context("Failed to get response text")?;

        // Debug Log
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(
                    f,
                    "Search Devices Query: hostname={} | Status: {} | Response: {}",
                    hostname, status, text
                )
                .unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("API search request failed with status: {} - {}", status, text);
        }

        let devices_response = serde_json::from_str(&text).context("Failed to parse JSON")?;
        Ok(devices_response)
    }

    async fn update_device_udf(&self, device_uid: &str, udf: &Udf) -> Result<()> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!("{}/api/v2/device/{}/udf", self.config.api_url, device_uid);

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .json(udf)
            .send()
            .await
            .context("Failed to send UDF update request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API UDF update failed with status: {} - {}", status, text);
        }

        Ok(())
    }

    async fn move_device(&self, device_uid: &str, site_uid: &str) -> Result<()> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!(
            "{}/api/v2/device/{}/site/{}",
            self.config.api_url, device_uid, site_uid
        );

        let response = self
            .client
            .put(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send move device request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API move device failed with status: {} - {}", status, text);
        }

        Ok(())
    }

    async fn update_device_warranty(&self, device_uid: &str, date: Option<String>) -> Result<()> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!("{}/api/v2/device/{}/warranty", self.config.api_url, device_uid);

        let body = serde_json::json!({
            "warrantyDate": date
        });

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send warranty update request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API warranty update failed with status: {} - {}", status, text);
        }

        Ok(())
    }

    async fn get_device_software(&self, device_uid: &str, page: i32, max: i32) -> Result<SoftwareResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!(
            "{}/api/v2/audit/device/{}/software?page={}&max={}",
            self.config.api_url, device_uid, page, max
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send software request")?;

        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API software request failed with status: {} - {}", status, text);
        }

        let text = response
            .text()
            .await
            .context("Failed to get response text")?;

        let software_response = serde_json::from_str(&text).context("Failed to parse software JSON")?;
        Ok(software_response)
    }
}
