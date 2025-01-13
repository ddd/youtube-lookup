# Use the official Rust image as the base image for building
FROM rust:1.75-slim-bullseye AS builder

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . .

# Build the application in release mode
RUN cargo build --release

# Create a minimal runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/youtube-lookup .

# Copy the static HTML file 
# Adjust the path if your HTML file is in a different location
COPY --from=builder /app/static/index.html ./static/index.html

# Expose the port the app runs on
EXPOSE 3000

# Set the startup command
CMD ["./youtube-lookup"]
