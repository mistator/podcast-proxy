FROM rust:1-slim-trixie AS build

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /target/release/podcast-proxy .

CMD ["/podcast-proxy"]