use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct Settings {
    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default = "default_config_file_path")]
    pub config_file_path: PathBuf,

    #[serde(default = "default_tokens_file_path")]
    pub tokens_file_path: PathBuf,

    #[serde(default = "default_mids_file_path")]
    pub mids_file_path: PathBuf,

    #[serde(default = "default_amqp_dsn")]
    pub amqp_dsn: String,
}

fn default_environment() -> String {
    String::from("LOCAL")
}

fn default_config_file_path() -> PathBuf {
    PathBuf::from("config.toml")
}

fn default_tokens_file_path() -> PathBuf {
    PathBuf::from("files/hermes_tokens.csv")
}

fn default_mids_file_path() -> PathBuf {
    PathBuf::from("files/perf_mids.csv")
}

fn default_amqp_dsn() -> String {
    String::from("amqp://localhost:5672")
}
