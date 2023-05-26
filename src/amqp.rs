use crate::models::Settings;
use amiquip::Connection;
use color_eyre::{eyre::eyre, Result};

pub fn connect(settings: Settings) -> Result<Connection> {
    let nice_err = |e| eyre!("Failed to connect to RabbitMQ: {e}");
    if settings.environment == "LOCAL" {
        Ok(Connection::insecure_open(&settings.amqp_dsn).map_err(nice_err)?)
    } else {
        Ok(Connection::open(&settings.amqp_dsn).map_err(nice_err)?)
    }
}
