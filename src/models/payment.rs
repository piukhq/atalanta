use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub amount: i16,
    pub transaction_date: DateTime<Utc>,
    pub merchant_name: String,
    pub transaction_id: String,
    pub auth_code: String,
    pub identifier: String,
    pub token: String,
}