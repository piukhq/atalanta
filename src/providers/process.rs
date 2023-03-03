#![warn(clippy::unwrap_used, clippy::expect_used)]
use amiquip::{
    Channel, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType,
    QueueDeclareOptions,
};
use color_eyre::Result;
use std::thread;
use std::time::{Duration, Instant};

use crate::configuration::load_config;
use crate::models::{Config, Transaction};
use crate::providers::*;
pub trait Sender {
    fn send(&self, transactions: String) -> Result<()>;
}

/// A struct that can send messages via SFTP.
pub struct SFTPSender {
    pub host: String,
    pub port: u16,
}

impl Sender for SFTPSender {
    fn send(&self, transactions: String) -> Result<()> {
        println!("SFTP: {transactions:?} to {}:{}", self.host, self.port);
        write_to_file(transactions)?;
        Ok(())
    }
}

/// A struct that can send messages via API.
pub struct APISender {
    pub url: String,
}

impl Sender for APISender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let message = transactions;

        let resp = client.post(&self.url).body(message).send()?;
        Ok(())
    }
}

/// A struct that can send messages to a blob storage.
pub struct BlobSender {
    pub container: String,
}

impl Sender for BlobSender {
    fn send(&self, transactions: String) -> Result<()> {
        println!("BLOB: {transactions:?} to {}", self.container);
        Ok(())
    }
}

pub trait Consumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>;
}

// A consumer that reads messages off a queue and sends them immediately.
pub struct InstantConsumer {
    pub channel: Channel,
}

impl Consumer for InstantConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        let config_data: Config = load_config()?;
        let routing_key = &config_data.routing_key;
        let queue_name = format!("perf-{}", config_data.deployed_slug);

        let mut count: u64 = 0;

        let exchange = self.channel.exchange_declare(
            ExchangeType::Topic,
            "transactions",
            ExchangeDeclareOptions::default(),
        )?;
        let queue = self.channel.queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
        )?;
        self.channel.queue_bind(
            queue.name(),
            exchange.name(),
            routing_key,
            Default::default(),
        )?;

        println!("Waiting for messages. Press Ctrl-C to exit.");
        println!("Routing key: {}", routing_key);
        let mut start = Instant::now();
        let mut output_transaction = String::new();

        let consumer = queue.consume(ConsumerOptions::default())?;
        for message in consumer.receiver().into_iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    if count == 0 {
                        start = Instant::now();
                    }
                    let tx: Transaction = rmp_serde::from_slice(&delivery.body).unwrap();
                    f(vec![tx])?;
                    consumer.ack(delivery)?;
                    count += 1;

                    if count == 100 {
                        let duration = start.elapsed();
                        println!("Final count: {}, duration = {:?}", count, duration);
                    }
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }

        Ok(())
    }
}

// A consumer that reads messages off a queue and sends them after a delay.
pub struct TimedConsumer {
    pub channel: Channel,
    pub delay: Duration,
}

impl Consumer for TimedConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        thread::sleep(self.delay);
        let config_data: Config = load_config()?;
        let routing_key = config_data.routing_key;
        let queue_name = format!("perf-{}", config_data.deployed_slug);

        let mut count: u64 = 0;

        let exchange = self.channel.exchange_declare(
            ExchangeType::Topic,
            "transactions",
            ExchangeDeclareOptions::default(),
        )?;
        let queue = self.channel.queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
        )?;
        self.channel.queue_bind(
            queue.name(),
            exchange.name(),
            &routing_key,
            Default::default(),
        )?;

        println!("Waiting for messages. Press Ctrl-C to exit.");
        println!("Routing key: {}", routing_key);
        let mut start = Instant::now();
        let mut output_transaction = String::new();
        let mut transactions: Vec<Transaction>= Vec::new();

        let consumer = queue.consume(ConsumerOptions::default())?;
        for message in consumer.receiver().into_iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    if count == 0 {
                        start = Instant::now();
                    }
                    count += 1;
                    transactions.push(rmp_serde::from_slice(&delivery.body).unwrap());
                    consumer.ack(delivery)?;

                    if count == 3 {
                        f(transactions.clone())?;
                        
                        count = 0;
                        transactions.clear();
                    }
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
        }

        Ok(())

    }
}
/// A trait for formatting transaction data for each retailer.
pub trait Formatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<String>;
}

pub struct VisaAuthFormatter {}

impl Formatter for VisaAuthFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<String> {
        let transaction = visa_auth(&transactions[0])?;

        Ok(transaction)
    }
}

pub struct VisaSettlementFormatter {}

impl Formatter for VisaSettlementFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<String> {
        let transaction = visa_settlement(&transactions[0])?;

        Ok(transaction)
    }
}

pub struct WasabiFormatter {}

impl Formatter for WasabiFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<String> {
        let transaction = wasabi_transaction(transactions)?;

        Ok(transaction)
    }
}

/// A generic function that can send messages via any Sender.
pub fn send_message<C, F, S>(consumer: C, formatter: F, sender: S) -> Result<()>
where
    C: Consumer,
    F: Formatter,
    S: Sender,
{
    consumer.consume(|transactions| {
        let transaction_data = formatter.format(transactions)?;
        sender.send(transaction_data)
    })
}

fn write_to_file(data: String) -> Result<()> {
    // Creates new `Writer` for `stdout`
    let path = "test_file.csv";

    let mut writer = csv::Writer::from_path(path)?;

    writer.serialize(data)?;

    writer.flush()?;

    Ok(())
}
