use amiquip::Connection;
use atalanta::senders::{APISender, AmexSender, SFTPSender};
use chrono::Duration;
use color_eyre::{eyre::eyre, Result};

use atalanta::configuration::{load_distributor_config, load_settings};
use atalanta::initialise::startup;
use atalanta::models::DistributorConfig;
use atalanta::providers::*;
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
        "wasabi-club" => {
            let consumer = OneShotConsumer { channel };
            let formatter = WasabiFormatter {};
            let sender = SFTPSender {
                host: "sftp://wasabi.com".to_owned(),
                port: 22,
            };

            start_consuming(consumer, formatter, sender)?;
        }
        "iceland-bonus-card" => {
            let consumer = OneShotConsumer { channel };
            let formatter = IcelandFormatter {};
            let sender = SFTPSender {
                host: "sftp://wasabi.com".to_owned(),
                port: 22,
            };

            start_consuming(consumer, formatter, sender)?;
        }
        "visa-auth" => {
            let consumer = InstantConsumer { config, channel };
            let formatter = VisaAuthFormatter {};
            let sender = APISender {
                url: "http://192.168.50.70:9090/auth_transactions/visa".to_owned(),
            };

            start_consuming(consumer, formatter, sender)?;
        }
        "visa-settlement" => {
            let consumer = DelayConsumer {
                config,
                channel,
                delay: Duration::seconds(10),
            };
            let formatter = VisaSettlementFormatter {};
            let sender = APISender {
                url: "http://192.168.50.70:9090/auth_transactions/visa".to_owned(),
            };

            start_consuming(consumer, formatter, sender)?;
        }
        "amex-auth" => {
            let consumer = InstantConsumer { config, channel };
            let formatter = AmexAuthFormatter {};
            let sender = AmexSender {
                url: "http://192.168.50.70:9090/auth_transactions".to_owned(),
            };

            start_consuming(consumer, formatter, sender)?;
        }

        _ => return Err(eyre!("No process available for {}", config.provider_slug)),
    }

    connection.close()?;

    Ok(())
}
