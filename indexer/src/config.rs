use config::{Config, ConfigError, File};
use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "./indexer/config/config.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub exclude: Vec<String>,
}

impl Configuration {
    pub fn read() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(CONFIG_FILE_PATH))?;
        s.try_into()
    }

    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder::new()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self { exclude: vec![] }
    }
}

pub struct ConfigurationBuilder {
    configuration: Configuration,
}

impl ConfigurationBuilder {
    pub fn new() -> Self {
        ConfigurationBuilder::default()
    }

    pub fn with_exclude(mut self, exclude: Vec<String>) -> Self {
        self.configuration.exclude = exclude;
        self
    }

    pub fn build(self) -> Configuration {
        self.configuration
    }
}

impl Default for ConfigurationBuilder {
    fn default() -> Self {
        ConfigurationBuilder {
            configuration: Configuration::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Configuration, ConfigurationBuilder};
    #[test]
    fn test_configuration_builder_exclude() {
        let configuration = ConfigurationBuilder::new()
            .with_exclude(vec!["currency".into(), "maps".into()])
            .build();
        assert_eq!(
            configuration.exclude,
            vec!["currency".to_string(), "maps".to_string()]
        );
    }

    #[test]
    fn test_configuration_read_default_config() {
        let default_config = Configuration::read().expect("Reading default configuration failed");
        assert_eq!(default_config.exclude.len(), 0);
    }
}
