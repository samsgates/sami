# syntax=docker/dockerfile:1.7
FROM node:22.23.0-bookworm-slim AS console-builder
WORKDIR /build/console
COPY console/package.json console/package-lock.json console/.npmrc ./
RUN npm ci --no-audit --no-fund
COPY console/ ./
RUN npm run build

FROM rust:1.86-bookworm AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY src/ ./src/
COPY docs/openapi.json ./docs/openapi.json
COPY packs/ ./packs/
RUN cargo build --locked --release --workspace

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 sami && useradd --uid 10001 --gid sami --no-create-home --shell /usr/sbin/nologin sami \
    && install -d -o sami -g sami /var/lib/sami /opt/sami/console /opt/sami/packs
COPY --from=rust-builder --chown=sami:sami /build/target/release/sami /usr/local/bin/sami
COPY --from=console-builder --chown=sami:sami /build/console/dist /opt/sami/console
COPY --from=rust-builder --chown=sami:sami /build/packs /opt/sami/packs
ENV SAMI_DATA_DIR=/var/lib/sami SAMI_CONSOLE_DIR=/opt/sami/console SAMI_ENV=production
WORKDIR /opt/sami
USER 10001:10001
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 CMD curl --fail --silent http://127.0.0.1:8080/ready || exit 1
ENTRYPOINT ["sami"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8080"]
