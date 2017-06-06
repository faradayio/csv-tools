# `geochunk`: Break data up into chunks of similar population

`geochunk` is intended for use in a distributed system.  It provides a
deterministic mapping from zip codes to "geochunks" that you can count on
remaining stable.

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
