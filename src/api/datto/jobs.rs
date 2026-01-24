use super::DattoClient;
use crate::api::datto::types::{
    ComponentsResponse, JobResult, JobStdOutput, QuickJobRequest, QuickJobResponse,
};
use anyhow::{Context, Result};

pub(crate) trait JobsApi {
    async fn get_job_result(&self, job_uid: &str, device_uid: &str) -> Result<JobResult>;
    async fn get_job_stdout(&self, job_uid: &str, device_uid: &str) -> Result<Vec<JobStdOutput>>;
    async fn get_job_stderr(&self, job_uid: &str, device_uid: &str) -> Result<Vec<JobStdOutput>>;
    async fn get_components(&self, page: Option<i32>) -> Result<ComponentsResponse>;
    async fn run_quick_job(&self, device_uid: &str, req: QuickJobRequest) -> Result<QuickJobResponse>;
}

impl JobsApi for DattoClient {
    async fn get_components(&self, page: Option<i32>) -> Result<ComponentsResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let mut url = format!("{}/api/v2/account/components", self.config.api_url);
        
        if let Some(p) = page {
            url.push_str(&format!("?page={}", p));
        }

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send components request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        // DEBUG LOG
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "COMPONENTS RESPONSE: {}", text).unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let components = serde_json::from_str::<ComponentsResponse>(&text)
            .context("Failed to parse components JSON")?;
        Ok(components)
    }

    async fn run_quick_job(&self, device_uid: &str, req: QuickJobRequest) -> Result<QuickJobResponse> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!("{}/api/v2/device/{}/quickjob", self.config.api_url, device_uid);

        let response = self
            .client
            .put(&url)
            .bearer_auth(access_token)
            .json(&req)
            .send()
            .await
            .context("Failed to send quick job request")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        // DEBUG LOG
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "QUICK JOB RESPONSE: {}", text).unwrap();
            });

        if !status.is_success() {
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let job_response = serde_json::from_str::<QuickJobResponse>(&text)
            .context("Failed to parse quick job response")?;
        
        Ok(job_response)
    }

    async fn get_job_result(&self, job_uid: &str, device_uid: &str) -> Result<JobResult> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;

        let url = format!(
            "{}/api/v2/job/{}/results/{}",
            self.config.api_url, job_uid, device_uid
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

        let text = response.text().await.context("Failed to get response text")?;

        // DEBUG LOG
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .map(|mut f| {
                use std::io::Write;
                writeln!(f, "JOB RESULT JSON: {}", text).unwrap();
            });

        // Try to parse as single object first
        match serde_json::from_str::<JobResult>(&text) {
            Ok(res) => Ok(res),
            Err(_) => {
                // If failed, try to parse as Vec<JobResult> and take the first one
                let list = serde_json::from_str::<Vec<JobResult>>(&text).context("Failed to parse JSON as Object or Array")?;
                list.into_iter().next().context("Job result list is empty")
            }
        }
    }

    async fn get_job_stdout(&self, job_uid: &str, device_uid: &str) -> Result<Vec<JobStdOutput>> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!(
            "{}/api/v2/job/{}/results/{}/stdout",
            self.config.api_url, job_uid, device_uid
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send stdout request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let output = response
            .json::<Vec<JobStdOutput>>()
            .await
            .context("Failed to parse stdout JSON")?;
        Ok(output)
    }

    async fn get_job_stderr(&self, job_uid: &str, device_uid: &str) -> Result<Vec<JobStdOutput>> {
        let access_token = self.access_token.as_ref().context("Not authenticated")?;
        let url = format!(
            "{}/api/v2/job/{}/results/{}/stderr",
            self.config.api_url, job_uid, device_uid
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send stderr request")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status: {} - {}", status, text);
        }

        let output = response
            .json::<Vec<JobStdOutput>>()
            .await
            .context("Failed to parse stderr JSON")?;
        Ok(output)
    }
}
