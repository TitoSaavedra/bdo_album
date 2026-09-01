# syntax=docker/dockerfile:1
#
# Builds the headless CLI (`scrape` + `download_daemon`, package
# `bdo-scraper-cli`) — the Tauri GUI in `src-tauri/` is never compiled here,
# only its Cargo.toml is copied in so the workspace graph resolves.
#
# Chromium + its Linux system deps are baked in at build time (not on first
# container start): the apt step installs into the image's rootfs either way
# (nothing to persist across a `--rm` container via a volume), so doing the
# Playwright driver/Chromium download in the same RUN is free — it removes
# first-run latency on the daily cron `scrape` run and lets the runtime
# container run as non-root always.

########## 1. builder ##########
FROM rust:1-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY cli ./cli
# Only src-tauri's manifest matters (that crate is never compiled here — we
# build -p bdo-scraper-cli only) but Cargo still parses the whole manifest
# graph, including resolving `lib.path`, so the source tree has to be
# present too (gen/, icons/ are excluded via .dockerignore).
COPY src-tauri ./src-tauri

RUN cargo build --release --locked -p bdo-scraper-cli

RUN cargo install playwright-rs --version 0.15.0 --locked --features cli \
      --root /build/playwright-rs-cli --bin playwright-rs

########## 2. runtime ##########
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Never assume UID/GID 1000 — fed from compose's ${PUID}/${PGID}.
ARG PUID=1000
ARG PGID=1000
RUN groupadd -g "${PGID}" scraper \
    && useradd -u "${PUID}" -g "${PGID}" -m -d /home/scraper -s /usr/sbin/nologin scraper

COPY --from=builder /build/target/release/scrape /usr/local/bin/scrape
COPY --from=builder /build/target/release/download_daemon /usr/local/bin/download_daemon
COPY --from=builder /build/playwright-rs-cli /opt/tools/playwright-rs-cli
ENV PLAYWRIGHT_RS_CLI_PATH=/opt/tools/playwright-rs-cli/bin/playwright-rs

# Bake driver + Chromium + system deps now (root, network available) instead
# of on first container start.
RUN /opt/tools/playwright-rs-cli/bin/playwright-rs install --driver-only \
    && /opt/tools/playwright-rs-cli/bin/playwright-rs install chromium --with-deps \
    && chown -R scraper:scraper /home/scraper /opt/tools

USER scraper
WORKDIR /home/scraper
ENTRYPOINT []
CMD ["/usr/local/bin/scrape"]
