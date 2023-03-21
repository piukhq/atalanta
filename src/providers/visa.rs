#![warn(clippy::unwrap_used, clippy::expect_used)]

use crate::{models::Transaction, providers::to_pounds};
use chrono::Utc;
use color_eyre::Result;
use serde_json::json;

pub fn visa_auth(transaction: &Transaction) -> Result<String> {
    let amount = to_pounds(transaction.amount);
    let auth = json!({
        "CardId": transaction.transaction_id[0..9],
        "ExternalUserId": transaction.token,
        "MessageElementsCollection": [
            {"Key": "Transaction.BillingAmount", "Value": amount},
            {"Key": "Transaction.TimeStampYYMMDD", "Value": transaction.transaction_date.to_string()},
            {"Key": "Transaction.MerchantCardAcceptorId", "Value": transaction.identifier},
            {"Key": "Transaction.MerchantAcquirerBin", "Value": "3423432"},
            {"Key": "Transaction.TransactionAmount", "Value": amount},
            {"Key": "Transaction.VipTransactionId", "Value": transaction.transaction_id},
            {"Key": "Transaction.VisaMerchantName", "Value": "Bink Shop"},
            {"Key": "Transaction.VisaMerchantId", "Value": transaction.identifier},
            {"Key": "Transaction.VisaStoreName", "Value": "Bink Shop"},
            {"Key": "Transaction.VisaStoreId", "Value": transaction.identifier},
            {"Key": "Transaction.SettlementDate", "Value": ""},
            {"Key": "Transaction.SettlementAmount", "Value": 0},
            {"Key": "Transaction.SettlementCurrencyCodeNumeric", "Value": 0},
            {"Key": "Transaction.SettlementBillingAmount", "Value": 0},
            {"Key": "Transaction.SettlementBillingCurrency", "Value": ""},
            {"Key": "Transaction.SettlementUSDAmount", "Value": 0},
            {"Key": "Transaction.CurrencyCodeNumeric", "Value": "840"},
            {"Key": "Transaction.BillingCurrencyCode", "Value": "840"},
            {"Key": "Transaction.USDAmount", "Value": amount},
            {"Key": "Transaction.MerchantLocalPurchaseDate ", "Value": "2019-12-19"},
            {"Key": "Transaction.MerchantGroup.0.Name", "Value": "TEST_MG"},
            {"Key": "Transaction.MerchantGroup.0.ExternalId", "Value": "MYSTORE"},
            {"Key": "Transaction.MerchantDateTimeGMT ", "Value": "2019-12-19T23:40:00"},
            {"Key": "Transaction.AuthCode", "Value": transaction.auth_code},
            {"Key": "Transaction.PanLastFour", "Value": transaction.last_four},
        ],
        "MessageId": "12345678",
        "MessageName": "AuthMessageTest",
        "UserDefinedFieldsCollection": [{"Key": "TransactionType", "Value": "AUTH"}],
        "UserProfileId": "f292f99d-babf-528a-8d8a-19fa5f14f4"
    });

    Ok(auth.to_string())
}

pub fn visa_settlement(transaction: &Transaction) -> Result<String> {
    let amount = to_pounds(transaction.amount);
    let settlement = json!(
        {
            "CardId": transaction.transaction_id[0..9],
            "ExternalUserId": transaction.token,
            "MessageElementsCollection": [
                {"Key": "Transaction.BillingAmount", "Value": amount},
                {"Key": "Transaction.TimeStampYYMMDD", "Value": transaction.transaction_date.to_string()},
                {"Key": "Transaction.MerchantCardAcceptorId", "Value": transaction.identifier},
                {"Key": "Transaction.MerchantAcquirerBin", "Value": "3423432"},
                {"Key": "Transaction.TransactionAmount", "Value": amount},
                {"Key": "Transaction.VipTransactionId", "Value": transaction.transaction_id},
                {"Key": "Transaction.VisaMerchantName", "Value": "Bink Shop"},
                {"Key": "Transaction.VisaMerchantId", "Value": transaction.identifier},
                {"Key": "Transaction.VisaStoreName", "Value": "Bink Shop"},
                {"Key": "Transaction.VisaStoreId", "Value": transaction.identifier},
                {"Key": "Transaction.SettlementDate", "Value": Utc::now()},
                {"Key": "Transaction.SettlementAmount", "Value": amount},
                {"Key": "Transaction.SettlementCurrencyCodeNumeric", "Value": 826},
                {"Key": "Transaction.SettlementBillingAmount", "Value": amount},
                {"Key": "Transaction.SettlementBillingCurrency", "Value": "GBP"},
                {"Key": "Transaction.SettlementUSDAmount", "Value": amount},
                {"Key": "Transaction.CurrencyCodeNumeric", "Value": "840"},
                {"Key": "Transaction.BillingCurrencyCode", "Value": "840"},
                {"Key": "Transaction.USDAmount", "Value": amount},
                {"Key": "Transaction.MerchantLocalPurchaseDate", "Value": "2019-12-19"},
                {"Key": "Transaction.MerchantGroup.0.Name", "Value": "TEST_MG"},
                {"Key": "Transaction.MerchantGroup.0.ExternalId", "Value": "MYSTORE"},
                {"Key": "Transaction.MerchantDateTimeGMT", "Value": transaction.transaction_date.to_string()},
                {"Key": "Transaction.AuthCode", "Value": transaction.auth_code},
                {"Key": "Transaction.PanLastFour", "Value": transaction.last_four},
            ],
            "MessageId": "12345678",
            "MessageName": "SettlementMessageTest",
            "UserDefinedFieldsCollection": [{"Key": "TransactionType", "Value": "SETTLE"}],
            "UserProfileId": "f292f99d-babf-528a-8d8a-19fa5f14f4",
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
        let pounds = to_pounds(235);
        assert_eq!("2.35", pounds);
    }

    #[test]
    fn visa_auth_valid() {
        let dt = Utc::now();
        let test_transaction = Transaction {
            amount: 245,
            transaction_date: dt,
            payment_provider: "visa".to_owned(),
            merchant_name: "test_merchant".to_string(),
            transaction_id: "test_transaction_id_1".to_string(),
            auth_code: "123456".to_string(),
            identifier: "12345678".to_string(),
            token: "98765432123456789".to_string(),
            first_six: "123456".to_owned(),
            last_four: "7890".to_owned(),
        };

        let json_result = visa_auth(&test_transaction);
        assert_eq!(
            json_result.unwrap(),
            json!({
                "CardId": "test_tran",
                "ExternalUserId": "98765432123456789",
                "MessageElementsCollection": [
                    {"Key": "Transaction.BillingAmount", "Value": "2.45"},
                    {"Key": "Transaction.TimeStampYYMMDD", "Value": dt.to_string()},
                    {"Key": "Transaction.MerchantCardAcceptorId", "Value": "12345678"},
                    {"Key": "Transaction.MerchantAcquirerBin", "Value": "3423432"},
                    {"Key": "Transaction.TransactionAmount", "Value": "2.45"},
                    {"Key": "Transaction.VipTransactionId", "Value": "test_transaction_id_1"},
                    {"Key": "Transaction.VisaMerchantName", "Value": "Bink Shop"},
                    {"Key": "Transaction.VisaMerchantId", "Value": "12345678"},
                    {"Key": "Transaction.VisaStoreName", "Value": "Bink Shop"},
                    {"Key": "Transaction.VisaStoreId", "Value": "12345678"},
                    {"Key": "Transaction.SettlementDate", "Value": ""},
                    {"Key": "Transaction.SettlementAmount", "Value": 0},
                    {"Key": "Transaction.SettlementCurrencyCodeNumeric", "Value": 0},
                    {"Key": "Transaction.SettlementBillingAmount", "Value": 0},
                    {"Key": "Transaction.SettlementBillingCurrency", "Value": ""},
                    {"Key": "Transaction.SettlementUSDAmount", "Value": 0},
                    {"Key": "Transaction.CurrencyCodeNumeric", "Value": "840"},
                    {"Key": "Transaction.BillingCurrencyCode", "Value": "840"},
                    {"Key": "Transaction.USDAmount", "Value": "2.45"},
                    {"Key": "Transaction.MerchantLocalPurchaseDate ", "Value": "2019-12-19"},
                    {"Key": "Transaction.MerchantGroup.0.Name", "Value": "TEST_MG"},
                    {"Key": "Transaction.MerchantGroup.0.ExternalId", "Value": "MYSTORE"},
                    {"Key": "Transaction.MerchantDateTimeGMT ", "Value": "2019-12-19T23:40:00"},
                    {"Key": "Transaction.AuthCode", "Value": "123456"},
                    {"Key": "Transaction.PanLastFour", "Value": "2345"},
                    ],
                    "MessageId": "12345678",
                    "MessageName": "AuthMessageTest",
                    "UserDefinedFieldsCollection": [{"Key": "TransactionType", "Value": "AUTH"}],
                    "UserProfileId": "f292f99d-babf-528a-8d8a-19fa5f14f4"
            })
            .to_string()
        );
    }
}
