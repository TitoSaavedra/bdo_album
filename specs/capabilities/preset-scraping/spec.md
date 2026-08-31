# Capability: preset-scraping

> LIVING truth — keep this current. Changes arrive as `specs/changes/<name>/spec-delta.md`
> and are merged here when the change is archived. This file should always describe what the
> system does *now*, not how it got here.

## Purpose
The Dashboard (`scraper/`) pulls popular Beauty Album presets from Garmoth, uploads their
images (and, once repaired/completed, their PAB files) to Cloudflare R2, and writes
metadata to the shared Postgres DB so `album/` can read it — with live progress in the
Dashboard UI and a live feed to any open Album instance.

## Requirements
- Garmoth's Cloudflare JS challenge means requests can't be plain HTTP — scraping runs
  through headless Chromium (`playwright-rs`), and even the "API" calls are routed
  through the browser's own TLS fingerprint rather than a bare HTTP client.
- Scraping is class-priority scheduled — not a flat queue — so favorited/active classes
  surface sooner.
- A scraping run is tracked as a session (`scrapper_sessions`/`scraper_sessions`) with
  running/done/error/cancelled status, per-class stats, and structured logs, all visible
  live in the Dashboard.
- A Garmoth session can be imported by pasting session data, which can drive
  auto-download of specific presets without a full scrape.
- On successful image upload, a `preset_uploaded` event fires (Postgres NOTIFY) so any
  open Album instance can patch its grid live; a PAB-only completion (no new images)
  fires the same event without bumping the "new preset" counters on the Album side.

## Behavior / scenarios
- **Given** a scraping session is running **when** a class finishes its batch **then** the
  Dashboard's live stats (fetched/images_ok/errors) update without a page reload.
- **Given** the Album's wishlist has presets queued for auto-download **when** the scraper's
  auto-download worker picks up `album_user_prefs.auto_download_requested_at` **then** it
  downloads and repairs the PAB, clears the request, and records an error instead if the
  attempt fails (so it can be retried from the Album side).
- **Given** a first-run install **when** the app needs the Playwright browser driver
  **then** `browser.rs::bootstrap_driver` shells out to a bundled CLI and blocks until it
  finishes, with no progress surfaced to the UI yet (tracked in the roadmap, Phase 2).

## Notes
- The scraper is the schema owner: every migration, including Album-only preference
  tables, lives in `scraper/src-tauri/migrations/` (see
  `specs/constitution/tech-stack.md`).
- PAB bytes are no longer patched on upload — R2 keeps the pristine original; the
  "editable" byte patch is applied client-side at export time instead (Album) /
  repaired locally via a standalone tool (scraper) when needed.
