# Use the official Rust image as the base image
FROM rust:latest as builder

WORKDIR /build_dir/
# Copy the local Cargo.toml and src/ directory into the container
COPY . .

# Build your Rust application
RUN cargo build --release

# FROM debian:latest
# COPY --from=builder /build_dir/target/release/dht_core .
ENV RUST_BACKTRACE=full

# Specify the command to run your application
CMD ["./target/release/dht_core"]
