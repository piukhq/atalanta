use crate::{formatters::to_pounds, models::Transaction};
use color_eyre::Result;
use serde_json::json;

pub struct Formatter;

impl super::Formatter for Formatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let metadata: serde_json::Value =
            serde_json::from_str(include_str!("stonegate_metadata.json"))?;
        let stonegate_transactions = transactions
            .into_iter()
            .map(|transaction| {
                json!({
                    "transaction_id": transaction.transaction_id,
                    "payment_card_type": card_type_name(transaction.payment_provider.as_str()),
                    "payment_card_first_six": transaction.first_six,
                    "payment_card_last_four": transaction.last_four,
                    "amount": to_pounds(transaction.amount),
                    "currency_code": "GBP",
                    "auth_code": transaction.auth_code,
                    "date": transaction.transaction_date.to_rfc3339(),
                    "merchant_identifier": transaction.identifier,
                    "retailer_location_id": transaction.identifier,
                    "metadata": metadata,
                    "items_ordered": include_str!("stonegate_order_items.json")
                })
            })
            .collect::<Vec<_>>();

        Ok(serde_json::to_string(&stonegate_transactions)?)
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

    use crate::formatters::Formatter as _;

    use super::*;

    #[test]
    fn stonegate_valid() -> Result<()> {
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

        let json_result = Formatter::format(test_transactions);

        let metadata: serde_json::Value =
            serde_json::from_str(include_str!("stonegate_metadata.json"))?;
        let expected_stonegate_tx_json = json!([
            {
                "transaction_id": "test_transaction_id_1",
                "payment_card_type": "VISA DEBIT",
                "payment_card_first_six": "123456",
                "payment_card_last_four": "7890",
                "amount": to_pounds(245),
                "currency_code": "GBP",
                "auth_code": "123456",
                "date": dt.to_rfc3339(),
                "merchant_identifier": "12345678",
                "retailer_location_id": "12345678",
                "metadata": metadata,
                "items_ordered": include_str!("stonegate_order_items.json")
            },
            {
                "transaction_id": "test_transaction_id_2",
                "payment_card_type": "VISA DEBIT",
                "payment_card_first_six": "123456",
                "payment_card_last_four": "7890",
                "amount": to_pounds(735),
                "currency_code": "GBP",
                "auth_code": "654321",
                "date": dt.to_rfc3339(),
                "merchant_identifier": "87654321",
                "retailer_location_id": "87654321",
                "metadata": metadata,
                "items_ordered": include_str!("stonegate_order_items.json")
            }
        ]);

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_result?)?,
            expected_stonegate_tx_json
        );

        Ok(())
    }
}
