# Dockerfile for building our Rust tools using GitHub Actions.
#
# Note that this actually gets "run" from the top-level directory and not the
# Rust subdirectory, so that we can see files in the top-level directories.

FROM ekidd/rust-musl-builder

# Add all the source code we'll need instead of trying to mount it, so that we
# don't have to mess around with UIDs.
ADD --chown=rust:rust . ./

WORKDIR /home/rust/src/fixed2csv

CMD cargo test && cargo clippy -- -D warnings && cargo deny check && cargo build --release
