use super::DattoClient;
use crate::api::datto::types::{self, CreateVariableRequest, SiteVariable, UpdateVariableRequest};
use anyhow::{Context, Result};

pub(crate) trait VariablesApi {
    async fn get_site_variables(&self, site_uid: &str) -> Result<Vec<SiteVariable>>;
    async fn create_site_variable(
        &self,
        site_uid: &str,
        req: CreateVariableRequest,
    ) -> Result<SiteVariable>;
    async fn update_site_variable(
        &self,
        site_uid: &str,
        variable_id: i32,
        req: UpdateVariableRequest,
    ) -> Result<SiteVariable>;
}

impl VariablesApi for DattoClient {
    async fn get_site_variables(&self, site_uid: &str) -> Result<Vec<SiteVariable>> {
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

    async fn create_site_variable(
        &self,
        site_uid: &str,
        req: CreateVariableRequest,
    ) -> Result<SiteVariable> {
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
        if text.trim().is_empty() || text == "null" {
            Ok(SiteVariable {
                id: 0,
                name: req.name,
                value: req.value,
                masked: req.masked,
            })
        } else {
            let variable =
                serde_json::from_str::<SiteVariable>(&text).context("Failed to parse response")?;
            Ok(variable)
        }
    }

    async fn update_site_variable(
        &self,
        site_uid: &str,
        variable_id: i32,
        req: UpdateVariableRequest,
    ) -> Result<SiteVariable> {
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
            Ok(SiteVariable {
                id: variable_id,
                name: req.name,
                value: req.value,
                masked: false,
            })
        } else {
            let variable =
                serde_json::from_str::<SiteVariable>(&text).context("Failed to parse response")?;
            Ok(variable)
        }
    }
}
