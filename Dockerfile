FROM rust:1-slim-bullseye AS build

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=build /target/release/podcast-proxy .

CMD ["/podcast-proxy"]