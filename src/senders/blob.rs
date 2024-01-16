use crate::{
    models::SenderConfig,
    services::blob::{send_to_blob_storage, BlobCredentials},
};

use super::Sender;
use color_eyre::{eyre::eyre, Result};

/// A struct that can send messages to a blob storage.
pub struct BlobSender {
    pub blob_credentials: BlobCredentials,
}

impl TryFrom<SenderConfig> for BlobSender {
    type Error = color_eyre::Report;

    fn try_from(config: SenderConfig) -> Result<Self> {
        match config {
            SenderConfig::Blob(config) => Ok(BlobSender {
                blob_credentials: BlobCredentials::new(
                    config.account,
                    config.access_key,
                    config.container,
                ),
            }),
            _ => Err(eyre!("Invalid sender config type, expected BLOB")),
        }
    }
}

impl Sender for BlobSender {
    fn send(&self, transactions: String) -> Result<()> {
        send_to_blob_storage(transactions, &self.blob_credentials)?;
        Ok(())
    }
}
