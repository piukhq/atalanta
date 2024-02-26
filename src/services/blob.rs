use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use color_eyre::Result;
use futures::stream::StreamExt;
use tokio;

pub struct BlobCredentials {
    pub account: String,
    pub access_key: String,
    pub container: String,
}

impl BlobCredentials {
    pub fn new(account: String, access_key: String, container: String) -> Self {
        Self {
            account,
            access_key,
            container,
        }
    }
}

#[tokio::main]
pub async fn send_to_blob_storage(content: String, credentials: &BlobCredentials) -> Result<()> {
    let storage_credentials = StorageCredentials::access_key(
        credentials.account.to_owned(),
        credentials.access_key.to_owned(),
    );
    let blob_client = ClientBuilder::new(&credentials.account, storage_credentials)
        .blob_client(&credentials.container, "test");

    blob_client
        .put_block_blob(content)
        .content_type("text/plain")
        .await?;

    Ok(())
}

#[tokio::main]
pub async fn file_from_blob_storage(
    container: String,
    credentials: &BlobCredentials,
) -> Result<()> {
    println!("TODO, get files from blob storage: {}", container);
    let storage_credentials = StorageCredentials::access_key(
        credentials.account.to_owned(),
        credentials.access_key.to_owned(),
    );
    let blob_client = ClientBuilder::new(&credentials.account, storage_credentials)
        .blob_client(&credentials.container, "test");
    let mut result: Vec<u8> = vec![];

    let mut stream = blob_client.get().into_stream();
    while let Some(value) = stream.next().await {
        let mut body = value?.data;
        // For each response, we stream the body instead of collecting it all
        // into one large allocation.
        while let Some(value) = body.next().await {
            let value = value?;
            result.extend(&value);
        }
    }

    Ok(())
}
