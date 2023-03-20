#![warn(clippy::unwrap_used, clippy::expect_used)]
use crate::models::Transaction;
use crate::providers::*;
use color_eyre::Result;
/// A trait for formatting transaction data for each retailer.

pub trait Formatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>>;
}

pub struct VisaAuthFormatter {}

impl Formatter for VisaAuthFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        let mut formatted_transactions: Vec<String> = Vec::new();
        formatted_transactions.push(visa_auth(&transactions[0])?);

        Ok(formatted_transactions)
    }
}

pub struct VisaSettlementFormatter {}

impl Formatter for VisaSettlementFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        let mut formatted_transactions: Vec<String> = Vec::new();

        for tx in transactions {
            let transaction = visa_settlement(&tx)?;
            formatted_transactions.push(transaction)
        }

        Ok(formatted_transactions)
    }
}

pub struct AmexAuthFormatter {}

impl Formatter for AmexAuthFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        let mut formatted_transactions: Vec<String> = Vec::new();
        formatted_transactions.push(amex_auth(&transactions[0])?);

        Ok(formatted_transactions)
    }
}
pub struct WasabiFormatter {}

impl Formatter for WasabiFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        let mut formatted_transactions: Vec<String> = Vec::new();

        formatted_transactions.push(wasabi_transaction(transactions)?);

        Ok(formatted_transactions)
    }
}

pub struct IcelandFormatter {}

impl Formatter for IcelandFormatter {
    fn format(&self, transactions: Vec<Transaction>) -> Result<Vec<String>> {
        let mut formatted_transactions: Vec<String> = Vec::new();

        formatted_transactions.push(iceland_transaction(transactions)?);

        Ok(formatted_transactions)
    }
}
