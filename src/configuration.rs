use crate::models::{DistributorConfig, Settings, TransactorConfig};
use color_eyre::Result;
use std::fs;
use toml;

pub fn load_transactor_config(settings: &Settings) -> Result<TransactorConfig> {
    let contents = fs::read_to_string(&settings.config_path)?;
    Ok(toml::from_str(&contents)?)
}

pub fn load_distributor_config(settings: &Settings) -> Result<DistributorConfig> {
    let contents = fs::read_to_string(&settings.config_path)?;
    Ok(toml::from_str(&contents)?)
}

pub fn load_settings() -> Result<Settings> {
    let env_settings = envy::from_env::<Settings>()?;

    Ok(env_settings)
}
