FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --no-default-features --features docker --all

FROM gcr.io/distroless/static AS runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rumia /
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/healthcheck /

VOLUME /filestore
EXPOSE 10032
CMD ["/rumia"]
HEALTHCHECK --interval=30s --timeout=30s --start-period=10s --retries=3 CMD ["/healthcheck", "http://localhost:10032/health"]