// Async HTTP boilerplate based on
// https://github.com/daboross/futures-example-2019/

#![feature(async_await)]

use common_failures::quick_main;
use env_logger;
use failure::{format_err, Error};
use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    future, FutureExt, TryFutureExt, TryStreamExt,
};
use hyper::rt::Stream;
use reqwest::r#async::Client;
use serde_json::json;
use std::{env, path::PathBuf, result, str};
use structopt::StructOpt;
use url::Url;

mod addresses;
mod async_util;
mod geocoder;
mod smartystreets;
mod structure;
mod unpack_vec;

use addresses::AddressColumnSpec;
use geocoder::geocode_stdio;
use smartystreets::MatchStrategy;
use structure::Structure;

type Result<T> = result::Result<T, Error>;

/// Our command-line arguments.
#[derive(Debug, StructOpt)]
#[structopt(help = "geocode CSV files passed on standard input")]
struct Opt {
    /// `strict` for valid postal addresses only, `range` for unknown addresses
    /// within a street's known range, and `invalid` to always generate some
    /// match.
    #[structopt(long = "match", default_value = "strict")]
    match_strategy: MatchStrategy,

    /// A JSON file describing what columns to geocode.
    #[structopt(long = "spec")]
    spec_path: PathBuf,
}

// Generate a boilerplate `main` function.
quick_main!(run);

/// Our main entry point.
fn run() -> Result<()> {
    // Set up basic logging.
    env_logger::init();

    // Parse our command-line arguments.
    let opt = Opt::from_args();
    let spec = AddressColumnSpec::from_path(&opt.spec_path)?;
    let structure = Structure::complete()?;

    // Call our geocoder asynchronously.
    let geocode_fut = geocode_stdio(spec, opt.match_strategy, structure);

    // Pass our future to our async runtime.
    let mut runtime =
        tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(geocode_fut.boxed().compat())?;
    Ok(())
}

/*
/// Example call to SmartyStreets.
async fn geocode_example() -> Result<()> {
    // Build our URL.
    let mut url = Url::parse("https://api.smartystreets.com/street-address")?;
    url.query_pairs_mut()
        .append_pair("auth-id", &env::var("SMARTYSTREETS_AUTH_ID")?)
        .append_pair("auth-token", &env::var("SMARTYSTREETS_AUTH_TOKEN")?)
        .finish();

    // Make the geocoding request.
    let client = Client::new();
    let response = client
        .post(url.as_str())
        .json(&json!([{
            "street": "275 Apple Tree Road",
            "city": "East Thetford",
            "state": "VT",
            "zip": "05043",
        }]))
        .send()
        .compat()
        .await?;
    let status = response.status();
    let body = response.into_body().concat2().compat().await?;
    let s = str::from_utf8(&body)?;

    // Check the request status.
    if status.is_success() {
        println!("response: {}", s);
        Ok(())
    } else {
        Err(format_err!("geocoding error: {}\n{}", status, s))
    }
}
*/
