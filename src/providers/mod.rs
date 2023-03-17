use rust_decimal::prelude::*;
use color_eyre::Result;

mod amex;
mod iceland;
mod formatters;
mod process;
mod visa;
mod wasabi;

pub use amex::{amex_auth, amex_settlement};
pub use visa::{visa_auth, visa_settlement};
pub use wasabi::wasabi_transaction;
pub use iceland::iceland_transaction;
pub use process::*;
pub use formatters::*;

pub fn to_pounds(amount: i16) -> Result<String> {
    Ok(Decimal::new(amount.into(), 2).to_string())
}