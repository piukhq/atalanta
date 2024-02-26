use chrono::Duration;
use color_eyre::{eyre::eyre, Result};

use atalanta::configuration::{load_distributor_config, load_settings};
use atalanta::initialise::startup;
use atalanta::models::{DistributorConfig, Settings};
use atalanta::{amqp, consumers, formatters, senders};
use tracing::info;

fn main() -> Result<()> {
    startup()?;

    let settings = load_settings()?;
    let config = load_distributor_config(&settings)?;

    info!(config.provider_slug, "distributing transactions");

    start_distributor(config, &settings)?;

    Ok(())
}

fn create_consumer<C: consumers::Consumer>(
    config: DistributorConfig,
    channel: amiquip::Channel,
    delay: Option<Duration>,
) -> C {
    match delay {
        Some(delay) => C::new_with_delay(config, channel, delay),
        None => C::new(config, channel),
    }
}

fn init_and_start_consuming<C, F, S>(
    settings: &Settings,
    config: DistributorConfig,
    delay: Option<Duration>,
) -> Result<()>
where
    C: consumers::Consumer,
    F: formatters::Formatter,
    S: senders::Sender,
{
    let mut connection = amqp::connect(settings)?;
    let channel = connection.open_channel(None)?;

    let consumer = create_consumer::<C>(config.clone(), channel, delay);
    let sender =
        S::try_from(config.sender).map_err(|_| eyre!("failed to create sender from config"))?;

    consumers::start_consuming::<C, F, S>(&consumer, &sender)?;

    connection.close()?;

    Ok(())
}

fn start_distributor(config: DistributorConfig, settings: &Settings) -> Result<()> {
    match config.provider_slug.as_str() {
        "costa" => init_and_start_consuming::<
            consumers::instant::Consumer,
            formatters::costa::Formatter,
            senders::api::APISender,
        >(settings, config, None)?,
        "stonegate" => {
            init_and_start_consuming::<
                consumers::instant::Consumer,
                formatters::stonegate::Formatter,
                senders::api::APISender,
            >(settings, config, None)?;
        }
        "tgi-fridays" => {
            init_and_start_consuming::<
                consumers::instant::Consumer,
                formatters::tgi_fridays::Formatter,
                senders::blob::Sender,
            >(settings, config, None)?;
        }
        "wasabi-club" => {
            init_and_start_consuming::<
                consumers::batch::Consumer,
                formatters::wasabi::Formatter,
                senders::sftp::Sender,
            >(settings, config, None)?;
        }
        "iceland-bonus-card" => {
            init_and_start_consuming::<
                consumers::batch::Consumer,
                formatters::iceland::Formatter,
                senders::blob::Sender,
            >(settings, config, None)?;
        }
        "visa-auth" => {
            init_and_start_consuming::<
                consumers::instant::Consumer,
                formatters::visa::AuthFormatter,
                senders::api::APISender,
            >(settings, config, None)?;
        }
        "visa-settlement" => init_and_start_consuming::<
            consumers::delay::Consumer,
            formatters::visa::SettlementFormatter,
            senders::api::APISender,
        >(settings, config, Some(Duration::seconds(10)))?,
        "amex-auth" => {
            init_and_start_consuming::<
                consumers::instant::Consumer,
                formatters::amex::AuthFormatter,
                senders::amex::Sender,
            >(settings, config, None)?;
        }
        _ => return Err(eyre!("No process available for {}", config.provider_slug)),
    }

    Ok(())
}
