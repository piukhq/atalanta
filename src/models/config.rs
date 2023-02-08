
#[derive(serde::Deserialize)]
pub struct Config {
    pub merchant_slug: String,
    pub transaction_rate: u16,
}
