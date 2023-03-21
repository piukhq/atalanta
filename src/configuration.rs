use crate::models::{Config, Settings};
use color_eyre::Result;
use std::fs;
use toml;

pub fn load_config() -> Result<Config> {
    let filename = "config.toml";
    let contents = fs::read_to_string(filename)?;
    let config: Config = toml::from_str(&contents)?;

    println!("Loading config, provider slug: {}", config.provider_slug);
    println!("Transaction rate: {}", config.transactions_per_second);

    Ok(config)
}

pub fn load_settings() -> Result<Settings> {
    let env_settings = envy::from_env::<Settings>()?;

    Ok(env_settings)
}
