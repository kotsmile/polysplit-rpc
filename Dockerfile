FROM rust:1.75 as build
WORKDIR /app
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release
CMD ["./app/target/release/polysplit-rpc"]


# FROM rust:1.75
# COPY --from=build /project/target/release/polysplit-rpc .
