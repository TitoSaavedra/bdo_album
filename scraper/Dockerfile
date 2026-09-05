# syntax=docker/dockerfile:1
#
# Builds the headless CLI (`scrape` + `download_daemon`, package
# `bdo-scraper-cli`) — the Tauri GUI in `src-tauri/` is never compiled here,
# only its Cargo.toml is copied in so the workspace graph resolves.
#
# Chromium + its Linux system deps are baked in at build time (not on first
# container start), so a daily `--rm` cron run of `scrape` never re-downloads
# ~240MB. Runtime container runs as root, not a security choice: verified by
# testing — `bdo_scraper_core::scraper::browser::bootstrap_driver` calls
# `install_browsers()`, which on Linux unconditionally re-runs the driver's
# own `install --with-deps` on *every* browser launch (not just when
# something is actually missing). `--with-deps` shells out to apt and, when
# not already root, tries `su` to escalate — which fails non-interactively
# (`su: Authentication failure`) with no root password configured. So a
# non-root runtime user can never launch the browser at all today, no matter
# what's pre-baked at build time. Dropping privileges here would require a
# code change upstream (skip the redundant `--with-deps` when deps are
# already known-present) — out of scope for this Dockerfile.

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

COPY --from=builder /build/target/release/scrape /usr/local/bin/scrape
COPY --from=builder /build/target/release/download_daemon /usr/local/bin/download_daemon
COPY --from=builder /build/playwright-rs-cli /opt/tools/playwright-rs-cli
ENV PLAYWRIGHT_RS_CLI_PATH=/opt/tools/playwright-rs-cli/bin/playwright-rs

# Bake driver + Chromium + system deps now (root, network available) instead
# of on first container start — first-run latency/flakiness avoided even
# though the runtime container also ends up root (see header comment).
RUN /opt/tools/playwright-rs-cli/bin/playwright-rs install --driver-only \
    && /opt/tools/playwright-rs-cli/bin/playwright-rs install chromium --with-deps

WORKDIR /root
ENTRYPOINT []
CMD ["/usr/local/bin/scrape"]
