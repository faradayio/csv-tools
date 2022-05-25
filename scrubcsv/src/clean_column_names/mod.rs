use std::str::FromStr;

use crate::{format_err, Error, Result};

use self::stable::StableCleaner;
use self::unique::Uniquifier;

mod stable;
mod unique;

#[derive(Debug, Clone, Copy)]
/// What algorithm should be use to clean column names?
pub enum ColumnNameCleanerType {
    /// Guarantee that all column names are unique, lowercase C identifiers.
    /// Conflicting column names will be suffixed to make them unique.
    Unique,
    /// Guarantee that all column names will always map to the _same_
    /// unique lowercase C identifier in an easily predictable fashion.
    /// This may fail if two conflicting column names are present.
    Stable,
}

impl ColumnNameCleanerType {
    /// Construct an appropriate `ColumnNameCleaner` instance.
    pub fn build_cleaner(self) -> Box<dyn ColumnNameCleaner> {
        match self {
            ColumnNameCleanerType::Unique => Box::new(Uniquifier::default()),
            ColumnNameCleanerType::Stable => Box::new(StableCleaner::default()),
        }
    }
}

impl FromStr for ColumnNameCleanerType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unique" => Ok(ColumnNameCleanerType::Unique),
            "stable" => Ok(ColumnNameCleanerType::Stable),
            _ => Err(format_err!(
                "invalid --clean-column-names argument: {:?}",
                s
            )),
        }
    }
}

/// Interface used to clean column names.
pub trait ColumnNameCleaner {
    /// Given a `name`, return an idenfitier to use as a column name.
    fn unique_id_for(&mut self, name: &str) -> Result<String>;
}
