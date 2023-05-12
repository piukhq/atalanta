use amiquip::{Connection, Exchange, Publish};
use atalanta::configuration::load_settings;
use color_eyre::Result;
use tracing::{debug, info};

#[tracing::instrument(ret)]
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let settings = load_settings()?;

    debug!("Starting connection to rabbitmq");
    // Open connection.
    let mut connection = Connection::insecure_open(&settings.amqp_dsn)?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);
    info!("Publish a message");
    // Publish a message to the "hello" queue.
    exchange.publish(Publish::new("hello there".as_bytes(), "hello"))?;
    info!("Message published");
    connection.close()?;

    Ok(())
}
