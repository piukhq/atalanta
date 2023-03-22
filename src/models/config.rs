#[derive(serde::Deserialize, Debug)]
pub struct TransactorConfig {
    pub provider_slug: String,
    pub routing_key: String,
    pub amount_min: i16,
    pub amount_max: i16,
    pub transactions_per_second: u64,
    pub percentage: [(String, i32); 3],
}

#[derive(serde::Deserialize, Debug)]
pub struct DistributorConfig {
    pub provider_slug: String,
    pub routing_key: String,
}
