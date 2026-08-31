# Change: album-ui-redesign

## Why
The Album app's current UI (dark-only, hand-rolled SCSS tokens, small monospace pills)
reads as dated — most recently visible in the filter toolbar (search/sort/region/upload
pills), which still feels cramped and inconsistent even after a first restyle pass. Phase
1 of the roadmap (`specs/constitution/roadmap.md`) calls for a real visual redesign of
the Album shell with light/dark theme support, not another incremental CSS patch.

## What changes
- **Layout direction is picked: "Command Bar."** Five static directions were compared in
  an HTML artifact (Rail, Command Bar, Atelier, Wall, Console — see
  `artifact-link.md` in this folder for the link and what each looked like).
  Command Bar won: no persistent sidebar, top sticky header (search + class pills), full-
  width grid.
- Adds a light theme alongside the current dark one (today: dark-only, tokens in
  `album/src/styles/_tokens.scss`, no `[data-theme]` switch).
- **The window draws its own titlebar** — `decorations: false` in both apps'
  `tauri.conf.json`, replaced with an in-app titlebar (app mark + name, minimize/
  maximize/close) driving `getCurrentWindow().minimize()/toggleMaximize()/close()`.
- **Preset detail becomes a grow-in-place transition, not a modal.** Clicking a card
  morphs its thumbnail into a full detail view in the same content area (View
  Transitions API — `document.startViewTransition()` — with an instant-swap fallback for
  unsupported browsers / `prefers-reduced-motion`), closed via a "‹ Back to grid"
  affordance rather than a dialog close button. Replaces `PresetDetail.svelte`'s current
  centered-modal-with-backdrop pattern.
- **Both real modules (Beauty + Face Grid) live inside every layout**, switched via an
  in-shell control (not just the existing top `<nav class="tab-nav">`). Face Grid's own
  internal arrangement (the 7-column character grid) must **not** be redesigned — it's
  driven by how Black Desert itself lays out `FaceTexture` slots on disk, not a UI choice.
- **Adds an in-app Settings surface** (gear icon in the Command Bar header) with:
  a Light/Dark toggle, and an **accent-color picker**. Correction from the first pass:
  the picker must change **only the accent** (buttons, focus/active states, modal/dialog
  accents, brand touches) — backgrounds/surfaces/borders/text stay the single dark-or-
  light neutral scale regardless of accent choice. (The exploration artifact currently
  varies the *whole* neutral scale per swatch — see Group 2 in `tasks.md`, that's a bug
  to fix in the artifact before this becomes real UI, not a spec for the real thing.)
- App gets renamed and rebranded: working name **"Facet"** (a faceted-gem mark — ties
  "face," what the app curates, to the gem-cutting motif already in BDO's own crafting/
  jewelry) applied in the exploration artifact's titlebar/logo. Alternatives considered:
  *Visage* (more literal), *Muse* (more generic/SaaS). **Not finalized** — needs a
  decision (see `tasks.md`, Group 1).
- Framework for the real implementation: still **not decided**. Leading candidate from a
  quick web check: Tailwind v4 + Bits UI (headless Svelte primitives) — gives full visual
  control rather than inheriting a kit's look, which matters since the goal (per the
  mission's public-release note) is a distinctive commercial feel, not a generic template.

## Affected capabilities
- `specs/capabilities/album-browsing` — its UI surface (toolbar → Command Bar header,
  grid, cards, detail modal → morph transition) is what gets rebuilt; its *behavior*
  (filters, favorites, wishlist, tiers) is explicitly not changing here.
- `specs/capabilities/face-grid` — gains a shared shell/titlebar and the module switcher,
  but its own internal layout (the 7-col grid) is explicitly *not* touched (see above).

## Out of scope
- No behavior/data changes — this is visual only (see `album-browsing`'s existing spec
  for the behavior that must keep working unchanged).
- Not touching `scraper/`'s Dashboard UI in this change.
- Not redesigning Face Grid's internal character-grid arrangement (constrained by how
  BDO itself stores face slots — see `specs/capabilities/face-grid/spec.md`).
- Not finalizing framework, app name, or accent color yet — all three are open decisions
  a human needs to make before implementation starts (`tasks.md`, Group 1).
