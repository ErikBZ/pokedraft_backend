FROM rust:1.81.0 as builder
WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo install --path .

FROM debian:bookworm-slim as runner

COPY requirements.txt ./
RUN apt update && apt install curl python3-full python3-pip -y
RUN pip install -r requirements.txt --break-system-packages

COPY --from=builder /usr/local/cargo/bin/pokedraft-backend /usr/local/bin/pokedraft-backend
COPY scripts scripts
COPY Rocket.toml ./
RUN chmod +x /scripts/entrypoint.sh

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["/scripts/entrypoint.sh"]
