use amiquip::{Channel, ConsumerMessage, ConsumerOptions};
use color_eyre::Result;
use tracing::info;

use crate::{
    consumers::queue_declare,
    models::{DistributorConfig, Transaction},
};

use super::Consumer;

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
