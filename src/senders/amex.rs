use crate::models::SenderConfig;

use color_eyre::{eyre::eyre, Result};
use serde_json::json;

pub struct Sender {
    pub url: String,
}

impl TryFrom<SenderConfig> for Sender {
    type Error = color_eyre::Report;

    fn try_from(value: SenderConfig) -> Result<Self> {
        if let SenderConfig::Amex(config) = value {
            Ok(Self { url: config.url })
        } else {
            Err(eyre!("Invalid sender config type, expected Amex"))
        }
    }
}

impl super::Sender for Sender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let authorize_url = format!("{}/{}", &self.url, "authorize");
        let client_id = std::env::var("AMEX_CLIENT_ID")?;
        let client_secret = std::env::var("AMEX_CLIENT_SECRET")?;
        let authorize_body = json!({
            "client_id": client_id,
            "client_secret": client_secret
        });

        let authorize_resp = client
            .post(authorize_url)
            .body(authorize_body.to_string())
            .send()?;
        let authorize_json: serde_json::Value = authorize_resp.json()?;
        println!("Token {}", authorize_json["api_key"]);
        let amex_url = format!("{}/{}", &self.url, "amex");
        let resp = client
            .post(amex_url)
            .header(
                "Authorization",
                format!(
                    "Token {}",
                    authorize_json["api_key"].to_string().replace('\"', "")
                ),
            )
            .body(transactions)
            .send()?;
        println!("{}", resp.status());

        Ok(())
    }
}
