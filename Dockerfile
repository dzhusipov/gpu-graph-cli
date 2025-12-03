# Build stage - используем тот же базовый образ для совместимости GLIBC
FROM nvidia/cuda:12.0.0-base-ubuntu22.04 AS builder

# Установка Rust и инструментов для компиляции
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    build-essential \
    pkg-config \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy manifest files
COPY Cargo.toml ./

# Create a dummy source file to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM nvidia/cuda:12.0.0-base-ubuntu22.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/gpu-graph-cli /usr/local/bin/gpu-graph-cli

# nvidia-smi should be available via nvidia runtime
# Make sure to run with: docker run --gpus all ...

ENTRYPOINT ["/usr/local/bin/gpu-graph-cli"]

