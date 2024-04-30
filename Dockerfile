FROM rust:latest as builder

RUN apt update && apt upgrade -y && apt install -y protobuf-compiler libprotobuf-dev

WORKDIR /usr/src/zkrpc

COPY . .

RUN cargo install --path ./zkrpc

# multistage run
FROM debian:latest


RUN mkdir -p /root/.space-dev/config

COPY --from=builder /usr/local/cargo/bin/zkrpc /usr/local/bin

COPY config/config.example.yaml /root/.space-dev/config/config.yaml

ENV ENV=dev

CMD ["zkrpc","server"]
