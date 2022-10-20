FROM rustlang/rust:nightly-bullseye AS builder

WORKDIR /build

COPY ./ .

# Build app (bin winn be in /build/target/release/swc)
RUN cargo build --release

## Final image
FROM debian:bullseye-slim
RUN apt update \
  && apt-get install -y ca-certificates \
  && update-ca-certificates

# Copy bin from builder to this new image
COPY --from=builder /build/target/release/swc /app/

CMD ["/app/swc"]
