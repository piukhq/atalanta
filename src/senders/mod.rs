use color_eyre::Result;
use serde_json::json;

pub trait Sender {
    fn send(&self, transactions: String) -> Result<()>;
}

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

/// A struct that can send messages via API.
pub struct APISender {
    pub url: String,
}

impl Sender for APISender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        println!("{:?}", transactions);

        let resp = client.post(&self.url).body(transactions).send()?;
        println!("{}", resp.status());

        Ok(())
    }
}

pub struct AmexSender {
    pub url: String,
}

impl Sender for AmexSender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        println!("{:?}", transactions);
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
