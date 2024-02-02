FROM rust:1.75 as build
RUN USER=root cargo new --bin project
WORKDIR /project
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm -rf src
COPY ./src ./src
RUN cargo build --release


FROM rust:1.75
COPY --from=build /project/target/release/polysplit-rpc .
CMD ["./polysplit-rpc"]
