// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// Enable clippy if we were asked to do so.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate csv;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use docopt::Docopt;
use serde::{Deserialize, Deserializer};
use serde::de::Error as DeError;
use std::io;
use std::process;
use std::result;

mod errors;
mod zip2010;

use errors::*;

/// Specify what data set we should use for generating chunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChunkType {
    /// Use 2010 census population data.
    Zip2010,
}

// Implement the `Deserialize` interface so that `docopt` can automatically
// parse this argument type from a string value.
impl<'de> Deserialize<'de> for ChunkType {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match &s[..] {
            "zip2010" => Ok(ChunkType::Zip2010),
            _ => {
                let msg = format!("Unknown chunk type \"{}\", try --help", s);
                Err(D::Error::custom(msg))
            }
        }
    }
}

const USAGE: &'static str = "
geochunk - Partition data sets by estimated population.

Usage:
  geochunk export <type> <population>
  geochunk csv <type> <population> <input-column>
  geochunk (--help | --version)

Options:
  --help        Show this screen.
  --version     Show version.

Commands:
  export        Export the geochunk mapping for use by another program.
  csv           Add a geochunk column to a CSV file (used in a pipeline).

Types:
  zip2010       Use 2010 Census zip code population data.
";

/// Our command-line arguments, which can be automatically deserialized by
/// `docopt`.
#[derive(Debug, Deserialize)]
struct Args {
    cmd_export: bool,
    cmd_csv: bool,
    arg_type: Option<ChunkType>,
    arg_population: Option<u64>,
    arg_input_column: Option<String>,
    flag_version: bool,
}

// Make a `main` function that calls `run` and prints out any errors.
quick_main!(run);

/// Our actual `main` function, called by the `quick_main!` macro above.
fn run() -> Result<()> {
    env_logger::init().expect("Could not initialize logging");
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    trace!("{:?}", args);

    // We have to handle `--version` ourselves.
    if args.flag_version {
        println!("geochunk {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    // Generate our table of chunks.
    let population = args.arg_population
        .expect("Population should have been required by docopt");
    let classifier = zip2010::Classifier::new(population);

    // Dispatch to an appropriate command handler.
    if args.cmd_export {
        let stdout = io::stdout();
        classifier.export(&mut stdout.lock())?;
    } else if args.cmd_csv {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let column = args.arg_input_column
            .expect("Column should have been required by docopt");
        classifier
            .transform_csv(&column, &mut stdin.lock(), &mut stdout.lock())?;
    } else {
        unreachable!("unknown subcommand, should have been caught by docopt");
    }

    Ok(())
}
