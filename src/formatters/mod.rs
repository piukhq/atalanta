use crate::models::Transaction;
use color_eyre::Result;
use rust_decimal::prelude::*;

pub mod amex_auth;
pub mod amex_settlement;
pub mod costa;
pub mod iceland;
pub mod stonegate;
pub mod tgi_fridays;
pub mod visa_auth;
pub mod visa_settlement;
pub mod wasabi;

#[must_use]
pub fn to_pounds(amount: i64) -> String {
    Decimal::new(amount, 2).to_string()
}

pub trait Formatter {
    /// Formats a list of transactions into a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the transactions cannot be formatted.
    fn format(transactions: Vec<Transaction>) -> Result<String>;
}
