use tokio;
/// the async function we'd like to call
async fn send_to_blob_storage(data: &str) {
    println!("upload blob: {data}");
}

/// method 1: futures::executor::block_on
fn call_1() {
    futures::executor::block_on(send_to_blob_storage("futures::executor::block_on"));
}

/// method 2: tokio::main
#[tokio::main]
async fn call_2() {
    send_to_blob_storage("tokio::main").await;
}

/// method 3: tokio::runtime::Runtime::block_on
fn call_3() {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(send_to_blob_storage("tokio::runtime::Runtime::block_on"));
}

/// synchronous main calling async send_to_blob_storage function
fn main() {
    call_1();
    call_2();
    call_3();
}