use trade_common::secret::SecretString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartMode {
    Resume,
    Fresh,
}
impl RestartMode {
    fn from_env() -> RestartMode {
        let env_value = std::env::var("RESTART_MODE").unwrap_or("".to_string());

        if env_value.to_lowercase().eq("resume") {
            return RestartMode::Resume;
        } else {
            RestartMode::Fresh
        }
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub rabbitmq: Option<RabbitMqConfig>,
    pub s3: Option<S3Config>,
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
            client_id: ensure_string_from_env("POE_CLIENT_ID"),
            client_secret: SecretString::new(ensure_string_from_env("POE_CLIENT_SECRET")),
            developer_mail: SecretString::new(ensure_string_from_env("POE_DEVELOPER_MAIL")),
            restart_mode: RestartMode::from_env(),
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
            let producer_routing_key = read_string_from_env("RABBITMQ_PRODUCER_ROUTING_KEY")
                .unwrap_or("poe-stash-indexer".into());

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
    pub bucket_name: String,
    pub region: String,
}

impl S3Config {
    pub fn from_env() -> Result<Option<S3Config>, std::env::VarError> {
        if let Ok(string) = std::env::var("S3_SINK_ENABLED") {
            if string.to_lowercase().eq("false") || string.eq("0") {
                return Ok(None);
            }

            let bucket_name = ensure_string_from_env("S3_SINK_BUCKET_NAME");
            let region = ensure_string_from_env("S3_SINK_REGION");

            Ok(Some(S3Config {
                bucket_name,
                region,
            }))
        } else {
            Ok(None)
        }
    }
}
