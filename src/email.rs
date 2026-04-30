use std::str::FromStr;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// How often customers should receive billing emails
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Frequency {
    /// Only send daily invoices. No monthly statement
    Daily(InvoicesToSend),
    /// Only send monthly statements. No daily invoices
    Monthly,
    /// Send invoices every day as they come in as well as a monthly statement
    MonthlyAndDaily(InvoicesToSend),
    /// Do not automatically send any invoices to this customer
    Never,
}

/// Which invoices to email to the customer
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum InvoicesToSend {
    /// Send all invoices regardless of payment status
    All,
    /// Send only invoices with a balance due
    Due,
    /// Send only invoices that are paid in full
    Paid,
}

/// Defines customer email preferences; such as email address, frequency of billing emails, and
/// whether they are to receive an emailed statement regardless of their ending balance
#[derive(Debug, Builder, Clone, PartialEq, Deserialize, Serialize)]
#[builder(setter(strip_option, into), pattern = "owned")]
pub struct EmailSettings {
    /// Customer's email address
    email: String,
    /// How frequently to send invoices
    frequency: Frequency,
    /// Whether to send statements even if the balance due is zero
    #[builder(default = false)]
    send_zero_balance_statements: bool,
}

impl EmailSettings {
    /// Change the email address
    pub fn with_email(&mut self, email: &str) {
        self.email = email.to_string();
    }

    /// Change how frequently invoice emails are sent
    pub fn with_frequency(&mut self, freq: Frequency) {
        self.frequency = freq;
    }

    /// Change whether to send the customer a statement even if their balance is zero
    pub fn with_send_zero_balance_statements(&mut self, send_zero_balance_statements: bool) {
        self.send_zero_balance_statements = send_zero_balance_statements;
    }

    /// Get the email address
    pub fn email(&self) -> String {
        self.email.to_string()
    }

    /// Get the frequency at which to send email invoices
    pub fn frequency(&self) -> Frequency {
        self.frequency.clone()
    }

    /// - `true` if the customer should be emailed a monthly statement even if their account balance
    /// is zero
    /// - `false` if the customer should not be emailed a monthly statement when their account
    /// balance is zero
    pub fn send_zero_balance_statements(&self) -> bool {
        self.send_zero_balance_statements
    }
}

impl FromStr for EmailSettings {
    type Err = EmailSettingsBuilderError;
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(EmailSettingsBuilderError::UninitializedField("email"));
        }
        let Ok(settings) = serde_json::from_str::<EmailSettings>(raw) else {
            return Ok(EmailSettingsBuilder::default()
                .email(raw)
                .frequency(Frequency::Never)
                .build()?);
        };
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_parse_from_str() {
        let json = serde_json::json!([
            {
                "email": "something@nothing.com",
                "frequency": {"Daily": "All"},
                "send_zero_balance_statements": true
            },
            {
                "email": "something@nothing2.com",
                "frequency": {"Daily": "Due"},
                "send_zero_balance_statements": true
            },
            {
                "email": "something@nothing3.com",
                "frequency": {"Daily": "Paid"},
                "send_zero_balance_statements": false
            },
            {
                "email": "something@nothing4.com",
                "frequency": {"MonthlyAndDaily": "Paid"},
                "send_zero_balance_statements": false
            },
            {
                "email": "something@nothing5.com",
                "frequency": {"MonthlyAndDaily": "All"},
                "send_zero_balance_statements": false
            },
            {
                "email": "something@nothing6.com",
                "frequency": {"MonthlyAndDaily": "Due"},
                "send_zero_balance_statements": false
            },
            {
                "email": "something@nothing7.com",
                "frequency": "Monthly",
                "send_zero_balance_statements": true
            },
            {
                "email": "something@nothing8.com",
                "frequency": "Never",
                "send_zero_balance_statements": true
            },
        ]);
        let settings = json
            .as_array()
            .unwrap()
            .iter()
            .map(Value::to_string)
            .map(|s| EmailSettings::from_str(&s).unwrap())
            .collect::<Vec<_>>();
        let check_settings = json
            .as_array()
            .unwrap()
            .iter()
            .map(|v| serde_json::from_value::<EmailSettings>(v.clone()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(settings, check_settings);

        assert!(EmailSettings::from_str("").is_err(),);
        assert!(EmailSettings::from_str(" ").is_err(),);
        assert_eq!(
            EmailSettings::from_str("a").unwrap(),
            EmailSettings {
                email: "a".to_string(),
                frequency: Frequency::Never,
                send_zero_balance_statements: false
            }
        );
    }
}
