//! Various subsets of data potenitally returned by SmartyStreets.

use csv::StringRecord;
use failure::format_err;
use serde_json::{self, Map, Value};
use std::borrow::Cow;

use crate::Result;

/// A subset of the fields returned by SmartyStreets.
#[derive(Debug)]
pub struct Structure {
    /// Number of columns added.
    column_count: usize,

    /// Fields that we want to include.
    ///
    /// WARNING: The correctness of the `traverse` function depends on `Map`
    /// being an order-preserving map type, which is set using `preserve_order`
    /// in `Cargo.toml`.
    fields: Map<String, Value>,
}

/// All the fields that we normally care about.
const COMPLETE: &str = include_str!("structures/complete.json");

impl Structure {
    /// A `Structure` including all the fields we normally care about.
    pub fn complete() -> Result<Structure> {
        Self::from_str(COMPLETE)
    }

    /// Parse a `Structure` from a string containing JSON.
    fn from_str(s: &str) -> Result<Structure> {
        // Parse our JSON and build our structure.
        let fields = serde_json::from_str(s)?;
        let mut structure = Structure {
            column_count: 0,
            fields,
        };

        // Update our column count.
        let mut count = 0;
        structure.traverse(|_path| {
            count += 1;
            Ok(())
        })?;
        structure.column_count = count;
        Ok(structure)
    }

    /// Add the column names specified in this `Structure` to a CSV header row.
    pub fn add_header_columns(
        &self,
        prefix: &str,
        header: &mut StringRecord,
    ) -> Result<()> {
        self.traverse(|path| {
            let last = path
                .last()
                .expect("should always have at least one path element");
            header.push_field(&format!("{}_{}", prefix, last));
            Ok(())
        })
    }

    /// Extract fields from `data` and merge them into `row`.
    ///
    /// PERFORMANCE: This is probably slower than it should be in a hot loop.
    pub fn add_value_columns_to_row(
        &self,
        data: &Value,
        row: &mut StringRecord,
    ) -> Result<()> {
        self.traverse(|path| {
            // Follow `path`.
            let mut focus = data;
            for key in path {
                if let Some(value) = focus.get(key) {
                    focus = value;
                } else {
                    // No value present, so push an empty field.
                    row.push_field("");
                    return Ok(());
                }
            }

            // Add the value to our row.
            let formatted = match focus {
                Value::Bool(b) => Cow::Borrowed(if *b { "T" } else { "F" }),
                Value::Null => Cow::Borrowed(""),
                Value::Number(n) => Cow::Owned(format!("{}", n)),
                Value::String(s) => Cow::Borrowed(&s[..]),
                Value::Array(_) | Value::Object(_) => {
                    return Err(format_err!(
                        "unexpected value at {:?}: {:?}",
                        path,
                        focus
                    ));
                }
            };
            row.push_field(&formatted);
            Ok(())
        })
    }

    /// Add empty columns to the row. We call this when we couldn't geocode an
    /// address.
    pub fn add_empty_columns_to_row(&self, row: &mut StringRecord) -> Result<()> {
        self.traverse(|_path| {
            row.push_field("");
            Ok(())
        })
    }

    /// Generic SmartyStreets result traverser. Calls `f` with the path to
    /// each key present in this `Structure`.
    fn traverse<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&[&str]) -> Result<()>,
    {
        let mut path = Vec::with_capacity(2);
        for (key, value) in &self.fields {
            path.push(&key[..]);
            match value {
                Value::Bool(true) => f(&path)?,
                Value::Bool(false) => {}
                Value::Object(map) => {
                    for (key, value) in map {
                        path.push(&key[..]);
                        match value {
                            Value::Bool(true) => f(&path)?,
                            Value::Bool(false) => {}
                            _ => {
                                return Err(format_err!(
                                    "invalid structure at {:?}: {:?}",
                                    path,
                                    value,
                                ));
                            }
                        }
                        path.pop();
                    }
                }
                _ => {
                    return Err(format_err!(
                        "invalid structure at {:?}: {:?}",
                        path,
                        value,
                    ));
                }
            }
            path.pop();
        }
        Ok(())
    }
}

#[test]
fn add_header_columns() {
    use std::iter::FromIterator;

    let structure = Structure::complete().unwrap();
    let mut header = StringRecord::from_iter(&["existing"]);
    structure.add_header_columns("x", &mut header).unwrap();
    let expected = StringRecord::from_iter(
        &[
            "existing",
            "x_addressee",
            "x_delivery_line_1",
            "x_delivery_line_2",
            "x_last_line",
            "x_delivery_point_barcode",
            "x_urbanization",
            "x_primary_number",
            "x_street_name",
            "x_street_predirection",
            "x_street_postdirection",
            "x_street_suffix",
            "x_secondary_number",
            "x_secondary_designator",
            "x_extra_secondary_number",
            "x_extra_secondary_designator",
            "x_pmb_designator",
            "x_pmb_number",
            "x_city_name",
            "x_default_city_name",
            "x_state_abbreviation",
            "x_zipcode",
            "x_plus4_code",
            "x_delivery_point",
            "x_delivery_point_check_digit",
            "x_record_type",
            "x_zip_type",
            "x_county_fips",
            "x_county_name",
            "x_carrier_route",
            "x_congressional_district",
            "x_building_default_indicator",
            "x_rdi",
            "x_elot_sequence",
            "x_elot_sort",
            "x_latitude",
            "x_longitude",
            "x_precision",
            "x_time_zone",
            "x_utc_offset",
            "x_dst",
            "x_dpv_match_code",
            "x_dpv_footnotes",
            "x_dpv_cmra",
            "x_dpv_vacant",
            "x_active",
            "x_ews_match",
            "x_footnotes",
            "x_lacslink_code",
            "x_lacslink_indicator",
            "x_suitelink_match",
        ][..],
    );
    assert_eq!(header, expected);
}

#[test]
fn add_value_columns() {
    use std::iter::FromIterator;

    let structure = Structure::complete().unwrap();

    let data: Value = serde_json::from_str(
        r#"{
    "addressee": "ACME, Inc.",
    "metadata": {
        "precision": "Zip5"
    }
}"#,
    )
    .unwrap();

    let mut row = StringRecord::from_iter(&["existing"]);
    structure.add_value_columns_to_row(&data, &mut row).unwrap();
    let expected = StringRecord::from_iter(
        &[
            "existing",
            "ACME, Inc.",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "Zip5",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
        ][..],
    );
    assert_eq!(row, expected);
}
