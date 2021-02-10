# `hashcsv`: Use CSV row contents to assign an ID to each row

`hashcsv` will take a CSV file as input, and output the same CSV data, appending an `id` column. The `id` column contains a UUID v5 hash of the normalized row contents. This tool is written in moderately optimized Rust and it should be suitable for large CSV files. It had a throughput of roughly 65 MiB/s when tested on a developer laptop.

## Usage

This can be invoked as either of:

```sh
hashcsv input.csv > output.csv
hashcsv < input.csv > output.csv
```

If `input.csv` contains:

```csv
a,b,c
1,2,3
1,2,3
4,5,6
```

Then `output.csv` will contain:

```csv
a,b,c,id
1,2,3,ab37bf3a-c35c-51a9-802d-8eda9ee2f50a
1,2,3,ab37bf3a-c35c-51a9-802d-8eda9ee2f50a
4,5,6,481492ee-82c7-58b9-95ec-d92cbcd332c4
```

There is also an option for renaming the `id` column. See `--help` for details.

## Limitations: Birthday problem

UUID v5 is based on an SHA hash, and it preserves 122 bits of the hash output.

This means that if you hash 2^(122/2) = 2^61 ≈ 2.3×10^18 rows, you should expect to have a 50% change of at least one collision. This is 2.3 _quintillion_ rows, which should be adequate for many applications. See [the birthday problem](https://en.wikipedia.org/wiki/Birthday_problem) for more information.

## Benchmarking

To measure throughput, build in release mode:

```sh
cargo build --release --target x86_64-unknown-linux-musl
```

Then use `pv` to measure output speed:

```sh
../target/x86_64-unknown-linux-musl/release/hashcsv test.csv | pv > /dev/null
```

To find where the hotspots are,

```sh
perf record --call-graph=lbr \
    ../target/x86_64-unknown-linux-musl/release/hashcsv test.csv > /dev/null
```
