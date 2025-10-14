# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-10-14

### Added

- `--output-stats-to-file <PATH>` option to write processing statistics to a JSON file. The output includes rows, bad_rows, elapsed_seconds, bytes_processed, and bytes_per_second.

### Changed

- Migrated from deprecated `structopt` to `clap` v4.5 with derive macros.
- Updated `env_logger` from 0.9 to 0.11.
- Updated `humansize` from 1.0 to 2.1.
- Updated all other dependencies to their latest compatible versions.

## [1.0.0] - 2022-05-25

### Added

- Official 1.0.0 release, because this program has been used in production for years.
- `--clean-column-names=stable` provides a new column-name-cleaning algorithm. This converts ASCII characters to lowercase, converts spaces to `_`, and verifies that the resulting column name is a unique, valid C identifier. The goal of this new column name cleaner is to make our output column names easily predictable.
- `--reserve-column-names=REGEX` will cause `scrubcsv` to fail if it would generate output columns matching `REGEX`.
- Binary builds for more platforms, including M1.

## [0.1.9] - 2020-07-14

### Added

- Write bad row info to debug log.
