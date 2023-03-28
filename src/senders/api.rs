use crate::models::SenderConfig;

use super::Sender;
use color_eyre::{eyre::eyre, Result};

pub struct APISender {
    pub url: String,
}

impl TryFrom<SenderConfig> for APISender {
    type Error = color_eyre::Report;

    fn try_from(config: SenderConfig) -> Result<Self> {
        match config {
            SenderConfig::API(config) => Ok(APISender { url: config.url }),
            _ => Err(eyre!("Invalid sender config type, expected API")),
        }
    }
}

impl Sender for APISender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        println!("{:?}", transactions);

        let resp = client.post(&self.url).body(transactions).send()?;
        println!("{}", resp.status());

        Ok(())
    }
}
