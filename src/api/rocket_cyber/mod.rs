pub mod incidents;
pub mod types;

use crate::config::RocketCyberConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RocketCyberClient {
    pub(crate) client: Client,
    pub(crate) config: RocketCyberConfig,
}

impl RocketCyberClient {
    pub fn new(config: RocketCyberConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, config })
    }
}
