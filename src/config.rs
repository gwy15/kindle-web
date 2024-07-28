use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub influx: InfluxConfig,
}

#[derive(Debug, Deserialize)]
pub struct InfluxConfig {
    pub domain: String,
    pub token: String,
    pub org: String,
    pub temp_query: String,
    pub humid_query: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
