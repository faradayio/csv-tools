//! Utilities for working with async tasks.

use failure::{format_err, ResultExt};
use futures::{
    compat::{Future01CompatExt},
};
use std::thread;
use tokio::{prelude::*, sync::mpsc};

use crate::Result;

/// Run a synchronous function `f` in a background worker thread and return its
/// value.
pub(crate) async fn run_sync_fn_in_background<F, T>(
    thread_name: String,
    f: F,
) -> Result<T>
where
    F: (FnOnce() -> Result<T>) + Send + 'static,
    T: Send + 'static,
{
    // Spawn a worker thread outside our thread pool to do the actual work.
    let (sender, receiver) = mpsc::channel(1);
    let thr = thread::Builder::new().name(thread_name);
    let handle = thr
        .spawn(move || {
            sender.send(f()).wait().expect(
                "should always be able to send results from background thread",
            );
        })
        .context("could not spawn thread")?;

    // Wait for our worker to report its results.
    let background_result = receiver.into_future().compat().await;
    let result = match background_result {
        // The background thread sent an `Ok`.
        Ok((Some(Ok(value)), _receiver)) => Ok(value),
        // The background thread sent an `Err`.
        Ok((Some(Err(err)), _receiver)) => Err(err),
        // The background thread exitted without sending anything. This
        // shouldn't happen.
        Ok((None, _receiver)) => {
            unreachable!("background thread did not send any results");
        }
        // We couldn't read a result from the background thread, probably
        // because it panicked.
        Err(_) => Err(format_err!("background thread panicked")),
    };

    // Block until our worker exits. This is a synchronous block in an
    // asynchronous task, but the background worker already reported its result,
    // so the wait should be short.
    handle.join().expect("background worker thread panicked");
    result
}
