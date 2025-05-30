FROM rust:latest

WORKDIR /usr/src/tank-game
COPY . .

RUN cargo install --path .

CMD ["tank-game"]
