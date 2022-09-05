FROM rust:latest

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk
RUN cargo install wasm-bindgen-cli
RUN cargo install cargo-watch

RUN apt update
RUN apt install -y redis-tools

COPY dev-entrypoint.sh /run.sh

ENTRYPOINT ["/run.sh"]
