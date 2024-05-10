FROM rust:latest as rust-build
WORKDIR /usr/samplegraph
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
RUN cargo build --release

FROM node:20 as node-build
WORKDIR /usr/client
COPY client .
RUN npm install
RUN npm run build

FROM debian:bookworm-slim as release
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
RUN ulimit -n 4096
WORKDIR /usr/client
COPY --from=node-build /usr/client/build ./build
WORKDIR /usr
COPY --from=rust-build /usr/samplegraph/target/release/samplegraph .
CMD ["./samplegraph"]
