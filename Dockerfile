FROM rust:latest as rust-build
WORKDIR /usr/samplegraph
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
RUN cargo build --release

FROM node:20 as node-build
WORKDIR /usr/frontend
COPY frontend .
RUN npm install
RUN npm run build

FROM gcr.io/distroless/cc-debian12:latest as release
WORKDIR /usr/frontend
COPY --from=node-build /usr/frontend/dist ./dist
WORKDIR /usr
COPY --from=rust-build /usr/samplegraph/target/release/samplegraph .
CMD ["./samplegraph"]
