//! Utilities for working with async tasks.

use failure::ResultExt;
use futures::executor::block_on;
use std::thread;
use tokio::sync::mpsc;

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
    let (mut sender, mut receiver) = mpsc::channel(1);
    let thr = thread::Builder::new().name(thread_name);
    let handle = thr
        .spawn(move || {
            if block_on(sender.send(f())).is_err() {
                panic!("should always be able to send results from background thread");
            }
        })
        .context("could not spawn thread")?;

    // Wait for our worker to report its results.
    let background_result = receiver.recv().await;
    let result = match background_result {
        // The background thread sent an `Ok`.
        Some(Ok(value)) => Ok(value),
        // The background thread sent an `Err`.
        Some(Err(err)) => Err(err),
        // The background thread exitted without sending anything. This
        // shouldn't happen.
        None => {
            unreachable!("background thread did not send any results");
        }
    };

    // Block until our worker exits. This is a synchronous block in an
    // asynchronous task, but the background worker already reported its result,
    // so the wait should be short.
    handle.join().expect("background worker thread panicked");
    result
}
