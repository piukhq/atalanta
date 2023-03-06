#![warn(clippy::unwrap_used, clippy::expect_used)]

use crate::{models::Transaction, providers::to_pounds};
use chrono::FixedOffset;
use color_eyre::Result;
use serde_json::json;

pub fn amex_auth(transaction: &Transaction) -> Result<String> {
    let mst_timezone = FixedOffset::west_opt(7 * 60 * 60).unwrap();
    let mst_datetime = transaction.transaction_date.with_timezone(&mst_timezone);
    let auth = json!({
        "transaction_id": transaction.transaction_id,
        "offer_id": transaction.transaction_id,
        "transaction_time": mst_datetime.to_string(),
        "transaction_amount": to_pounds(transaction.amount)?,
        "cm_alias": transaction.token,
        "merchant_number": transaction.identifier,
        "approval_code": transaction.auth_code,
    });

    Ok(auth.to_string())
}

pub fn amex_settlement(transaction: &Transaction) -> Result<String> {
    println!("{}", transaction.merchant_name);

    let settlement = json!(
        {
            "transactionId": transaction.transaction_id,
            "offerId": transaction.transaction_id,
            "transactionDate": transaction.transaction_date.to_string(),
            "transactionAmount": to_pounds(transaction.amount)?,
            "cardToken": transaction.token,
            "merchantNumber": transaction.identifier,
            "approvalCode": transaction.auth_code,
            "dpan": "firstsixXXXXXlastfour",
            "partnerId": "AADP0050",
            "recordId": "0224133845625011230183160001602891525AADP00400",
            "currencyCode": "840"
        }
    );

    Ok(settlement.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn to_pounds_success() {
        let pounds = to_pounds(235).unwrap();
        println!("Pounds = {}", pounds);
        assert_eq!("2.35", pounds);
    }

    #[test]
    fn amex_auth_valid() {
        let dt = Utc::now();
        let test_transaction =Transaction {
            amount: 245,
            transaction_date: dt,
            merchant_name: "test_merchant".to_string(),
            transaction_id: "test_transaction_id_1".to_string(),
            auth_code: "123456".to_string(),
            identifier: "12345678".to_string(),
            token: "98765432123456789".to_string(),
        };

        let json_result = amex_auth(&test_transaction);
        let mst_timezone = FixedOffset::west_opt(7 * 60 * 60).unwrap();
        let auth_tx_json = json!({
            "transaction_id": "test_transaction_id_1".to_string(),
            "offer_id": "test_transaction_id_1".to_string(),
            "transaction_time": dt.with_timezone(&mst_timezone).to_string(),
            "transaction_amount": "2.45",
            "cm_alias": "98765432123456789".to_string(),
            "merchant_number": "12345678".to_string(),
            "approval_code": "123456".to_string(),
        }).to_string();
    
        assert_eq!(
            json_result.unwrap(),
            auth_tx_json
        );
    }

    #[test]
    fn amex_settlement_valid() {
        let dt = Utc::now();
        let test_transaction =Transaction {
            amount: 245,
            transaction_date: dt,
            merchant_name: "test_merchant".to_string(),
            transaction_id: "test_transaction_id_1".to_string(),
            auth_code: "123456".to_string(),
            identifier: "12345678".to_string(),
            token: "98765432123456789".to_string(),
        };

        let json_result = amex_settlement(&test_transaction);
        let settlement_tx_json = json!({
            "transactionId": "test_transaction_id_1".to_string(),
            "offerId": "test_transaction_id_1".to_string(),
            "transactionDate": dt.to_string(),
            "transactionAmount": "2.45",
            "cardToken": "98765432123456789".to_string(),
            "merchantNumber": "12345678".to_string(),
            "approvalCode": "123456".to_string(),
            "dpan": "firstsixXXXXXlastfour",
            "partnerId": "AADP0050",
            "recordId": "0224133845625011230183160001602891525AADP00400",
            "currencyCode": "840"
        }).to_string();
    
        assert_eq!(
            json_result.unwrap(),
            settlement_tx_json
        );
    }
}
