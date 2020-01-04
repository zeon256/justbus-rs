#FROM rust:latest as builder
#
#WORKDIR /home/rust
#COPY . .
#RUN cargo build --release
#ENTRYPOINT ["./target/release/justbus-rs"]
#
#FROM scratch
#WORKDIR /home/rust/
#COPY --from=builder /home/rust/target/release/justbus-rs .
#ENTRYPOINT ["./justbus-rs"]

FROM debian:latest
WORKDIR /home/rust/
RUN apt-get update \
    && apt-get install -y tzdata libcurl4 libssl-dev \
    && rm -r /var/lib/apt/lists/* \
    && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false
COPY ./target/release/justbus-rs .
ENTRYPOINT ["./justbus-rs"]