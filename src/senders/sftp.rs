use super::Sender;
use color_eyre::Result;

/// A struct that can send messages via SFTP.
pub struct SFTPSender {
    pub host: String,
    pub port: u16,
}

impl Sender for SFTPSender {
    fn send(&self, transactions: String) -> Result<()> {
        println!("SFTP: {transactions:?} to {}:{}", self.host, self.port);
        Ok(())
    }
}
