use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct Settings {
    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default = "default_config_path")]
    pub config_path: PathBuf,

    #[serde(default = "default_amqp_dsn")]
    pub amqp_dsn: String,
}

fn default_environment() -> String {
    String::from("LOCAL")
}

fn default_config_path() -> PathBuf {
    PathBuf::from("config.toml")
}

fn default_amqp_dsn() -> String {
    String::from("amqp://localhost:5672")
}
