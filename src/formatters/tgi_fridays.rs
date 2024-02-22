#![warn(clippy::unwrap_used, clippy::expect_used)]

use crate::{formatters::to_pounds, models::Transaction};
use color_eyre::Result;
use serde_json::json;

use super::Formatter;

pub struct TGIFridaysFormatter;

fn gratuitise(amount: i64) -> (i64, i64) {
    let gratuity = (amount as f64 * 0.1).round() as i64;
    (amount - gratuity, gratuity)
}

impl Formatter for TGIFridaysFormatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let tgi_fridays_transactions = transactions
            .into_iter()
            .map(|transaction| {
                let (amount, gratuity) = gratuitise(transaction.amount);
                json!({
                    "transaction_id": transaction.transaction_id,
                    "payment_card_type": card_type_name(transaction.payment_provider.as_str()),
                    "payment_card_first_six": transaction.first_six,
                    "payment_card_last_four": transaction.last_four,
                    "amount": to_pounds(amount),
                    "gratuity_amount": to_pounds(gratuity),
                    "currency_code": "GBP",
                    "auth_code": transaction.auth_code,
                    "date": transaction.transaction_date.to_rfc3339(),
                    "merchant_identifier": transaction.identifier,
                    "retailer_location_id": transaction.identifier,
                })
            })
            .collect::<Vec<_>>();

        Ok(serde_json::to_string(&tgi_fridays_transactions)?)
    }
}

fn card_type_name(payment_provider: &str) -> String {
    match payment_provider {
        "amex" => "AMEX CREDIT".to_owned(),
        "mastercard" => "DEBIT MASTERCARD".to_owned(),
        "visa" => "VISA DEBIT".to_owned(),
        _ => "Unknown".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn tgi_fridays_valid() -> Result<()> {
        let dt = Utc::now();
        let test_transactions = vec![
            Transaction {
                amount: 245,
                transaction_date: dt,
                payment_provider: "visa".to_owned(),
                merchant_name: "test_merchant".to_owned(),
                transaction_id: "test_transaction_id_1".to_owned(),
                auth_code: "123456".to_owned(),
                identifier: "12345678".to_owned(),
                identifier_type: "PRIMARY".to_owned(),
                token: "98765432123456789".to_owned(),
                first_six: "123456".to_owned(),
                last_four: "7890".to_owned(),
            },
            Transaction {
                amount: 735,
                transaction_date: dt,
                payment_provider: "visa".to_owned(),
                merchant_name: "test_merchant2".to_owned(),
                transaction_id: "test_transaction_id_2".to_owned(),
                auth_code: "654321".to_owned(),
                identifier: "87654321".to_owned(),
                identifier_type: "PRIMARY".to_owned(),
                token: "98765432123456789".to_owned(),
                first_six: "123456".to_owned(),
                last_four: "7890".to_owned(),
            },
        ];

        let json_result = TGIFridaysFormatter::format(test_transactions);
        let expected_tgi_fridays_tx_json = json!([
            {
                "transaction_id": "test_transaction_id_1",
                "payment_card_type": "VISA DEBIT",
                "payment_card_first_six": "123456",
                "payment_card_last_four": "7890",
                "amount": to_pounds(220),
                "gratuity_amount": to_pounds(25),
                "currency_code": "GBP",
                "auth_code": "123456",
                "date": dt.to_rfc3339(),
                "merchant_identifier": "12345678",
                "retailer_location_id": "12345678",
            },
            {
                "transaction_id": "test_transaction_id_2",
                "payment_card_type": "VISA DEBIT",
                "payment_card_first_six": "123456",
                "payment_card_last_four": "7890",
                "amount": to_pounds(661),
                "gratuity_amount": to_pounds(74),
                "currency_code": "GBP",
                "auth_code": "654321",
                "date": dt.to_rfc3339(),
                "merchant_identifier": "87654321",
                "retailer_location_id": "87654321",
            }
        ]);

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_result?)?,
            expected_tgi_fridays_tx_json
        );

        Ok(())
    }
}
