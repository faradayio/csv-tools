// Async HTTP boilerplate based on
// https://github.com/daboross/futures-example-2019/

#![feature(async_await)]

use failure::{format_err, Error};
use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
    future, FutureExt, TryFutureExt, TryStreamExt,
};
use hyper::rt::Stream;
use reqwest::r#async::Client;
use serde_json::json;
use std::{env, result, str};
use url::Url;

mod addresses;
mod unpack_vec;

type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    let mut runtime =
        tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(geocode_example().boxed().compat())?;
    Ok(())
}

/// Example call to Smartystreetss.
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
