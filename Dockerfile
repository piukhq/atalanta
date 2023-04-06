# https://github.com/LukeMathWalker/cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# prepare build manifest
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# build dependencies
FROM chef AS builder 

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build the actual application
COPY . .
RUN cargo build --release

# minimal runtime image
FROM ubuntu:22.04 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/transactor /app/target/release/distributor /usr/local/bin/
