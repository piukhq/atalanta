pub mod batch;
pub mod delay;
pub mod instant;

use std::collections::BTreeMap;

use amiquip::{Channel, ExchangeDeclareOptions, ExchangeType, Queue, QueueDeclareOptions};
use color_eyre::Result;

use crate::{
    formatters::Formatter,
    models::{DistributorConfig, Transaction},
    senders::Sender,
};

pub trait Consumer {
    fn new(config: DistributorConfig, channel: Channel) -> Self;
    fn new_with_delay(config: DistributorConfig, channel: Channel, delay: chrono::Duration)
        -> Self;

    /// Consumes messages from a queue and invokes the given function with the transactions.
    ///
    /// # Errors
    ///
    /// Returns an error if messages cannot be consumed and parsed.
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>;
}

/// A generic function that can start any consumer with a given transaction formatter & sender.
///
/// # Errors
///
/// Returns an error if the consumer cannot consume messages or the sender cannot send them.
pub fn start_consuming<C, F, S>(consumer: &C, sender: &S) -> Result<()>
where
    C: Consumer,
    F: Formatter,
    S: Sender,
{
    consumer.consume(|transactions| {
        let transaction_data = F::format(transactions)?;
        sender.send(transaction_data)
    })
}

fn queue_declare<'a>(
    config: &DistributorConfig,
    channel: &'a Channel,
    options: QueueDeclareOptions,
) -> Result<Queue<'a>> {
    let exchange = channel.exchange_declare(
        ExchangeType::Topic,
        "transactions",
        ExchangeDeclareOptions::default(),
    )?;

    let name = format!("perf-{}", config.provider_slug);
    let queue = channel.queue_declare(&name, options)?;

    channel.queue_bind(
        queue.name(),
        exchange.name(),
        &config.routing_key,
        BTreeMap::default(),
    )?;

    Ok(queue)
}
