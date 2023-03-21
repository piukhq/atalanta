use chrono::{DateTime, FixedOffset};
use color_eyre::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Transaction {
    amount: String,
    transaction_date: DateTime<FixedOffset>,
    merchant_name: String,
    transaction_id: String,
    auth_code: String,
}

fn create_transactions() -> Result<Vec<Transaction>> {
    let toffee_transaction = vec![
        Transaction {
            amount: "2.60".to_string(),
            transaction_date: DateTime::parse_from_str(
                "2014-11-28 21:00:09 +00:00",
                "%Y-%m-%d %H:%M:%S %:z",
            )?,
            merchant_name: "Bink toffee".to_string(),
            transaction_id: "1234567890987654321234567".to_string(),
            auth_code: "098765".to_string(),
        },
        Transaction {
            amount: "42.67".to_string(),
            transaction_date: DateTime::parse_from_str(
                "2014-11-20 21:00:09 +00:00",
                "%Y-%m-%d %H:%M:%S %:z",
            )?,
            merchant_name: "Bink toffee".to_string(),
            transaction_id: "12345678909887654".to_string(),
            auth_code: "023454".to_string(),
        },
    ];

    Ok(toffee_transaction)
}

/// Inserts data into writer and writes to a file
///
/// # Error
///
/// If an error occurs, the error is returned to `main`
fn write_to_file(path: &str) -> Result<()> {
    // Creates new `Writer` for `stdout`
    let mut writer = csv::Writer::from_path(path)?;
    let toffee_transaction = create_transactions()?;
    // We don't explicitly write the header record
    for item in toffee_transaction.iter() {
        writer.serialize(item)?;
    }

    writer.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    write_to_file("my_data.csv")?;

    Ok(())
}
