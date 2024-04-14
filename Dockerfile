FROM rust:1.76.0-bookworm as builder

WORKDIR /usr/src/omnity

COPY . .

RUN cargo build -p runes_oracle --release

FROM debian:bookworm-slim

COPY --from=builder /usr/src/omnity/target/release/runes_oracle /usr/local/bin
RUN apt-get update && apt-get install -y openssl

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
