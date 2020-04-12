FROM rust:1.40

WORKDIR /usr/src/server

COPY Cargo.lock .
COPY Cargo.toml .

RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY ./src src

RUN cargo build --release
RUN cargo install --path .

EXPOSE 80

CMD ["server", "serve", "-p", "80"]
