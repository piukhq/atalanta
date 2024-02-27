use amiquip::{Channel, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use color_eyre::Result;
use tracing::info;

use crate::{
    consumers::queue_declare,
    models::{DistributorConfig, Transaction},
};

/// A consumer that reads messages off a queue and sends them immediately.
/// Useful for auth providers that generally run in realtime.
pub struct Consumer {
    pub config: DistributorConfig,
    pub channel: Channel,
}

impl super::Consumer for Consumer {
    fn new(config: DistributorConfig, channel: Channel) -> Self {
        Self { config, channel }
    }

    fn new_with_delay(
        config: DistributorConfig,
        channel: Channel,
        _delay: chrono::Duration,
    ) -> Self {
        Self { config, channel }
    }

    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        let queue = queue_declare(&self.config, &self.channel, QueueDeclareOptions::default())?;

        info!(self.config.routing_key, "waiting for messages");

        let consumer = queue.consume(ConsumerOptions::default())?;
        for message in consumer.receiver() {
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
