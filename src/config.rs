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
    pub sophos: SophosConfig,
}

#[derive(Clone, Debug)]
pub struct SophosConfig {
    pub partner_id: String,
    pub client_id: String,
    pub secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        // Datto Config
        let api_url = env::var("DATTO_API_URL").context("DATTO_API_URL must be set")?;
        let api_key = env::var("DATTO_API_KEY").context("DATTO_API_KEY must be set")?;
        let secret_key = env::var("DATTO_SECRET_KEY").context("DATTO_SECRET_KEY must be set")?;

        let datto_config = DattoConfig {
            api_url,
            api_key,
            secret_key,
        };

        // RocketCyber Config
        let rocket_url = env::var("ROCKET_CYBER_URL").context("ROCKET_CYBER_URL must be set")?;
        let rocket_secret =
            env::var("ROCKET_CYBER_SECRET").context("ROCKET_CYBER_SECRET must be set")?;

        let rocket_config = RocketCyberConfig {
            api_url: rocket_url,
            api_key: rocket_secret,
        };

        // Sophos Config
        let partner_id = env::var("SOPHOS_PARTER_ID").context("SOPHOS_PARTER_ID must be set")?;
        let client_id = env::var("SOPHOS_CLIENT_ID").context("SOPHOS_CLIENT_ID must be set")?;
        let secret = env::var("SOPHOS_SECRET").context("SOPHOS_SECRET must be set")?;

        let sophos_config = SophosConfig {
            partner_id,
            client_id,
            secret,
        };

        Ok(Self {
            datto: datto_config,
            rocket: rocket_config,
            sophos: sophos_config,
        })
    }
}
