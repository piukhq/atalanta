use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use color_eyre::Result;
use futures::stream::StreamExt;

#[tokio::main]
async fn send_to_blob_storage(data: String) -> Result<()> {
    let account: String = "binkuksouthdev".to_owned();
    let access_key =
        "L/xU6NZswZAJbFhKjIGr0feakhY8QsCw4oUuj6bXNfxhWQv2caNkDo8czIu05DBcaZbSL7vfpYGP7OZsbpXuhw=="
            .to_owned();
    let container: String = "harmonia-imports-test/wasabi".to_owned();
    let blob_name: String = "wasabi".to_owned();

    let storage_credentials = StorageCredentials::access_key(account.clone(), access_key);
    let blob_client =
        ClientBuilder::new(account, storage_credentials).blob_client(&container, blob_name);

    println!("upload blob: {data}");

    blob_client
        .put_block_blob(data)
        .content_type("text/plain")
        .await?;

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
        
    println!("result: {:?}", result);
    
    Ok(())
}

fn main() -> Result<()> {
    send_to_blob_storage("Some test data for harmonia perf testing".to_owned())?;

    Ok(())
}
