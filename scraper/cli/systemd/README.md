# Headless scraper on Ubuntu (systemd)

Two independent units:

- **`bdo-scraper.service`** + **`bdo-scraper.timer`** — the `scrape` binary. A
  oneshot run, triggered daily by the timer (`RandomizedDelaySec=300` spreads
  the actual start out over a 5-minute window so it doesn't always hit
  Garmoth at exactly the same second). Walks all ~32 seeded classes
  individually × 7 day-windows × 10 regions × mode=both, parallelism 2 — the
  same scope as running the desktop GUI with every filter left at its
  default, minus the "all classes in one global-ranking request" shortcut
  the GUI takes when its class filter is empty.
- **`bdo-scraper-download-daemon.service`** — the `download_daemon` binary. A
  continuous worker draining the album's auto-download queue (presets marked
  "wanted"). `Type=simple`, `Restart=on-failure`, always on.

Both binaries are Tauri-free (`bdo-scraper-core` + `bdo-scraper-cli`, see
`scraper/Cargo.toml`'s `[workspace]`) — no GUI, no display required.

## 1. Build

On a Linux host (no cross-compilation needed — build directly on the target,
or on an equivalent Ubuntu box/CI runner):

```bash
cd scraper
cargo build --release -p bdo-scraper-cli
# binaries land at scraper/target/release/{scrape,download_daemon}
```

## 2. Install the playwright-rs bootstrap CLI

Both binaries launch headless Chromium via `playwright-rs`, which needs its
driver fetched once via a small bootstrap CLI (same one the desktop app
bundles, and the same command CI already runs for the Windows build):

```bash
cargo install playwright-rs --version 0.15.0 --locked --features cli \
  --root /opt/bdo-scraper/tools/playwright-rs-cli --bin playwright-rs
```

`scrape`/`download_daemon` look for it at
`./tools/playwright-rs-cli/bin/playwright-rs` relative to their own
executable by default, or at `$PLAYWRIGHT_RS_CLI_PATH` if set (see
`.env.example`). Either place the `tools/` directory next to the installed
binaries under `/opt/bdo-scraper/`, or point `PLAYWRIGHT_RS_CLI_PATH` at
wherever `--root` above put it.

On first run, this CLI in turn downloads the actual Playwright driver
(~90 MB) and Chromium itself (~150 MB) into the service user's cache dir —
requires outbound internet the first time only.

### System libraries for headless Chromium

Prefer letting Playwright's own driver run its `install-deps` step (it knows
the right package set per distro) over hand-maintaining an `apt` list. If
that isn't available, this is the known-good fallback set on Debian/Ubuntu:

```bash
sudo apt-get install -y \
  libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 \
  libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libgbm1 libasound2 \
  libpango-1.0-0 libcairo2 fonts-liberation
```

## 3. Garmoth session (`garmoth_auth.json`)

`download_daemon` needs an authenticated Garmoth session to click the
"Download" button on a preset page (`scrape` does not — it only reads the
public `search-advanced` API). This session is a Discord OAuth cookie; there
is no way to obtain it without a human logging into garmoth.com through a
real browser, so this step stays manual — it just moves from "open the
Windows GUI" to "copy one file to the server":

1. On Windows, open the desktop GUI (`BDO Dashboard`, Tauri identifier
   `com.bdo.dashboard`) and import a session there as usual (Cookie-Editor
   export → the app's "import Garmoth session" flow). This writes
   `garmoth_auth.json` into the app's Tauri `app_data_dir()`, typically:

   ```
   %APPDATA%\com.bdo.dashboard\garmoth_auth.json
   ```

2. Copy that file to the server, to the path `GARMOTH_SESSION_FILE` points at
   (see `.env.example` — defaults to `/opt/bdo-scraper/garmoth_auth.json`):

   ```bash
   scp "%APPDATA%\com.bdo.dashboard\garmoth_auth.json" \
       bdoscraper@server:/opt/bdo-scraper/garmoth_auth.json
   ```

3. **This session expires periodically** (Discord OAuth cookies aren't
   forever) — when `download_daemon`'s logs show download failures with
   "download button not found ... imported session is likely missing or
   expired", repeat steps 1–2 with a freshly exported cookie.

## 4. Configure and install the units

```bash
sudo useradd --system --home /opt/bdo-scraper --shell /usr/sbin/nologin bdoscraper
sudo mkdir -p /opt/bdo-scraper
sudo cp target/release/scrape target/release/download_daemon /opt/bdo-scraper/
sudo cp scraper/cli/systemd/.env.example /opt/bdo-scraper/.env
sudo $EDITOR /opt/bdo-scraper/.env   # fill in DATABASE_URL, R2_*, GARMOTH_SESSION_FILE
sudo chown -R bdoscraper:bdoscraper /opt/bdo-scraper

sudo cp scraper/cli/systemd/bdo-scraper.service \
        scraper/cli/systemd/bdo-scraper.timer \
        scraper/cli/systemd/bdo-scraper-download-daemon.service \
        /etc/systemd/system/

sudo systemctl daemon-reload
sudo systemctl enable --now bdo-scraper.timer
sudo systemctl enable --now bdo-scraper-download-daemon.service
```

Check status / logs:

```bash
systemctl list-timers bdo-scraper.timer
journalctl -u bdo-scraper -f
journalctl -u bdo-scraper-download-daemon -f

# Trigger a run immediately without waiting for the timer:
sudo systemctl start bdo-scraper.service
```
