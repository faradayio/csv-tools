# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2025-03-04

### Added

- Added `--select-columns` which accepts a CSV of headers (in their final post-processed form) and only returns those headers. While this is technically redundant with `xsv select`, it allows earlier field selection when processing very large files.

## [1.0.0] - 2022-05-25

### Added

- Official 1.0.0 release, because this program has been used in production for years.
- `--clean-column-names=stable` provides a new column-name-cleaning algorithm. This converts ASCII characters to lowercase, converts spaces to `_`, and verifies that the resulting column name is a unique, valid C identifier. The goal of this new column name cleaner is to make our output column names easily predictable.
- `--reserve-column-names=REGEX` will cause `scrubcsv` to fail if it would generate output columns matching `REGEX`.
- Binary builds for more platforms, including M1.

## [0.1.9] - 2020-07-14

### Added

- Write bad row info to debug log.
