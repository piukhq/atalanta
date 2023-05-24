use crate::models::Transaction;
use color_eyre::Result;
use rust_decimal::prelude::*;

mod amex;
pub use amex::AmexAuthFormatter;
mod iceland;
pub use iceland::IcelandFormatter;
mod visa;
pub use visa::{VisaAuthFormatter, VisaSettlementFormatter};
mod wasabi;
pub use wasabi::WasabiFormatter;
mod costa;
pub use costa::CostaFormatter;

pub fn to_pounds(amount: i64) -> String {
    Decimal::new(amount, 2).to_string()
}

pub trait Formatter {
    fn format(transactions: Vec<Transaction>) -> Result<String>;
}
