use crate::config::SophosConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    // refresh_token: String, // Not using refresh token yet, grant_type is client_credentials
    // token_type: String,
    // expires_in: u64,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Fields used in Debug or might be used later
#[serde(rename_all = "camelCase")]
struct WhoAmIResponse {
    id: String,
    id_type: String,
    api_hosts: ApiHosts,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ApiHosts {
    global: String,
}

#[derive(Clone, Debug)]
pub struct SophosClient {
    pub(crate) client: Client,
    pub(crate) config: SophosConfig,
    pub(crate) access_token: Option<String>,
}

impl SophosClient {
    pub fn new(config: SophosConfig) -> Result<Self> {
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
        let url = "https://id.sophos.com/api/v2/oauth2/token";

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.secret),
            ("scope", "token"),
        ];

        let response = self
            .client
            .post(url)
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

    pub async fn whoami(&self) -> Result<String> {
        let url = "https://api.central.sophos.com/whoami/v1";

        // Ensure we have a token
        let token = self.access_token.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to send whoami request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Whoami failed: {} - {}", status, text);
        }

        let whoami_response = response
            .json::<WhoAmIResponse>()
            .await
            .context("Failed to parse whoami response")?;
        println!("Whoami response: {:#?}", whoami_response);

        Ok(whoami_response.id)
    }

    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Tenant> {
        let url = format!(
            "https://api.central.sophos.com/partner/v1/tenants/{}",
            tenant_id
        );
        let token = self.access_token.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Partner-ID", &self.config.partner_id)
            .send()
            .await
            .context("Failed to send get_tenant request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Get tenant failed: {} - {}", status, text);
        }

        let tenant = response
            .json::<Tenant>()
            .await
            .context("Failed to parse tenant response")?;

        Ok(tenant)
    }

    pub async fn get_tenants(&self) -> Result<Vec<Tenant>> {
        let url = "https://api.central.sophos.com/partner/v1/tenants";
        let token = self.access_token.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Partner-ID", &self.config.partner_id)
            .send()
            .await
            .context("Failed to send get_tenants request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Get tenants failed: {} - {}", status, text);
        }

        let response_json = response
            .json::<TenantsResponse>()
            .await
            .context("Failed to parse tenants response")?;

        Ok(response_json.items)
    }

    pub async fn get_cases(&self, tenant_id: &str, data_region: &str) -> Result<Vec<Case>> {
        let url = format!(
            "https://api-{}.central.sophos.com/cases/v1/cases",
            data_region
        );
        let token = self.access_token.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to send get_cases request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Get cases failed: {} - {}", status, text);
        }

        let response_json = response
            .json::<CasesResponse>()
            .await
            .context("Failed to parse cases response")?;

        Ok(response_json.items)
    }

    pub async fn get_endpoints(
        &self,
        tenant_id: &str,
        data_region: &str,
        hostname_contains: &str,
    ) -> Result<Vec<Endpoint>> {
        let url = format!(
            "https://api-{}.central.sophos.com/endpoint/v1/endpoints",
            data_region
        );
        let token = self.access_token.as_ref().context("Not authenticated")?;

        let params = [("hostnameContains", hostname_contains)];

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", tenant_id)
            .query(&params)
            .send()
            .await
            .context("Failed to send get_endpoints request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Get endpoints failed: {} - {}", status, text);
        }

        let response_json = response
            .json::<EndpointsResponse>()
            .await
            .context("Failed to parse endpoints response")?;

        Ok(response_json.items)
    }
    pub async fn start_scan(
        &self,
        tenant_id: &str,
        data_region: &str,
        endpoint_id: &str,
    ) -> Result<()> {
        let url = format!(
            "https://api-{}.central.sophos.com/endpoint/v1/endpoints/{}/scans",
            data_region, endpoint_id
        );
        let token = self.access_token.as_ref().context("Not authenticated")?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to send start_scan request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Start scan failed: {} - {}", status, text);
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub data_region: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TenantsResponse {
    items: Vec<Tenant>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Case {
    pub id: String,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CasesResponse {
    items: Vec<Case>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointHealth {
    pub overall: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointIsolation {
    pub is_isolated: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub id: String,
    pub hostname: String,
    pub health: Option<EndpointHealth>,
    pub isolation: Option<EndpointIsolation>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EndpointsResponse {
    items: Vec<Endpoint>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_sophos_auth_and_whoami() -> Result<()> {
        let config = Config::from_env()?;
        let mut client = SophosClient::new(config.sophos)?;

        client
            .authenticate()
            .await
            .context("Authentication failed")?;
        assert!(client.access_token.is_some());

        let id = client.whoami().await.context("Whoami failed")?;
        println!("Authenticated as ID: {}", id);

        Ok(())
    }
}
