use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use color_eyre::Result;
//use tokio::runtime::Handle;

async fn send_to_blob_storage(data: &str) -> Result<()>{
    //let handle = Handle::current();
    //handle.enter();
    
    let account: String = "binkuksouthdev".to_string();
    let access_key = "L/xU6NZswZAJbFhKjIGr0feakhY8QsCw4oUuj6bXNfxhWQv2caNkDo8czIu05DBcaZbSL7vfpYGP7OZsbpXuhw==".to_string();
    let container: String = "harmonia-imports-test".to_string();
    let blob_name:String = "wasabi".to_string();
    
    let storage_credentials = StorageCredentials::Key(account.clone(), access_key);
    let blob_client = ClientBuilder::new(account, storage_credentials).blob_client(&container, blob_name);

    blob_client.put_block_blob("testing blobs").content_type("text/plain").await?;
    println!("upload blob: {data}");
    Ok(())
}

fn send_blob(data: String) -> Result<()> {
    let result = futures::executor::block_on(send_to_blob_storage(&data));

    println!("{:?}", result);
    Ok(())
}

fn main() -> Result<()> {
    send_blob("Some test data for harmonia perf testing".to_string())?;

    Ok(())
}
