use amiquip::{Connection, Exchange, Publish};
use chrono::Utc;
use color_eyre::Result;
use rand::Rng;
use atalanta::models::{Transaction, Config};
use std::fs;
use tracing::{info,};
use toml;
use uuid::Uuid;


#[tracing::instrument(ret)]
fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();

    println!("Hello from transactor!");
    let config_data: Config = load_config()?;

    println!("Configuration: '{}'", config_data.merchant_slug);

    let tx = create_transaction(config_data)?;

    queue_transaction(tx)?;

    Ok(())
}

fn load_config() -> Result<Config> {
    let filename = "wasabi-club.toml";

    let contents = fs::read_to_string(filename)?;

    let conf: Config = toml::from_str(&contents)?;

    println!("Merchant slug:'{}'", conf.merchant_slug);
    println!("{}", conf.transaction_rate);

    return Ok(conf);
}

fn create_transaction(conf: Config) -> Result<Transaction> {

    return Ok(Transaction{
        amount: rand::thread_rng().gen_range(0..100),
        transaction_date: Utc::now(),
        merchant_name: conf.merchant_slug,
        transaction_id: Uuid::new_v4().to_string(),
        auth_code: create_auth_code()?
    });
}

fn create_auth_code() -> Result<String> {
    let number = rand::thread_rng().gen_range(9..1000000);
    return Ok(format!("{:0>6}", number.to_string()));
}

fn queue_transaction(transaction: Transaction) -> Result<()> {
    // Open connection.
    let mut connection = Connection::insecure_open("amqp://localhost:5672")?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);
    info!("Publish a message");
    // Publish a message to the "new_transaction" queue.
    exchange.publish(Publish::new(&rmp_serde::to_vec(&transaction).unwrap(), "new_transaction"))?;
    info!("Message published");
    connection.close()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_auth_code_success() {
        let auth_code = create_auth_code().unwrap();
        println!("Random number = {}", auth_code);
        assert_eq!(6, auth_code.chars().count());
    }
}