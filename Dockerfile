FROM rust:1.75.0 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .


FROM debian:bookworm-slim as runner
COPY --from=builder /usr/local/cargo/bin/pokedraft-backend /usr/local/bin/pokedraft-backend
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["pokedraft-backend"]