use trade_common::secret::SecretString;

use self::user_config::UserConfiguration;

#[derive(Debug)]
pub struct Configuration {
    pub user_config: UserConfiguration,
    pub database_url: Option<String>,
    pub rabbitmq: Option<RabbitMqConfig>,
    pub metrics_port: u32,
    pub client_id: String,
    pub client_secret: SecretString,
}

impl Configuration {
    pub fn from_env() -> Result<Configuration, std::env::VarError> {
        Ok(Configuration {
            database_url: read_string_from_env("DATABASE_URL"),
            metrics_port: read_int_from_env("METRICS_PORT").unwrap_or(4000),
            rabbitmq: RabbitMqConfig::from_env()?,
            user_config: UserConfiguration::default(),
            client_id: ensure_string_from_env("POE_CLIENT_ID"),
            client_secret: SecretString::new(ensure_string_from_env("POE_CLIENT_SECRET")),
        })
    }
}

fn ensure_string_from_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Missing environment variable {}", name))
}

fn read_string_from_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

#[allow(dead_code)]
fn ensure_int_from_env(name: &str) -> u32 {
    ensure_string_from_env(name).parse().unwrap()
}

fn read_int_from_env(name: &str) -> Option<u32> {
    std::env::var(name).map(|s| s.parse::<u32>().unwrap()).ok()
}

#[derive(Debug, Clone)]
pub struct RabbitMqConfig {
    pub connection_url: String,
    pub producer_routing_key: String,
}

impl RabbitMqConfig {
    pub fn from_env() -> Result<Option<RabbitMqConfig>, std::env::VarError> {
        if let Ok(string) = std::env::var("RABBITMQ_SINK_ENABLED") {
            if string.to_lowercase().eq("false") || string.eq("0") {
                return Ok(None);
            }

            let connection_url =
                std::env::var("RABBITMQ_URL").expect("RABBITMQ_URL environment variable");
            let producer_routing_key = std::env::var("RABBITMQ_PRODUCER_ROUTING_KEY")
                .expect("RABBITMQ_PRODUCER_ROUTING_KEY environment variable");

            Ok(Some(RabbitMqConfig {
                connection_url,
                producer_routing_key,
            }))
        } else {
            Ok(None)
        }
    }
}

pub mod user_config {

    use serde::Deserialize;

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
                restart_mode: RestartMode::Fresh,
            }
        }
    }

    #[derive(Default)]
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
                .with_restart_mode(RestartMode::Resume)
                .build();
            assert_eq!(configuration.restart_mode, RestartMode::Resume);
        }

        #[test]
        fn test_default_restart_mode_is_fresh() {
            let configuration = UserConfigurationBuilder::new().build();
            assert_eq!(configuration.restart_mode, RestartMode::Fresh);
        }
    }
}
