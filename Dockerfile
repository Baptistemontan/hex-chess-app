FROM rust:latest as builder
WORKDIR /dist
RUN cargo install --locked cargo-leptos
RUN rustup toolchain install nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
COPY . .
RUN cargo leptos build --release

FROM ubuntu:latest
COPY --from=builder /dist/target/site /site
COPY --from=builder /dist/target/server/release/hex-chess-app /hex-chess-app
ARG PORT
ENV LEPTOS_OUTPUT_NAME="hex-chess-app"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:${PORT}"
ENV LEPTOS_RELOAD_PORT="3001"
ENTRYPOINT ["/hex-chess-app"]
EXPOSE 3000
