use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use crate::models::SenderConfig;
use uuid::Uuid;

use super::Sender;
use color_eyre::{eyre::eyre, Result};

/// A struct that can send messages to a blob storage.
pub struct BlobSender {
    pub account: String,
    pub access_key: String,
    pub container: String,
}

impl TryFrom<SenderConfig> for BlobSender {
    type Error = color_eyre::Report;

    fn try_from(config: SenderConfig) -> Result<Self> {
        match config {
            SenderConfig::Blob(config) => Ok(BlobSender {
                account: config.account,
                access_key: config.access_key,
                container: config.container
            }),
            _ => Err(eyre!("Invalid sender config type, expected BLOB")),
        }
    }
}

impl Sender for BlobSender {
    fn send(&self, transactions: String) -> Result<()> {
        println!("BLOB: {transactions:?} to {}", self.container);
        send_to_blob_storage(
            &self.account,
            &self.access_key,
            &self.container,
            transactions
        )?;
        Ok(())
    }
}


#[tokio::main]
async fn send_to_blob_storage(account: &String, access_key: &String, container: &String, data: String) -> Result<()> {
    let blob_name: String = Uuid::new_v4().to_string();

    let storage_credentials = StorageCredentials::Key(account.into(),access_key.into());
    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(container, blob_name);

    println!("upload blob: {data}");

    blob_client
        .put_block_blob(data)
        .content_type("text/plain")
        .await?;
    Ok(())
}