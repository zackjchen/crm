use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    // pub host: String,
    pub port: u16,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let ret: Result<AppConfig, _> = match (
            fs::File::open("metadata.yml"),
            fs::File::open("/etc/config/metadata.yml"),
            std::env::var("METADATA_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (Err(_), Ok(reader), _) => serde_yaml::from_reader(reader),
            (Err(_), Err(_), Ok(reader)) => serde_yaml::from_reader(fs::File::open(reader)?),
            _ => bail!("Config file not found"),
        };

        Ok(ret?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_config() -> anyhow::Result<()> {
        let config = AppConfig::load()?;
        println!("{:?}", config);
        Ok(())
    }
}
