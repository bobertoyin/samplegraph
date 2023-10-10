FROM rust:latest as rust-build
WORKDIR /usr/server
COPY server .
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
WORKDIR /usr/server
COPY --from=rust-build /usr/server/target/release/server .
CMD ["./server"]
