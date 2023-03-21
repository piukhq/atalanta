#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub merchant_slug: String,
    pub payment_provider: String,
    pub deployed_slug: String,
    pub routing_key: String,
    pub batch_size: usize,
    pub amount_min: i16,
    pub amount_max: i16,
    pub transactions_per_second: u64,
    pub maximum_number_transactions: u64,
    pub percentage: [(String, i32); 3],
}
