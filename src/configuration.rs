use crate::models::{Config, Settings};
use color_eyre::Result;
use std::fs;
use toml;

pub fn load_config() -> Result<Config> {
    let filename = "wasabi-club.toml";

    let contents = fs::read_to_string(filename)?;

    let conf: Config = toml::from_str(&contents)?;

    println!("Loading config, merchant slug:'{}'", conf.merchant_slug);
    println!("Transaction rate: {}", conf.transaction_rate);

    Ok(conf)
}

pub fn load_settings() -> Result<Settings> {
    let env_settings = envy::from_env::<Settings>()?;

    Ok(env_settings)
}
