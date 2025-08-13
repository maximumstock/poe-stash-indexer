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
    std::env::var(name).unwrap_or_else(|_| panic!("Missing environment variable {name}"))
}

#[allow(dead_code)]
fn ensure_int_from_env(name: &str) -> u32 {
    ensure_string_from_env(name).parse().unwrap()
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
