# Tech Stack

## Languages & runtimes
- TypeScript (frontend), Rust stable (backend) — both apps.
- Node.js 22+ / pnpm for the frontend toolchain; Python 3 only for the standalone DB
  migration runner (`scripts/migrate.py`).

## Frameworks
- **Tauri 2** — desktop shell for both apps (`scraper/src-tauri`, `album/src-tauri`).
- **Svelte 5** (runes: `$state`, `$derived`, `$effect`, `$props` — no Svelte 4 stores) +
  Vite. Component-local SCSS only (`.scss` file per component, never inline style blocks).
- Rust side: commands stay thin → service → repository. No business logic in
  `#[tauri::command]` handlers.

## Data & storage
- **PostgreSQL** (`bdo_album` db) is the single shared store both apps read/write —
  `scraper` writes, `album` reads (+ writes its own `album_*` preference tables:
  favorites, wanted, discards).
- **Deployment topology** — two Postgres targets, selected purely by *which* `.env`
  a binary loads (same filename `.env` in both cases, different location — no
  dev/prod switch in code; each app's `lib.rs` already tries the exe-adjacent
  `.env` first via `dotenvy::from_path`, then falls back to `dotenvy::dotenv()`
  searching up from the CWD):
  - **Dev** — local via `docker-compose.yml`; `src-tauri/.env` in each project.
    Day-to-day development stays here, unchanged.
  - **Prod** — a single self-hosted Postgres instance (Docker) on Tito's home
    server, shared across *all* his personal projects, reachable over LAN only
    (no VPN, no port forwarded to the internet). Each project gets its own
    database inside that one instance — this project's is `bdo_album` — plus its
    own least-privilege Postgres role (never the shared superuser). The prod
    `.env` lives next to the installed app's executable and points at the
    server's LAN address.
- **Cloudflare R2** for preset images and PAB files, via a custom `R2Client` (S3-compatible).
- `sqlx` (Rust) for all Postgres access — hand-written SQL, no query builder/ORM.

## Key libraries & external services
- **Garmoth.com** — source of preset data; scraped through `playwright-rs` (headless
  Chromium) specifically to pass Cloudflare's JS challenge; API calls routed through the
  browser's TLS fingerprint.
- **Tauri updater plugin** — auto-update, manifests (`*-latest.json`) published as GitHub
  Release assets.
- **svelte-i18n** — UI copy lives in `en.json`/`es.json`; `en` is the actually-maintained
  locale (fallback), `es` stays sparse by convention.
- PG `LISTEN`/`NOTIFY` — live cross-app sync (scraper's uploads/downloads reflected live
  in the album UI without polling).

## Notable tradeoffs
- **No shared crate/workspace between the two Tauri apps** — chosen for independence
  (either app can be built/shipped without the other); the cost is some duplicated
  patterns (e.g. both read `DATABASE_URL` from their own `.env`).
- **Hand-written SQL over an ORM** — direct control over the exact joins needed for the
  album's filters (has_pab, is_wanted, is_discarded, live counts), at the cost of more
  boilerplate per query.
- **Custom Python migration runner instead of `sqlx-cli`** — one standalone script
  (`scripts/migrate.py`) tracks applied migrations in `_sqlx_migrations` compatibly with
  sqlx's own scheme, but isn't run automatically on app start; must be run by hand after
  pulling schema changes.
- **`scraper/src-tauri/migrations/` is the schema of record for both apps** — even
  album-only preference tables (e.g. `album_class_favorites`, `album_user_prefs`) are
  migrated from the scraper project, since it's the one that "writes."
- **One shared Postgres instance for prod, across all personal projects** — cheaper to
  run and back up than one instance per project, at the cost of a single point of
  failure spanning every project. Mitigated by giving each project its own database
  and its own role inside that instance, so a bug or leak in one app can't reach
  another's data.

## Conventions
- Semantic versioning, one shared version number bumped together in both apps'
  `tauri.conf.json` before every push to the `releases` branch (PATCH = fixes/infra,
  MINOR = new features/UI reworks, MAJOR = unused pre-1.0).
- No automatic build/test in normal *dev* flow (per project convention — day-to-day
  verification is manual, `pnpm tauri dev` + eyeballing the running app).
- CI (`.github/workflows/release.yml`) builds both apps' Windows MSI installers in
  parallel and publishes updater manifests only once both succeed.
- CI (`.github/workflows/validate.yml`) runs on every push to `master` and on PRs:
  `svelte-check` + `cargo check` for each app (Windows runners, since both apps have
  `#[cfg(windows)]` code paths that a Linux check would silently skip). `album`'s
  Rust check runs with `SQLX_OFFLINE=true` against the committed
  `album/src-tauri/.sqlx` cache — `scraper` doesn't use sqlx's compile-time
  `query!`/`query_as!` macros, so it needs no DB/offline cache to check.
