use amiquip::{Connection, Exchange, Publish, Result};
use tracing::{info, debug};

#[tracing::instrument(ret)]
fn main() -> Result<()> {
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();

    debug!("Starting connection to rabbitmq");
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);
    info!("Publish a message");
    // Publish a message to the "hello" queue.
    exchange.publish(Publish::new("hello there".as_bytes(), "hello"))?;
    info!("Message published");
    connection.close()
}