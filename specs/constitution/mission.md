# Mission

## What we're building
A two-app desktop suite for Black Desert Online "Beauty Album" character presets. The
Dashboard (`scraper/`) scrapes popular presets from Garmoth and writes metadata + images
(Cloudflare R2) to a shared PostgreSQL database. The Album (`album/`) reads that same
database so the user can browse presets, curate favorites/wishlist/discards, and apply
them onto their own in-game characters via the face grid.

## Who it's for
Tito, the sole developer and sole user, for personal use browsing and curating presets
for his own BDO characters. Not a public product today — the README's own TODO on
distributing `.env` config to "end users" is explicitly deferred until (if ever) this
goes beyond personal/internal use.

## Tone / product feel
A fast, dense, keyboard-and-mouse desktop tool — not a consumer app. Dark-themed,
information-dense grids, hover-revealed actions, live-updating counts. Utilitarian first;
polish is welcome but never at the cost of speed of browsing/curating a large preset
catalog.

## What success looks like
The Album app makes it fast to find, favorite, and export the right preset among
hundreds/thousands scraped by the Dashboard, with the two apps staying in sync live
(PG LISTEN/NOTIFY) while both are open.

## Non-goals (optional)
- Not building for multiple concurrent end users or a hosted/multi-tenant service.
- Not solving `.env`/credential distribution to other people right now (tracked in the
  roadmap backlog, not blocking).
- No shared Rust crate/workspace between `scraper/` and `album/` — they stay fully
  independent Tauri projects on purpose.
