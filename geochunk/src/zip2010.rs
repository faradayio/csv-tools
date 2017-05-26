//! Support for chunks based on 2010 census population data.

use csv;
#[cfg(test)]
use env_logger;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::default::Default;
use std::io::prelude::*;
use std::str::from_utf8;

use errors::*;

/// The length of a basic zip code, in digits.
const ZIP_CODE_LENGTH: usize = 5;

/// Classifies Zip codes into geochunks based on 2010 census population data.
pub struct Classifier {
    /// The approximate number of people we want to put in each chunk.
    target_population: u64,
    /// Map from zip code prefixes to chunk IDs.
    chunk_id_for_prefix: HashMap<String, String>,
}

impl Classifier {
    /// Create a new classifier, specifying how many people we'd ideally
    /// want to see in each chunk.
    pub fn new(target_population: u64) -> Classifier {
        let prefix_population = PrefixPopulation::new();
        let mut chunk_id_for_prefix = HashMap::<String, String>::new();
        prefix_population.build_chunks_recursive(target_population,
                                                 "",
                                                 &mut chunk_id_for_prefix);
        Classifier {
            target_population: target_population,
            chunk_id_for_prefix: chunk_id_for_prefix,
        }
    }

    /// Return the column name to use for the geochunk column.  This encodes
    /// the parameters we used to configure the geochunks, to help prevent
    /// messing them up in the real world.
    pub fn geochunk_column_name(&self) -> String {
        format!("geochunk_zip2010_{}", self.target_population)
    }

    /// Given a zip code, return the geochunk identifier.  Returns an error
    /// if the `zip` code is invalid.
    pub fn chunk_for(&self, zip: &str) -> Result<&str> {
        for i_rev in 0..(ZIP_CODE_LENGTH + 1) {
            let i = ZIP_CODE_LENGTH - i_rev;
            if let Some(chunk_id) = self.chunk_id_for_prefix.get(&zip[..i]) {
                return Ok(chunk_id);
            }
        }
        Err(format!("Cannot find chunk for zip code \"{}\"", zip).into())
    }

    /// Export this mapping as a CSV file.
    pub fn export(&self, out: &mut Write) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(out);
        wtr.serialize(["zip", &self.geochunk_column_name()])?;
        for zip_int in 0..100000 {
            let zip = format!("{:05}", zip_int);
            let chunk_id = self.chunk_for(&zip)?;
            wtr.serialize([&zip[..], &chunk_id])?;
        }
        Ok(())
    }

    /// Read a CSV file, add a geochunk column, and write it back out again.
    pub fn transform_csv(&self,
                         input_column: &str,
                         input: &mut Read,
                         output: &mut Write)
                         -> Result<()> {
        let mut rdr = csv::Reader::from_reader(input);
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(output);

        // Extract our headers.
        let mut headers = rdr.headers()?.to_owned();

        // Look up the header index for our zip code column.
        let zip_col_idx = headers
            .iter()
            .position(|h| h == input_column)
            .ok_or_else(|| Error::no_such_column(input_column))?;

        // Add our output column and write our headers.
        headers.push_field(&self.geochunk_column_name());
        wtr.write_record(headers.iter())?;

        // According to BurntSushi at
        // https://github.com/BurntSushi/rust-csv/issues/76 ,
        // this should be the fastest way to write this loop.  This matters
        // because we may have millions of rows and hundreds of columns.
        let mut row = csv::ByteRecord::new();
        while rdr.read_byte_record(&mut row)? {
            let zip = from_utf8(&row[zip_col_idx])
                .chain_err(|| Error::non_utf8_zip(row.position()))?
                .to_owned();
            row.push_field(self.chunk_for(&zip)?.as_bytes());
            wtr.write_byte_record(&row)?;
        }
        Ok(())
    }
}

#[test]
fn classifies_sample_zip_codes_as_expected() {
    let _ = env_logger::init();
    let classifier = Classifier::new(250000);
    assert_eq!(classifier.chunk_for("01000").unwrap(), "010_0");
    assert_eq!(classifier.chunk_for("07720").unwrap(), "077_1");
}

type PrefixPopulationMaps = [HashMap<String, u64>; ZIP_CODE_LENGTH + 1];

/// Directly include our zip code population data in our application binary
/// for ease of distribution and packaging.
const ZIP_POPULATION_CSV: &'static str = include_str!("zip2010.csv");

/// The population associated with a zip code prefix.
struct PrefixPopulation {
    maps: PrefixPopulationMaps,
}

impl PrefixPopulation {
    fn new() -> PrefixPopulation {
        let mut maps = PrefixPopulationMaps::default();

        let mut rdr = csv::Reader::from_reader(ZIP_POPULATION_CSV.as_bytes());
        for row in rdr.records() {
            let (zip, pop): (String, u64) =
                row.expect("Invalid CSV data built into executable")
                    .deserialize(None)
                    .expect("Invalid CSV data built into executable");

            // For each prefix of this zip code, increment the population of
            // that prefix.
            for prefix_len in 0..maps.len() {
                // This is a very long way of writing `(... ||= 0) += pop`.
                match maps[prefix_len].entry(zip[0..prefix_len].to_owned()) {
                    Entry::Vacant(vacant) => {
                        vacant.insert(pop);
                    }
                    Entry::Occupied(mut occupied) => {
                        *occupied.get_mut() += pop;
                    }
                }
            }
        }

        PrefixPopulation { maps: maps }
    }

    /// Look up the population of a zip code prefix.  Calling this function
    /// with invalid data will panic, since this is intended to be called using
    /// purely compile-time data.
    fn lookup(&self, prefix: &str) -> u64 {
        if prefix.len() > ZIP_CODE_LENGTH {
            panic!("Invalid zip code prefix: {}", prefix);
        }
        // Look up the prefix, and return 0 if it isn't in our map.
        self.maps[prefix.len()]
            .get(prefix)
            .cloned()
            .unwrap_or_default()
    }

    // Build zip code chunks based on population data.
    fn build_chunks_recursive(&self,
                              target_population: u64,
                              prefix: &str,
                              chunk_id_for_prefix: &mut HashMap<String, String>) {
        let prefix_pop = self.lookup(prefix);
        if prefix_pop <= target_population {
            // We're small enough to fill a chunk on our own.
            trace!("Mapping {} (pop {}) to {}", prefix, prefix_pop, prefix);
            chunk_id_for_prefix.insert(prefix.to_owned(), prefix.to_owned());
        } else {
            // Check each possible "child" of this prefix, recursing for any
            // that are greater than or equal to our target size.  Collect
            // the smaller children in `leftovers`.
            let mut leftovers = vec![];
            for digit in 0..10 {
                let child_prefix = format!("{}{}", prefix, digit);
                let child_pop = self.lookup(&child_prefix);
                if child_pop >= target_population {
                    self.build_chunks_recursive(target_population,
                                                &child_prefix,
                                                chunk_id_for_prefix);
                } else {
                    leftovers.push(child_prefix);
                }
            }

            // Group our leftovers into chunks with names like `{prefix}_{i}`.
            // It's important to include the zero-length chunks here, so that
            // post-2010 zip codes can be placed in some chunk.
            let mut chunk_idx: u64 = 0;
            let mut chunk_pop: u64 = 0;
            for child_prefix in leftovers {
                let child_pop = self.lookup(&child_prefix);
                assert!(child_pop < target_population);
                if chunk_pop + child_pop > target_population {
                    chunk_idx += 1;
                    chunk_pop = 0;
                }
                chunk_pop += child_pop;
                let chunk_id = format!("{}_{}", prefix, chunk_idx);
                trace!("Mapping {} (pop {}) to {}",
                       child_prefix,
                       child_pop,
                       chunk_id);
                chunk_id_for_prefix.insert(child_prefix, chunk_id);
            }
        }
    }
}
