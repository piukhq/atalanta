use std::{io::Write, net::TcpStream, path::Path};

use crate::models::SenderConfig;

use super::Sender;
use color_eyre::{eyre::eyre, Result};
use ssh2::Session;
use tracing::{debug, info};
use uuid::Uuid;

/// A struct that can send messages via SFTP.
pub struct SFTPSender {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub upload_path: String,
}

impl TryFrom<SenderConfig> for SFTPSender {
    type Error = color_eyre::Report;

    fn try_from(value: SenderConfig) -> Result<Self> {
        if let SenderConfig::SFTP(config) = value {
            Ok(SFTPSender {
                host: config.host,
                port: config.port,
                username: config.username,
                upload_path: config.upload_path,
            })
        } else {
            Err(eyre!("Invalid sender config type, expected SFTP"))
        }
    }
}

impl Sender for SFTPSender {
    fn send(&self, transactions: String) -> Result<()> {
        let mut sess = Session::new()?;
        let mut agent = sess.agent()?;
        agent.connect()?;

        debug!("Connecting to {}:{}...", self.host, self.port);
        let tcp = TcpStream::connect(format!("{}:{}", self.host, self.port))?;
        sess.set_tcp_stream(tcp);

        debug!("Handshaking");
        sess.handshake()?;

        debug!("Trying identities");
        agent.list_identities()?;
        for identity in agent.identities()? {
            debug!("Trying identity: {:?}", identity.comment());
            if let Ok(()) = agent.userauth(&self.username, &identity) {
                info!("Authenticated with identity: {:?}", identity.comment());
                break;
            }
        }

        if !sess.authenticated() {
            return Err(eyre!("None of the identities worked, cannot authenticate."));
        }

        info!(
            "Connected to {}:{} as {}",
            self.host, self.port, self.username
        );

        debug!("Opening SFTP session");
        let sftp = sess.sftp()?;

        let filename = Uuid::new_v4().to_string();
        let path = Path::new(&self.upload_path).join(filename);
        debug!("Uploading {}", path.to_string_lossy());
        let mut file = sftp.create(&path)?;

        debug!("Writing transactions to file");
        write!(file, "{}", transactions)?;

        info!("Transactions sent successfully");

        Ok(())
    }
}
