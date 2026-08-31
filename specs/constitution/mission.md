# Mission

## What we're building
A two-app desktop suite for Black Desert Online "Beauty Album" character presets. The
Dashboard (`scraper/`) scrapes popular presets from Garmoth and writes metadata + images
(Cloudflare R2) to a shared PostgreSQL database. The Album (`album/`) reads that same
database so the user can browse presets, curate favorites/wishlist/discards, and apply
them onto their own in-game characters via the face grid.

## Who it's for
Today: Tito, the sole developer and sole user, for personal use browsing and curating
presets for his own BDO characters. Not a public product today — the README's own TODO
on distributing `.env` config to "end users" is explicitly deferred until (if ever) this
goes beyond personal/internal use.

Future intent (confirmed 2026-08-30): only the **Album** app (`album/`) is a candidate
for eventually going public ("liberar" — a public release for other BDO players). The
Dashboard/scraper stays internal/personal — it's the data-collection tool, not something
meant for other people to run. This means Phase 1's UI redesign should aim at a
commercial-grade visual quality for `album/` specifically, not just a personal-tool
polish pass; it also means the `.env`/credential-distribution TODO and any
multi-user/hosting questions become real (not hypothetical) once a public release is
actually scheduled — they're still not scheduled today, but no longer purely
hypothetical either.

## Tone / product feel
Today, still a fast, dense, keyboard-and-mouse desktop tool — not a consumer app.
Dark-themed, information-dense grids, hover-revealed actions, live-updating counts.
Utilitarian first; polish is welcome but never at the cost of speed of browsing/curating
a large preset catalog.

For the Album's Phase 1 redesign specifically, aim past "personal tool" polish toward a
**modern, commercial-product** feel (the kind of visual quality a small paid app or a
polished SaaS would ship) — since it's the piece that may eventually go public. Light and
dark themes both need to hold up to that bar, not just dark.

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
