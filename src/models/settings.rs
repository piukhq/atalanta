use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct Settings {
    #[serde(default = "default_environment")]
    pub environment: String,

    #[serde(default = "default_config_path")]
    pub config_path: PathBuf,
}

fn default_environment() -> String {
    String::from("LOCAL")
}

fn default_config_path() -> PathBuf {
    PathBuf::from("config.toml")
}
