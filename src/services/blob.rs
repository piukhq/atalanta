use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use color_eyre::Result;
use tokio;

pub struct BlobCredentials {
    account: String,
    access_key: String,
    container: String,
    name: String,
}

impl BlobCredentials {
    pub fn new(account: String, access_key: String, container: String, name: String) -> Self {
        Self {
            account,
            access_key,
            container,
            name,
        }
    }
}

#[tokio::main]
pub async fn send_to_blob_storage(content: String, credentials: BlobCredentials) -> Result<()> {
    // let account = "binkuksouthdev";
    // let access_key =
    //     "L/xU6NZswZAJbFhKjIGr0feakhY8QsCw4oUuj6bXNfxhWQv2caNkDo8czIu05DBcaZbSL7vfpYGP7OZsbpXuhw==";
    let storage_credentials =
        StorageCredentials::Key(credentials.account.clone(), credentials.access_key);
    let blob_client = ClientBuilder::new(credentials.account, storage_credentials)
        .blob_client(&credentials.container, credentials.name);

    blob_client
        .put_block_blob(content)
        .content_type("text/plain")
        .await?;

    Ok(())
}
