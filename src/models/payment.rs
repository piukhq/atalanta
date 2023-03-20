use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    pub amount: i16,
    pub transaction_date: DateTime<Utc>,
    pub merchant_name: String,
    pub transaction_id: String,
    pub auth_code: String,
    pub identifier: String,
    pub token: String,
    pub first_six: String,
    pub last_four: String,
}
