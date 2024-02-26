use atalanta::consumers::{
    start_consuming, BatchConsumer, Consumer, DelayConsumer, InstantConsumer,
};
use atalanta::senders::{APISender, AmexSender, BlobSender, SFTPSender, Sender};
use chrono::Duration;
use color_eyre::{eyre::eyre, Result};

use atalanta::configuration::{load_distributor_config, load_settings};
use atalanta::initialise::startup;
use atalanta::models::{DistributorConfig, Settings};
use atalanta::{amqp, formatters::*};
use tracing::info;

fn main() -> Result<()> {
    startup()?;

    let settings = load_settings()?;
    let config = load_distributor_config(&settings)?;

    info!(config.provider_slug, "distributing transactions");

    start_distributor(config, settings)?;

    Ok(())
}

fn create_consumer<C: Consumer>(
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
    settings: Settings,
    config: DistributorConfig,
    delay: Option<Duration>,
) -> Result<()>
where
    C: Consumer,
    F: Formatter,
    S: Sender,
{
    let mut connection = amqp::connect(&settings)?;
    let channel = connection.open_channel(None)?;

    let consumer = create_consumer::<C>(config.clone(), channel, delay);
    let sender =
        S::try_from(config.sender).map_err(|_| eyre!("failed to create sender from config"))?;

    start_consuming::<C, F, S>(consumer, sender)?;

    connection.close()?;

    Ok(())
}

fn start_distributor(config: DistributorConfig, settings: Settings) -> Result<()> {
    match config.provider_slug.as_str() {
        "costa" => init_and_start_consuming::<InstantConsumer, CostaFormatter, APISender>(
            settings, config, None,
        )?,
        "stonegate" => init_and_start_consuming::<InstantConsumer, StonegateFormatter, APISender>(
            settings, config, None,
        )?,
        "tgi-fridays" => {
            init_and_start_consuming::<InstantConsumer, TGIFridaysFormatter, BlobSender>(
                settings, config, None,
            )?
        }
        "wasabi-club" => init_and_start_consuming::<BatchConsumer, WasabiFormatter, SFTPSender>(
            settings, config, None,
        )?,
        "iceland-bonus-card" => {
            init_and_start_consuming::<BatchConsumer, IcelandFormatter, BlobSender>(
                settings, config, None,
            )?
        }
        "visa-auth" => init_and_start_consuming::<InstantConsumer, VisaAuthFormatter, APISender>(
            settings, config, None,
        )?,
        "visa-settlement" => init_and_start_consuming::<
            DelayConsumer,
            VisaSettlementFormatter,
            APISender,
        >(settings, config, Some(Duration::seconds(10)))?,
        "amex-auth" => init_and_start_consuming::<InstantConsumer, AmexAuthFormatter, AmexSender>(
            settings, config, None,
        )?,
        _ => return Err(eyre!("No process available for {}", config.provider_slug)),
    }

    Ok(())
}
