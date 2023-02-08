#![warn(clippy::unwrap_used, clippy::expect_used)]

use amiquip::{Connection, Exchange, Publish};
use chrono::Utc;
use color_eyre::Result;
use rand::Rng;
use std::{thread, time};
use tracing::info;
use uuid::Uuid;

use atalanta::configuration::{load_config, load_settings};
use atalanta::initialise::startup;
use atalanta::models::{Config, Transaction, Settings};

#[tracing::instrument(ret)]
fn main() -> Result<()> {
    startup()?;

    println!("Starting transactor!");

    let config_data: Config = load_config()?;
    let settings: Settings = load_settings()?;

    println!("Configuration: '{}'", config_data.merchant_slug);

    let start = time::Instant::now();
    let transaction_count: u64 = transaction_producer(config_data, settings)?;
    let duration = start.elapsed();
    println!(
        "Final count: {}, duration = {:?}",
        transaction_count, duration
    );

    Ok(())
}

fn transaction_producer(config_data: Config, settings: Settings) -> Result<u64> {
    //Manages the process of creating raw transactions
    let mut count: u64 = 0;
    let transactions_per_minute: u64 = 60;
    let transactions_per_second: u64 = transactions_per_minute / 60;
    let delay = time::Duration::from_secs(transactions_per_second);
    let mut tx: Transaction;

    let mut queue_connection;
    if settings.environment == "LOCAL"{ 
        queue_connection = connect_to_local_queue()?;
    }
    else {
        queue_connection = connect_to_live_queue()?;

    }
    // Open a channel - None says let the library choose the channel ID.
    let channel = queue_connection.open_channel(None)?;
 
    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    loop {
        count += 1;
        println!("Count: {}", count);

        tx = create_transaction(&config_data)?;

        queue_transaction(&exchange, tx)?;

        if count == 1000 {
            println!("Finished");
            break;
        }
    }

    queue_connection.close()?;

    Ok(count)
}

fn create_transaction(conf: &Config) -> Result<Transaction> {
    return Ok(Transaction {
        amount: rand::thread_rng().gen_range(0..100),
        transaction_date: Utc::now(),
        merchant_name: conf.merchant_slug.clone(),
        transaction_id: Uuid::new_v4().to_string(),
        auth_code: create_auth_code()?,
    });
}

fn create_auth_code() -> Result<String> {
    let number = rand::thread_rng().gen_range(9..1000000);
    return Ok(format!("{:0>6}", number.to_string()));
}

fn connect_to_local_queue() -> Result<Connection> {
    // Open insecure connection for local testing only.
    let connection = Connection::insecure_open("amqp://localhost:5672")?;

    Ok(connection)
}

fn connect_to_live_queue() -> Result<(Connection)> {
    let connection = Connection::open("amqp://localhost:5672")?;

    Ok(connection)
}

fn queue_transaction(exchange: &Exchange, transaction: Transaction) -> Result<()> {
    info!("Publish a message");
    // Publish a message to the "new_transaction" queue.
    exchange.publish(Publish::new(
        &rmp_serde::to_vec(&transaction).unwrap(),
        "new_transaction",
    ))?;
    info!("Message published");

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
