# https://github.com/LukeMathWalker/cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# prepare build manifest
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# build/install dependencies
FROM chef AS builder 
ARG LINKERD_AWAIT_VERSION=0.2.7

RUN wget -O linkerd-await https://github.com/linkerd/linkerd-await/releases/download/release/v${LINKERD_AWAIT_VERSION}/linkerd-await-v${LINKERD_AWAIT_VERSION}-amd64
RUN chmod +x linkerd-await

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build the actual application
COPY . .
RUN cargo build --features vendored-openssl --release

# minimal runtime image
FROM ubuntu:22.04 AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder \
  /app/target/release/transactor \
  /app/target/release/distributor \
  /app/linkerd-await \
  /usr/local/bin/

ENTRYPOINT [ "linkerd-await", "--" ]
