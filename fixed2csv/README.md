# `fixed2csv`: A quick-and-dirty tool for converting fixed-width fields to CSV

You can do this with `awk`, but this should be faster.

To install:

```sh
cargo install -f --git https://github.com/faradayio/fixed2csv.git fixed2csv
```

Given an input file `input.txt`:

```txt
first     last      middle
John      Smith     Q
Sally     Jones
```

You should be able to run:

```sh
$ fixed2csv -v 10 10 6 < input.txt
first,last,middle
John,Smith,Q
Sally,Jones,
Processed 65 B in 1s, 65 B/s
```
