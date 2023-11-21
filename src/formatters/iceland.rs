#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::{formatters::to_pounds, models::Transaction};
use chrono_tz::Europe::London;
use chrono::prelude::*;
use color_eyre::Result;
use csv::WriterBuilder;
use serde::Serialize;

use super::Formatter;

#[derive(Serialize)]
pub struct IcelandTransaction {
    #[serde(rename = "TransactionCardFirst6")]
    pub first_six: String,
    #[serde(rename = "TransactionCardLast4")]
    pub last_four: String,
    #[serde(rename = "TransactionCardExpiry")]
    pub expiry: String,
    #[serde(rename = "TransactionCardSchemeId")]
    pub card_scheme_id: String,
    #[serde(rename = "TransactionCardScheme")]
    pub card_scheme_name: String,
    #[serde(rename = "TransactionStore_Id")]
    pub identifier: String,
    #[serde(rename = "TransactionTimestamp")]
    pub transaction_date: String,
    #[serde(rename = "TransactionAmountValue")]
    pub amount: String,
    #[serde(rename = "TransactionAmountUnit")]
    pub amount_unit: String,
    #[serde(rename = "TransactionCashbackValue")]
    pub cashback_value: String,
    #[serde(rename = "TransactionCashbackUnit")]
    pub cashback_unit: String,
    #[serde(rename = "TransactionId")]
    pub transaction_id: String,
    #[serde(rename = "TransactionAuthCode")]
    pub auth_code: String,
}

pub struct IcelandFormatter;

impl Formatter for IcelandFormatter {
    fn format(transactions: Vec<Transaction>) -> Result<String> {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);

        // TODO: card_scheme name and number, first six and last four.
        for transaction in transactions {
            let iceland_tx = IcelandTransaction {
                first_six: transaction.first_six,
                last_four: transaction.last_four,
                expiry: "01/80".to_owned(),
                card_scheme_id: "6".to_owned(),
                card_scheme_name: card_type_name(transaction.payment_provider.as_str()),
                identifier: transaction.identifier.clone(),
                transaction_date: date_to_timezone(&transaction.transaction_date),
                amount: to_pounds(transaction.amount),
                amount_unit: "GBP".to_owned(),
                cashback_value: ".00".to_owned(),
                cashback_unit: "GBP".to_owned(),
                transaction_id: transaction.transaction_id,
                auth_code: transaction.auth_code.clone(),
            };

            wtr.serialize(iceland_tx)?;
        }
        let data = String::from_utf8(wtr.into_inner()?)?;

        Ok(data)
    }
}

fn card_type_name(payment_provider: &str) -> String {
    match payment_provider {
        "amex" => "Amex".to_owned(),
        "mastercard" => "MasterCard/MasterCard One".to_owned(),
        "visa" => "Visa".to_owned(),
        _ => "Unknown".to_owned(),
    }
}

fn date_to_timezone(date: &DateTime<Utc>) -> String {
    let tz_date = date.with_timezone(&London);
    return tz_date.format("%Y-%m-%d %H:%M:%S").to_string();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use pretty_assertions::assert_eq;

    #[test]
    fn iceland_transaction_valid() -> Result<()> {
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

        let iceland_tx = IcelandFormatter::format(test_transactions)?;

        assert_eq!(iceland_tx.len(), 485);

        Ok(())
    }

    #[test]
    fn convert_to_timezone_datetime() -> Result<()> {
        // All transaction dates and times are generated as UTC. Therefore we
        // need to make datetimes created during summer months for Iceland to be aware of daylight savings.
        // Check the time changes across a daylight savings change
        let naivedatetime_utc = NaiveDate::from_ymd_opt(2016, 10, 29).unwrap().and_hms_opt(12, 0, 0).unwrap();
        let datetime_utc = DateTime::<Utc>::from_utc(naivedatetime_utc, Utc);

        let day_later = datetime_utc + Duration::hours(24); // same datetime 24 hours later in GMT/UTC


        assert_eq!("2016-10-29 13:00:00", date_to_timezone(&datetime_utc));
        assert_eq!("2016-10-30 12:00:00", date_to_timezone(&day_later));
        Ok(())
    }

}
