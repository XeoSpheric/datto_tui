use super::DattoClient;
use crate::api::datto::types::ActivityLogsResponse;
use anyhow::{Context, Result};

pub(crate) trait ActivityApi {
    #[allow(clippy::too_many_arguments)]
    async fn get_activity_logs(
        &self,
        page: Option<String>,
        size: i32,
        order: Option<String>,
        from: Option<String>,
        until: Option<String>,
        entities: Option<Vec<String>>,
        categories: Option<Vec<String>>,
        actions: Option<Vec<String>>,
        site_ids: Option<Vec<i32>>,
        user_ids: Option<Vec<i32>>,
    ) -> Result<ActivityLogsResponse>;
}

impl ActivityApi for DattoClient {
    async fn get_activity_logs(
        &self,
        page: Option<String>,
        size: i32,
        order: Option<String>,
        from: Option<String>,
        until: Option<String>,
        entities: Option<Vec<String>>,
        categories: Option<Vec<String>>,
        actions: Option<Vec<String>>,
        site_ids: Option<Vec<i32>>,
        user_ids: Option<Vec<i32>>,
    ) -> Result<ActivityLogsResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let mut url = format!(
            "{}/api/v2/activity-logs?size={}",
            self.config.api_url, size
        );

        if let Some(val) = page {
             if !val.is_empty() {
                url.push_str(&format!("&page={}", val));
             }
        }

        if let Some(val) = order {
            url.push_str(&format!("&order={}", val));
        }
        if let Some(val) = from {
            url.push_str(&format!("&from={}", val));
        }
        if let Some(val) = until {
            url.push_str(&format!("&until={}", val));
        }
        if let Some(vals) = entities {
            for v in vals {
                url.push_str(&format!("&entities={}", v));
            }
        }
        if let Some(vals) = categories {
            for v in vals {
                url.push_str(&format!("&categories={}", v));
            }
        }
        if let Some(vals) = actions {
            for v in vals {
                url.push_str(&format!("&actions={}", v));
            }
        }
        if let Some(vals) = site_ids {
            for v in vals {
                url.push_str(&format!("&siteIds={}", v));
            }
        }
        if let Some(vals) = user_ids {
            for v in vals {
                url.push_str(&format!("&userIds={}", v));
            }
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

        let response_body = response
            .json::<ActivityLogsResponse>()
            .await
            .context("Failed to parse JSON")?;
        Ok(response_body)
    }
}
