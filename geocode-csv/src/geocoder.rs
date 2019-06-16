//! Geocoding support.

use common_failures::prelude::*;
use csv::{self, StringRecord};
use failure::{format_err, ResultExt};
use futures::{compat::Future01CompatExt, future, FutureExt, TryFutureExt};
use hyper::Client;
use hyper_tls::HttpsConnector;
use log::{debug, error, trace};
use std::{cmp::max, io, sync::Arc, thread::sleep, time::Duration};
use tokio::{
    prelude::*,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::addresses::AddressColumnSpec;
use crate::async_util::run_sync_fn_in_background;
use crate::smartystreets::{
    AddressRequest, MatchStrategy, SharedHyperClient, SmartyStreets,
};
use crate::structure::Structure;
use crate::Result;

/// The number of chunks to buffer on our internal channels.
const CHANNEL_BUFFER: usize = 8;

/// The number of concurrent workers to run.
const CONCURRENCY: usize = 48;

/// The number of addresses to pass to SmartyStreets at one time.
const GEOCODE_SIZE: usize = 72;

/// Data about the CSV file that we include with every chunk to be geocoded.
struct Shared {
    /// Which columns contain addresses that we need to geocode?
    spec: AddressColumnSpec<usize>,
    /// Which SmartyStreets outputs do we want to store in our output?
    structure: Structure,
    /// The header of the output CSV file.
    out_headers: StringRecord,
}

/// A chunk to geocode.
struct Chunk {
    /// Shared information about the CSV file, including headers.
    shared: Arc<Shared>,
    /// The rows to geocode.
    rows: Vec<StringRecord>,
}

/// A message sent on our channel.
enum Message {
    /// A chunk to geocode.
    Chunk(Chunk),

    /// The end of our stream. Sent when all data has been processed
    /// successfuly.
    EndOfStream,
}

/// Read CSVs from standard input, geocode them, and write them to standard
/// output.
pub async fn geocode_stdio(
    spec: AddressColumnSpec<String>,
    match_strategy: MatchStrategy,
    structure: Structure,
) -> Result<()> {
    // Set up bounded channels for communication between the sync and async
    // worlds.
    let (in_tx, in_rx) = mpsc::channel::<Message>(CHANNEL_BUFFER);
    let (out_tx, out_rx) = mpsc::channel::<Message>(CHANNEL_BUFFER);

    // Hook up our inputs and outputs, which are synchronous functions running
    // in their own threads.
    let read_fut = run_sync_fn_in_background("read CSV".to_owned(), move || {
        read_csv_from_stdin(spec, structure, in_tx)
    });
    let write_fut = run_sync_fn_in_background("write CSV".to_owned(), move || {
        write_csv_to_stdout(out_rx)
    });

    // Create a shared `hyper::Client` with a connection pool, so that we can
    // use keep-alive.
    let client = Arc::new(
        Client::builder()
            .keep_alive(true)
            .build(HttpsConnector::new(4)?),
    );

    // Geocode each chunk that we see, with up to `CONCURRENCY` chunks being
    // geocoded at a time.
    let geocode_fut = in_rx
        .map_err(|e| e.into())
        // Turn input messages into futures that yield output messages.
        .map(move |message| {
            geocode_message(client.clone(), match_strategy, message)
                .boxed()
                .compat()
        })
        // Turn output message futures into output messages in parallel.
        .buffered(CONCURRENCY)
        .forward(out_tx)
        .map(|_| ())
        .compat()
        // Convert from "complex type implementing future" to an abstract
        // `Box<dyn Future<...>>` to avoid weird type errors in our callers.
        .boxed();

    // Wait for all three of our processes to finish.
    let (read_result, geocode_result, write_result) =
        future::join3(read_fut, geocode_fut, write_fut).await;

    // Wrap any errors with context.
    let read_result: Result<()> = read_result
        .context("error reading input")
        .map_err(|e| e.into());
    let geocode_result: Result<()> = geocode_result
        .context("error geocoding")
        .map_err(|e| e.into());
    let write_result: Result<()> = write_result
        .context("error writing output")
        .map_err(|e| e.into());

    // Print if one of the processes fails, it will usually cause the other two
    // to fail. We could try to figure out the "root" cause for the user, or we
    // could just print out all the errors and let the user sort them out. :-(
    let mut failed = false;
    if let Err(err) = &read_result {
        failed = true;
        eprintln!("{}", err.display_causes_and_backtrace());
    }
    if let Err(err) = &geocode_result {
        failed = true;
        eprintln!("{}", err.display_causes_and_backtrace());
    }
    if let Err(err) = &write_result {
        failed = true;
        eprintln!("{}", err.display_causes_and_backtrace());
    }

    if failed {
        Err(format_err!("geocoding stdio failed"))
    } else {
        Ok(())
    }
}

/// Read a CSV file and write it as messages to `tx`.
fn read_csv_from_stdin(
    spec: AddressColumnSpec<String>,
    structure: Structure,
    mut tx: Sender<Message>,
) -> Result<()> {
    // Open up our CSV file and get the headers.
    let stdin = io::stdin();
    let mut rdr = csv::Reader::from_reader(stdin.lock());
    let in_headers = rdr.headers()?.to_owned();
    debug!("input headers: {:?}", in_headers);

    // Convert our column spec from using header names to header indices.
    let spec = spec.convert_to_indices_using_headers(&in_headers)?;

    // Decide how big to make our chunks. We want to geocode no more
    // `GEOCODE`-size addresses at a time, and each input row may generate up to
    // `spec.prefix_count()` addresses.
    let chunk_size = max(1, GEOCODE_SIZE / spec.prefix_count());

    // Build our output headers.
    let mut out_headers = in_headers.clone();
    for prefix in spec.prefixes() {
        structure.add_header_columns(prefix, &mut out_headers)?;
    }
    debug!("output headers: {:?}", out_headers);

    // Build our shared CSV file metadata, and wrap it with a reference count.
    let shared = Arc::new(Shared {
        spec,
        structure,
        out_headers,
    });

    // Group up the rows into chunks and send them to `tx`.
    let mut sent_chunk = false;
    let mut rows = Vec::with_capacity(chunk_size);
    for row in rdr.records() {
        let row = row?;
        rows.push(row);
        if rows.len() >= chunk_size {
            trace!("sending {} input rows", rows.len());
            tx = tx
                .send(Message::Chunk(Chunk {
                    shared: shared.clone(),
                    rows,
                }))
                .wait()
                .context("could not send rows to geocoder (perhaps it failed)")?;
            sent_chunk = true;
            rows = Vec::with_capacity(chunk_size);
        }
    }

    // Send a final chunk if either (1) we never sent a chunk, or (2) we have
    // rows that haven't been sent yet.
    if !sent_chunk || !rows.is_empty() {
        trace!("sending final {} input rows", rows.len());
        tx = tx
            .send(Message::Chunk(Chunk {
                shared: shared.clone(),
                rows,
            }))
            .wait()
            .context("could not send rows to geocoder (perhaps it failed)")?;
    }

    // Confirm that we've seen the end of the stream.
    trace!("sending end-of-stream for input");
    tx.send(Message::EndOfStream)
        .wait()
        .context("could not send end-of-stream to geocoder (perhaps it failed)")?;

    debug!("done sending input");
    Ok(())
}

/// Receive chunks of a CSV file from `rx` and write them to standard output.
fn write_csv_to_stdout(mut rx: Receiver<Message>) -> Result<()> {
    let stdout = io::stdout();
    let mut wtr = csv::Writer::from_writer(stdout.lock());

    let mut headers_written = false;
    let mut end_of_stream_seen = false;
    while !end_of_stream_seen {
        let rx_result = rx.into_future().wait();
        match rx_result {
            // We received a value on our stream.
            Ok((Some(message), new_rx)) => {
                rx = new_rx;
                match message {
                    Message::Chunk(chunk) => {
                        trace!("received {} output rows", chunk.rows.len());
                        if !headers_written {
                            wtr.write_record(&chunk.shared.out_headers)?;
                            headers_written = true;
                        }
                        for row in chunk.rows {
                            wtr.write_record(&row)?;
                        }
                    }
                    Message::EndOfStream => {
                        trace!("received end-of-stream for output");
                        assert!(headers_written);
                        end_of_stream_seen = true;
                    }
                }
            }
            // The background thread exitted without sending anything. This
            // shouldn't happen.
            Ok((None, _rx)) => {
                error!("did not receive end-of-stream");
                return Err(format_err!(
                    "did not receive end-of-stream from geocoder (perhaps it failed)"
                ));
            }
            // We couldn't read a result from the background thread, probably
            // because it panicked.
            Err(_) => {
                error!("background thread panicked");
                return Err(format_err!("background thread panicked"));
            }
        }
    }
    Ok(())
}

/// Geocode a `Message`. This is just a wrapper around `geocode_chunk`.
async fn geocode_message(
    client: SharedHyperClient,
    match_strategy: MatchStrategy,
    message: Message,
) -> Result<Message> {
    match message {
        Message::Chunk(chunk) => {
            trace!("geocoding {} rows", chunk.rows.len());
            Ok(Message::Chunk(
                geocode_chunk(client, match_strategy, chunk).await?,
            ))
        }
        Message::EndOfStream => {
            trace!("geocoding received end-of-stream");
            Ok(Message::EndOfStream)
        }
    }
}

/// Geocode a `Chunk`.
async fn geocode_chunk(
    client: SharedHyperClient,
    match_strategy: MatchStrategy,
    mut chunk: Chunk,
) -> Result<Chunk> {
    // Build a list of addresses to geocode.
    let prefixes = chunk.shared.spec.prefixes();
    let mut addresses = vec![];
    for prefix in &prefixes {
        let column_keys = chunk
            .shared
            .spec
            .get(prefix)
            .expect("should always have prefix");
        for row in &chunk.rows {
            addresses.push(AddressRequest {
                address: column_keys.extract_address_from_record(row)?,
                match_strategy,
            });
        }
    }
    let addresses_len = addresses.len();

    // Create a SmartyStreets client.
    let smartystreets = SmartyStreets::new(client)?;

    // Geocode our addresses.
    //
    // TODO: Retry on failure.
    trace!("geocoding {} addresses", addresses_len);
    let mut failures: u8 = 0;
    let geocoded = loop {
        // TODO: The `clone` here is expensive. We might want to move the
        // `retry` loop inside of `street_addresses`.
        let result = smartystreets.street_addresses(addresses.clone()).await;
        match result {
            Err(ref err) if failures < 5 => {
                failures += 1;
                debug!("retrying smartystreets error: {}", err);
                sleep(Duration::from_secs(2));
            }
            Err(err) => {
                return Err(err)
                    .context("smartystreets error")
                    .map_err(|e| e.into());
            }
            Ok(geocoded) => {
                break geocoded;
            }
        }
    };
    trace!("geocoded {} addresses", addresses_len);

    // Add address information to our , output rows.
    for geocoded_for_prefix in geocoded.chunks(chunk.rows.len()) {
        assert_eq!(geocoded_for_prefix.len(), chunk.rows.len());
        for (response, row) in geocoded_for_prefix.iter().zip(&mut chunk.rows) {
            if let Some(response) = response {
                chunk
                    .shared
                    .structure
                    .add_value_columns_to_row(&response.fields, row)?;
            } else {
                chunk.shared.structure.add_empty_columns_to_row(row)?;
            }
        }
    }
    Ok(chunk)
}
