use trade_common::secret::SecretString;

use crate::sinks::{postgres::PostgresConfig, rabbitmq::RabbitMqConfig, s3::S3Config};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartMode {
    Resume,
    Fresh,
}
impl RestartMode {
    fn from_env() -> RestartMode {
        let env_value = std::env::var("RESTART_MODE").unwrap_or("".to_string());

        if env_value.to_lowercase().eq("resume") {
            RestartMode::Resume
        } else {
            RestartMode::Fresh
        }
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub rabbitmq: Option<RabbitMqConfig>,
    pub s3: Option<S3Config>,
    pub postgres: Option<PostgresConfig>,
    pub metrics_port: u32,
    pub client_id: String,
    pub client_secret: SecretString,
    pub developer_mail: SecretString,
    pub restart_mode: RestartMode,
}

impl Configuration {
    pub fn from_env() -> Result<Configuration, std::env::VarError> {
        Ok(Configuration {
            metrics_port: read_int_from_env("METRICS_PORT").unwrap_or(4000),
            rabbitmq: RabbitMqConfig::from_env()?,
            s3: S3Config::from_env()?,
            postgres: PostgresConfig::from_env()?,
            client_id: ensure_string_from_env("POE_CLIENT_ID"),
            client_secret: SecretString::new(ensure_string_from_env("POE_CLIENT_SECRET")),
            developer_mail: SecretString::new(ensure_string_from_env("POE_DEVELOPER_MAIL")),
            restart_mode: RestartMode::from_env(),
        })
    }
}

pub fn ensure_string_from_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Missing environment variable {}", name))
}

pub fn read_string_from_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

#[allow(dead_code)]
fn ensure_int_from_env(name: &str) -> u32 {
    ensure_string_from_env(name).parse().unwrap()
}

pub fn read_int_from_env(name: &str) -> Option<u32> {
    std::env::var(name).map(|s| s.parse::<u32>().unwrap()).ok()
}
