# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2017-02-09

### Changed

- Inner loop optimized, raising throughput from ~256 KiB/s to 65 MiB/s. I had suspected that two memory allocations per CSV row were slowing us down, but I am surprised that eliminating them improved performance by a factor of _250×_. As always, `malloc` is highly expensive in hot loops.

## [0.1.0] - 2017-02-05

### Added

- Initial release for internal use. This was only released as manually-built binaries.
