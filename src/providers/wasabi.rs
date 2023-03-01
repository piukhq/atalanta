#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::configuration::load_config;
use crate::models::Transaction;
use num::pow;
use color_eyre::Result;
use rust_decimal::prelude::*;

pub struct WasabiTransaction {
    pub store_no: String,
    pub entry_no: String,
    pub transaction_no: String,
    pub tender_type: String,
    pub amount: String,
    pub card_number: String,
    pub card_type_name: String,
    pub auth_code: String,
    pub authorization_ok: String,
    pub date: String,
    pub time: String,
    pub eft_merchant_no: String,
    pub receipt_no: String,
}

pub fn wasabi_transaction(transaction: &Transaction) -> Result<String> {
    let config_data: Config = load_config()?;

    let transaction = WasabiTransaction {
        store_no: "A076".to_string(),
        entry_no: "16277".to_string(),
        transaction_no: "123456789".to_string(),
        tender_type:"3".to_string(),
        amount: to_pounds(transaction.amount)?,
        card_number: "first_six******last_four".to_string(),
        card_type_name: config_data.payment_provider,
        auth_code: transaction.auth_code,
        authorization_ok: "1".to_string(),
        date: transaction.transaction_date.format("%Y-%m-%d").to_string(),
        time: transaction.transaction_date.format("%H-%M-%S").to_string(),
        eft_merchant_no: transaction.identifier,
        receipt_no: padded_random_int(12, 13) //"0000A0{str(randint(0, 10 ** 12)).rjust(13, '0')}",
    }
    let out_transaction = format!("{:?}", transaction);
    Ok(out_transaction)
}

fn to_pounds(amount: i16) -> Result<String> {
    Ok(Decimal::new(amount.into(), 2).to_string())
}

fn padded_random_int(raise_power: u32, num_chars: i32) -> Result<String> {
    let upper_value = 10u32.pow(raise_power);
    //let number = rand::thread_rng().gen_range(1..1);
    Ok(upper_value.to_string())

}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn wasabi_transaction_valid() {
        let dt = Utc::now();
        let test_transaction = Transaction {
            amount: 245,
            transaction_date: dt,
            merchant_name: "test_merchant".to_string(),
            transaction_id: "test_transaction_id_1".to_string(),
            auth_code: "123456".to_string(),
            identifier: "12345678".to_string(),
            token: "98765432123456789".to_string(),
        };

        let wasabi_tx = wasabi_transaction(&test_transaction).unwrap();

        let expected_wasabi_tx = (
            "A076",
            "16277",
            "123456789",
            "3",
            "2.45",
            "first_six******last_four",
            "test_merchant",
            "123456",
            "1",
            dt.to_string(),
            dt.to_string(),
            "12345678",
            "test_transaction_id_1",
        );
        let expected_transaction = format!("{:?}", expected_wasabi_tx);
        assert_eq!(wasabi_tx, expected_transaction);
    }

    #[test]
    fn test_padded_random_int() {
        let value = padded_random_int(12, 13).unwrap();
        assert_eq!(value, "100000000000".to_string());
    }
}
