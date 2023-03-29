use std::thread::sleep;

use amiquip::{Channel, ConsumerMessage, ConsumerOptions};
use chrono::{Duration, Utc};
use color_eyre::Result;
use tracing::{info, trace};

use crate::{
    consumers::queue_declare,
    models::{DistributorConfig, Transaction},
};

use super::Consumer;

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