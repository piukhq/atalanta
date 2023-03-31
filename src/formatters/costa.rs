#![warn(clippy::unwrap_used, clippy::expect_used)]

use crate::{formatters::to_pounds, models::Transaction};
use color_eyre::Result;
use serde_json::json;

use super::Formatter;

pub struct CostaFormatter;

impl Formatter for CostaFormatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let mut costa_transactions = json!([]);

        for transaction in transactions {
            let tx = json!({
                "transaction_id": transaction.transaction_id,
                "payment_card_type": transaction.payment_provider,
                "payment_card_first_six": transaction.first_six,
                "payment_card_last_four": transaction.last_four,
                "amount": to_pounds(transaction.amount),
                "currency_code": "GBP",
                "auth_code": transaction.auth_code,
                "date": transaction.transaction_date,
                "merchant_identifier": transaction.identifier,
                "retailer_location_id": transaction.auth_code,
                "metadata": include_str!("costa_metadata.json"),
                "items_ordered": include_str!("costa_order_items.json")
            });

            costa_transactions.as_array_mut().unwrap().push(tx);
        }

        Ok(costa_transactions.to_string())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn costa_valid() {
        let dt = Utc::now();
        let test_transactions =  vec![
            Transaction {
                amount: 245,
                transaction_date: dt,
                payment_provider: "visa".to_owned(),
                merchant_name: "test_merchant".to_owned(),
                transaction_id: "test_transaction_id_1".to_owned(),
                auth_code: "123456".to_owned(),
                identifier: "12345678".to_owned(),
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
                token: "98765432123456789".to_owned(),
                first_six: "123456".to_owned(),
                last_four: "7890".to_owned(),
            },

        ];

        let json_result = CostaFormatter::format(test_transactions);

        let expected_costa_tx_json = json!([{
            "transaction_id": "test_transaction_id_1",
            "payment_card_type": "visa",
            "payment_card_first_six": "123456",
            "payment_card_last_four": "7890",
            "amount": to_pounds(245),
            "currency_code": "GBP",
            "auth_code": "123456",
            "date": dt,
            "merchant_identifier": "12345678",
            "retailer_location_id": "123456",
            "metadata": include_str!("costa_metadata.json"),
            "items_ordered": include_str!("costa_order_items.json")
            },
            {
                "transaction_id": "test_transaction_id_2",
                "payment_card_type": "visa",
                "payment_card_first_six": "123456",
                "payment_card_last_four": "7890",
                "amount": to_pounds(735),
                "currency_code": "GBP",
                "auth_code": "654321",
                "date": dt,
                "merchant_identifier": "87654321",
                "retailer_location_id": "654321",
                "metadata": include_str!("costa_metadata.json"),
                "items_ordered": include_str!("costa_order_items.json")
            }
        ]);

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_result.unwrap()).unwrap(),
            expected_costa_tx_json
        );
    }
}
