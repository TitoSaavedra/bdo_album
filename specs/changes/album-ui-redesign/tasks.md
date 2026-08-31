# Tasks — album-ui-redesign

Small, ordered groups. Check off as they merge. Do groups one at a time where mistakes
compound (security, migrations, data).

## Group 1 — Close the open decisions (before writing any real component code)
- [x] Frontend approach: **Tailwind v4 + Bits UI** (decided 2026-08-31). Headless
      primitives keep full visual control instead of inheriting a kit's look; same base
      shadcn-svelte builds on, so its recipes stay usable later if wanted. The
      exploration artifact already previews it (Tailwind loaded live there).
- [ ] Pick the final app name: "Facet" (working choice, applied in the artifact) vs.
      "Visage" vs. "Muse" vs. something else.
- [ ] Pick the final accent color (or keep it user-configurable via Settings — see
      Group 2 — in which case pick the *default*).
- [ ] Fill this change's `spec-delta.md` once the above are locked — it should describe
      the concrete edit to `specs/capabilities/album-browsing`'s spec (new UI surface,
      Settings capability, titlebar) and to `specs/capabilities/face-grid`'s spec
      (shared shell only, layout untouched).

## Group 2 — Fix the palette system in the exploration artifact
- [ ] In the artifact (`artifact-link.md` has the URL), change the 4 swatches so they
      only override `--m-accent` / `--m-accent-ink` (and anything that visually derives
      from the accent — button fills, focus rings, modal/dialog accent touches). Stop
      varying `--m-bg` / `--m-surface` / `--m-card` / `--m-elevated` / `--m-border` /
      `--m-border-soft` / `--m-text` / `--m-sub` / `--m-mute` per swatch — those come
      only from the Light/Dark toggle, never from the accent choice.
- [ ] Re-validate the 3 non-amber accents still hold acceptable contrast against the
      *unchanged* dark and light neutral scales once they're no longer paired with their
      own custom backgrounds.

## Group 3 — Design-system foundation in `album/`
- [ ] Stand up the chosen frontend approach (Group 1) in `album/` — Tailwind config (or
      equivalent), token definitions for dark + light (`[data-theme]` pattern, matching
      the artifact's `--m-*` naming or a cleaned-up equivalent).
- [ ] `decorations: false` in `album/src-tauri/tauri.conf.json` (and mirror in
      `scraper/src-tauri/tauri.conf.json` only if the same titlebar treatment is wanted
      there — not decided, Dashboard UI is explicitly out of scope for *this* change).
- [ ] Build the real titlebar component (app mark, name, minimize/maximize/close via
      `getCurrentWindow()`), shared across both modules.
- [ ] Build the Settings component (Light/Dark + accent picker per the corrected
      Group 2 behavior), persisted (e.g. to a local pref / `localStorage` equivalent —
      decide storage during implementation).

## Group 4 — Rebuild the Beauty module as Command Bar
- [ ] Replace `ClassList.svelte`'s sidebar+toolbar with the Command Bar header (search +
      sort + class pills + favorite-creator chips as an inline strip, per the artifact).
- [ ] Rebuild `PresetCard.svelte` to match the new visual language (rounded-xl, tier
      glow via `color-mix()`, hover-reveal creator row) — keep all existing behavior
      (favorite-creator heart, wishlist heart, discard, tier priority) unchanged, this is
      a visual pass per `album-browsing`'s spec.
- [ ] Replace `PresetDetail.svelte`'s modal with the View-Transitions morph-expand
      pattern from the artifact, including the `prefers-reduced-motion` / unsupported-
      browser fallback.
- [ ] Wire the module switcher (Beauty ↔ Face Grid) into the new shared shell, replacing
      or absorbing the existing `<nav class="tab-nav">` in `App.svelte`.

## Group 5 — Carry the shell to Face Grid
- [ ] `FaceGridView.svelte` picks up the same titlebar + module switcher.
- [ ] **Do not** touch `CharacterGrid.svelte`'s internal layout (fixed 7-column grid) —
      confirmed constraint, it mirrors how BDO itself lays out `FaceTexture` slots.

## Group 6 — Validate
- [ ] Run through `references/validate.md`'s checklist once Groups 1–5 are done; confirm
      `album-browsing`'s existing behavior scenarios (filters, favorites, wishlist, live
      updates) still hold with the new UI before archiving this change.
