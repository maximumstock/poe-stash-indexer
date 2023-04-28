#[derive(Debug)]
pub struct Config {
    pub(crate) metrics_port: u32,
    pub(crate) db_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            metrics_port: int_from_env("METRICS_PORT")?,
            db_url: str_from_env("TRADE_API_DATABASE_URL")?
        })
    }
}

fn int_from_env(key: &str) -> Result<u32, Box<dyn std::error::Error>> {
    str_from_env(key)?.parse::<u32>().map_err(|e| e.into())
}

fn str_from_env(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(std::env::var(key).unwrap_or_else(|_| panic!("{} environment variable", key)))
}

