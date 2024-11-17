# Stage 1: Builder
FROM rust:1.73 as builder

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Install Trunk for building the frontend
RUN cargo install trunk --version 0.16.0
RUN cargo install wasm-bindgen-cli --version 0.2.80

# Set the working directory
WORKDIR /usr/src/myapp

# Copy Cargo.toml and Cargo.lock first for better caching
COPY Cargo.toml Cargo.lock ./

# Fetch dependencies (this layer will be cached if Cargo.toml and Cargo.lock haven't changed)
RUN cargo fetch

# Copy the rest of the source code
COPY . .

# **Remove the `cargo install --path .` step**
# This is unnecessary for a library crate and can interfere with the build process.

# Build the project for the WASM target
RUN cargo build --release --target wasm32-unknown-unknown

# Build the frontend assets using Trunk
RUN trunk build --release

# **Optional: Verify the contents of the `dist` directory**
RUN ls -la /usr/src/myapp/dist

# Stage 2: Runtime
FROM python:3.11-slim

# Set the working directory
WORKDIR /usr/src/myapp/dist

# Copy the built frontend assets from the builder stage
COPY --from=builder /usr/src/myapp/dist /usr/src/myapp/dist

# Expose port 8080
EXPOSE 8080

# Serve the frontend using Python's HTTP server
CMD ["python3", "-m", "http.server", "8080", "--bind", "0.0.0.0"]