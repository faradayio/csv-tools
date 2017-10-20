# `geochunk`: Break data up into chunks of similar population

`geochunk` is intended for use in a distributed system.  It provides a deterministic mapping from zip codes to "geochunks" that you can count on remaining stable.  Geochunks will try to approximate the population size that you specify.

```
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
```

## How it works

See the [Jupyter notebook][notebook], which explains the algorithm. We use census data to build variable-length zip code prefixes, and then try to group those prefixes together in a way that balances population size as much as possible.

[notebook]: ./notebook/ZipDistribution.ipynb

## Installing

[Binary releases are available][releases] for OS X and Linux. To install these, unzip the file and copy `geochunk` to `/usr/local/bin` or another directory in your `PATH`:

```sh
unzip geochunk-v0.1.4-osx.zip
sudo cp geochunk /usr/local/bin/
```

You can also install from source:

```sh
# Mac and Linux.
curl https://sh.rustup.rs -sSf | sh
cargo install geochunk

# On Windows, see https://www.rustup.rs/ for instructions on installing
# Rust, then run:
cargo install geochunk
```

Windows hasn't been tested, but it should work, perhaps after some tweaking. If it doesn't, please feel free to submit issues, PRs or even an [AppVeyor][] build configuration. In general, Rust command-line tools should work fine on Windows.

[releases]: https://github.com/faradayio/geochunk/releases
[AppVeyor]: https://www.appveyor.com/
