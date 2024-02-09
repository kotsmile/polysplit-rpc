# Builder stage
FROM rust:1.75 as builder
WORKDIR /app
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Production stage
FROM rust:1.75
WORKDIR /app  # Use a leading slash to be explicit about the path being absolute
COPY --from=builder /app/target/release/polysplit-rpc ./polysplit-rpc
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["./polysplit-rpc"]
