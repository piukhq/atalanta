use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub amount: i64,
    pub transaction_date: DateTime<Utc>,
    pub payment_provider: String,
    pub merchant_name: String,
    pub transaction_id: String,
    pub auth_code: String,
    pub identifier: String,
    pub token: String,
    pub first_six: String,
    pub last_four: String,
}
