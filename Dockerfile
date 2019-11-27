#---------------------------------
# BASE BUILDER IMAGE
#---------------------------------

FROM rust:latest AS base_builder

RUN apt-get update && apt-get install -y cmake golang automake autoconf libtool libssl-dev musl-tools pkg-config

RUN rustup update

#---------------------------------
# QUICHE BUILDER IMAGE
#---------------------------------

FROM base_builder AS quiche_builder

WORKDIR /build

RUN git clone --recursive --single-branch https://github.com/cloudflare/quiche.git
RUN cd quiche/deps/boringssl && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_POSITION_INDEPENDENT_CODE=on .. && \
    make -j`nproc` && \
    cd .. && \
    mkdir -p .openssl/lib && \
    cp build/crypto/libcrypto.a build/ssl/libssl.a .openssl/lib && \
    ln -s $PWD/include .openssl
RUN cd quiche && \
    QUICHE_BSSL_PATH=$PWD/deps/boringssl \
    cargo build --release --examples --features pkg-config-meta

#---------------------------------
# APP BUILDER IMAGE
#---------------------------------

FROM base_builder AS app_builder

WORKDIR /build

ENV RUSTFLAGS="-C target-feature=+crt-static"

RUN rustup default nightly

RUN rustup target add x86_64-unknown-linux-musl

RUN git clone https://github.com/asurbernardo/http3support.git
RUN cd http3support && \
    PKG_CONFIG_ALLOW_CROSS=1 RUSTFLAGS=-Clinker=musl-gcc\
        cargo build --release --target x86_64-unknown-linux-musl
RUN ls -la /build/http3support/target/x86_64-unknown-linux-musl/release

#---------------------------------
# FINAL IMAGE CONTAINING BINARIES
#---------------------------------

FROM scratch

COPY --from=quiche_builder /build/quiche/target/release/examples/http3-client .
COPY --from=app_builder /build/http3support/target/x86_64-unknown-linux-musl/release/http3support .
COPY --from=app_builder /build/http3support/private ./private
COPY --from=app_builder /build/http3support/static ./static
COPY --from=app_builder /build/http3support/Rocket.toml .

EXPOSE 8000

ENTRYPOINT ["/http3support"]