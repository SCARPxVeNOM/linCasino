FROM rust:1.86-slim

SHELL ["bash", "-c"]

RUN apt-get update && apt-get install -y \
    pkg-config \
    protobuf-compiler \
    clang \
    make \
    jq

# Install wasm32-unknown-unknown target for WebAssembly compilation
RUN rustup target add wasm32-unknown-unknown

# Install Linera services with memory optimizations
# Build with single job to reduce memory usage
ENV CARGO_BUILD_JOBS=1
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# Install services separately to reduce memory pressure
RUN cargo install --locked linera-storage-service@0.15.7
RUN cargo install --locked linera-service@0.15.7

RUN apt-get install -y curl
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm http-server

WORKDIR /build

HEALTHCHECK CMD ["curl", "-s", "http://localhost:5173"]

ENTRYPOINT bash /build/run.bash

