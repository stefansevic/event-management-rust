# Multi-stage build za sve servise

# Stage 1: Kompajliranje 
FROM rust:latest AS builder

WORKDIR /app
COPY . .

ARG SERVICE_NAME
RUN cargo build --release --bin ${SERVICE_NAME}

#  Stage 2: Minimalan image za pokretanje 
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

ARG SERVICE_NAME
COPY --from=builder /app/target/release/${SERVICE_NAME} /usr/local/bin/service

CMD ["/usr/local/bin/service"]
