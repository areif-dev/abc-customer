pub mod email;
pub mod error;

use csv::StringRecord;
use derive_builder::Builder;
pub use error::Error;
use std::{collections::HashMap, path::Path, str::FromStr};

use crate::email::EmailSettings;

#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(setter(into))]
pub struct AbcCustomer {
    code: String,
    name: String,
    address: Option<String>,
    zip: Option<String>,
    email: Option<EmailSettings>,
    #[builder(default = Vec::new())]
    phone: Vec<String>,
    terms: PaymentTerms,
    tax_code: String,
    tin: Option<String>,
    jdf_id: Option<String>,
}

pub type AbcCustomersByCode = HashMap<String, AbcCustomer>;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PaymentTerms {
    /// Full payment is due at the time of product or service delivery
    CashOnDelivery,
    /// Full payment is due within the specified number of days from the date of the invoice
    Net(u32),
}

impl AbcCustomer {
    fn first_and_last_names(full_name: &str) -> (String, String) {
        let name_in_order = if full_name.contains(",") {
            let mut comma_split: Vec<String> = full_name
                .split(",")
                .filter_map(|n| {
                    if n.trim().is_empty() {
                        None
                    } else {
                        Some(n.trim().to_string())
                    }
                })
                .collect();
            comma_split.reverse();
            comma_split.join(" ")
        } else {
            full_name.to_string()
        };
        let mut space_split = name_in_order.split(" ");
        let first_name = space_split.next().unwrap_or("").to_string();
        let last_name = space_split.collect::<Vec<_>>().join(" ");
        match (first_name.is_empty(), last_name.is_empty()) {
            (true, true) => (String::new(), String::from("Receiving")),
            _ => (first_name, last_name),
        }
    }

    /// Get the ID of the [`AbcCustomer`]. Something like "DOEJO 0"
    pub fn code(&self) -> String {
        self.code.to_string()
    }

    /// The full name as it appears in ABC. A person will be something like "DOE, JOHN". A company
    /// should be "JOHN DOE, INC"
    pub fn full_name(&self) -> String {
        self.name.to_string()
    }

    /// Parses the customer's name and returns only the "first" name. For a person like "DOE,
    /// JOHN", this will return "JOHN". For a company like "JOHN DOE, INC.", this will return
    /// "INC."
    pub fn first_name(&self) -> String {
        let (first, _) = Self::first_and_last_names(&self.name);
        first
    }

    /// Parses the customer's name and returns only the "last" name. For a person like "DOE, JOHN",
    /// this will return "DOE". For a company like "JOHN DOE, INC.", this will return "JOHN DOE"
    pub fn last_name(&self) -> String {
        let (_, last) = Self::first_and_last_names(&self.name);
        last
    }

    /// The customer's billing address including building number and street name
    pub fn address(&self) -> Option<String> {
        self.address.to_owned()
    }

    /// The customer's billing zip code
    pub fn zip(&self) -> Option<String> {
        self.zip.to_owned()
    }

    /// Object describing the customer's invoice emailing preferences (if present)
    pub fn email(&self) -> Option<EmailSettings> {
        self.email.to_owned()
    }

    /// List of phone numbers associated with the customer
    pub fn phone(&self) -> Vec<String> {
        self.phone.to_owned()
    }

    /// Default payment terms on the customer's account
    pub fn terms(&self) -> PaymentTerms {
        self.terms
    }

    /// Customer's default tax code
    pub fn tax_code(&self) -> String {
        self.tax_code.to_string()
    }

    /// Customer's tax id number (if present)
    pub fn tin(&self) -> Option<String> {
        self.tin.to_owned()
    }

    /// Customer's John Deere Financial id number (if present)
    pub fn jdf_id(&self) -> Option<String> {
        self.jdf_id.to_owned()
    }

    /// Set the ID of the [`AbcCustomer`].
    pub fn set_code(&mut self, code: impl Into<String>) {
        self.code = code.into();
    }

    /// Set the full name of the [`AbcCustomer`].
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Set the customer's billing address.
    pub fn set_address(&mut self, address: impl Into<Option<String>>) {
        self.address = address.into();
    }

    /// Set the customer's billing zip code.
    pub fn set_zip(&mut self, zip: impl Into<Option<String>>) {
        self.zip = zip.into();
    }

    /// Set the customer's invoice emailing preferences.
    pub fn set_email(&mut self, email: impl Into<Option<EmailSettings>>) {
        self.email = email.into();
    }

    /// Set the list of phone numbers associated with the customer.
    pub fn set_phone(&mut self, phone: impl Into<Vec<String>>) {
        self.phone = phone.into();
    }

    /// Set the default payment terms on the customer's account.
    pub fn set_terms(&mut self, terms: PaymentTerms) {
        self.terms = terms;
    }

    /// Set the customer's default tax code.
    pub fn set_tax_code(&mut self, tax_code: impl Into<String>) {
        self.tax_code = tax_code.into();
    }

    /// Set the customer's tax id number.
    pub fn set_tin(&mut self, tin: impl Into<Option<String>>) {
        self.tin = tin.into();
    }

    /// Set the customer's John Deere Financial id number.
    pub fn set_jdf_id(&mut self, jdf_id: impl Into<Option<String>>) {
        self.jdf_id = jdf_id.into();
    }

    /// Create a map of skus to [`AbcCustomer`]s by parsing ABC database export files.
    ///
    /// In order to run a database export, run report 7-10, select "C" (Customer) as the file to export. All
    /// other parameters can be skipped or left as default. Run the report to the Screen. After a
    /// few seconds, two files should be created at
    /// C:\ABC Software\Database Export\Company001\Data\customer.data and
    /// C:\ABC Software\Database Export\Company001\Data\customer_posted.data.
    ///
    /// # Arguments
    /// * `customer_path` - The path to the item.data file generated by the db export. This will
    /// probably be C:\ABC Software\Database Export\Company001\Data\customer.data.
    ///
    /// # Returns
    /// A [`HashMap`] of ABC customer IDs to the [`AbcCustomer`] they belong to
    ///
    /// # Errors
    /// The data files are long, and ABC does not always produce them correctly. Therefore, if any
    /// required fields are missing or if certain values cannot be parsed,
    /// then an [`ParseCustomerError`] will be returned
    pub fn from_db_export(customer_path: impl AsRef<Path>) -> Result<AbcCustomersByCode, Error> {
        fn parse_record(row: StringRecord) -> Result<(String, AbcCustomer), Error> {
            let code = row.get(0).ok_or((
                AbcCustomerBuilderError::UninitializedField("code"),
                row.clone(),
            ))?;
            let name = row.get(1).ok_or((
                AbcCustomerBuilderError::UninitializedField("name"),
                row.clone(),
            ))?;
            let address = row.get(3).map_or(None, |a| {
                if a.is_empty() {
                    None
                } else {
                    Some(a.to_string())
                }
            });
            let zip = row.get(4).map_or(None, |z| {
                if z.is_empty() {
                    None
                } else {
                    Some(z.to_string())
                }
            });
            let email = row.get(6).map_or(None, |e| EmailSettings::from_str(e).ok());
            let tax = row
                .get(11)
                .map_or("PA", |e| if e.is_empty() { "PA" } else { e });

            // Phone numbers may exist at any of indexes 7, 27, 34
            let phone: Vec<String> = [7usize, 27, 34]
                .iter()
                .filter_map(|i| {
                    row.get(*i).map_or(None, |s| {
                        if s.is_empty() {
                            None
                        } else {
                            Some(s.to_string())
                        }
                    })
                })
                .collect();
            let terms: PaymentTerms = row
                .get(15)
                .unwrap_or("")
                .parse()
                .unwrap_or(PaymentTerms::CashOnDelivery);
            let tin = row.get(31).map_or(None, |e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e.to_string())
                }
            });
            let jdf_id = row.get(36).map_or(None, |e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e.to_string())
                }
            });

            Ok((
                code.to_owned(),
                AbcCustomerBuilder::default()
                    .code(code)
                    .name(name)
                    .address(address)
                    .zip(zip)
                    .email(email)
                    .phone(phone)
                    .terms(terms)
                    .tax_code(tax)
                    .jdf_id(jdf_id)
                    .tin(tin)
                    .build()
                    .map_err(|e| (e, row))?,
            ))
        }

        let mut customer_data = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .from_path(customer_path)
            .map_err(|e| (e, "failed to open customer.data file"))?;

        let mut customers = HashMap::new();
        for row in customer_data.records() {
            let row = row.map_err(|e| {
                (
                    e,
                    "customer.data opened successfully, but could not read a particular record",
                )
            })?;
            let (code, customer) = parse_record(row)?;
            customers.insert(code.to_string(), customer);
        }
        Ok(customers)
    }
}

impl FromStr for PaymentTerms {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let lower = value.to_lowercase();
        if lower.contains("cash") {
            return Ok(PaymentTerms::CashOnDelivery);
        }
        if lower.contains("net") {
            let days = lower
                .chars()
                .filter(|c| c.is_digit(10))
                .collect::<String>()
                .parse::<u32>()
                .map_err(|e| {
                    Error::ParsePaymentTermsError(
                        format!(
                            "parsing payment terms failed. Net due days could not be read from {value}. Caused by {e}",
                        ),
                    )
                })?;
            return Ok(PaymentTerms::Net(days));
        }
        Err(Error::ParsePaymentTermsError(format!(
            "could not parse valid payment terms from {value}",
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::email::{EmailSettingsBuilder, Frequency};

    use super::*;

    #[test]
    fn test_parser() {
        let customer_path = "./customer.data";
        let customers = AbcCustomer::from_db_export(customer_path).unwrap();
        let known_customers = AbcCustomersByCode::from([
            (
                ".CASH".to_string(),
                AbcCustomerBuilder::default()
                    .code(".CASH")
                    .name("Cash Sale")
                    .address(None)
                    .zip(None)
                    .tax_code("PA")
                    .tin(None)
                    .email(
                        EmailSettingsBuilder::default()
                            .email("something@nothing.com")
                            .frequency(Frequency::Never)
                            .build()
                            .unwrap(),
                    )
                    .jdf_id(None)
                    .terms("CASH".parse::<PaymentTerms>().unwrap())
                    .build()
                    .unwrap(),
            ),
            (
                "SOMECODE".to_string(),
                AbcCustomerBuilder::default()
                    .code("SOMECODE")
                    .name("SOME COMPANY")
                    .address(Some("1119 CENTRE AVE".to_string()))
                    .email(None)
                    .zip(Some("19601".to_string()))
                    .phone(["(123)456-7890".to_string()])
                    .tax_code("PA/")
                    .terms("NET 30".parse::<PaymentTerms>().unwrap())
                    .tin(Some("123456789".to_string()))
                    .jdf_id(None)
                    .build()
                    .unwrap(),
            ),
            (
                "OTHERCODE".to_string(),
                AbcCustomerBuilder::default()
                    .code("OTHERCODE")
                    .name("OTHER COMPANY")
                    .address(Some("7180 BERNVILLE RD".to_string()))
                    .zip(Some("19506".to_string()))
                    .phone(["(987)654-3210".to_string(), "(123)456-7890".to_string()])
                    .email(
                        EmailSettingsBuilder::default()
                            .email("ree@ree.ree")
                            .frequency(Frequency::Never)
                            .build()
                            .unwrap(),
                    )
                    .tax_code("PA")
                    .terms("NET30".parse::<PaymentTerms>().unwrap())
                    .tin(None)
                    .jdf_id(None)
                    .build()
                    .unwrap(),
            ),
            (
                "SURGI 0".to_string(),
                AbcCustomerBuilder::default()
                    .code("SURGI 0")
                    .name("SURNAME, GIVENNAME")
                    .address(Some("300 PENN VALLEY RD".to_string()))
                    .zip(Some("19506".to_string()))
                    .email(None)
                    .phone(["(321)195-5731".to_string()])
                    .tax_code("PA")
                    .terms("CASH".parse::<PaymentTerms>().unwrap())
                    .tin("123456789".to_string())
                    .jdf_id(None)
                    .build()
                    .unwrap(),
            ),
            (
                "REEEEEEEE".to_string(),
                AbcCustomerBuilder::default()
                    .code("REEEEEEEE")
                    .name("REEEE COMPANY")
                    .address(Some("7180 BERNVILLE RD".to_string()))
                    .zip(Some("19506".to_string()))
                    .email(
                        EmailSettingsBuilder::default()
                            .email("something@nothing.com")
                            .frequency(Frequency::Daily(email::InvoicesToSend::All))
                            .send_zero_balance_statements(true)
                            .build()
                            .unwrap(),
                    )
                    .phone(["(987)654-3210".to_string(), "(123)456-7890".to_string()])
                    .tax_code("PA")
                    .terms("NET30".parse::<PaymentTerms>().unwrap())
                    .tin(None)
                    .jdf_id(None)
                    .build()
                    .unwrap(),
            ),
        ]);
        for (code, customer) in customers {
            assert_eq!(&customer, known_customers.get(&code).unwrap());
        }
    }

    #[test]
    fn test_first_and_last_names() {
        let names = [
            "SURNAME, GIVENNAME",
            "SURNAME,GIVENNAME",
            "JUSTONENAME",
            "",
            "Company Name",
            "Givenname Surname",
            "This Has Many Names",
            "Surname, Givenname Middlename",
            "Surname, Givenname, Bull",
            "strayComma,",
        ];
        let mapped: Vec<(String, String)> = names
            .into_iter()
            .map(AbcCustomer::first_and_last_names)
            .collect();
        let expected = [
            ("GIVENNAME", "SURNAME"),
            ("GIVENNAME", "SURNAME"),
            ("JUSTONENAME", ""),
            ("", "Receiving"),
            ("Company", "Name"),
            ("Givenname", "Surname"),
            ("This", "Has Many Names"),
            ("Givenname", "Middlename Surname"),
            ("Bull", "Givenname Surname"),
            ("strayComma", ""),
        ];
        assert_eq!(
            mapped,
            expected
                .into_iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect::<Vec<_>>()
        );
    }
}
