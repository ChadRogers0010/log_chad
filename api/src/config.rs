use std::net::Ipv4Addr;

use serde::Deserialize;

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub address: Ipv4Addr,
    pub port: u16,
    pub log_level: String,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let builder = config::Config::builder()
            .set_default("ADDRESS", "127.0.0.1")?
            .set_default("PORT", 3000u16)?
            .set_default("LOG_LEVEL", "info")?
            .set_default("api_key", "supersecretcode")?
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("CHAD_LOG").separator("__"));

        builder.build()?.try_deserialize()
    }

    pub fn default() -> Self {
        Self {
            address: Ipv4Addr::new(127, 0, 0, 1),
            port: 3000u16,
            log_level: String::from("info"),
            api_key: Some(String::from("supersecretcode")),
        }
    }
}
