use config::{Config, ConfigError, File};
use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "./config/config.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct UserConfiguration {
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

impl UserConfiguration {
    pub fn read() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        // Its fine if the file does not exist
        let _ = s.merge(File::with_name(CONFIG_FILE_PATH));
        Ok(s.try_into().unwrap_or_default())
    }

    #[allow(dead_code)]
    pub fn builder() -> UserConfigurationBuilder {
        UserConfigurationBuilder::new()
    }
}

impl Default for UserConfiguration {
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

pub struct UserConfigurationBuilder {
    #[allow(dead_code)]
    configuration: UserConfiguration,
}

impl UserConfigurationBuilder {
    #[allow(dead_code)]
    pub fn new() -> Self {
        UserConfigurationBuilder::default()
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
    pub fn build(self) -> UserConfiguration {
        self.configuration
    }
}

impl Default for UserConfigurationBuilder {
    fn default() -> Self {
        UserConfigurationBuilder {
            configuration: UserConfiguration::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{RestartMode, UserConfigurationBuilder};
    #[test]
    fn test_configuration_builder_with_categories() {
        let configuration = UserConfigurationBuilder::new()
            .with_categories(vec!["currency".into(), "maps".into()])
            .build();
        assert_eq!(
            configuration.filter.item_categories,
            Some(vec!["currency".to_string(), "maps".to_string()])
        );
    }

    #[test]
    fn test_configuration_builder_with_restart_mode() {
        let configuration = UserConfigurationBuilder::new()
            .with_restart_mode(RestartMode::Fresh)
            .build();
        assert_eq!(configuration.restart_mode, RestartMode::Fresh);
    }

    #[test]
    fn test_default_restart_mode_is_resume() {
        let configuration = UserConfigurationBuilder::new().build();
        assert_eq!(configuration.restart_mode, RestartMode::Resume);
    }
}
