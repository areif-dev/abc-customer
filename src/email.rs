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
    pub fn parse_from_str(raw: &str) -> Option<EmailSettings> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        let Ok(settings) = serde_json::from_str::<EmailSettings>(raw) else {
            return Some(
                EmailSettingsBuilder::default()
                    .email(raw)
                    .frequency(Frequency::Never)
                    .build()
                    .ok()?,
            );
        };
        Some(settings)
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
            .map(|s| EmailSettings::parse_from_str(&s).unwrap())
            .collect::<Vec<_>>();
        let check_settings = json
            .as_array()
            .unwrap()
            .iter()
            .map(|v| serde_json::from_value::<EmailSettings>(v.clone()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(settings, check_settings);

        assert_eq!(EmailSettings::parse_from_str(""), None);
        assert_eq!(EmailSettings::parse_from_str(" "), None);
        assert_eq!(
            EmailSettings::parse_from_str("a"),
            Some(EmailSettings {
                email: "a".to_string(),
                frequency: Frequency::Never,
                send_zero_balance_statements: false
            })
        );
    }
}
