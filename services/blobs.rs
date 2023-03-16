use tokio;
use color_eyre::Result;

async fn send_to_blob_storage(data: &str) {
    let account = "binkuksouthdev";
    let access_key = "L/xU6NZswZAJbFhKjIGr0feakhY8QsCw4oUuj6bXNfxhWQv2caNkDo8czIu05DBcaZbSL7vfpYGP7OZsbpXuhw=="
    let storage_credentials = StorageCredentials::Key(account.clone(), access_key);
    let blob_client = ClientBuilder::new(account, storage_credentials).blob_client(&container, blob_name);

    blob_client.put_block_blob(data).content_type("text/plain").await?;
    println!("upload blob: {data}");
}

fn send_blob(data: String) -> Result<()> {

    futures::executor::block_on(send_to_blob_storage(data));

    Ok(())
}