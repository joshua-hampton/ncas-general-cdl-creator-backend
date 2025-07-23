FROM rust:1.87 AS builder
WORKDIR /usr/src/rust-backend
COPY . .
RUN cargo install --path .

FROM ubuntu:noble
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/rust-backend /usr/local/bin/rust-backend
CMD ["rust-backend"]

