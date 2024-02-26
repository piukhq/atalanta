use crate::models::{DistributorConfig, Settings, TransactorConfig};
use color_eyre::{eyre::eyre, Result};
use std::fs;
use toml;
use tracing::info;

/// Reads the transactor configuration from the file specified in `settings.config_file_path`.
///
/// # Errors
///
/// This function will return an error if the file cannot be read or if the file is not valid `TOML`.
pub fn load_transactor_config(settings: &Settings) -> Result<TransactorConfig> {
    info!(?settings.config_file_path, "reading transactor config");
    let contents = fs::read_to_string(&settings.config_file_path)?;
    toml::from_str(&contents).map_err(|e| {
        eyre!(
            "failed to load transactor config from {}:\n{}",
            settings.config_file_path.to_string_lossy(),
            e
        )
    })
}

/// Reads the distributor configuration from the file specified in `settings.config_file_path`.
///
/// # Errors
///
/// This function will return an error if the file cannot be read or if the file is not valid `TOML`.
pub fn load_distributor_config(settings: &Settings) -> Result<DistributorConfig> {
    info!(?settings.config_file_path, "reading distributor config");
    let contents = fs::read_to_string(&settings.config_file_path)?;
    toml::from_str(&contents).map_err(|e| {
        eyre!(
            "failed to load distributor config from {}:\n{}",
            settings.config_file_path.to_string_lossy(),
            e
        )
    })
}

/// Creates a [`Settings`] instance from environment variables.
///
/// # Errors
///
/// This function will return an error if the environment variables are missing or invalid.
pub fn load_settings() -> Result<Settings> {
    info!("reading settings from environment");
    let env_settings = envy::from_env::<Settings>()?;

    Ok(env_settings)
}
