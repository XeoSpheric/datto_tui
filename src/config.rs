use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct DattoConfig {
    pub api_url: String,
    pub api_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug)]
pub struct RocketCyberConfig {
    pub api_url: String,
    pub api_key: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub datto: DattoConfig,
    pub rocket: RocketCyberConfig,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let api_url = env::var("DATTO_API_URL").context("DATTO_API_URL must be set")?;
        let api_key = env::var("DATTO_API_KEY").context("DATTO_API_KEY must be set")?;
        let secret_key = env::var("DATTO_SECRET_KEY").context("DATTO_SECRET_KEY must be set")?;

        let datto_config = DattoConfig {
            api_url,
            api_key,
            secret_key,
        };

        let rocket_url = env::var("ROCKET_CYBER_URL").context("ROCKET_CYBER_URL must be set")?;
        let rocket_secret =
            env::var("ROCKET_CYBER_SECRET").context("ROCKET_CYBER_SECRET must be set")?;

        let rocket_config = RocketCyberConfig {
            api_url: rocket_url,
            api_key: rocket_secret,
        };

        Ok(Self {
            datto: datto_config,
            rocket: rocket_config,
        })
    }
}
