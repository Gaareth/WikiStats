FROM rust:1.89 AS builder
WORKDIR /usr/src

# Download the target for static linking.
RUN rustup target add x86_64-unknown-linux-musl

RUN rustup toolchain install nightly
RUN rustup target add x86_64-unknown-linux-musl --toolchain nightly

RUN apt-get update && \
    apt-get install -y \
        perl \
        perl-modules \
        build-essential \
        musl-tools \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates



# Copy the source and build the application.
COPY . .
RUN cargo +nightly build --bin server --target x86_64-unknown-linux-musl --release

# Copy the statically-linked binary into a scratch container.
FROM alpine:latest

# Add a non-root user
RUN adduser -D -u 1000 spuser

# Create app and logs directories
WORKDIR /app
RUN mkdir logs

# Copy the static binary
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/server .

# Set ownership and switch user
RUN chown -R spuser:spuser /app
USER spuser

# Expose port and run
EXPOSE 1870
ENTRYPOINT ["./server"]
