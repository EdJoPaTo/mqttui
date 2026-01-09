FROM docker.io/library/rust:1.74.0-bookworm AS builder
WORKDIR /usr/local/src
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /usr/local/src/target/release/mqttui /usr/local/bin/mqttui
ENTRYPOINT ["/usr/local/bin/mqttui"]