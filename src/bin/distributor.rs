use amiquip::Connection;
use color_eyre::Result;
use std::time::Duration;

use atalanta::configuration::{load_config, load_settings};
use atalanta::initialise::startup;
use atalanta::models::{Config, Settings};
use atalanta::providers::*;

fn main() -> Result<()> {
    startup()?;

    let config_data: Config = load_config()?;
    let settings: Settings = load_settings()?;

    println!("Distributing {} transactions.", config_data.merchant_slug);

    routing(settings, config_data)?;
    // transaction_consumer()?;

    Ok(())
}

fn routing(settings: Settings, config_data: Config) -> Result<()> {
    let mut provider = String::with_capacity(25);

    if settings.environment == "LOCAL" {
        provider = config_data.deployed_slug;
    } else {
        println!("Live configuration and settings missing")
    }

    // Create rabbitmq connection and channel
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;
    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    match provider.as_str() {
        "wasabi-club" => {
            let consumer = DelayConsumer {
                channel,
                delay: Duration::from_secs(10),
            };
            let formatter = WasabiFormatter {};
            let sender = SFTPSender {
                host: "sftp://wasabi.com".to_string(),
                port: 22,
            };

            send_message(consumer, formatter, sender)?;
        }
        "iceland-bonus-card" => {
            let consumer = DelayConsumer {
                channel,
                delay: Duration::from_secs(10),
            };
            let formatter = IcelandFormatter {};
            let sender = SFTPSender {
                host: "sftp://wasabi.com".to_string(),
                port: 22,
            };

            send_message(consumer, formatter, sender)?;
        }
        "visa-auth" => {
            let consumer = InstantConsumer { channel };
            let formatter = VisaAuthFormatter {};
            let sender = APISender {
                url: "http://192.168.50.70:9090/auth_transactions/visa".to_string(),
            };

            send_message(consumer, formatter, sender)?;
        }
        "visa-settlement" => {
            let consumer = DelayConsumer {
                channel,
                delay: Duration::from_secs(10),
            };
            let formatter = VisaSettlementFormatter {};
            let sender = APISender {
                url: "http://192.168.50.70:9090/auth_transactions/visa".to_string(),
            };

            send_message(consumer, formatter, sender)?;
        }
        "amex-auth" => {
            let consumer = InstantConsumer { channel };
            let formatter = AmexAuthFormatter {};
            let sender = AmexSender {
                url: "http://192.168.50.70:9090/auth_transactions".to_string(),
            };

            send_message(consumer, formatter, sender)?;
        }

        _ => panic!("No process available for {}", provider),
    }

    connection.close()?;

    Ok(())
}
