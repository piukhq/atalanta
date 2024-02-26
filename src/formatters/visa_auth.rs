use crate::{formatters::to_pounds, models::Transaction};
use color_eyre::{eyre::eyre, Result};
use serde_json::json;

pub struct Formatter;

impl super::Formatter for Formatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let transaction = transactions
            .into_iter()
            .next()
            .ok_or_else(|| eyre!("Expected at least one transaction."))?;

        let amount = to_pounds(transaction.amount);
        let date = transaction.transaction_date.to_rfc3339();
        let auth = json!({
            "CardId": transaction.transaction_id[0..9],
            "ExternalUserId": transaction.token,
            "MessageElementsCollection": [
                {"Key": "Transaction.BillingAmount", "Value": amount},
                {"Key": "Transaction.TimeStampYYMMDD", "Value": date},
                {"Key": "Transaction.MerchantCardAcceptorId", "Value": primary_identifier(&transaction)},
                {"Key": "Transaction.MerchantAcquirerBin", "Value": "3423432"},
                {"Key": "Transaction.TransactionAmount", "Value": amount},
                {"Key": "Transaction.VipTransactionId", "Value": transaction.transaction_id},
                {"Key": "Transaction.VisaMerchantName", "Value": transaction.merchant_name},
                {"Key": "Transaction.VisaMerchantId", "Value": psimi_identifier(&transaction)},
                {"Key": "Transaction.VisaStoreName", "Value": "Bink Shop"},
                {"Key": "Transaction.VisaStoreId", "Value": secondary_identifier(&transaction)},
                {"Key": "Transaction.SettlementDate", "Value": ""},
                {"Key": "Transaction.SettlementAmount", "Value": 0},
                {"Key": "Transaction.SettlementCurrencyCodeNumeric", "Value": 0},
                {"Key": "Transaction.SettlementBillingAmount", "Value": 0},
                {"Key": "Transaction.SettlementBillingCurrency", "Value": ""},
                {"Key": "Transaction.SettlementUSDAmount", "Value": 0},
                {"Key": "Transaction.CurrencyCodeNumeric", "Value": "840"},
                {"Key": "Transaction.BillingCurrencyCode", "Value": "840"},
                {"Key": "Transaction.USDAmount", "Value": amount},
                {"Key": "Transaction.MerchantLocalPurchaseDate", "Value": "2019-12-19"},
                {"Key": "Transaction.MerchantGroup.0.Name", "Value": "TEST_MG"},
                {"Key": "Transaction.MerchantGroup.0.ExternalId", "Value": "MYSTORE"},
                {"Key": "Transaction.MerchantDateTimeGMT", "Value": date},
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
}

fn primary_identifier(transaction: &Transaction) -> String {
    match transaction.identifier_type.as_str() {
        "PRIMARY" => transaction.identifier.clone(),
        _ => "PRIM11111".to_owned(),
    }
}

fn secondary_identifier(transaction: &Transaction) -> String {
    match transaction.identifier_type.as_str() {
        "SECONDARY" => transaction.identifier.clone(),
        _ => "SEC222222".to_owned(),
    }
}

fn psimi_identifier(transaction: &Transaction) -> String {
    match transaction.identifier_type.as_str() {
        "PSIMI" => transaction.identifier.clone(),
        _ => "PSIMI3333".to_owned(),
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
        assert_eq!("2.35", pounds);
    }

    #[test]
    fn visa_auth_valid() -> Result<()> {
        let dt = Utc::now();
        let test_transaction = Transaction {
            amount: 245,
            transaction_date: dt,
            payment_provider: "visa".to_owned(),
            merchant_name: "Bink Shop".to_owned(),
            transaction_id: "test_transaction_id_1".to_owned(),
            auth_code: "123456".to_owned(),
            identifier: "12345678".to_owned(),
            identifier_type: "PRIMARY".to_owned(),
            token: "98765432123456789".to_owned(),
            first_six: "123456".to_owned(),
            last_four: "7890".to_owned(),
        };

        let json_result = Formatter::format(vec![test_transaction]);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json_result?)?,
            json!({
                "CardId": "test_tran",
                "ExternalUserId": "98765432123456789",
                "MessageElementsCollection": [
                    {"Key": "Transaction.BillingAmount", "Value": "2.45"},
                    {"Key": "Transaction.TimeStampYYMMDD", "Value": dt.to_rfc3339()},
                    {"Key": "Transaction.MerchantCardAcceptorId", "Value": "12345678"},
                    {"Key": "Transaction.MerchantAcquirerBin", "Value": "3423432"},
                    {"Key": "Transaction.TransactionAmount", "Value": "2.45"},
                    {"Key": "Transaction.VipTransactionId", "Value": "test_transaction_id_1"},
                    {"Key": "Transaction.VisaMerchantName", "Value": "Bink Shop"},
                    {"Key": "Transaction.VisaMerchantId", "Value": "PSIMI3333"},
                    {"Key": "Transaction.VisaStoreName", "Value": "Bink Shop"},
                    {"Key": "Transaction.VisaStoreId", "Value": "SEC222222"},
                    {"Key": "Transaction.SettlementDate", "Value": ""},
                    {"Key": "Transaction.SettlementAmount", "Value": 0},
                    {"Key": "Transaction.SettlementCurrencyCodeNumeric", "Value": 0},
                    {"Key": "Transaction.SettlementBillingAmount", "Value": 0},
                    {"Key": "Transaction.SettlementBillingCurrency", "Value": ""},
                    {"Key": "Transaction.SettlementUSDAmount", "Value": 0},
                    {"Key": "Transaction.CurrencyCodeNumeric", "Value": "840"},
                    {"Key": "Transaction.BillingCurrencyCode", "Value": "840"},
                    {"Key": "Transaction.USDAmount", "Value": "2.45"},
                    {"Key": "Transaction.MerchantLocalPurchaseDate", "Value": "2019-12-19"},
                    {"Key": "Transaction.MerchantGroup.0.Name", "Value": "TEST_MG"},
                    {"Key": "Transaction.MerchantGroup.0.ExternalId", "Value": "MYSTORE"},
                    {"Key": "Transaction.MerchantDateTimeGMT", "Value": dt.to_rfc3339()},
                    {"Key": "Transaction.AuthCode", "Value": "123456"},
                    {"Key": "Transaction.PanLastFour", "Value": "7890"},
                    ],
                    "MessageId": "12345678",
                    "MessageName": "AuthMessageTest",
                    "UserDefinedFieldsCollection": [{"Key": "TransactionType", "Value": "AUTH"}],
                    "UserProfileId": "f292f99d-babf-528a-8d8a-19fa5f14f4"
            })
        );

        Ok(())
    }
}
