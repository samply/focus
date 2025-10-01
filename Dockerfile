FROM rust:1 AS builder
WORKDIR /usr/src/app
COPY . .
RUN --mount=type=cache,target=/usr/src/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --path .

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /usr/local/cargo/bin/focus /usr/local/bin/
CMD ["focus"]
