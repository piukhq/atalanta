use amiquip::{Channel, ConsumerMessage::Delivery, ConsumerOptions, QueueDeclareOptions};
use color_eyre::{eyre::eyre, Result};

use tracing::{debug, info, warn};

use crate::models::{DistributorConfig, Transaction};

use super::queue_declare;

/// A consumer that reads all messages off a queue and sends them as a batch.
/// Useful for file-based providers that run as a scheduled process.
pub struct Consumer {
    pub channel: Channel,
    pub config: DistributorConfig,
}

impl super::Consumer for Consumer {
    fn new(config: DistributorConfig, channel: Channel) -> Self {
        Self { channel, config }
    }

    fn new_with_delay(
        config: DistributorConfig,
        channel: Channel,
        _delay: chrono::Duration,
    ) -> Self {
        Self { channel, config }
    }

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

        if !queue_is_consumable(&queue)? {
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
        let message_count = queue.declared_message_count().ok_or_else(|| {
            eyre!(
                "unable to determine message count for queue {}.",
                queue.name()
            )
        })?;
        for messages in consumer
            .receiver()
            .iter()
            .take(message_count as usize)
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

fn queue_is_consumable(queue: &amiquip::Queue) -> Result<bool> {
    // if the queue is empty or we can't determine the message count, quit early.
    let message_count = queue.declared_consumer_count().ok_or_else(|| {
        eyre!(
            "unable to determine message count for queue {}.",
            queue.name()
        )
    })?;

    let consumer_count = queue.declared_consumer_count().ok_or_else(|| {
        eyre!(
            "unable to determine consumer count for queue {}.",
            queue.name()
        )
    })?;

    if message_count == 0 {
        warn!("queue {} is empty, not consuming messages.", queue.name());
        Ok(false)
    }
    // if the queue already has consumers, quit early.
    else if consumer_count > 0 {
        warn!(
            "queue {} already has consumers, not consuming messages.",
            queue.name()
        );
        Ok(false)
    } else {
        Ok(true)
    }
}
