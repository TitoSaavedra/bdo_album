# Exploration artifact

**URL:** https://claude.ai/code/artifact/2a59bae2-5b74-4041-8776-8ab22fdef8b8

Private Claude Artifact (owned by the human — visit `claude.ai/code/artifacts` or run
`/artifacts` in Claude Code if the link above ever 404s and it needs re-finding). Static
HTML, Tailwind via CDN, no backend — nothing here is wired to real data. It exists purely
to compare layout directions before writing any Svelte.

## What's in it
- **5 layout tabs** (top nav): Rail, Command Bar *(chosen — opens by default)*, Atelier,
  Wall, Console. Same dummy preset/creator/class data reused across all 5 for a fair
  comparison.
- Each layout has a **Beauty / Face Grid module switch** built into its own chrome (the
  switch's position differs per layout on purpose — that placement is part of what's
  being compared, not just the layout shape itself).
- The whole thing floats as a fake OS window: **own titlebar** (no browser/OS chrome),
  ambient desktop backdrop behind it.
- **Command Bar only:** click a preset card — it morphs into a full detail view via the
  View Transitions API instead of opening a modal. Gear icon in the header opens an
  in-app **Settings** popover (Light/Dark + accent-color swatches).
- Top-right of the *page* (outside the fake window): a page-chrome dark/light toggle
  (unrelated to the app's own theme) and a 4-swatch palette row (Amber / Violet / Coral /
  Champagne Teal) that repaints the whole mockup live.

## Resolved: palette swatches now accent-only
Fixed 2026-08-30 (`tasks.md`, Group 2). The swatches previously re-themed the **entire
neutral scale** (background, surface, card, border, text) per swatch, not just the
accent. They now only override `--m-accent` / `--m-accent-ink` — background/surface/
border/text always come from the Light/Dark toggle alone, never from the accent choice.
Contrast was re-checked for the 3 non-Amber accents; see Group 2's notes in `tasks.md`
for the one borderline pattern found (accent-as-plain-text on a neutral background,
already borderline for Amber too — not a regression from this fix).

## Also applied: app renamed to Muse
The artifact's titlebar mark and `APP_NAME` now read **"Muse"** (final name, decided
2026-08-30 — see `tasks.md`, Group 1), replacing the working name "Facet". The
faceted-gem logo mark is unchanged.
