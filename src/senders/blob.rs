use crate::{
    models::SenderConfig,
    services::blob::{send_to_blob_storage, Credentials},
};

use color_eyre::{eyre::eyre, Result};
use pollster::FutureExt;

/// A struct that can send messages to a blob storage.
pub struct Sender {
    pub blob_credentials: Credentials,
}

impl TryFrom<SenderConfig> for Sender {
    type Error = color_eyre::Report;

    fn try_from(config: SenderConfig) -> Result<Self> {
        match config {
            SenderConfig::Blob(config) => Ok(Self {
                blob_credentials: Credentials::new(
                    config.account,
                    config.access_key,
                    config.container,
                ),
            }),
            _ => Err(eyre!("Invalid sender config type, expected BLOB")),
        }
    }
}

impl super::Sender for Sender {
    fn send(&self, transactions: String) -> Result<()> {
        send_to_blob_storage(transactions, &self.blob_credentials).block_on()?;
        Ok(())
    }
}
