//! Geocoding support.

use csv::{self, StringRecord};
use failure::format_err;
use futures::{compat::Stream01CompatExt, FutureExt, StreamExt, TryFutureExt};
use std::{cmp::max, io, sync::Arc};
use tokio::{
    prelude::*,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::addresses::AddressColumnSpec;
use crate::async_util::run_sync_fn_in_background;
use crate::smartystreets::MatchStrategy;
use crate::structure::Structure;
use crate::{Error, Result};

/// The number of chunks to buffer on our internal channels.
const CHANNEL_BUFFER: usize = 8;

/// The number of concurrent workers to run.
const CONCURRENCY: usize = 48;

/// The number of addresses to pass to SmartyStreets at one time.
const GEOCODE_SIZE: usize = 72;

/// Data about the CSV file that we include with every chunk to be geocoded.
struct Shared {
    /// The header of the input CSV file.
    in_headers: StringRecord,
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

    // Geocode each chunk that we see, with up to `CONCURRENCY` chunks being
    // geocoded at a time.
    let geocode_fut = in_rx
        .map_err(|e| e.into())
        // Turn input messages into futures that yield output messages.
        .map(move |message| geocode_message(match_strategy, message).boxed().compat())
        // Turn output message futures into output messages in parallel.
        .buffered(CONCURRENCY)
        .forward(out_tx);

    unimplemented!()
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

    // Build our shared CSV file metadata, and wrap it with a reference count.
    let shared = Arc::new(Shared {
        in_headers,
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
            tx = tx
                .send(Message::Chunk(Chunk {
                    shared: shared.clone(),
                    rows,
                }))
                .wait()?;
            sent_chunk = true;
            rows = Vec::with_capacity(chunk_size);
        }
    }

    // Send a final chunk if either (1) we never sent a chunk, or (2) we have
    // rows that haven't been sent yet.
    if !sent_chunk || !rows.is_empty() {
        tx = tx
            .send(Message::Chunk(Chunk {
                shared: shared.clone(),
                rows,
            }))
            .wait()?;
    }

    // Confirm that we've seen the end of the stream.
    tx.send(Message::EndOfStream).wait()?;
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
                        if !headers_written {
                            wtr.write_record(&chunk.shared.out_headers)?;
                            headers_written = true;
                        }
                        for row in chunk.rows {
                            wtr.write_record(&row)?;
                        }
                    }
                    Message::EndOfStream => {
                        assert!(headers_written);
                        end_of_stream_seen = true;
                    }
                }
            }
            // The background thread exitted without sending anything. This
            // shouldn't happen.
            Ok((None, _rx)) => {
                return Err(format_err!("did not receive end-of-stream"));
            }
            // We couldn't read a result from the background thread, probably
            // because it panicked.
            Err(_) => {
                return Err(format_err!("background thread panicked"));
            }
        }
    }
    Ok(())
}

/// Geocode a `Message`. This is just a wrapper around `geocode_chunk`.
async fn geocode_message(
    match_strategy: MatchStrategy,
    message: Message,
) -> Result<Message> {
    match message {
        Message::Chunk(chunk) => {
            Ok(Message::Chunk(geocode_chunk(match_strategy, chunk).await?))
        }
        Message::EndOfStream => Ok(Message::EndOfStream),
    }
}

/// Geocode a `Chunk`.
async fn geocode_chunk(match_strategy: MatchStrategy, chunk: Chunk) -> Result<Chunk> {
    let prefixes = chunk.shared.spec.prefixes();
    //let mut addresses = vec![];
    for row in &chunk.rows {
        for prefix in &prefixes {
            //chunk.shared.spec.
            //addresses.push()
            unimplemented!()
        }
    }
    unimplemented!()
}
