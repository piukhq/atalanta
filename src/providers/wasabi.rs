#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::{models::Transaction, providers::to_pounds};
use color_eyre::Result;
use csv::Writer;
use rand::Rng;
use serde::Serialize;

#[derive(Serialize)]
pub struct WasabiTransaction {
    pub store_no: String,
    pub entry_no: String,
    pub transaction_no: String,
    pub tender_type: String,
    pub amount: String,
    pub card_number: String,
    pub card_type_name: String,
    pub auth_code: String,
    pub authorization_ok: String,
    pub date: String,
    pub time: String,
    pub eft_merchant_no: String,
    pub receipt_no: String,
}

pub fn wasabi_transaction(transactions: Vec<Transaction>) -> Result<String> {
    let mut wtr = Writer::from_writer(vec![]);

    for transaction in transactions {
        let wasabi_tx = WasabiTransaction {
            store_no: "A076".to_owned(),
            entry_no: "16277".to_owned(),
            transaction_no: "123456789".to_owned(),
            tender_type: "3".to_owned(),
            amount: to_pounds(transaction.amount),
            card_number: format!("{}******{}", transaction.first_six, transaction.last_four),
            card_type_name: transaction.payment_provider.clone(),
            auth_code: transaction.auth_code.clone(),
            authorization_ok: "1".to_owned(),
            date: transaction.transaction_date.format("%Y-%m-%d").to_string(),
            time: transaction.transaction_date.format("%H-%M-%S").to_string(),
            eft_merchant_no: transaction.identifier.clone(),
            receipt_no: padded_random_int(12, 13),
        };

        wtr.serialize(wasabi_tx)?;
    }
    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}

fn padded_random_int(raise_power: u32, num_chars: u32) -> String {
    let upper_value = 10_u64.pow(raise_power);
    let number = rand::thread_rng().gen_range(1..upper_value);
    format!("{:0num_chars$}", number, num_chars = num_chars as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_padded_random_int() {
        let value = padded_random_int(12, 13);
        assert_eq!(value.len(), 13);
    }

    #[test]
    fn wasabi_transaction_valid() {
        let dt = Utc::now();

        let test_transactions = vec![
            Transaction {
                amount: 260,
                transaction_date: dt,
                payment_provider: "visa".to_owned(),
                merchant_name: "Bink toffee".to_owned(),
                transaction_id: "1234567890987654321234567".to_owned(),
                auth_code: "098765".to_owned(),
                identifier: "1111111111".to_owned(),
                token: "a_token_001".to_owned(),
                first_six: "123456".to_owned(),
                last_four: "7890".to_owned(),
            },
            Transaction {
                amount: 4267,
                transaction_date: dt,
                payment_provider: "visa".to_owned(),
                merchant_name: "Bink toffee".to_owned(),
                transaction_id: "12345678909887654".to_owned(),
                auth_code: "023454".to_owned(),
                identifier: "1111111112".to_owned(),
                token: "a_token_002".to_owned(),
                first_six: "123456".to_owned(),
                last_four: "7890".to_owned(),
            },
        ];

        let wasabi_tx = wasabi_transaction(test_transactions).unwrap();

        assert_eq!(wasabi_tx.len(), 368);
    }
}
