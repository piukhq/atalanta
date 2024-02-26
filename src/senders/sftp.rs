use std::{
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
};

use crate::models::SenderConfig;

use color_eyre::{eyre::eyre, Result};
use ssh2::Session;
use tracing::debug;
use uuid::Uuid;

/// A struct that can send messages via SFTP.
pub struct Sender {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub key_file_path: PathBuf,
    pub upload_path: PathBuf,
}

impl TryFrom<SenderConfig> for Sender {
    type Error = color_eyre::Report;

    fn try_from(value: SenderConfig) -> Result<Self> {
        if let SenderConfig::SFTP(config) = value {
            Ok(Self {
                host: config.host,
                port: config.port,
                username: config.username,
                key_file_path: config.key_file_path,
                upload_path: config.upload_path,
            })
        } else {
            Err(eyre!("Invalid sender config type, expected SFTP"))
        }
    }
}

fn write_file(sftp: &ssh2::Sftp, path: &Path, content: &str) -> Result<()> {
    let filename = format!("{}.csv", Uuid::new_v4());
    let path = path.join(filename);
    debug!("Uploading {}", path.to_string_lossy());
    let mut file = sftp.create(&path)?;

    debug!("Writing transactions to file");
    write!(file, "{content}")?;

    Ok(())
}

impl super::Sender for Sender {
    fn send(&self, transactions: String) -> Result<()> {
        let mut sess = Session::new()?;
        sess.set_tcp_stream(TcpStream::connect(format!("{}:{}", self.host, self.port))?);
        sess.handshake()?;
        sess.userauth_pubkey_file(&self.username, None, &self.key_file_path, None)?;
        if !sess.authenticated() {
            return Err(eyre!("None of the identities worked, cannot authenticate."));
        }
        let sftp = sess.sftp()?;
        write_file(&sftp, Path::new(&self.upload_path), &transactions)?;
        Ok(())
    }
}
