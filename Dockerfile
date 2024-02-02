# Builder stage
FROM rust:1.70 as builder
WORKDIR /app
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

# Production stage
FROM rust:1.70
WORKDIR /app  # Use a leading slash to be explicit about the path being absolute
COPY --from=builder /app/target/release/polysplit-rpc ./polysplit-rpc
CMD ["./polysplit-rpc"]
