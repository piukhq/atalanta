use crate::models::SenderConfig;

use super::Sender;
use color_eyre::{eyre::eyre, Result};

/// A struct that can send messages via SFTP.
pub struct SFTPSender {
    pub host: String,
    pub port: u16,
}

impl TryFrom<SenderConfig> for SFTPSender {
    type Error = color_eyre::Report;

    fn try_from(value: SenderConfig) -> Result<Self> {
        if let SenderConfig::SFTP(config) = value {
            Ok(SFTPSender {
                host: config.host,
                port: config.port,
            })
        } else {
            Err(eyre!("Invalid sender config type, expected SFTP"))
        }
    }
}

impl Sender for SFTPSender {
    fn send(&self, transactions: String) -> Result<()> {
        println!("SFTP: {transactions:?} to {}:{}", self.host, self.port);
        Ok(())
    }
}
