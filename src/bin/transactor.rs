#![warn(clippy::unwrap_used, clippy::expect_used)]

use amiquip::{Connection, Exchange, ExchangeDeclareOptions, ExchangeType, Publish};
use chrono::Utc;
use color_eyre::{eyre::eyre, Result};
use csv::StringRecord;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;
use std::fs::File;
use std::path::Path;
use std::time;
use tracing::debug;
use tracing::info;
use uuid::Uuid;

use atalanta::configuration::{load_settings, load_transactor_config};
use atalanta::initialise::startup;
use atalanta::models::{Settings, Transaction, TransactorConfig};

fn main() -> Result<()> {
    info!("starting transactor");
    startup()?;

    let settings = load_settings()?;
    let config = load_transactor_config(&settings)?;

    let payment_card_tokens =
        load_payment_card_tokens(&config.provider_slug, &settings.tokens_file_path)?;
    let identifiers = load_retailer_identifiers(&config.provider_slug, &settings.mids_file_path)?;

    transaction_producer(config, settings, payment_card_tokens, identifiers)
}

fn load_payment_card_tokens(
    retailer_slug: &String,
    tokens_file_path: &Path,
) -> Result<Vec<StringRecord>> {
    // Load token and slugs derived from the Hermes database
    //Only tokens related to the current retailer are loaded
    let mut tokens = Vec::new();

    let file = File::open(tokens_file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    for result in rdr.records() {
        let record = result?;
        if record.iter().any(|field| field == retailer_slug) {
            tokens.push(record);
        }
    }

    info!(
        "loaded {} tokens from {} for retailer {retailer_slug}",
        tokens.len(),
        tokens_file_path.display()
    );

    Ok(tokens)
}

fn load_retailer_identifiers(
    retailer_slug: &String,
    mids_file_path: &Path,
) -> Result<Vec<StringRecord>> {
    //Only identifiers related to the current retailer are loaded
    let mut identifiers = Vec::new();

    let file = File::open(mids_file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    for result in rdr.records() {
        let record = result?;
        if record.iter().any(|field| field == retailer_slug) {
            identifiers.push(record);
        }
    }

    info!(
        "loaded {} identifiers from {} for retailer {retailer_slug}",
        identifiers.len(),
        mids_file_path.display()
    );

    Ok(identifiers)
}

fn transaction_producer(
    config_data: TransactorConfig,
    settings: Settings,
    payment_card_tokens: Vec<StringRecord>,
    identifiers: Vec<StringRecord>,
) -> Result<()> {
    //Manages the process of creating raw transactions
    let delay = time::Duration::from_millis(1000 / config_data.transactions_per_second);
    println!("Delay setting - TODO: {}", delay.as_millis());

    let mut tx: Transaction;

    let mut connection = connect_to_amqp(settings)?;

    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    // let exchange = Exchange::direct(&channel);
    let exchange = channel.exchange_declare(
        ExchangeType::Topic,
        "transactions",
        ExchangeDeclareOptions::default(),
    )?;

    loop {
        // Select a payment provider based on weighted selection,
        // visa provides many more transactions than mastercard or amex
        let payment_provider = select_payment_provider(&config_data.percentage)?;
        let routing_key = format!(
            "transactions.{}.{}",
            payment_provider, config_data.provider_slug
        );

        //Select a token to use for this payment provider, along with first six and last four
        //This could be an inefficient process since we have to look through a list of StringRecords
        let payment_details =
            select_payment_details(&payment_card_tokens, payment_provider.clone())?;
        let identifier_details =
            select_identifiers_per_payment_provider(&identifiers, payment_provider.clone())?;
        tx = create_transaction(
            &config_data,
            &payment_provider,
            &payment_details,
            &identifier_details,
        )?;

        debug!(?tx);
        queue_transaction(&exchange, tx, &routing_key)?;

        std::thread::sleep(delay);
    }
}

fn select_payment_details(
    payment_card_tokens: &Vec<StringRecord>,
    payment_provider: String,
) -> Result<Vec<StringRecord>> {
    let mut provider_list = Vec::new();
    for item in payment_card_tokens {
        if item[2] == payment_provider {
            provider_list.push(item.clone());
        }
    }

    Ok(provider_list)
}

//Provided with a set of retailer specific identifier records
//select a subset of identifiers based on the payment provider
fn select_identifiers_per_payment_provider(
    identifiers: &Vec<StringRecord>,
    payment_provider: String,
) -> Result<Vec<StringRecord>> {
    let mut identifier_list = Vec::new();
    for item in identifiers {
        if item[1] == payment_provider {
            identifier_list.push(item.clone());
        }
    }

    Ok(identifier_list)
}

fn create_transaction(
    config: &TransactorConfig,
    payment_provider: &String,
    payment_card_tokens: &Vec<StringRecord>,
    identifiers: &Vec<StringRecord>,
) -> Result<Transaction> {
    let token = payment_card_tokens
        .choose(&mut rand::thread_rng())
        .ok_or(eyre!("failed to select payment card token"))?;

    let identifier = identifiers
        .choose(&mut rand::thread_rng())
        .ok_or(eyre!("failed to select identifier"))?;

    Ok(Transaction {
        amount: rand::thread_rng().gen_range(config.amount_min..config.amount_max),
        transaction_date: Utc::now(),
        payment_provider: payment_provider.to_string(),
        merchant_name: config.provider_slug.clone(),
        transaction_id: Uuid::new_v4().to_string(),
        auth_code: create_auth_code()?,
        identifier: identifier[2].to_string(),
        token: token[0].to_string(),
        first_six: token[3].to_string(),
        last_four: token[4].to_string(),
    })
}

fn select_payment_provider(percentages: &[(String, i32); 3]) -> Result<String> {
    let dist = WeightedIndex::new(percentages.iter().map(|item| item.1))?;
    let mut rng = thread_rng();
    let provider = percentages[dist.sample(&mut rng)].0.clone();
    Ok(provider)
}

fn create_auth_code() -> Result<String> {
    let number = rand::thread_rng().gen_range(9..1000000);
    Ok(format!("{:0>6}", number))
}

fn connect_to_amqp(settings: Settings) -> Result<Connection> {
    if settings.environment == "LOCAL" {
        Ok(Connection::insecure_open(&settings.amqp_dsn)?)
    } else {
        Ok(Connection::open(&settings.amqp_dsn)?)
    }
}

fn queue_transaction(
    exchange: &Exchange,
    transaction: Transaction,
    routing_key: &String,
) -> Result<()> {
    // Publish a message to the "new_transaction" queue.
    exchange.publish(Publish::new(&rmp_serde::to_vec(&transaction)?, routing_key))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_auth_code_success() -> Result<()> {
        let auth_code = create_auth_code()?;
        assert_eq!(auth_code.len(), 6);
        Ok(())
    }
}
