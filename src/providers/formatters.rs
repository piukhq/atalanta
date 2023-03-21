#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::models::Transaction;
use crate::providers::*;
use color_eyre::Result;

pub trait Formatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>>;
}

pub struct VisaAuthFormatter {}

impl Formatter for VisaAuthFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        Ok(vec![visa_auth(&transactions[0])?])
    }
}

pub struct VisaSettlementFormatter {}

impl Formatter for VisaSettlementFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        Ok(vec![visa_settlement(&transactions[0])?])
    }
}

pub struct AmexAuthFormatter {}

impl Formatter for AmexAuthFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        Ok(vec![amex_auth(&transactions[0])?])
    }
}
pub struct WasabiFormatter {}

impl Formatter for WasabiFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        Ok(vec![wasabi_transaction(transactions)?])
    }
}

pub struct IcelandFormatter {}

impl Formatter for IcelandFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        Ok(vec![iceland_transaction(transactions)?])
    }
}
