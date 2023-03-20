use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use color_eyre::Result;

#[tokio::main]
async fn send_to_blob_storage(data: String) -> Result<()> {
    let account: String = "binkuksouthdev".to_string();
    let access_key =
        "L/xU6NZswZAJbFhKjIGr0feakhY8QsCw4oUuj6bXNfxhWQv2caNkDo8czIu05DBcaZbSL7vfpYGP7OZsbpXuhw=="
            .to_string();
    let container: String = "harmonia-imports-test".to_string();
    let blob_name: String = "wasabi".to_string();

    let storage_credentials = StorageCredentials::Key(account.clone(), access_key);
    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(&container, blob_name);

    println!("upload blob: {data}");

    blob_client
        .put_block_blob(data)
        .content_type("text/plain")
        .await?;
    Ok(())
}

fn main() -> Result<()> {
    send_to_blob_storage("Some test data for harmonia perf testing".to_owned())?;

    Ok(())
}
