FROM rustlang/rust:nightly-bullseye-slim AS builder

ENV \
    TERM=xterm-color \
    TIMEZONE=Europe/Amsterdam

WORKDIR /usr/src/swc
COPY . .

RUN apt update \
  && apt-get install -y ca-certificates \
  && update-ca-certificates

RUN cargo test --lib
RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /usr/src/swc/target/release/swc /usr/local/bin/swc
CMD ["swc"]
