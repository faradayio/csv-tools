//! Interface to SmartyStreets REST API.

use failure::{format_err, ResultExt};
use futures::stream::StreamExt;
use hyper::{client::Client, client::HttpConnector, Body, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::{
    env,
    str::{self, FromStr},
    sync::Arc,
};
use url::Url;

use crate::addresses::Address;
use crate::unpack_vec::unpack_vec;
use crate::{Error, Result};

/// A `hyper` client shared between multiple workers.
pub type SharedHyperClient = Arc<Client<HttpsConnector<HttpConnector>>>;

/// Credentials for authenticating with SmartyStreets.
#[derive(Debug, Clone)]
pub struct Credentials {
    auth_id: String,
    auth_token: String,
}

impl Credentials {
    /// Create new SmartyStreets credentials from environment variables.
    fn from_env() -> Result<Credentials> {
        let auth_id = env::var("SMARTYSTREETS_AUTH_ID")
            .context("could not read SMARTYSTREETS_AUTH_ID")?;
        let auth_token = env::var("SMARTYSTREETS_AUTH_TOKEN")
            .context("could not read SMARTYSTREETS_AUTH_TOKEN")?;
        Ok(Credentials {
            auth_id,
            auth_token,
        })
    }
}

/// What match candidates should we output when geocoding?
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
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

impl Default for MatchStrategy {
    fn default() -> Self {
        MatchStrategy::Strict
    }
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
#[derive(Clone, Debug, Serialize)]
pub struct AddressRequest {
    /// The address to geocode.
    #[serde(flatten)]
    pub address: Address,

    /// What match strategy should we use?
    #[serde(rename = "match")]
    pub match_strategy: MatchStrategy,
}

/// A SmartyStreets address response.
#[derive(Clone, Debug, Deserialize)]
pub struct AddressResponse {
    /// The index of the corresponding `AddressRequest`.
    pub input_index: usize,

    /// Fields returned by SmartyStreets. We could actually represent this as
    /// serveral large structs with known fields, and it would probably be
    /// faster, but this way requires less code for now.
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

/// The real implementation of `SmartyStreetsApi`.
pub struct SmartyStreets {
    credentials: Credentials,
    client: SharedHyperClient,
}

impl SmartyStreets {
    /// Create a new SmartyStreets client.
    pub fn new(client: SharedHyperClient) -> Result<SmartyStreets> {
        Ok(SmartyStreets {
            credentials: Credentials::from_env()?,
            client,
        })
    }

    /// Geocode addresses using SmartyStreets.
    pub async fn street_addresses(
        &self,
        requests: Vec<AddressRequest>,
    ) -> Result<Vec<Option<AddressResponse>>> {
        street_addresses_impl(self.credentials.clone(), self.client.clone(), requests)
            .await
    }
}

async fn street_addresses_impl(
    credentials: Credentials,
    client: SharedHyperClient,
    requests: Vec<AddressRequest>,
) -> Result<Vec<Option<AddressResponse>>> {
    // Build our URL.
    let mut url = Url::parse("https://api.smartystreets.com/street-address")?;
    url.query_pairs_mut()
        .append_pair("auth-id", &credentials.auth_id)
        .append_pair("auth-token", &credentials.auth_token)
        .finish();

    // Make the geocoding request.
    let req = Request::builder()
        .method("POST")
        .uri(url.as_str())
        .header("Content-Type", "application/json; charset=utf-8")
        .body(Body::from(serde_json::to_string(&requests)?))?;
    let res = client.request(req).await?;
    let status = res.status();
    let mut body = res.into_body();
    let mut body_data = vec![];
    while let Some(chunk_result) = body.next().await {
        let chunk = chunk_result?;
        body_data.extend(&chunk[..]);
    }

    // Check the request status.
    if status.is_success() {
        let resps: Vec<AddressResponse> = serde_json::from_slice(&body_data)?;
        Ok(unpack_vec(resps, requests.len(), |resp| resp.input_index)?)
    } else {
        Err(format_err!(
            "geocoding error: {}\n{}",
            status,
            String::from_utf8_lossy(&body_data),
        ))
    }
}
