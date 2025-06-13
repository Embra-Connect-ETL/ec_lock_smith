FROM rust:1.73-slim-bullseye

WORKDIR /app

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8089

COPY ./target/x86_64-unknown-linux-musl/release/ec_secrets_management /app/
COPY ./public /app/public/

CMD ["/app/ec_secrets_management"]

EXPOSE 8089