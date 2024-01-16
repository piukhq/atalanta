#![warn(clippy::unwrap_used, clippy::expect_used)]

use amiquip::{Exchange, ExchangeDeclareOptions, ExchangeType, Publish};
use atalanta::amqp;
use chrono::Utc;
use color_eyre::{eyre::eyre, Result};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::time;
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

    transaction_producer(config, settings, &payment_card_tokens, &identifiers)
}

#[derive(Debug, Deserialize)]
struct TokenRecord {
    token: String,
    retailer_slug: String,
    first_six: String,
    last_four: String,
    payment_slug: String,
}

fn load_payment_card_tokens(
    retailer_slug: &str,
    tokens_file_path: &Path,
) -> Result<Vec<TokenRecord>> {
    // Load token and slugs derived from the Hermes database
    //Only tokens related to the current retailer are loaded
    let mut tokens = Vec::new();

    let file = File::open(tokens_file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: TokenRecord = result?;
        if record.retailer_slug == retailer_slug {
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

#[derive(Debug, Deserialize, Serialize)]
enum IdentifierType {
    #[serde(rename = "PRIMARY")]
    PrimaryMID,
    #[serde(rename = "SECONDARY")]
    SecondaryMID,
    #[serde(rename = "PSIMI")]
    Psimi,
}

impl ToString for IdentifierType {
    fn to_string(&self) -> String {
        match self {
            IdentifierType::PrimaryMID => "PRIMARY".to_owned(),
            IdentifierType::SecondaryMID => "SECONDARY".to_owned(),
            IdentifierType::Psimi => "PSIMI".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct IdentifierRecord {
    retailer_slug: String,
    payment_slug: String,
    identifier: String,
    identifier_type: IdentifierType,
    _location_id: Option<String>,
    _merchant_internal_id: Option<String>,
}

fn load_retailer_identifiers(
    retailer_slug: &str,
    mids_file_path: &Path,
) -> Result<Vec<IdentifierRecord>> {
    //Only identifiers related to the current retailer are loaded
    let mut identifiers = Vec::new();

    let file = File::open(mids_file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: IdentifierRecord = result?;
        if record.retailer_slug == retailer_slug {
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
    payment_card_tokens: &[TokenRecord],
    identifiers: &[IdentifierRecord],
) -> Result<()> {
    //Manages the process of creating raw transactions
    let delay = time::Duration::from_millis(1000 / config_data.transactions_per_second);
    println!("Delay setting - TODO: {}", delay.as_millis());

    let mut tx: Transaction;

    let mut connection = amqp::connect(settings)?;

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
        let payment_details = select_payment_details(payment_card_tokens, &payment_provider);
        let identifier_details =
            select_identifiers_per_payment_provider(identifiers, &payment_provider);
        tx = create_transaction(
            &config_data,
            &payment_provider,
            &payment_details,
            &identifier_details,
        )?;

        queue_transaction(&exchange, tx, &routing_key)?;

        std::thread::sleep(delay);
    }

    Ok(())
}

fn select_payment_details<'a>(
    payment_card_tokens: &'a [TokenRecord],
    payment_provider: &str,
) -> Vec<&'a TokenRecord> {
    payment_card_tokens
        .iter()
        .filter(|token| token.payment_slug == payment_provider)
        .collect()
}

//Provided with a set of retailer specific identifier records
//select a subset of identifiers based on the payment provider
fn select_identifiers_per_payment_provider<'a>(
    identifiers: &'a [IdentifierRecord],
    payment_provider: &str,
) -> Vec<&'a IdentifierRecord> {
    identifiers
        .iter()
        .filter(|identifier| identifier.payment_slug == payment_provider)
        .collect()
}

fn create_transaction(
    config: &TransactorConfig,
    payment_provider: &str,
    payment_card_tokens: &[&TokenRecord],
    identifiers: &[&IdentifierRecord],
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
        identifier: identifier.identifier.clone(),
        identifier_type: identifier.identifier_type.to_string(),
        token: token.token.clone(),
        first_six: token.first_six.clone(),
        last_four: token.last_four.clone(),
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
    use pretty_assertions::assert_eq;

    #[test]
    fn create_auth_code_success() -> Result<()> {
        let auth_code = create_auth_code()?;
        assert_eq!(auth_code.len(), 6);
        Ok(())
    }

    #[test]
    fn create_transaction_success() -> Result<()> {
        let transactor_config = TransactorConfig {
            provider_slug: "test_slug".to_string(),
            amount_min: 10,
            amount_max: 100,
            transactions_per_second: 1,
            percentage: [
                ("visa".to_string(), 100),
                ("mastercard".to_string(), 0),
                ("amex".to_string(), 0),
            ],
        };

        let token_record = [&TokenRecord {
            token: "test_token".to_string(),
            retailer_slug: "wasabi_club".to_string(),
            first_six: "666666".to_string(),
            last_four: "4444".to_string(),
            payment_slug: "visa".to_string(),
        }];

        let identifier_records = [&IdentifierRecord {
            retailer_slug: "wasabi_club".to_string(),
            payment_slug: "visa".to_string(),
            identifier: "123456789".to_string(),
            identifier_type: IdentifierType::PrimaryMID,
            _location_id: Some("loc_id_123456".to_string()),
            _merchant_internal_id: Some("mi_1234567".to_string()),
        }];

        let expected_transaction = Transaction {
            amount: 2000,
            transaction_date: Utc::now(),
            payment_provider: "visa".to_string(),
            merchant_name: "wasabi-club".to_string(),
            transaction_id: Uuid::new_v4().to_string(),
            auth_code: "123456".to_string(),
            identifier: identifier_records[0].identifier.clone(),
            identifier_type: identifier_records[0].identifier_type.to_string(),
            token: token_record[0].token.clone(),
            first_six: token_record[0].first_six.clone(),
            last_four: token_record[0].last_four.clone(),
        };

        let test_transaction = create_transaction(
            &transactor_config,
            "visa",
            &token_record,
            &identifier_records,
        )?;
        assert_eq!(test_transaction.identifier, expected_transaction.identifier);
        assert_eq!(test_transaction.identifier_type, "PRIMARY");

        Ok(())
    }
}
