FROM rust:latest as builder
WORKDIR /dist
RUN cargo install --locked cargo-leptos
RUN rustup toolchain install nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
COPY . .
# RUN cargo update
RUN cargo leptos build --release

FROM ubuntu:latest
COPY --from=builder /dist/target/site /site
COPY --from=builder /dist/target/server/release/hex-chess-app /hex-chess-app
COPY --from=builder /dist/start.sh /start.sh
RUN apt-get update
RUN apt-get install wget -y
RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb
RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb
ENV RUST_LOG="info"
ENV LEPTOS_OUTPUT_NAME="hex-chess-app"
ENV LEPTOS_SITE_ROOT="site"
ENV APP_ENVIRONMENT="production"
RUN chmod 777 /start.sh
ENTRYPOINT ["/start.sh"]
EXPOSE ${PORT}
