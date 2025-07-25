FROM rust:latest

WORKDIR /usr/src/client
COPY . .

RUN cargo install --path .

CMD ["client"]
