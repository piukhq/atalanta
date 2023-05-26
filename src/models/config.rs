use std::path::PathBuf;

#[derive(serde::Deserialize)]
pub struct TransactorConfig {
    pub provider_slug: String,
    pub amount_min: i64,
    pub amount_max: i64,
    pub transactions_per_second: u64,
    pub percentage: [(String, i32); 3],
}

#[derive(serde::Deserialize, Clone)]
pub struct DistributorConfig {
    pub provider_slug: String,
    pub routing_key: String,
    pub batch_size: usize,

    pub sender: SenderConfig,
}

#[derive(serde::Deserialize, Clone)]
pub enum SenderConfig {
    API(APISenderConfig),
    Amex(APISenderConfig),
    SFTP(SFTPSenderConfig),
    Blob(BlobSenderConfig),
}

#[derive(serde::Deserialize, Clone)]
pub enum APISenderHeaderValue {
    Literal(String),
    Secret(PathBuf),
}

#[derive(serde::Deserialize, Clone)]
pub struct APISenderHeader {
    pub name: String,
    pub value: APISenderHeaderValue,
}

#[derive(serde::Deserialize, Clone)]
pub struct APISenderConfig {
    pub url: String,
    #[serde(default)]
    pub headers: Vec<APISenderHeader>,
}

#[derive(serde::Deserialize, Clone)]
pub struct SFTPSenderConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub key_file_path: PathBuf,
    pub upload_path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct BlobSenderConfig {
    pub account: String,
    pub access_key: String,
    pub container: String,
}
