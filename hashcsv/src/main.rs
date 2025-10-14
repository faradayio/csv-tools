use anyhow::{Context, Result};
use clap::Parser;
use csv::ByteRecord;
use log::debug;
use std::{
    fs::File,
    io::{stdin, stdout, Cursor, Read, Write},
    path::PathBuf,
    process,
};
use uuid::Uuid;

/// Use reasonably large input and output buffers. In other CSV tools, this
/// seems to give us a performance boost of around 5-10% compared to the
/// standard 8 KiB buffer used by `csv`.
const BUFFER_SIZE: usize = 256 * 1024;

/// Command-line options.
#[derive(Debug, Parser)]
#[command(
    about = "Add an `id` column to a CSV file based on a hash of the other columns",
    version
)]
struct Opt {
    /// Input file (uses stdin if omitted).
    input: Option<PathBuf>,

    /// The column name for the new, hash-based ID column.
    #[arg(long = "id-column-name", short = 'c', default_value = "id")]
    id_column_name: String,
}

/// Our main entry point. Calls `run` and prints out any errors.
fn main() {
    // Set up logging.
    env_logger::init();

    // Parse our command-line arguments.
    let opt: Opt = Opt::parse();
    debug!("Options: {:#?}", opt);

    if let Err(err) = run(&opt) {
        eprintln!("ERROR: {}", err);
        let mut source = err.source();
        while let Some(cause) = source {
            eprintln!("  caused by: {}", cause);
            source = cause.source();
        }
        process::exit(1);
    }
}

/// Do the actual work, returning an error if something goes wrong.
fn run(opt: &Opt) -> Result<()> {
    // Build our CSV reader.
    let input = get_input(opt)?;
    let mut rdr_builder = csv::ReaderBuilder::new();
    rdr_builder.has_headers(true);
    rdr_builder.buffer_capacity(BUFFER_SIZE);
    let mut rdr = rdr_builder.from_reader(input);

    // We lock `stdout`, giving us exclusive access. In the past, this has made
    // an enormous difference in performance.
    let stdout = stdout();
    let output = stdout.lock();

    // Build our CSV writer.
    let mut wtr = csv::WriterBuilder::new()
        .buffer_capacity(BUFFER_SIZE)
        .from_writer(output);

    // Handle our headers.
    let mut header = rdr
        .byte_headers()
        .context("cannot read headers")?
        .to_owned();
    header.push_field(opt.id_column_name.as_bytes());
    wtr.write_byte_record(&header)
        .context("cannot write headers")?;

    // Set up a "namespace", which is required to build UUID v5 hash-style
    // UUIDs.
    let namespace = "9270e21d-47c3-4395-bf32-c1797115f3fa"
        .parse::<Uuid>()
        .expect("could not parse UUID in source");

    // Add an ID value to each row of the file. We pre-allocate a `ByteRecord`
    // and each of our buffers to avoid needing to allocate memory on every pass
    // through the loop. This makes `hashcsv` roughly 250x faster than if we
    // allocate memory each time through the loop.
    let mut record = ByteRecord::new();
    let mut hash_buffer = vec![];
    let mut uuid_buffer = vec![];
    while rdr
        .read_byte_record(&mut record)
        .context("cannot read record")?
    {
        // Build a UUID v5 based on a hash of all the fields in our record.
        hash_buffer.clear();
        for field in &record {
            hash_buffer.extend_from_slice(field);
            hash_buffer.push(0x0);
        }
        let uuid = Uuid::new_v5(&namespace, &hash_buffer);

        // Write our UUID as a string and append it to `record`.
        uuid_buffer.clear();
        {
            let mut uuid_buffer_cursor = Cursor::new(&mut uuid_buffer);
            write!(&mut uuid_buffer_cursor, "{}", uuid)
                .context("could not write to buffer")?;
        }
        record.push_field(&uuid_buffer);

        // Write our modified record.
        wtr.write_byte_record(&record)
            .context("cannot write record")?;
    }

    // Finish writing.
    wtr.flush().context("error writing records")?;

    Ok(())
}

/// Get our input stream, either `stdin` or a file.
fn get_input(opt: &Opt) -> Result<Box<dyn Read>> {
    if let Some(path) = &opt.input {
        let file = File::open(path.as_path())
            .with_context(|| format!("could not open {}", path.display()))?;
        Ok(Box::new(file))
    } else {
        Ok(Box::new(stdin()))
    }
}
