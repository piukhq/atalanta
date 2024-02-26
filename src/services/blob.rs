use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use color_eyre::Result;

pub struct Credentials {
    pub account: String,
    pub access_key: String,
    pub container: String,
}

impl Credentials {
    #[must_use]
    pub const fn new(account: String, access_key: String, container: String) -> Self {
        Self {
            account,
            access_key,
            container,
        }
    }
}

/// Sends the given content to blob storage using the provided credentials.
///
/// # Errors
///
/// Returns an error if the connection to blob storage or the upload fails.
pub async fn send_to_blob_storage(content: String, credentials: &Credentials) -> Result<()> {
    let storage_credentials =
        StorageCredentials::access_key(credentials.account.clone(), credentials.access_key.clone());
    let blob_client = ClientBuilder::new(&credentials.account, storage_credentials)
        .blob_client(&credentials.container, "test");

    blob_client
        .put_block_blob(content)
        .content_type("text/plain")
        .await?;

    Ok(())
}
