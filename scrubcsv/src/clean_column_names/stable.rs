use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{format_err, Result};

use super::ColumnNameCleaner;

lazy_static! {
    /// Regular expression that all output columns must match.
    ///
    /// This is basically equivalent to "lowercase C identifier", and it
    /// corresponds to a valid BigQuery column name. (BigQuery allows mixed-case
    /// names in SQL source, but it ignores case.)
    static ref SAFE_NAME_REGEX: Regex =
        Regex::new("^[_a-z][_a-z0-9]*$").expect("invalid regex in source");
}

/// A column name cleaner that produces stable column names.
#[derive(Default)]
pub struct StableCleaner {
    /// Identifiers that we have already generated, mapped to the column names
    /// we generated them from.
    used: HashMap<String, String>,
}

impl ColumnNameCleaner for StableCleaner {
    fn unique_id_for(&mut self, name: &str) -> Result<String> {
        let normalized = name.to_ascii_lowercase().replace(' ', "_");

        // Check to make sure it's a "safe" name.
        if !SAFE_NAME_REGEX.is_match(&normalized) {
            return Err(format_err!("invalid column name: {:?}", name));
        }

        // Check to make sure we haven't already used this name.
        if let Some(conflicting_name) =
            self.used.insert(normalized.to_owned(), name.to_owned())
        {
            return Err(format_err!(
                "conflicting column names {:?} and {:?} would both map to {:?}",
                conflicting_name,
                name,
                normalized
            ));
        };
        Ok(normalized)
    }
}
