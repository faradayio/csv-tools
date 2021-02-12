# Miscellaneous Faraday CSV tools

This repository contains tools for manipulating CSV files, all written in Rust. Most of these are a page or two long, and they're intended to run as quickly as possible without requiring heroic optimization. If you like these, you will probably also like [`xsv`](https://github.com/BurntSushi/xsv), which provides an extensive toolkit of CSV-processing utilities written in Rust.

- [`catcsv`](./catcsv): Concatenate directory of CSV files into a single CSV stream, decompressing as needed.
- [`fixed2csv`](./fixed2csv): Convert fixed-width fields to CSV.
- [`geochunk`](./geochunk): Add a column to a CSV file that groups records into similarly-sized chunks based on US ZIP codes and census data.
- [`geocode-csv`](./geocode-csv): Geocode CSV files in bulk using the SmartyStreets API.
- [`scrubcsv`](./scrubcsv): Turn messy, slightly corrupt CSV files into something clean and standardized.
- [`hashcsv`](./hashcsv): Add a new column to a CSV file, containing a hash of the other columns. Useful for de-duplicating.
