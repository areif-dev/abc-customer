use std::fmt::{Debug, Display};

use csv::StringRecord;

use crate::AbcCustomerBuilderError;

/// General purpose type for handling errors that arise in relation to processing [`crate::AbcCustomer`]s
#[derive(Debug)]
pub enum Error {
    /// Occurs when building a [`crate::AbcCustomer`] fails. Forwards the underlying error as well
    /// as the record that caused the error
    BuilderError {
        inner: AbcCustomerBuilderError,
        record: StringRecord,
    },
    /// Could not read valid [`crate::PaymentTerms`] from a given string. Includes context around
    /// what happened
    ParsePaymentTermsError(String),
    /// Reading the customer.data file failed for some reason. Forwards the underlying error as
    /// well as a string with additional context
    CsvError(csv::Error, &'static str),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match self {
            Self::CsvError(e, c) => format!("{e}. Context: {c}"),
            Self::BuilderError { inner, record } => {
                format!("{:?} failed to parse due to {inner}", record)
            }
            Self::ParsePaymentTermsError(c) => format!("can't parse payment terms due to {c}"),
        };
        write!(f, "problem with AbcCustomer lib. Context: {inner}")
    }
}

impl From<(csv::Error, &'static str)> for Error {
    fn from((inner, ctx): (csv::Error, &'static str)) -> Self {
        Error::CsvError(inner, ctx)
    }
}

impl From<(AbcCustomerBuilderError, StringRecord)> for Error {
    fn from((inner, record): (AbcCustomerBuilderError, StringRecord)) -> Self {
        Error::BuilderError { inner, record }
    }
}
