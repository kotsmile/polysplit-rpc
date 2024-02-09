# # Builder stage
# FROM rust:1.75 as builder
# WORKDIR /app
# COPY ./Cargo.lock ./Cargo.lock
# COPY ./Cargo.toml ./Cargo.toml
# COPY ./src ./src
# COPY ./.sqlx ./.sqlx
# COPY ./migrations ./migrations
# ENV SQLX_OFFLINE=true
# RUN cargo build --release
#
# # Production stage
# FROM rust:1.75
# WORKDIR /app  # Use a leading slash to be explicit about the path being absolute
# COPY --from=builder /app/target/release/polysplit-rpc ./polysplit-rpc
# CMD ["./polysplit-rpc"]
#
ARG RUST_VERSION=1.75
FROM rust:1.75 AS build

# Capture dependencies
WORKDIR /app/
RUN cargo init --bin
COPY Cargo.toml Cargo.lock .
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release
RUN rm /app/src/main.rs

COPY ./src ./src
COPY ./.sqlx ./.sqlx
COPY ./migrations ./migrations

RUN --mount=type=cache,target=/usr/local/cargo/registry \
  touch /app/src/main.rs && \
  cargo build --release 

CMD ["/app/target/release/polysplit-rpc"]

# Again, our final image is the same - a slim base and just our app
FROM rust:1.75 AS app
WORKDIR /app/
COPY --from=build /app/target/release/polysplit-rpc /app/polysplit-rpc
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["/app/polysplit-rpc"]
