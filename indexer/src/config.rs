use config::{Config, ConfigError, File};
use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "./config/config.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub filter: Filter,
    pub restart_mode: RestartMode,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Filter {
    pub item_categories: Option<Vec<String>>,
    pub leagues: Option<Vec<String>>,
}
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum RestartMode {
    Resume,
    Fresh,
}

impl Configuration {
    pub fn read() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(CONFIG_FILE_PATH))?;
        s.try_into()
    }

    #[allow(dead_code)]
    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder::new()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            filter: Filter {
                item_categories: None,
                leagues: None,
            },
            restart_mode: RestartMode::Resume,
        }
    }
}

pub struct ConfigurationBuilder {
    #[allow(dead_code)]
    configuration: Configuration,
}

impl ConfigurationBuilder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ConfigurationBuilder::default()
    }

    #[allow(dead_code)]
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.configuration.filter.item_categories = Some(categories);
        self
    }

    #[allow(dead_code)]
    pub fn with_restart_mode(mut self, restart_mode: RestartMode) -> Self {
        self.configuration.restart_mode = restart_mode;
        self
    }

    #[allow(dead_code)]
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
    use super::{ConfigurationBuilder, RestartMode};
    #[test]
    fn test_configuration_builder_with_categories() {
        let configuration = ConfigurationBuilder::new()
            .with_categories(vec!["currency".into(), "maps".into()])
            .build();
        assert_eq!(
            configuration.filter.item_categories,
            Some(vec!["currency".to_string(), "maps".to_string()])
        );
    }

    #[test]
    fn test_configuration_builder_with_restart_mode() {
        let configuration = ConfigurationBuilder::new()
            .with_restart_mode(RestartMode::Fresh)
            .build();
        assert_eq!(configuration.restart_mode, RestartMode::Fresh);
    }

    #[test]
    fn test_default_restart_mode_is_resume() {
        let configuration = ConfigurationBuilder::new().build();
        assert_eq!(configuration.restart_mode, RestartMode::Resume);
    }
}
