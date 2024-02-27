pub mod amex;
pub mod api;
pub mod blob;
pub mod sftp;

use color_eyre::Result;

use crate::models::SenderConfig;

pub trait Sender: TryFrom<SenderConfig> {
    /// Sends a formatted set of transactions to a destination.
    ///
    /// # Errors
    ///
    /// Returns an error if the transactions cannot be sent.
    fn send(&self, transactions: String) -> Result<()>;
}
