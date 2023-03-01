
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub merchant_slug: String,
    pub payment_provider: String,
    pub transaction_rate: u16,
    pub deployed_slug: String,
    pub routing_key: String,
}
