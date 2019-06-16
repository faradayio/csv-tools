//! Types related to addresses.

use csv::StringRecord;
use failure::{format_err, ResultExt};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fs::File, path::Path};

use crate::Result;

/// An address record that we can pass to SmartyStreets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Address {
    /// Either the street, or the entire address as a string. This must always
    /// be present.
    pub street: String,
    /// The city, if any.
    pub city: Option<String>,
    /// The state, if any.
    pub state: Option<String>,
    /// The zipcode, if any.
    pub zipcode: Option<String>,
}

/// Either a column name, or a list of names.
///
/// `K` is typically either a `String` (for a column name) or a `usize` (for a
/// column index).
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(untagged, deny_unknown_fields)]
pub enum ColumnKeyOrKeys<K: Eq> {
    /// The name of a single column.
    Key(K),
    /// The names of multiple columns, which should be joined using a space.
    Keys(Vec<K>),
}

impl ColumnKeyOrKeys<usize> {
    /// Given a CSV row, extract an `Address` value to send to SmartyStreets.
    pub fn extract_from_record<'a>(
        &self,
        record: &'a StringRecord,
    ) -> Result<Cow<'a, str>> {
        match self {
            ColumnKeyOrKeys::Key(key) => Ok(Cow::Borrowed(&record[*key])),
            ColumnKeyOrKeys::Keys(keys) => Ok(Cow::Owned(
                keys.iter()
                    .map(|key| &record[*key])
                    .collect::<Vec<_>>()
                    .join(" "),
            )),
        }
    }
}

/// The column names from a CSV file that we want to use as addresses.
///
/// `K` is typically either a `String` (for a column name) or a `usize` (for a
/// column index).
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AddressColumnKeys<K: Default + Eq> {
    /// The name of street column or columns. May also be specified as
    /// "house_number_and_street" or "address".
    #[serde(alias = "house_number_and_street", alias = "address")]
    pub street: ColumnKeyOrKeys<K>,
    /// The city column, if any.
    #[serde(default)]
    pub city: Option<K>,
    /// The state column, if any.
    #[serde(default)]
    pub state: Option<K>,
    /// The zipcode column, if any. May also be specified as
    /// "postcode".
    #[serde(default, alias = "postcode")]
    pub zipcode: Option<K>,
}

impl AddressColumnKeys<usize> {
    /// Given a CSV row, extract an `Address` value to send to SmartyStreets.
    pub fn extract_address_from_record<'a>(
        &self,
        record: &'a StringRecord,
    ) -> Result<Address> {
        Ok(Address {
            street: self.street.extract_from_record(record)?.into_owned(),
            city: self.city.map(|c| record[c].to_owned()),
            state: self.state.map(|s| record[s].to_owned()),
            zipcode: self.zipcode.map(|z| record[z].to_owned()),
        })
    }
}

#[test]
fn extract_simple_address_from_record() {
    use std::iter::FromIterator;
    let record = StringRecord::from_iter(&[
        "1600 Pennsylvania Avenue NW, Washington DC, 20500",
    ]);
    let keys = AddressColumnKeys {
        street: ColumnKeyOrKeys::Key(0),
        city: None,
        state: None,
        zipcode: None,
    };
    assert_eq!(
        keys.extract_address_from_record(&record).unwrap(),
        Address {
            street: "1600 Pennsylvania Avenue NW, Washington DC, 20500".to_owned(),
            city: None,
            state: None,
            zipcode: None,
        },
    );
}

#[test]
fn extract_complex_address_from_record() {
    use std::iter::FromIterator;
    let record = StringRecord::from_iter(&[
        "1600",
        "Pennsylvania Avenue NW",
        "Washington",
        "DC",
        "20500",
    ]);
    let keys = AddressColumnKeys {
        street: ColumnKeyOrKeys::Keys(vec![0, 1]),
        city: Some(2),
        state: Some(3),
        zipcode: Some(4),
    };
    assert_eq!(
        keys.extract_address_from_record(&record).unwrap(),
        Address {
            street: "1600 Pennsylvania Avenue NW".to_owned(),
            city: Some("Washington".to_owned()),
            state: Some("DC".to_owned()),
            zipcode: Some("20500".to_owned()),
        },
    );
}

/// A map from column prefixes (e.g. "home", "work") to address column keys.
///
/// `K` is typically either a `String` (for a column name) or a `usize` (for a
/// column index).
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct AddressColumnSpec<Key: Default + Eq> {
    /// A map from output column prefixes to address column keys.
    #[serde(flatten)]
    address_columns_by_prefix: HashMap<String, AddressColumnKeys<Key>>,
}

impl<Key: Default + Eq> AddressColumnSpec<Key> {
    /// The number of prefixes we want to include in our output.
    pub fn prefix_count(&self) -> usize {
        self.address_columns_by_prefix.len()
    }

    /// The address prefixes we want to include in our output.
    ///
    /// This **MUST** return the prefixes in the same order every time or our
    /// output will be corrupted.
    pub fn prefixes(&self) -> Vec<&str> {
        let mut prefixes = self
            .address_columns_by_prefix
            .keys()
            .map(|k| &k[..])
            .collect::<Vec<_>>();
        // Do not remove this `sort`!
        prefixes.sort();
        prefixes
    }

    /// Look up an `AddressColumnKeys` by prefix.
    pub fn get(&self, prefix: &str) -> Option<&AddressColumnKeys<Key>> {
        self.address_columns_by_prefix.get(prefix)
    }
}

impl AddressColumnSpec<String> {
    /// Load an `AddressColumnSpec` from a file.
    pub fn from_path(path: &Path) -> Result<Self> {
        let f = File::open(path)
            .with_context(|_| format_err!("cannot open {}", path.display()))?;
        Ok(serde_json::from_reader(f)
            .with_context(|_| format_err!("error parsing {}", path.display()))?)
    }

    /// Given an `AddressColumnSpec` using strings, and the header row of a CSV
    /// file, convert it into a `AddressColumnSpec<usize>` containing the column
    /// indices.
    pub fn convert_to_indices_using_headers(
        &self,
        headers: &StringRecord,
    ) -> Result<AddressColumnSpec<usize>> {
        let mut header_columns = HashMap::new();
        for (idx, header) in headers.iter().enumerate() {
            if let Some(_existing) = header_columns.insert(header, idx) {
                return Err(format_err!("duplicate header column `{}`", header));
            }
        }
        self.convert_to_indices(&header_columns)
    }
}

#[test]
fn convert_address_column_spec_to_indices() {
    use std::iter::FromIterator;
    let headers = StringRecord::from_iter(&[
        "home_number",
        "home_street",
        "home_city",
        "home_state",
        "home_zip",
        "work_address",
    ]);
    let address_column_spec_json = r#"{
   "home": {
       "house_number_and_street": ["home_number", "home_street"],
       "city": "home_city",
       "state": "home_state",
       "postcode": "home_zip"
   },
   "work": {
       "address": "work_address"
   }
}"#;
    let address_column_spec: AddressColumnSpec<String> =
        serde_json::from_str(address_column_spec_json).unwrap();

    let mut expected = HashMap::new();
    expected.insert(
        "home".to_owned(),
        AddressColumnKeys {
            street: ColumnKeyOrKeys::Keys(vec![0, 1]),
            city: Some(2),
            state: Some(3),
            zipcode: Some(4),
        },
    );
    expected.insert(
        "work".to_owned(),
        AddressColumnKeys {
            street: ColumnKeyOrKeys::Key(5),
            city: None,
            state: None,
            zipcode: None,
        },
    );
    assert_eq!(
        address_column_spec
            .convert_to_indices_using_headers(&headers)
            .unwrap(),
        AddressColumnSpec::<usize> {
            address_columns_by_prefix: expected,
        },
    );
}

/// A value which can be converted from using string indices to numeric indices.
trait ConvertToIndices {
    type Output;

    /// Convert this value from using string indices to numeric indices.
    fn convert_to_indices(
        &self,
        header_columns: &HashMap<&str, usize>,
    ) -> Result<Self::Output>;
}

impl ConvertToIndices for String {
    type Output = usize;

    fn convert_to_indices(
        &self,
        header_columns: &HashMap<&str, usize>,
    ) -> Result<Self::Output> {
        header_columns
            .get(&self[..])
            .map(|idx| *idx)
            .ok_or_else(|| format_err!("could not find column `{}` in header", self))
    }
}

impl ConvertToIndices for ColumnKeyOrKeys<String> {
    type Output = ColumnKeyOrKeys<usize>;

    fn convert_to_indices(
        &self,
        header_columns: &HashMap<&str, usize>,
    ) -> Result<Self::Output> {
        match self {
            ColumnKeyOrKeys::Key(key) => Ok(ColumnKeyOrKeys::Key(
                key.convert_to_indices(header_columns)?,
            )),
            ColumnKeyOrKeys::Keys(keys) => Ok(ColumnKeyOrKeys::Keys(
                keys.iter()
                    .map(|k| k.convert_to_indices(header_columns))
                    .collect::<Result<Vec<_>>>()?,
            )),
        }
    }
}

impl ConvertToIndices for AddressColumnKeys<String> {
    type Output = AddressColumnKeys<usize>;

    fn convert_to_indices(
        &self,
        header_columns: &HashMap<&str, usize>,
    ) -> Result<Self::Output> {
        Ok(AddressColumnKeys {
            street: self.street.convert_to_indices(header_columns)?,
            city: self
                .city
                .as_ref()
                .map(|c| c.convert_to_indices(header_columns))
                .transpose()?,
            state: self
                .state
                .as_ref()
                .map(|s| s.convert_to_indices(header_columns))
                .transpose()?,
            zipcode: self
                .zipcode
                .as_ref()
                .map(|z| z.convert_to_indices(header_columns))
                .transpose()?,
        })
    }
}

impl ConvertToIndices for AddressColumnSpec<String> {
    type Output = AddressColumnSpec<usize>;

    fn convert_to_indices(
        &self,
        header_columns: &HashMap<&str, usize>,
    ) -> Result<Self::Output> {
        let mut address_columns_by_prefix = HashMap::new();
        for (prefix, address_columns) in &self.address_columns_by_prefix {
            address_columns_by_prefix.insert(
                prefix.to_owned(),
                address_columns.convert_to_indices(header_columns)?,
            );
        }
        Ok(AddressColumnSpec {
            address_columns_by_prefix,
        })
    }
}
