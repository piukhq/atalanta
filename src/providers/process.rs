#![warn(clippy::unwrap_used, clippy::expect_used)]
use amiquip::{
    Channel, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType,
    QueueDeclareOptions,
};
use color_eyre::Result;
use serde_json::{json};
use std::{thread, fs};
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
        println!("{:?}", transactions);

        let resp = client.post(&self.url).body(transactions).send()?;

        Ok(())
    }
}

pub struct AmexSender {
    pub url: String,
}

impl Sender for AmexSender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        println!("{:?}", transactions);
        let authorize_url = format!("{}/{}", &self.url, "authorize");
        let client_id = std::env::var("AMEX_CLIENT_ID").expect("Client id not in environment variables");
        let client_secret = std::env::var("AMEX_CLIENT_SECRET").expect("Client secret not in environment variables");
        let authorize_body = json!({
            "client_id": client_id,
            "client_secret": client_secret
        });

        let authorize_resp = client.post(authorize_url).body(authorize_body.to_string()).send()?;
        let authorize_text: String = authorize_resp.text().unwrap();
        let api_key: serde_json::Value = serde_json::from_str(&authorize_text)?;
        println!("Token {}", api_key["api_key"]);
        let amex_url = format!("{}/{}", &self.url, "amex");
        let resp = client
        .post(amex_url)
        .header("Authorization", format!("Token {}", api_key["api_key"].to_string().replace("\"", "")))
        .body(transactions)
        .send()?;

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

        let mut count: u32 = 0;

        let exchange = self.channel.exchange_declare(
            ExchangeType::Topic,
            "transactions",
            ExchangeDeclareOptions::default(),
        )?;
        let queue = self.channel.queue_declare(
            &queue_name,
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
        let initial_number_transaction_on_queue = queue.declared_message_count().unwrap();
        let mut local_batch_size = config_data.batch_size;
        if initial_number_transaction_on_queue > 0 && initial_number_transaction_on_queue < local_batch_size{
            local_batch_size = initial_number_transaction_on_queue;
        }

        let mut start = Instant::now();
        let mut transactions: Vec<Transaction>= Vec::new();

        let consumer = queue.consume(ConsumerOptions::default())?;
        loop {
            println!("Consumer sleep - waiting" );
            std::thread::sleep(Duration::from_secs(30));
            println!("Waking up after sleep");

            for message in consumer.receiver().try_iter() {
                println!("Consuming messages");
                match message {
                    ConsumerMessage::Delivery(delivery) => {
                        if count == 0 {
                            start = Instant::now();
                        }
                        count += 1;
                        transactions.push(rmp_serde::from_slice(&delivery.body).unwrap());
                        consumer.ack(delivery)?;

                        if count == local_batch_size {
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
            println!("Consumer ended - waiting" );
            let duration = start.elapsed();
            if duration >= Duration::from_secs(120) {
                break;
            }
        }

        Ok(())

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
        sender.send(transaction_data.join("\n"))
    })
}

fn write_to_file(data: String) -> Result<()> {
    // Creates new `Writer` for `stdout`
    let path = "test_file.csv";
    fs::write(path, data)?;

    Ok(())
}
