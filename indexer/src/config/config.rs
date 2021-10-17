use super::user_config::UserConfiguration;

#[derive(Debug)]
pub struct Configuration {
    pub user_config: UserConfiguration,
    pub database_url: String,
    pub rabbitmq: Option<RabbitMqConfig>,
    pub metrics_port: usize,
}

impl Configuration {
    pub fn from_env() -> Result<Configuration, std::env::VarError> {
        Ok(Configuration {
            database_url: ensure_string_from_env("DATABASE_URL"),
            metrics_port: ensure_int_from_env("METRICS_PORT"),
            rabbitmq: RabbitMqConfig::from_env()?,
            user_config: UserConfiguration::read()
                .expect("Your configuration file is malformed. Please check."),
        })
    }
}

fn ensure_string_from_env(name: &str) -> String {
    std::env::var(name).expect(&format!("Missing environment variable {}", name))
}

fn ensure_int_from_env(name: &str) -> usize {
    ensure_string_from_env(name).parse().unwrap()
}

#[derive(Debug)]
pub struct RabbitMqConfig {
    pub connection_url: String,
    pub producer_routing_key: String,
}

impl RabbitMqConfig {
    pub fn from_env() -> Result<Option<RabbitMqConfig>, std::env::VarError> {
        let enabled =
            std::env::var("RABBITMQ_SINK_ENABLED").expect("Missing RABBITMQ_SINK_ENABLED");

        Ok(enabled)
            .map(|s| s.to_lowercase().eq(&"true") || s.eq(&"1"))
            .map(|enabled| {
                if enabled {
                    let connection_url =
                        std::env::var("RABBITMQ_URL").expect("Missing RABBITMQ_URL");
                    let producer_routing_key = std::env::var("RABBITMQ_PRODUCER_ROUTING_KEY")
                        .expect("Missing RABBITMQ_PRODUCER_ROUTING_KEY");

                    Some(RabbitMqConfig {
                        connection_url,
                        producer_routing_key,
                    })
                } else {
                    None
                }
            })
    }
}
