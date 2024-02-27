use std::{fs, path::PathBuf};

use crate::models;

use color_eyre::{eyre::eyre, Result};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use tracing::{error, info};

enum APISenderHeaderValue {
    Literal(String),
    Secret(PathBuf),
}

impl APISenderHeaderValue {
    fn resolve(&self) -> Result<String> {
        match self {
            Self::Literal(v) => Ok(v.clone()),
            Self::Secret(path) => fs::read_to_string(path)
                .map(|s| s.trim().to_owned())
                .map_err(|e| {
                    eyre!(
                        "Failed to read header secret file {}: {e}",
                        path.to_string_lossy()
                    )
                }),
        }
    }
}

impl From<models::APISenderHeaderValue> for APISenderHeaderValue {
    fn from(value: models::APISenderHeaderValue) -> Self {
        match value {
            models::APISenderHeaderValue::Literal(value) => Self::Literal(value),
            models::APISenderHeaderValue::Secret(path) => Self::Secret(path),
        }
    }
}

struct APISenderHeader {
    name: String,
    value: APISenderHeaderValue,
}

impl From<models::APISenderHeader> for APISenderHeader {
    fn from(header: models::APISenderHeader) -> Self {
        Self {
            name: header.name,
            value: header.value.into(),
        }
    }
}

pub struct Sender {
    pub url: String,
    headers: Vec<APISenderHeader>,
}

impl TryFrom<models::SenderConfig> for Sender {
    type Error = color_eyre::Report;

    fn try_from(config: models::SenderConfig) -> Result<Self> {
        match config {
            models::SenderConfig::API(config) => Ok(Self {
                url: config.url,
                headers: config
                    .headers
                    .iter()
                    .map(|header| Into::<APISenderHeader>::into(header.clone()))
                    .collect(),
            }),
            _ => Err(eyre!("Invalid sender config type, expected API")),
        }
    }
}

impl super::Sender for Sender {
    #[tracing::instrument(skip_all, name = "APISender::send")]
    fn send(&self, transactions: String) -> Result<()> {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        for header in &self.headers {
            let value = header.value.resolve()?;
            headers.insert(
                HeaderName::from_bytes(header.name.as_bytes())?,
                HeaderValue::from_str(&value)?,
            );
        }

        match client
            .post(&self.url)
            .headers(headers)
            .body(transactions)
            .send()
        {
            Ok(resp) => info!("response status: {}", resp.status()),
            Err(e) => error!("connection error, transaction discarded: {e}"),
        };

        Ok(())
    }
}
