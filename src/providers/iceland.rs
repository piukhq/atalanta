#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::{models::Transaction, providers::to_pounds};
use color_eyre::Result;
use csv::WriterBuilder;
use serde::Serialize;

#[derive(Serialize)]
pub struct IcelandTransaction {
    #[serde(rename = "TransactionCardFirst6")]
    pub first_six: String,
    #[serde(rename = "TransactionCardLast4")]
    pub last_four: String,
    #[serde(rename = "TransactionCardExpiry")]
    pub expiry: String,
    #[serde(rename = "TransactionCardSchemeId")]
    pub card_scheme_id: String,
    #[serde(rename = "TransactionCardScheme")]
    pub card_scheme_name: String,
    #[serde(rename = "TransactionStore_Id")]
    pub identifier: String,
    #[serde(rename = "TransactionTimestamp")]
    pub transaction_date: String,
    #[serde(rename = "TransactionAmountValue")]
    pub amount: String,
    #[serde(rename = "TransactionAmountUnit")]
    pub amount_unit: String,
    #[serde(rename = "TransactionCashbackValue")]
    pub cashback_value: String,
    #[serde(rename = "TransactionCashbackUnit")]
    pub cashback_unit: String,
    #[serde(rename = "TransactionId")]
    pub transaction_id: String,
    #[serde(rename = "TransactionAuthCode")]
    pub auth_code: String,
}

pub fn iceland_transaction(transactions: Vec<Transaction>) -> Result<String> {
    let mut wtr = WriterBuilder::new().from_writer(vec![]);

    // TODO: card_scheme name and number, first six and last four.
    for transaction in transactions {
        let iceland_tx = IcelandTransaction {
            first_six: "123456".to_string(),
            last_four: "4444".to_string(),
            expiry: "01/80".to_string(),
            card_scheme_id: "6".to_string(),
            card_scheme_name: "Visa Debit".to_string(),
            identifier: transaction.identifier.clone(),
            transaction_date: transaction.transaction_date.to_string(),
            amount: to_pounds(transaction.amount)?,
            amount_unit: "GBP".to_string(),
            cashback_value: ".00".to_string(),
            cashback_unit: "GBP".to_string(),
            transaction_id: transaction.transaction_id,
            auth_code: transaction.auth_code.clone(),
        };

        wtr.serialize(iceland_tx)?;
    }
    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn iceland_transaction_valid() {
        let dt = Utc::now();

        let test_transactions = vec![
            Transaction {
                amount: 260,
                transaction_date: dt,
                merchant_name: "Bink toffee".to_string(),
                transaction_id: "1234567890987654321234567".to_string(),
                auth_code: "098765".to_string(),
                identifier: "1111111111".to_string(),
                token: "a_token_001".to_string(),
            },
            Transaction {
                amount: 4267,
                transaction_date: dt,
                merchant_name: "Bink toffee".to_string(),
                transaction_id: "12345678909887654".to_string(),
                auth_code: "023454".to_string(),
                identifier: "1111111112".to_string(),
                token: "a_token_002".to_string(),
            },
        ];

        let iceland_tx = iceland_transaction(test_transactions).unwrap();

        assert_eq!(iceland_tx.len(), 519);
    }
}
