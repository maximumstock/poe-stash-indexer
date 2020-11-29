use config::{Config, ConfigError, File};
use serde::Deserialize;

const CONFIG_FILE_PATH: &str = "./indexer/config/config.toml";

#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub filter: Filter,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Filter {
    pub item_categories: Option<Vec<String>>,
    pub leagues: Option<Vec<String>>,
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

    pub fn has_categories_filter(&self) -> bool {
        self.filter.item_categories.is_some()
    }

    pub fn has_leagues_filter(&self) -> bool {
        self.filter.leagues.is_some()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            filter: Filter {
                item_categories: None,
                leagues: None,
            },
        }
    }
}

pub struct ConfigurationBuilder {
    configuration: Configuration,
}

impl ConfigurationBuilder {
    pub fn new() -> Self {
        ConfigurationBuilder::default()
    }

    pub fn with_item_categories(mut self, categories: Vec<String>) -> Self {
        self.configuration.filter.item_categories = Some(categories);
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
    fn test_configuration_builder_include() {
        let configuration = ConfigurationBuilder::new()
            .with_item_categories(vec!["currency".into(), "maps".into()])
            .build();
        assert_eq!(
            configuration.filter.item_categories,
            Some(vec!["currency".to_string(), "maps".to_string()])
        );
    }
}
