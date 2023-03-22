#![warn(clippy::unwrap_used, clippy::expect_used)]
use std::thread::sleep;

use amiquip::{
    Channel, ConsumerMessage, ConsumerOptions, ExchangeDeclareOptions, ExchangeType, Queue,
    QueueDeclareOptions,
};
use chrono::{Duration, Utc};
use color_eyre::Result;
use tracing::{info, trace};

use crate::models::{DistributorConfig, Transaction};
use crate::providers::*;
use crate::senders::Sender;

pub trait Consumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>;
}

fn queue_declare<'a>(config: &DistributorConfig, channel: &'a Channel) -> Result<Queue<'a>> {
    let exchange = channel.exchange_declare(
        ExchangeType::Topic,
        "transactions",
        ExchangeDeclareOptions::default(),
    )?;

    let name = format!("perf-{}", config.provider_slug);
    let queue = channel.queue_declare(&name, QueueDeclareOptions::default())?;

    channel.queue_bind(
        queue.name(),
        exchange.name(),
        &config.routing_key,
        Default::default(),
    )?;

    Ok(queue)
}

/// A consumer that reads messages off a queue and sends them immediately.
/// Useful for auth providers that generally run in realtime.
pub struct InstantConsumer {
    pub config: DistributorConfig,
    pub channel: Channel,
}

impl Consumer for InstantConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        let queue = queue_declare(&self.config, &self.channel)?;

        info!(self.config.routing_key, "waiting for messages");

        let consumer = queue.consume(ConsumerOptions::default())?;
        for message in consumer.receiver().into_iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let tx: Transaction = rmp_serde::from_slice(&delivery.body)?;
                    f(vec![tx])?;
                    consumer.ack(delivery)?;
                }
                other => {
                    info!(message = ?other, "consumer ended");
                    break;
                }
            }
        }

        Ok(())
    }
}

/// A consumer that reads messages off a queue and sends them after a delay.
/// Useful for settlement providers that send transactions one at a time, usually some time after
/// the corresponding auth transaction was sent.
pub struct DelayConsumer {
    pub config: DistributorConfig,
    pub channel: Channel,
    pub delay: Duration,
}

impl Consumer for DelayConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        let queue = queue_declare(&self.config, &self.channel)?;

        let consumer = queue.consume(ConsumerOptions::default())?;

        info!(self.config.routing_key, "waiting for messages");
        for message in consumer.receiver().iter() {
            trace!("message received");
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let tx: Transaction = rmp_serde::from_slice(&delivery.body)?;

                    let now = Utc::now();
                    let send_at = tx.transaction_date + self.delay;
                    let delay = send_at - now;
                    match delay.to_std() {
                        Ok(delay) => {
                            info!(?send_at, ?delay, "waiting");
                            sleep(delay);
                        }
                        Err(_) => {
                            info!("delay < 0; transaction must be sent immediately");
                        }
                    }

                    consumer.ack(delivery)?;
                    f(vec![tx])?;
                }
                other => {
                    info!(message = ?other, "consumer ended");
                    break;
                }
            }
        }

        Ok(())
    }
}

/// A consumer that reads all messages off a queue and sends them as a batch.
/// Useful for file-based providers that run as a scheduled process.
pub struct OneShotConsumer {
    pub channel: Channel,
}

impl Consumer for OneShotConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        todo!()
    }
}

/// A generic function that can start any consumer with a given transaction formatter & sender.
pub fn start_consuming<C, F, S>(consumer: C, formatter: F, sender: S) -> Result<()>
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
