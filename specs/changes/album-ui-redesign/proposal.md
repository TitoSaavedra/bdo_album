# Change: album-ui-redesign

## Why
The Album app's current UI (dark-only, hand-rolled SCSS tokens, small monospace pills)
reads as dated — most recently visible in the filter toolbar (search/sort/region/upload
pills), which still feels cramped and inconsistent even after a first restyle pass. Phase
1 of the roadmap (`specs/constitution/roadmap.md`) calls for a real visual redesign of
the Album shell with light/dark theme support, not another incremental CSS patch.

## What changes
- A new design direction for the Album's main shell (sidebar/class-list/toolbar + preset
  grid) — evaluated first as static layout comparisons (no wiring), reviewed by the human
  before any component code changes.
- Adds a light theme alongside the current dark one (today: dark-only, tokens in
  `album/src/styles/_tokens.scss`, no `[data-theme]` switch).
- Likely swaps the hand-rolled SCSS token system for a framework-backed one — candidates
  to compare during the spike: Tailwind v4 + a headless Svelte component layer (Bits UI /
  Melt UI), a pre-built kit (Skeleton v3, shadcn-svelte port, DaisyUI), or a disciplined
  hand-rolled design-token system kept as-is but restructured for light/dark. Not decided
  yet — the layout spike and this decision are sequenced before spec-delta/tasks are
  filled in.

## Affected capabilities
- `specs/capabilities/album-browsing` — its UI surface (toolbar, grid, cards, detail
  modal) is what gets restyled; its *behavior* (filters, favorites, wishlist, tiers) is
  explicitly not changing here.

## Out of scope
- No behavior/data changes — this is visual only (see `album-browsing`'s existing spec
  for the behavior that must keep working unchanged).
- Not touching `scraper/`'s Dashboard UI in this change.
- Not deciding the framework yet — that follows once a layout direction is picked from
  the visual comparison.
