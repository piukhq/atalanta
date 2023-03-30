use amiquip::{Channel, ConsumerMessage::Delivery, ConsumerOptions, QueueDeclareOptions};
use color_eyre::Result;

use crate::models::{DistributorConfig, Transaction};

use super::{queue_declare, Consumer};

/// A consumer that reads all messages off a queue and sends them as a batch.
/// Useful for file-based providers that run as a scheduled process.
pub struct BatchConsumer {
    pub channel: Channel,
    pub config: DistributorConfig,
}

impl Consumer for BatchConsumer {
    fn consume<F>(&self, _f: F) -> Result<()>
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

        // FIXME: this is definitely not ideal. if another consumer connects,
        // the messages will go down faster than expected and this will hang
        // waiting for the last few to land.
        let consumer = queue.consume(ConsumerOptions {
            no_ack: true,
            ..Default::default()
        })?;
        let messages = consumer
            .receiver()
            .iter()
            .take(queue.declared_message_count().unwrap_or(0) as usize)
            .filter_map(|message| {
                if let Delivery(message) = message {
                    Some(message)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        println!("Done consuming messages:");
        for message in messages {
            println!("- {:?}", message);
        }

        Ok(())
    }
}
