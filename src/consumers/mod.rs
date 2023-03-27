mod delay;
pub use delay::DelayConsumer;

mod instant;
pub use instant::InstantConsumer;

mod batch;
pub use batch::BatchConsumer;

use amiquip::{Channel, ExchangeDeclareOptions, ExchangeType, Queue, QueueDeclareOptions};
use color_eyre::Result;

use crate::{
    formatters::Formatter,
    models::{DistributorConfig, Transaction},
    senders::Sender,
};

pub trait Consumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>;
}

/// A generic function that can start any consumer with a given transaction formatter & sender.
pub fn start_consuming<C, F, S>(consumer: C, sender: S) -> Result<()>
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
