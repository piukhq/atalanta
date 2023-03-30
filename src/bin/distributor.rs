use amiquip::Connection;
use atalanta::consumers::{start_consuming, BatchConsumer, DelayConsumer, InstantConsumer};
use atalanta::senders::{APISender, AmexSender, SFTPSender};
use chrono::Duration;
use color_eyre::{eyre::eyre, Result};

use atalanta::configuration::{load_distributor_config, load_settings};
use atalanta::formatters::*;
use atalanta::initialise::startup;
use atalanta::models::DistributorConfig;
use tracing::info;

fn main() -> Result<()> {
    startup()?;

    let settings = load_settings()?;
    let config = load_distributor_config(&settings)?;

    info!(config.provider_slug, "distributing transactions");

    start_distributor(config)?;

    Ok(())
}

fn start_distributor(config: DistributorConfig) -> Result<()> {
    // Create rabbitmq connection and channel
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;
    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    match config.provider_slug.as_str() {
        "costa" => {
            let consumer = InstantConsumer {
                config: config.clone(),
                channel,
            };
            let sender = APISender::try_from(config.sender)?;
            start_consuming::<_, CostaFormatter, _>(consumer, sender)?;
        }
        "wasabi-club" => {
            let consumer = BatchConsumer {
                config: config.clone(),
                channel,
            };
            let sender = SFTPSender::try_from(config.sender)?;
            start_consuming::<_, WasabiFormatter, _>(consumer, sender)?;
        }
        "iceland-bonus-card" => {
            let consumer = BatchConsumer {
                config: config.clone(),
                channel,
            };
            let sender = SFTPSender::try_from(config.sender)?;
            start_consuming::<_, IcelandFormatter, _>(consumer, sender)?;
        }
        "visa-auth" => {
            let consumer = InstantConsumer {
                config: config.clone(),
                channel,
            };
            let sender = APISender::try_from(config.sender)?;
            start_consuming::<_, VisaAuthFormatter, _>(consumer, sender)?;
        }
        "visa-settlement" => {
            let consumer = DelayConsumer {
                config: config.clone(),
                channel,
                delay: Duration::seconds(10),
            };
            let sender = APISender::try_from(config.sender)?;
            start_consuming::<_, VisaSettlementFormatter, _>(consumer, sender)?;
        }
        "amex-auth" => {
            let consumer = InstantConsumer {
                config: config.clone(),
                channel,
            };
            let sender = AmexSender::try_from(config.sender)?;
            start_consuming::<_, AmexAuthFormatter, _>(consumer, sender)?;
        }

        _ => return Err(eyre!("No process available for {}", config.provider_slug)),
    }

    connection.close()?;

    Ok(())
}
