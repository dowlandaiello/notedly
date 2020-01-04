FROM rust:1.40

WORKDIR /usr/src/server

COPY . .

EXPOSE 80

RUN cargo install --path .

CMD ["server", "serve", "-p", "80"]
