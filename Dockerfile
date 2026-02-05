# LinCasino Docker Image
# Builds Rust/Linera environment with Node.js for the casino platform

FROM rust:1.86-slim

LABEL maintainer="LinCasino Team"
LABEL description="Linera Casino - Multi-chain casino with Poker, Rummy, and Roulette"
LABEL version="1.0"

SHELL ["bash", "-c"]

# Install all system dependencies in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    protobuf-compiler \
    clang \
    make \
    jq \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install wasm32-unknown-unknown target for WebAssembly compilation
RUN rustup target add wasm32-unknown-unknown

# Install Linera services with memory optimizations
# Build with single job to reduce memory usage
ENV CARGO_BUILD_JOBS=1
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# Install services separately to reduce memory pressure
RUN cargo install --locked linera-storage-service@0.15.7 \
    && cargo install --locked linera-service@0.15.7

# Install Node.js LTS and npm packages
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && npm install -g pnpm http-server \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Expose all required ports
EXPOSE 5173 5174 5175 8080 8081 8082 8083

HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=5 \
    CMD curl -sf http://localhost:5173 || exit 1

ENTRYPOINT ["bash", "/build/run.bash"]

