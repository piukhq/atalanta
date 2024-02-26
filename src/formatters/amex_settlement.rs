use crate::{formatters::to_pounds, models::Transaction};
use color_eyre::{eyre::eyre, Result};
use serde_json::json;

struct Formatter;

impl super::Formatter for Formatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let transaction = transactions
            .into_iter()
            .next()
            .ok_or_else(|| eyre!("Expected at least one transaction for Amex settlement."))?;

        let settlement = json!(
            {
                "transactionId": transaction.transaction_id,
                "offerId": transaction.transaction_id,
                "transactionDate": transaction.transaction_date.to_string(),
                "transactionAmount": to_pounds(transaction.amount),
                "cardToken": transaction.token,
                "merchantNumber": transaction.identifier,
                "approvalCode": transaction.auth_code,
                "dpan": "",
                "partnerId": "AADP0050",
                "recordId": "0224133845625011230183160001602891525AADP00400",
                "currencyCode": "840"
            }
        );

        Ok(settlement.to_string())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use pretty_assertions::assert_eq;

    use crate::formatters::Formatter as _;

    use super::*;

    #[test]
    fn to_pounds_success() {
        let pounds = to_pounds(235);
        assert_eq!(pounds, "2.35");
    }

    #[test]
    fn amex_settlement_valid() -> Result<()> {
        let dt = Utc::now();
        let test_transaction = Transaction {
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
        };

        let json_result = Formatter::format(vec![test_transaction]);
        let settlement_tx_json = json!({
            "transactionId": "test_transaction_id_1",
            "offerId": "test_transaction_id_1",
            "transactionDate": dt.to_string(),
            "transactionAmount": "2.45",
            "cardToken": "98765432123456789",
            "merchantNumber": "12345678",
            "approvalCode": "123456",
            "dpan": "",
            "partnerId": "AADP0050",
            "recordId": "0224133845625011230183160001602891525AADP00400",
            "currencyCode": "840"
        });

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_result?)?,
            settlement_tx_json
        );

        Ok(())
    }
}
