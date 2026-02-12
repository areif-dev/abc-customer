use derive_builder::Builder;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Frequency {
    /// Only send daily invoices. No monthly statement
    Daily(InvoicesToSend),
    /// Only send monthly statements. No daily invoices
    Monthly,
    /// Send invoices every day as they come in as well as a monthly statement
    MonthlyAndDaily(InvoicesToSend),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum InvoicesToSend {
    /// Send all invoices regardless of payment status
    All,
    /// Send only invoices with a balance due
    Due,
    /// Send only invoices that are paid in full
    Paid,
}

#[derive(Debug, Builder, Clone, PartialEq)]
#[builder(setter(strip_option, into), pattern = "owned")]
pub struct EmailSettings {
    /// Customer's email address
    email: String,
    /// How frequently to send invoices
    frequency: Frequency,
    /// Whether to send statements even if the balance due is zero
    send_zero_balance_statements: bool,
}
