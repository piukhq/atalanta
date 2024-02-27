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
    macro_rules! init_and_start_consuming {
        ($consumer:ident, $formatter:ident, $sender:ident) => {
            init_and_start_consuming::<
                consumers::$consumer::Consumer,
                formatters::$formatter::Formatter,
                senders::$sender::Sender,
            >(settings, config, None)?
        };

        ($consumer:ident, $formatter:ident, $sender:ident, $delay_seconds:expr) => {
            init_and_start_consuming::<
                consumers::$consumer::Consumer,
                formatters::$formatter::Formatter,
                senders::$sender::Sender,
            >(settings, config, Some(Duration::seconds($delay_seconds)))?
        };
    }

    match config.provider_slug.as_str() {
        "costa" => init_and_start_consuming!(instant, costa, api),
        "stonegate" => init_and_start_consuming!(instant, stonegate, api),
        "tgi-fridays" => init_and_start_consuming!(instant, tgi_fridays, blob),
        "wasabi-club" => init_and_start_consuming!(batch, wasabi, sftp),
        "iceland-bonus-card" => init_and_start_consuming!(batch, iceland, blob),
        "visa-auth" => init_and_start_consuming!(instant, visa_auth, api),
        "visa-settlement" => init_and_start_consuming!(delay, visa_settlement, api, 10),
        "amex-auth" => init_and_start_consuming!(instant, amex_auth, amex),
        _ => return Err(eyre!("No process available for {}", config.provider_slug)),
    }

    Ok(())
}
