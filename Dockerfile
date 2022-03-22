FROM rust:1.59.0 AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev libssl-dev
RUN update-ca-certificates

WORKDIR /build

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release -j 8 --bin pod-lifetime-limiter

FROM alpine:3.15.1

WORKDIR /build

ENV USER=lifetimeuser
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

# Copy our build
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/pod-lifetime-limiter ./

RUN chown -R $USER:$USER .

USER "${USER}":"${USER}"

CMD ["./pod-lifetime-limiter"]
