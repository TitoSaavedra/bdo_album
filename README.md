# BDO Album

A two-app desktop suite for browsing and curating Black Desert Online "Beauty Album" character presets, built with [Tauri 2](https://tauri.app/) and [Svelte 5](https://svelte.dev/).

- **[Dashboard](scraper/)** — scrapes popular presets from [Garmoth](https://garmoth.com), uploads preset images to Cloudflare R2, and writes metadata to PostgreSQL.
- **[Album](album/)** — reads the same PostgreSQL database and lets the user browse presets, manage favorites/discards, and import them onto their own in-game characters (via face grid).

The two apps are fully independent Tauri/Rust projects (no shared crate or workspace); they only share a Postgres database and an R2 bucket.

## Architecture

```
┌─────────────┐        ┌──────────────────┐        ┌─────────────┐
│  Dashboard  │──write→│    PostgreSQL     │←─read──│    Album    │
│ (scraper/)  │        │   (bdo_album db)  │        │  (album/)   │
└──────┬──────┘        └──────────────────┘        └─────────────┘
       │
       ▼
┌─────────────┐
│ Cloudflare R2│  ← preset images
└─────────────┘
```

- **Frontend**: Svelte 5 (runes), TypeScript, SCSS, Vite
- **Backend**: Rust, Tauri 2, `sqlx` (PostgreSQL)
- **Scraping**: `playwright-rs` (headless Chromium) to get past Cloudflare's JS challenge on Garmoth
- **Storage**: PostgreSQL for metadata, Cloudflare R2 for preset images
- **Auto-update**: Tauri updater plugin, manifests published as GitHub Release assets

## Getting started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org/) 22+ and [pnpm](https://pnpm.io/)
- [Docker](https://www.docker.com/) (for local PostgreSQL)

### Infrastructure

```bash
docker compose up -d   # PostgreSQL on localhost:5432
```

### Environment variables

Each app reads its own `src-tauri/.env` (gitignored). See [scraper/src-tauri/.env.example](scraper/src-tauri/.env.example) and [album/src-tauri/.env.example](album/src-tauri/.env.example) for the required keys (database connection string, plus R2 credentials for the scraper).

### Run

```bash
# Dashboard / scraper
cd scraper
pnpm install
pnpm tauri dev

# Album viewer
cd album
pnpm install
pnpm tauri dev
```

## Building & releasing

Releases are built by [`.github/workflows/release.yml`](.github/workflows/release.yml). Pushing to the `releases` branch:

1. Creates a draft GitHub Release.
2. Builds both apps' Windows MSI installers in parallel jobs.
3. Publishes updater manifests (`scraper-latest.json`, `album-latest.json`) and un-drafts the release once both builds succeed.

Bump the `version` field in both `scraper/src-tauri/tauri.conf.json` and `album/src-tauri/tauri.conf.json` together before pushing to `releases` — they ship as one combined release.

## TODO

- [ ] **Figure out how to distribute `.env` config to end users.** Right now `.env` is bundled directly into each MSI as a Tauri bundle resource (`"resources": [".env"]` in `tauri.conf.json`), which means the database connection string — and, for the scraper, the R2 credentials — end up embedded in the installer shipped to users. This needs a real solution before this goes beyond personal/internal use: e.g. a first-run setup screen that stores config in the OS app-data dir, a remote config endpoint, or per-build secrets injection scoped to the intended user.
