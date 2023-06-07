FROM rust:1.69.0-alpine3.17

Run apk add --update g++
RUN cargo install cargo-watch --locked

RUN mkdir -p /project
WORKDIR /project
