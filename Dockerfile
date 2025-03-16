# First stage: Build the application
FROM rust:slim as builder

WORKDIR /app

# Copy over the manifest files
COPY Cargo.toml Cargo.lock* ./

# This is a trick to build the dependencies first
# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY src/ src/

# Build the application
RUN touch src/main.rs && \
    cargo build --release

# Second stage: Create the runtime image
FROM alpine:latest

# Install dependencies required for running dynamically linked Rust binary
RUN apk add --no-cache ca-certificates libgcc libssl1.1 musl-dev

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/identity-server .

# Set the binary as executable
RUN chmod +x /app/identity-server

# Create a non-root user to run the application
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

# Command to run the executable
CMD ["./identity-server"]

