# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2022-05-25

### Added

- Build new binaries, including Mac M1.

## [1.0.0] - 2021-02-10

### Changed

- Stabilized CLI interface.
- Set up automatic binary releases using GitHub Actions.

## [0.1.1] - 2017-02-09

### Changed

- Inner loop optimized, raising throughput from ~256 KiB/s to 65 MiB/s. I had suspected that two memory allocations per CSV row were slowing us down, but I am surprised that eliminating them improved performance by a factor of _250×_. As always, `malloc` is highly expensive in hot loops.

## [0.1.0] - 2017-02-05

### Added

- Initial release for internal use. This was only released as manually-built binaries.
