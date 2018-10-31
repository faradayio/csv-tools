extern crate csv;
extern crate env_logger;
extern crate failure;
extern crate humansize;
extern crate humantime;
#[macro_use]
extern crate log;
extern crate structopt;

use failure::Error;
use humansize::{FileSize, file_size_opts};
use humantime::format_duration;
use std::{
    cmp::min,
    io::{BufReader, prelude::*, stdin, stdout},
    time::{Duration, SystemTime},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Convert fixed-width fields on stdin to CSV data on stdout.
struct Opt {
    /// Print summary statistics.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// One of more field widths, as separate command-line arguments.
    field_widths: Vec<usize>,
}

/// Our main entry point.
fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    debug!("Options: {:?}", opt);

    // Keep track of how much time this takes.
    let start = SystemTime::now();

    let mut input = stdin();
    let mut output = stdout();
    let total = extract_fields(&mut input, &opt.field_widths, &mut output)?;

    // Print our some summary statistics.
    if opt.verbose {
        let elapsed = start.elapsed()?;
        let simple_elapsed = if elapsed.as_secs() > 0 {
            Duration::from_secs(elapsed.as_secs())
        } else {
            elapsed
        };
        eprintln!(
            "Processed {} in {}, {}/s",
            total.file_size(file_size_opts::BINARY)
                .expect("size can never be negative"),
            format_duration(simple_elapsed),
            (total as u64 / elapsed.as_secs()).file_size(file_size_opts::BINARY)
                .expect("size can never be negative"),
        );
    }

    Ok(())
}

/// Extract fixed-width fields from the input, and write to the output as a CSV.
fn extract_fields(
    r: &mut Read,
    field_widths: &[usize],
    w: &mut Write,
) -> Result<usize, Error> {
    // Wrap up our I/O streams to get the APIs we'll need.
    let mut buffered = BufReader::new(r);
    let mut out = csv::Writer::from_writer(w);

    // Allocate working buffers.
    let mut line = vec![];
    let mut record = csv::ByteRecord::new();

    // Keep track of how much data we've read.
    let mut total = 0;

    // Loop over input lines.
    while buffered.read_until(b'\n', &mut line)? > 0 {
        // Update our running total.
        total += line.len();

        // Calculate `end`, the length of our line with any trailing '\r' or
        // '\n' character stripped.
        let mut end = line.len() - 1;
        if end > 0 && line[end-1] == b'\r' {
            end -= 1;
        }

        // Add each of our columns to the record and write it.
        let mut offset = 0;
        for width in field_widths {
            // Get our column.
            let mut field = &line[min(offset, end)..min(offset+width, end)];

            // Strip spaces and add to our output record.
            while field.len() > 0 && field[field.len()-1] == b' ' {
                field = &field[..field.len()-1];
            }
            record.push_field(field);

            // Increment our offset.
            offset += width;
        }
        out.write_byte_record(&record)?;

        // Clear our buffers.
        record.clear();
        line.clear();
    }
    Ok(total)
}

#[test]
fn extracts_fields() {
    use std::io::Cursor;

    let input = "\
first     last      middle
John      Smith     Q
Sally     Jones
";
    let cols = &[10,10,6];
    let expected = "\
first,last,middle
John,Smith,Q
Sally,Jones,
";

    let mut r = Cursor::new(input);
    let mut w = vec![];

    extract_fields(&mut r, cols, &mut w).unwrap();
    assert_eq!(String::from_utf8_lossy(&w), expected);
}
