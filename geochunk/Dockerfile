FROM ekidd/rust-musl-builder

# We need to add the source code to an image because rust-musl-builder
# assumes uid 1000, but TravisCI is now using uid 2000. So we copy our
# source code in and fix the permissions.
ADD . ./
RUN sudo chown -R rust:rust .
CMD cargo build --release
