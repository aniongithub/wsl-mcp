FROM rust:1-bookworm

RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt
