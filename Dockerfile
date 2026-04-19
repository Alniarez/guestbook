FROM rust:1.87-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/guestbook .
VOLUME ["/app/data"]
EXPOSE 50007
CMD ["./guestbook"]
