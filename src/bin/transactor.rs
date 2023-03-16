#![warn(clippy::unwrap_used, clippy::expect_used)]

use amiquip::{Connection, Exchange, ExchangeDeclareOptions, ExchangeType, Publish};
use chrono::Utc;
use color_eyre::Result;
use csv::StringRecord;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;
use std::fs::File;
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

    let payment_card_tokens = load_payment_card_tokens(&config_data.merchant_slug)?;

    let start = time::Instant::now();
    let transaction_count: u64 = transaction_producer(config_data, settings, payment_card_tokens)?;
    let duration = start.elapsed();
    println!(
        "Final count: {}, duration = {:?}",
        transaction_count, duration
    );

    Ok(())
}

fn load_payment_card_tokens(merchant_slug: &String) -> Result<Vec<StringRecord>> {
    // Load token and slugs derived from the Hemres database
    //Only tokens related to the current retailer are loaded
    let mut tokens = Vec::new();

    let file_path = "./files/hermes_tokens.csv";
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
    .has_headers(false)
    .from_reader(file);

    for result in rdr.records() {
        let record = result?;
        if record.iter().any(|field| field == merchant_slug) {
            tokens.push(record);
        }
    }

    Ok(tokens)
}


fn transaction_producer(config_data: Config, settings: Settings, payment_card_tokens: Vec<StringRecord>) -> Result<u64> {
    //Manages the process of creating raw transactions
    let mut count: u64 = 0;
    let mut total_count: u64 = 0;

    let delay = time::Duration::from_millis(1000 / config_data.transactions_per_second);
    println!("Delay setting - TODO: {}", delay.as_millis());

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
        total_count += 1;
        println!("Count: {}", count);

        // Select a payment provider based on weighted selection,
        // visa provides many more transactions than mastercard or amex
        let payment_provider = select_payment_provider(&config_data.percentage)?;
        let payment_key = format!("perf-{}", payment_provider);
        let merchant_key = format!("perf-{}", config_data.merchant_slug);
        let routing_key = format!("transactions.{}.{}", payment_key, merchant_key);
        println!("routing_key: {}", routing_key);

        //Select a token to use for this payment provider, along with first six and last four
        //This could be an inefficient process since we have to look through a list of StringRecords
        let payment_details = select_payment_details(&payment_card_tokens, payment_provider)?;
        tx = create_transaction(&config_data, &payment_details)?;

        if total_count >= config_data.maximum_number_transactions {
            println!("Produced {} transactions.", count);
            break;
        }

        queue_transaction(&exchange, tx, &routing_key)?;

        std::thread::sleep(delay);
    }

    queue_connection.close()?;

    Ok(count)
}

fn select_payment_details(payment_card_tokens: &Vec<StringRecord>, payment_provider: String)  -> Result<Vec<StringRecord>> {

    let mut provider_list = Vec::new();
    for item in payment_card_tokens {
        if item[2] == payment_provider {
            provider_list.push(item.clone());
        }
    }

    Ok(provider_list)

}

fn create_transaction(config: &Config, payment_card_tokens: &Vec<StringRecord>) -> Result<Transaction> {
    let token = payment_card_tokens.choose(&mut rand::thread_rng());
    return Ok(Transaction {
        amount: rand::thread_rng().gen_range(config.amount_min..config.amount_max),
        transaction_date: Utc::now(),
        merchant_name: config.merchant_slug.clone(),
        transaction_id: Uuid::new_v4().to_string(),
        auth_code: create_auth_code()?,
        identifier: "12345678".to_string(),
        token: token.unwrap()[0].to_string(),
        first_six: token.unwrap()[3].to_string(),
        last_four: token.unwrap()[4].to_string(),
    });
}

fn select_payment_provider(percentages: &[(String, i32); 3]) -> Result<String> {
    let dist = WeightedIndex::new(percentages.iter().map(|item| item.1)).unwrap();
    let mut rng = thread_rng();
    let provider = &percentages[dist.sample(&mut rng)].0;

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
