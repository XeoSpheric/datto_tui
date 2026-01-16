use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: String,
    pub api_key: String,
    pub secret_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let api_url = env::var("DATTO_API_URL").context("DATTO_API_URL must be set")?;
        let api_key = env::var("DATTO_API_KEY").context("DATTO_API_KEY must be set")?;
        let secret_key = env::var("DATTO_SECRET_KEY").context("DATTO_SECRET_KEY must be set")?;

        Ok(Self {
            api_url,
            api_key,
            secret_key,
        })
    }
}
