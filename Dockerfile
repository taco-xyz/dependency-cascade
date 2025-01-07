FROM rust:1.75-slim as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/dependency-cascade /usr/local/bin/
ENTRYPOINT ["dependency-cascade"]