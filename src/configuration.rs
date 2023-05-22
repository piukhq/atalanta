use crate::models::{DistributorConfig, Settings, TransactorConfig};
use color_eyre::{eyre::eyre, Result};
use std::fs;
use toml;
use tracing::{debug, info};

pub fn load_transactor_config(settings: &Settings) -> Result<TransactorConfig> {
    info!(?settings.config_file_path, "reading transactor config");
    let metadata = std::fs::metadata(&settings.config_file_path)?;
    debug!(?metadata, "transactor config file");
    let contents = fs::read_to_string(&settings.config_file_path)?;
    toml::from_str(&contents).map_err(|e| {
        eyre!(
            "failed to load transactor config from {}:\n{}",
            settings.config_file_path.to_string_lossy(),
            e
        )
    })
}

pub fn load_distributor_config(settings: &Settings) -> Result<DistributorConfig> {
    info!(?settings.config_file_path, "reading distributor config");
    let metadata = std::fs::metadata(&settings.config_file_path)?;
    debug!(?metadata, "distributor config file");
    let contents = fs::read_to_string(&settings.config_file_path)?;
    toml::from_str(&contents).map_err(|e| {
        eyre!(
            "failed to load distributor config from {}:\n{}",
            settings.config_file_path.to_string_lossy(),
            e
        )
    })
}

pub fn load_settings() -> Result<Settings> {
    info!("reading settings from environment");
    let env_settings = envy::from_env::<Settings>()?;

    Ok(env_settings)
}
