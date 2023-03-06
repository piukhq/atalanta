use rust_decimal::prelude::*;
use color_eyre::Result;

mod visa;
mod wasabi;
mod process;
mod amex;

pub use visa::{visa_auth, visa_settlement};
pub use wasabi::wasabi_transaction;
pub use process::*;

pub fn to_pounds(amount: i16) -> Result<String> {
    Ok(Decimal::new(amount.into(), 2).to_string())
}