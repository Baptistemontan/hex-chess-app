FROM rust:latest as builder
WORKDIR /build
RUN apt update
RUN apt install pkg-config -y
RUN rustup toolchain install nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN cargo install --locked cargo-leptos

COPY . .

RUN cargo leptos build --release

FROM rust:latest
COPY --from=builder /build/target/server/release/hex-chess-app /hex-chess-app
COPY --from=builder /build/target/site /site
ENV LEPTOS_OUTPUT_NAME="hex-chess-app"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="pkg"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_RELOAD_PORT="3001"
ENTRYPOINT ["/hex-chess-app"]
EXPOSE 3000
