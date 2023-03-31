use amiquip::{Channel, ConsumerMessage::Delivery, ConsumerOptions, QueueDeclareOptions};
use color_eyre::{eyre::eyre, Result};
use tracing::{debug, info, warn};

use crate::models::{DistributorConfig, Transaction};

use super::{queue_declare, Consumer};

/// A consumer that reads all messages off a queue and sends them as a batch.
/// Useful for file-based providers that run as a scheduled process.
pub struct BatchConsumer {
    pub channel: Channel,
    pub config: DistributorConfig,
}

impl Consumer for BatchConsumer {
    fn consume<F>(&self, f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        let queue = queue_declare(
            &self.config,
            &self.channel,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
        )?;

        // if the queue is empty or we can't determine the message count, quit early.
        let message_count = check_queue_message_count(&queue)?;
        if message_count == 0 {
            warn!("queue {} is empty, not consuming messages.", queue.name());
            return Ok(());
        }
        info!("queue {} has {} messages.", queue.name(), message_count);

        // if the queue already has consumers, quit early.
        if !no_other_consumers(&queue)? {
            warn!(
                "queue {} already has consumers, not consuming messages.",
                queue.name()
            );
            return Ok(());
        }

        let consumer = queue.consume(ConsumerOptions {
            no_ack: true,
            ..Default::default()
        })?;

        // FIXME: this is definitely not ideal. if another consumer connects,
        // the messages will go down faster than expected and this will hang
        // waiting for the last few to land.
        // we check for other consumers above, but that's not a guarantee.
        for messages in consumer
            .receiver()
            .iter()
            .take(message_count)
            .filter_map(|message| {
                if let Delivery(message) = message {
                    Some(message)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .chunks(self.config.batch_size)
        {
            let transactions = messages
                .iter()
                .map(|message| rmp_serde::from_slice::<Transaction>(&message.body))
                .collect::<Result<Vec<_>, rmp_serde::decode::Error>>()?;

            info!("sending batch of {} transactions.", transactions.len());
            f(transactions)?;
        }

        debug!("finished consuming messages from queue {}.", queue.name());

        Ok(())
    }
}

fn check_queue_message_count(queue: &amiquip::Queue) -> Result<usize> {
    if let Some(count) = queue.declared_message_count() {
        Ok(count as usize)
    } else {
        Err(eyre!(
            "unable to determine message count for queue {}.",
            queue.name()
        ))
    }
}

fn no_other_consumers(queue: &amiquip::Queue) -> Result<bool> {
    if let Some(count) = queue.declared_consumer_count() {
        Ok(count == 0)
    } else {
        Err(eyre!(
            "unable to determine consumer count for queue {}.",
            queue.name()
        ))
    }
}
