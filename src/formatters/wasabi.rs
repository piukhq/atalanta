use crate::{formatters::to_pounds, models::Transaction};
use color_eyre::Result;
use csv::Writer;
use rand::Rng;
use serde::Serialize;

#[derive(Serialize)]
pub struct TransactionRecord {
    #[serde(rename = "Store No_")]
    pub store_no: String,
    #[serde(rename = "Entry No_")]
    pub entry_no: String,
    #[serde(rename = "Transaction No_")]
    pub transaction_no: String,
    #[serde(rename = "Tender Type")]
    pub tender_type: String,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Card Number")]
    pub card_number: String,
    #[serde(rename = "Card Type Name")]
    pub card_type_name: String,
    #[serde(rename = "Auth_code")]
    pub auth_code: String,
    #[serde(rename = "Authorisation Ok")]
    pub authorization_ok: String,
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Time")]
    pub time: String,
    #[serde(rename = "EFT Merchant No_")]
    pub eft_merchant_no: String,
    #[serde(rename = "Receipt No_")]
    pub receipt_no: String,
}

pub struct Formatter;

impl super::Formatter for Formatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let mut wtr = Writer::from_writer(vec![]);

        for transaction in transactions {
            let wasabi_tx = TransactionRecord {
                store_no: "A076".to_owned(),
                entry_no: "16277".to_owned(),
                transaction_no: transaction.transaction_id,
                tender_type: "3".to_owned(),
                amount: to_pounds(transaction.amount),
                card_number: format!("{}******{}", transaction.first_six, transaction.last_four),
                card_type_name: card_type_name(transaction.payment_provider.as_str()),
                auth_code: transaction.auth_code.clone(),
                authorization_ok: "1".to_owned(),
                date: transaction.transaction_date.format("%d/%m/%Y").to_string(),
                time: transaction.transaction_date.format("%H:%M:%S").to_string(),
                eft_merchant_no: transaction.identifier.clone(),
                receipt_no: format!("0000A{}", padded_random_int(13, 14)),
            };

            wtr.serialize(wasabi_tx)?;
        }
        let data = String::from_utf8(wtr.into_inner()?)?;
        Ok(data)
    }
}

fn padded_random_int(raise_power: u32, num_chars: u32) -> String {
    let upper_value = 10_u64.pow(raise_power);
    let number = rand::thread_rng().gen_range(1..upper_value);
    format!("{:0num_chars$}", number, num_chars = num_chars as usize)
}

fn card_type_name(payment_provider: &str) -> String {
    match payment_provider {
        "amex" => "American Express".to_owned(),
        "mastercard" => "Mastercard".to_owned(),
        "visa" => "Visa".to_owned(),
        _ => "Unknown".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::formatters::Formatter as _;

    use super::*;
    use chrono::Utc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_padded_random_int() {
        let value = padded_random_int(13, 14);
        assert_eq!(value.len(), 14);
    }

    #[test]
    fn wasabi_transaction_valid() -> Result<()> {
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
                identifier_type: "PRIMARY".to_owned(),
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
                identifier_type: "PRIMARY".to_owned(),
                token: "a_token_002".to_owned(),
                first_six: "123456".to_owned(),
                last_four: "7890".to_owned(),
            },
        ];

        let wasabi_tx = Formatter::format(test_transactions)?;

        // 1 header, 2 transactions, 1 newline
        assert_eq!(wasabi_tx.split('\n').count(), 4);

        Ok(())
    }
}
