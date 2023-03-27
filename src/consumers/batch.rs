use amiquip::Channel;
use color_eyre::Result;

use crate::models::Transaction;

use super::Consumer;

/// A consumer that reads all messages off a queue and sends them as a batch.
/// Useful for file-based providers that run as a scheduled process.
pub struct BatchConsumer {
    pub channel: Channel,
}

impl Consumer for BatchConsumer {
    fn consume<F>(&self, _f: F) -> Result<()>
    where
        F: Fn(Vec<Transaction>) -> Result<()>,
    {
        todo!()
    }
}
