use std::fmt::{Debug, Display};

use csv::StringRecord;

use crate::AbcCustomerBuilderError;

pub type Context = String;

pub struct Error {
    kind: ErrorKind,
    context: Vec<Context>,
}

#[derive(Debug)]
pub enum ErrorKind {
    BuilderError {
        inner: AbcCustomerBuilderError,
        record: &'static str,
    },
    ParsePaymentTermsError(Context),
    CsvError(csv::Error),
}

pub trait AddContext<T> {
    /// If `self` is `Ok`, returns the value unchanged.
    /// If `self` is `Err`, appends `ctx` to the error’s internal `Context`
    /// and returns the mutated error.
    fn add_context(self, ctx: &str) -> Result<T, Error>;
}

impl Error {
    fn add_context(self, ctx: &str) -> Error {
        let mut existing = self.context.clone();
        existing.push(ctx.to_string());
        Self {
            context: existing,
            ..self
        }
    }
}

impl std::error::Error for Error {}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut context = self.context.clone();
        context.reverse();
        let context = if context.is_empty() {
            String::from("no context")
        } else {
            context.join(" -> ")
        };
        write!(f, "{context}")
    }
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error {
            context: vec![format!("{:?}", value)],
            kind: ErrorKind::CsvError(value),
        }
    }
}

impl From<AbcCustomerBuilderError> for Error {
    fn from(value: AbcCustomerBuilderError) -> Self {
        Error {
            context: vec![format!("{:?}", value)],
            kind: ErrorKind::BuilderError(value),
        }
    }
}

impl From<(AbcCustomerBuilderError, &'static str)> for Error {
    fn from(value: (AbcCustomerBuilderError, &'static str)) -> Self {
        Error
    }
}

impl<T> AddContext<T> for Result<T, Error> {
    fn add_context(self, ctx: &str) -> Result<T, Error> {
        match self {
            Ok(d) => Ok(d),
            Err(e) => Err(e.add_context(ctx)),
        }
    }
}
