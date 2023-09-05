use trade_common::secret::SecretString;

#[derive(Debug)]
pub struct Configuration {
    pub s3: Option<S3Config>,
    pub client_id: String,
    pub client_secret: SecretString,
    pub developer_mail: SecretString,
}

impl Configuration {
    pub fn from_env() -> anyhow::Result<Configuration, std::env::VarError> {
        Ok(Configuration {
            s3: S3Config::from_env()?,
            client_id: ensure_string_from_env("POE_CLIENT_ID"),
            client_secret: SecretString::new(ensure_string_from_env("POE_CLIENT_SECRET")),
            developer_mail: SecretString::new(ensure_string_from_env("POE_DEVELOPER_MAIL")),
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

            let connection_url = ensure_string_from_env("RABBITMQ_URL");
            let producer_routing_key = ensure_string_from_env("RABBITMQ_PRODUCER_ROUTING_KEY");

            Ok(Some(RabbitMqConfig {
                connection_url,
                producer_routing_key,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: SecretString,
    pub bucket_name: String,
    pub region: String,
}

impl S3Config {
    pub fn from_env() -> Result<Option<S3Config>, std::env::VarError> {
        if let Ok(string) = std::env::var("S3_SINK_ENABLED") {
            if string.to_lowercase().eq("false") || string.eq("0") {
                return Ok(None);
            }

            let access_key = ensure_string_from_env("S3_SINK_ACCESS_KEY");
            let secret_key = SecretString::new(ensure_string_from_env("S3_SINK_SECRET_KEY"));
            let bucket_name = ensure_string_from_env("S3_SINK_BUCKET_NAME");
            let region = ensure_string_from_env("S3_SINK_REGION");

            Ok(Some(S3Config {
                access_key,
                secret_key,
                bucket_name,
                region,
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
