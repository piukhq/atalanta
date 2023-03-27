use super::Sender;
use color_eyre::Result;

/// A struct that can send messages to a blob storage.
pub struct BlobSender {
    pub container: String,
}

impl Sender for BlobSender {
    fn send(&self, transactions: String) -> Result<()> {
        println!("BLOB: {transactions:?} to {}", self.container);
        Ok(())
    }
}
