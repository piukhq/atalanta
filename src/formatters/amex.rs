use crate::{formatters::to_pounds, models::Transaction};
use chrono::FixedOffset;
use color_eyre::{eyre::eyre, Result};
use serde_json::json;

use super::Formatter;

pub struct AuthFormatter;

impl Formatter for AuthFormatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let transaction = transactions
            .into_iter()
            .next()
            .ok_or_else(|| eyre!("Expected at least one transaction."))?;

        let mst_timezone =
            FixedOffset::west_opt(7 * 60 * 60).ok_or(eyre!("Failed to construct MST timezone"))?;
        let mst_datetime = transaction.transaction_date.with_timezone(&mst_timezone);
        let auth = json!({
            "transaction_id": transaction.transaction_id,
            "offer_id": transaction.transaction_id,
            "transaction_time": mst_datetime.to_string(),
            "transaction_amount": to_pounds(transaction.amount),
            "cm_alias": transaction.token,
            "merchant_number": transaction.identifier,
            "approval_code": transaction.auth_code,
        });

        Ok(auth.to_string())
    }
}

struct AmexSettlementFormatter;

impl Formatter for AmexSettlementFormatter {
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

    use super::*;

    #[test]
    fn to_pounds_success() {
        let pounds = to_pounds(235);
        assert_eq!(pounds, "2.35");
    }

    #[test]
    fn amex_auth_valid() -> Result<()> {
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

        let json_result = AuthFormatter::format(vec![test_transaction]);
        let mst_timezone = FixedOffset::west_opt(7 * 60 * 60)
            .ok_or_else(|| eyre!("failed to create MST timezone"))?;
        let auth_tx_json = json!({
            "transaction_id": "test_transaction_id_1",
            "offer_id": "test_transaction_id_1",
            "transaction_time": dt.with_timezone(&mst_timezone).to_string(),
            "transaction_amount": "2.45",
            "cm_alias": "98765432123456789",
            "merchant_number": "12345678",
            "approval_code": "123456",
        });

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_result?)?,
            auth_tx_json
        );

        Ok(())
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

        let json_result = AmexSettlementFormatter::format(vec![test_transaction]);
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
