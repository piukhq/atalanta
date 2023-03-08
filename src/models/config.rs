#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub merchant_slug: String,
    pub payment_provider: String,
    pub deployed_slug: String,
    pub routing_key: String,
    pub batch_size: u32,
    pub amount_min: i16,
    pub amount_max: i16,
    pub transactions_per_second: u64,
}
