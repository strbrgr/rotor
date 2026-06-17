# syntax=docker/dockerfile:1.4
FROM rust:1.95-slim AS builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY server/ .
RUN cargo build --release --bins

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

FROM runtime AS sse
COPY --from=builder /build/target/release/sse /usr/local/bin/
EXPOSE 3001
CMD ["sse"]

FROM runtime AS gateway
COPY --from=builder /build/target/release/gateway /usr/local/bin/
EXPOSE 8080
CMD ["gateway"]

FROM runtime AS writer
COPY --from=builder /build/target/release/writer /usr/local/bin/
CMD ["writer"]

FROM runtime AS sensor
COPY --from=builder /build/target/release/sensor /usr/local/bin/
CMD ["sensor", "100"]
