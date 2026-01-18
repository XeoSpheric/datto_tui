pub mod types;

use crate::config::DattoAvConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use types::AgentDetail;

#[derive(Clone, Debug)]
pub struct DattoAvClient {
    pub(crate) client: Client,
    pub(crate) config: DattoAvConfig,
}

impl DattoAvClient {
    pub fn new(config: DattoAvConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, config })
    }

    /// Fetch agent details by hostname using a filter
    pub async fn get_agent_details(&self, hostname: &str) -> Result<Vec<AgentDetail>> {
        let url = format!("{}/api/AgentDetails", self.config.url);

        // Filter: {"where":{"hostname":"[INSERT HOSTNAME HERE]"}}
        // Loopback filter often passed as "filter" query param
        let filter_json = serde_json::json!({
            "where": {
                "hostname": hostname.to_lowercase()
            }
        });

        // Pass as "filter" query parameter
        let params = [("filter", filter_json.to_string())];

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("{}", self.config.secret))
            .header("Accept", "application/json")
            .query(&params)
            .send()
            .await
            .context("Failed to send get_agent_details request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Get agent details failed: {} - {}", status, text);
        }

        let agents = response
            .json::<Vec<AgentDetail>>()
            .await
            .context("Failed to parse agent details response")?;

        Ok(agents)
    }

    /// Fetch single agent detail by ID
    pub async fn get_agent_detail(&self, id: &str) -> Result<AgentDetail> {
        let url = format!("{}/api/AgentDetails/{}", self.config.url, id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("{}", self.config.secret))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send get_agent_detail request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Get agent detail failed: {} - {}", status, text);
        }

        let agent = response
            .json::<AgentDetail>()
            .await
            .context("Failed to parse agent detail response")?;

        Ok(agent)
    }

    /// Trigger a scan for an agent
    pub async fn scan_agent(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/Agents/scan", self.config.url);

        let body = serde_json::json!({
            "id": id
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("{}", self.config.secret))
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send scan_agent request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Scan agent failed: {} - {}", status, text);
        }

        Ok(())
    }

    pub async fn get_agent_alerts(&self, agent_id: &str) -> Result<Vec<types::Alert>> {
        let url = format!("{}/api/Alerts", self.config.url);

        // Filter by agentId and sort by createdOn DESC, limit 5
        let filter = serde_json::json!({
            "where": {
                "agentId": agent_id
            },
            "order": "createdOn DESC",
            "limit": 5
        });

        let query = [("filter", filter.to_string())];

        // Note: The example filter string provided by user was JSON encoded string.
        // reqwest can handle query params, but we need to ensure it's passed as 'filter={json}'

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("{}", self.config.secret))
            .header("Accept", "application/json")
            .query(&query)
            .send()
            .await
            .context("Failed to fetch alerts")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch alerts: {} - {}", status, text);
        }

        let alerts: Vec<types::Alert> = response
            .json()
            .await
            .context("Failed to parse alerts response")?;

        Ok(alerts)
    }
}
