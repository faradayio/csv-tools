# Miscellaneous Faraday CSV tools

This repository contains tools for manipulating CSV files, all written in Rust. Most of these are a page or two long, and they're intended to run as quickly as possible without requiring heroic optimization. If you like these, you will probably also like [`xsv`](https://github.com/BurntSushi/xsv), which provides an extensive toolkit of CSV-processing utilities written in Rust.

- [`hashcsv`](./hashcsv): Add a new column to a CSV file, containing a hash of the other columns. Useful for de-duplicating.

## To merge

These should probably be moved into this monorepo.

- https://github.com/faradayio/scrubcsv
- https://github.com/faradayio/geocode-csv
- https://github.com/faradayio/catcsv
- https://github.com/faradayio/fixed2csv
- https://github.com/faradayio/geochunk
