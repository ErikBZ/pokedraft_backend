FROM rust:1.77.1 as builder
WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo install --path .

FROM debian:bookworm-slim as runner
RUN apt update && apt install curl -y
COPY --from=builder /usr/local/cargo/bin/pokedraft-backend /usr/local/bin/pokedraft-backend
COPY Rocket.toml Rocket.toml
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["pokedraft-backend"]
