use config::{Config, ConfigError, File};
use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "./indexer/config/config.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub exclude: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(CONFIG_FILE_PATH))?;
        s.try_into()
    }
}
