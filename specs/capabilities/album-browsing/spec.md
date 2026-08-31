# Capability: album-browsing

> LIVING truth — keep this current. Changes arrive as `specs/changes/<name>/spec-delta.md`
> and are merged here when the change is archived. This file should always describe what the
> system does *now*, not how it got here.

## Purpose
Lets the user browse presets scraped into the shared Postgres DB, filter/sort them, and
curate personal state (class favorites, creator favorites, wishlist, discards) — all from
the `album/` app, which only reads scraped data but owns its own `album_*` preference
tables.

## Requirements
- Browsing is scoped to one selected class at a time, except the favorite-creator filter,
  which shows one creator's presets across every class.
- Filters compose: search (title/nickname/character), region, upload-date window, sort
  (downloads/views/likes) — except the creator filter, which intentionally ignores
  region/upload-date (a favorited creator's whole catalog should always show up).
- A preset's visual state in the grid is a strict priority order: has a PAB file
  (downloaded) > wishlisted > from a favorited creator > plain. Only the highest
  applicable tier is shown per card.
- Favoriting (class or creator) and wishlisting are optimistic in the UI (state updates
  immediately) and persisted async via a Tauri command; a failed persist is silently
  non-fatal (the in-memory state is source of truth for the session).
- Discarding a preset hides it for the rest of the session and persists
  `album_user_prefs.is_discarded`.
- Live uploads/downloads from the scraper (via PG LISTEN/NOTIFY) patch the currently
  open class's grid in place without a full reload; this does not apply while browsing
  by favorite creator (cross-class, no single class to patch).

## Behavior / scenarios
- **Given** a class is selected **when** the user types in search or changes
  region/sort/upload-date **then** the grid reloads from `get_presets` with those filters
  and the sidebar's per-class counts update.
- **Given** the user favorites a creator (heart on a card, in the detail modal, or the
  sidebar chip cloud) **when** that creator has other presets in the loaded grid **then**
  those cards immediately show the red "favorite creator" outline, unless they're already
  downloaded or wishlisted.
- **Given** the user selects a favorited creator from the sidebar **when** the filter is
  active **then** the grid shows every non-discarded preset by that creator across all
  classes, tagged with each preset's class, ignoring region/upload-date.
- **Given** the wishlist has queued presets **when** the user opens "auto-download" **then**
  those presets can be sent to the scraper's auto-download queue
  (`album_user_prefs.auto_download_requested_at`), picked up by `scraper`'s worker.

## Notes
- Backing tables: `scraper_presets`/`scraper_classes` (read-only from album's side),
  `album_user_prefs` (is_wanted/is_discarded/auto_download_* per preset),
  `album_class_favorites`, `album_creator_favorites` — all migrated from
  `scraper/src-tauri/migrations/` even though only `album` writes to the `album_*` ones.
  See `specs/constitution/tech-stack.md` for why.
- The sidebar filter toolbar (search, sort segmented control, filters popover, active
  chips) and the favorite-creator chip cloud are the pieces Phase 1's visual redesign
  will most directly touch — see `specs/constitution/roadmap.md`.
