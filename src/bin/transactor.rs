#![warn(clippy::unwrap_used, clippy::expect_used)]

use amiquip::{Connection, Exchange, ExchangeDeclareOptions, ExchangeType, Publish};
use chrono::Utc;
use color_eyre::Result;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;
use std::time;
use uuid::Uuid;

use atalanta::configuration::{load_config, load_settings};
use atalanta::initialise::startup;
use atalanta::models::{Config, Settings, Transaction};

#[tracing::instrument(ret)]
fn main() -> Result<()> {
    startup()?;

    println!("Starting transactor!");

    let config_data: Config = load_config()?;
    let settings: Settings = load_settings()?;

    println!("Configuration: '{:?}'", config_data);

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
    println!("Delay setting - TODO: {}", delay.as_secs());
    
    let mut tx: Transaction;

    let mut queue_connection;
    if settings.environment == "LOCAL" {
        queue_connection = connect_to_local_queue()?;
    } else {
        queue_connection = connect_to_live_queue()?;
    }
    // Open a channel - None says let the library choose the channel ID.
    let channel = queue_connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    // let exchange = Exchange::direct(&channel);
    let exchange = channel.exchange_declare(
        ExchangeType::Topic,
        "transactions",
        ExchangeDeclareOptions::default(),
    )?;

    loop {
        count += 1;
        println!("Count: {}", count);

        tx = create_transaction(&config_data)?;

        // Select a payment provider based on weighted selection,
        // visa provides many more transactions than mastercard or amex
        let payment_provider = select_payment_provider()?;
        let payment_key = format!("perf-{}", payment_provider);
        let merchant_key = format!("perf-{}",config_data.merchant_slug);
        println!("Weighted payment slug choice - TODO: {}", payment_key);
        let routing_key = format!("transactions.{}.{}", payment_key, merchant_key);
        println!("routing_key: {}", routing_key);

        queue_transaction(&exchange, tx, &routing_key)?;

        if count == 5 {
            println!("Finished");
            break;
        }
    }

    queue_connection.close()?;

    Ok(count)
}

fn create_transaction(config: &Config) -> Result<Transaction> {
    return Ok(Transaction {
        amount: rand::thread_rng().gen_range(0..100),
        transaction_date: Utc::now(),
        merchant_name: config.merchant_slug.clone(),
        transaction_id: Uuid::new_v4().to_string(),
        auth_code: create_auth_code()?,
        identifier: "12345678".to_string(),
        token: "token_1234".to_string(),
    });
}

fn select_payment_provider() -> Result<String> {
    let choices = ["visa", "mastercard", "amex"];
    let weights = [5, 2, 1];
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = thread_rng();
    let provider = choices[dist.sample(&mut rng)];

    Ok(provider.to_string())
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

fn connect_to_live_queue() -> Result<Connection> {
    let connection = Connection::open("amqp://localhost:5672")?;

    Ok(connection)
}

fn queue_transaction(
    exchange: &Exchange,
    transaction: Transaction,
    routing_key: &String,
) -> Result<()> {
    // Publish a message to the "new_transaction" queue.
    exchange.publish(Publish::new(
        &rmp_serde::to_vec(&transaction).unwrap(),
        routing_key,
    ))?;

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
