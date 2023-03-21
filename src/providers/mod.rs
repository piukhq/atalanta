use rust_decimal::prelude::*;

mod amex;
mod formatters;
mod iceland;
mod process;
mod visa;
mod wasabi;

pub use amex::{amex_auth, amex_settlement};
pub use formatters::*;
pub use iceland::iceland_transaction;
pub use process::*;
pub use visa::{visa_auth, visa_settlement};
pub use wasabi::wasabi_transaction;

pub fn to_pounds(amount: i16) -> String {
    Decimal::new(amount.into(), 2).to_string()
}
