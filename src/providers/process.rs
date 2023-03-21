#![warn(clippy::unwrap_used, clippy::expect_used)]
use amiquip::{
    Channel, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType,
    QueueDeclareOptions,
};
use color_eyre::Result;
use std::time::{Duration, Instant};

use crate::configuration::load_config;
use crate::models::{Config, Transaction};
use crate::providers::*;
use crate::senders::Sender;

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
        let queue = self
            .channel
            .queue_declare(queue_name, QueueDeclareOptions::default())?;
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
pub struct DelayConsumer {
    pub channel: Channel,
    pub delay: Duration,
}

impl Consumer for DelayConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        let config_data: Config = load_config()?;
        let routing_key = config_data.routing_key;
        let queue_name = format!("perf-{}", config_data.deployed_slug);

        let mut count: u32 = 0;

        let exchange = self.channel.exchange_declare(
            ExchangeType::Topic,
            "transactions",
            ExchangeDeclareOptions::default(),
        )?;
        let queue = self
            .channel
            .queue_declare(&queue_name, QueueDeclareOptions::default())?;
        self.channel.queue_bind(
            queue.name(),
            exchange.name(),
            &routing_key,
            Default::default(),
        )?;

        println!("Waiting for messages. Press Ctrl-C to exit.");
        println!("Routing key: {}", routing_key);
        let initial_number_transaction_on_queue = queue.declared_message_count().unwrap();
        println!(
            "{} queue has {} transactions",
            queue_name, initial_number_transaction_on_queue
        );
        let mut start = Instant::now();
        let mut transactions: Vec<Transaction> = Vec::new();

        let consumer = queue.consume(ConsumerOptions::default())?;

        let no = consumer.receiver().len();

        for message in consumer.receiver().iter() {
            println!("Consuming messages");
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    if count == 0 {
                        start = Instant::now();
                    }
                    count += 1;
                    transactions.push(rmp_serde::from_slice(&delivery.body).unwrap());
                    consumer.ack(delivery)?;

                    if count == config_data.batch_size {
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

        if !transactions.is_empty() {
            f(transactions.clone())?;
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
