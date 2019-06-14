//! Interface to SmartyStreets REST API.

use failure::format_err;
use serde::{Deserialize, Serialize};
use serde_json;
use std::str::FromStr;

use crate::addresses::Address;
use crate::{Error, Result};

/// What match candidates should we output when geocoding?
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchStrategy {
    /// Only match valid USPS addresses.
    Strict,
    /// Match addresses that are within the known range on a street,
    /// but which are not valid USPS addresses.
    Range,
    /// Return a candidate for every address.
    Invalid,
}

impl FromStr for MatchStrategy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Do this manually instead of including another library to generate it,
        // or quoting values and parsing them with `serde_json`.
        match s {
            "strict" => Ok(MatchStrategy::Strict),
            "range" => Ok(MatchStrategy::Range),
            "invalid" => Ok(MatchStrategy::Invalid),
            _ => Err(format_err!("unknown match strategy {:?}", s)),
        }
    }
}

/// A SmartyStreets address request.
#[derive(Debug, Serialize)]
pub struct AddressRequest<'a> {
    /// The address to geocode.
    #[serde(flatten)]
    pub address: Address<'a>,

    /// What match strategy should we use?
    #[serde(rename = "match")]
    pub match_strategy: MatchStrategy,
}

/// A SmartyStreets address response.
#[derive(Debug, Deserialize)]
pub struct AddressResponse {
    /// Fields returned by SmartyStreets. We could actually represent this as
    /// serveral large structs with known fields, and it would probably be
    /// faster, but this way requires less code for now.
    #[serde(flatten)]
    fields: serde_json::Value,
}

/// An interface to SmartyStreets.
pub trait SmartyStreetsApi {
    /// Geocode street addresses using SmartyStreets.
    fn street_addresses(&self, requests: &[AddressRequest]) -> Result<Vec<Option<AddressResponse>>>;
}

/// The real implementation of `SmartyStreetsApi`.
pub struct SmartyStreets {

}
