FROM rust:1.63 as builder
WORKDIR /usr/src/application

COPY Cargo.lock Cargo.toml .
RUN mkdir src && touch src/lib.rs && cargo build --release && rm -rf src

COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /usr/src/application/target/release/argocd-workload /usr/local/bin/argocd-workload

USER 65532:65532
EXPOSE 8080/tcp
CMD ["/usr/local/bin/argocd-workload"]
