#---------------------------------
# BUILDER IMAGE
#---------------------------------

FROM rust:latest AS builder

# Dependencies
RUN apt-get update && apt-get install -y cmake golang automake autoconf libtool libssl-dev musl-tools pkg-config

RUN rustup target add x86_64-unknown-linux-musl

ENV RUSTFLAGS="-C target-feature=+crt-static"

WORKDIR /build

# Build boringssl and quiche
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
    QUICHE_BSSL_PATH=$PWD/deps/boringssl RUSTFLAGS=-Clinker=musl-gcc\
    cargo build --release --examples --features pkg-config-meta --target x86_64-unknown-linux-musl

# Build web binaries
RUN git clone https://github.com/asurbernardo/http3support.git
RUN cd http3support && \
    PKG_CONFIG_ALLOW_CROSS=1 RUSTFLAGS=-Clinker=musl-gcc\
        cargo build --release --target x86_64-unknown-linux-musl

#---------------------------------
# FINAL IMAGE CONTAINING BINARIES
#---------------------------------

FROM scratch

# Dependencies
# RUN apk update && apk add --no-cache ca-certificates

RUN addgroup -g 1000 app
RUN adduser -D -s /bin/sh -u 1000 -G app app

WORKDIR /home/app/bin/

COPY --from=builder /build/quiche/target/release/examples/http3-client .
RUN chown app:app http3-client

COPY --from=builder /build/http3support/target/release/http3support .
RUN chown app:app http3support

EXPOSE 443

ENTRYPOINT ["./http3support"]