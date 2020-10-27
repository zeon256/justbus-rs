FROM rust:latest as builder

WORKDIR /home/rust
COPY . .
RUN cargo build --release --features swisstable
RUN strip ./target/release/justbus-rs
ENTRYPOINT ["./target/release/justbus-rs"]

FROM debian:latest
WORKDIR /home/rust/
RUN apt-get update \
    && apt-get install -y tzdata libcurl4 libssl-dev \
    && rm -r /var/lib/apt/lists/* \
    && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false
COPY --from=builder /home/rust/target/release/justbus-rs .
ENTRYPOINT ["./justbus-rs"]