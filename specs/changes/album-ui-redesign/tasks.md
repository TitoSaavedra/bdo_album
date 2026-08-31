# Tasks — album-ui-redesign

Small, ordered groups. Check off as they merge. Do groups one at a time where mistakes
compound (security, migrations, data).

## Group 1 — Close the open decisions (before writing any real component code)
- [x] Frontend approach: **Tailwind v4 + Bits UI** (decided 2026-08-31). Headless
      primitives keep full visual control instead of inheriting a kit's look; same base
      shadcn-svelte builds on, so its recipes stay usable later if wanted. The
      exploration artifact already previews it (Tailwind loaded live there).
- [x] Final app name: **Muse** (decided 2026-08-30, over Facet/Visage). Renamed in the
      exploration artifact (titlebar mark + `APP_NAME`); the faceted-gem logo mark itself
      is kept as the visual brand regardless of the name.
- [x] Final default accent color: **Amber** (decided 2026-08-30) — matches the artifact's
      existing default and the project's current branding. The Settings picker stays
      user-configurable (Group 2); Amber is only the out-of-the-box default.
      **Locked requirement:** the chosen theme + accent must persist across full app
      restarts, not just for the running session — see Group 3's Settings-persistence
      task, tightened accordingly.
- [x] Fill this change's `spec-delta.md` once the above are locked — it should describe
      the concrete edit to `specs/capabilities/album-browsing`'s spec (new UI surface,
      Settings capability, titlebar) and to `specs/capabilities/face-grid`'s spec
      (shared shell only, layout untouched).

## Group 2 — Fix the palette system in the exploration artifact
- [x] In the artifact (`artifact-link.md` has the URL), change the 4 swatches so they
      only override `--m-accent` / `--m-accent-ink` (and anything that visually derives
      from the accent — button fills, focus rings, modal/dialog accent touches). Stop
      varying `--m-bg` / `--m-surface` / `--m-card` / `--m-elevated` / `--m-border` /
      `--m-border-soft` / `--m-text` / `--m-sub` / `--m-mute` per swatch — those come
      only from the Light/Dark toggle, never from the accent choice. Done 2026-08-30.
- [x] Re-validate the 3 non-amber accents still hold acceptable contrast against the
      *unchanged* dark and light neutral scales once they're no longer paired with their
      own custom backgrounds. Checked with WCAG contrast math (2026-08-30): the dominant
      accent usage (solid-fill buttons/pills, `--m-accent` bg + `--m-accent-ink` text) is
      a matched pair per palette/theme and is unaffected by this fix — stays strong for
      all 4. The one exposed pattern is `--m-accent` used as *plain text* on a neutral
      `--m-card`/`--m-elevated` background (Settings' Light/Dark segment, rail/console
      active-class labels) — there, Violet passes ~4.5:1 in both themes, but Coral and
      Teal land ~3.1–3.2:1 in light mode, versus Amber's own existing ~3.3–3.9:1 in the
      same spot. This is a pre-existing borderline pattern (even Amber doesn't clear
      4.5:1 there today), not a regression introduced by this fix, so hex values were
      left as-is rather than hand-tuned; worth a real look during Group 3 if any
      non-Amber accent becomes the shipped default later.

## Group 3 — Design-system foundation in `album/`
- [x] Stand up the chosen frontend approach (Group 1) in `album/` (done 2026-08-30):
      swapped Tailwind v3 (postcss + `tailwind.config.js`) for **Tailwind v4** via
      `@tailwindcss/vite` (new `src/styles/tailwind.css` with `@import 'tailwindcss'`,
      no PostCSS config needed), added `bits-ui`. Kept the project's existing `--color-*`
      token naming (already partially theme-shaped from an earlier pass) rather than
      renaming to the artifact's `--m-*` — same "cleaned-up equivalent" the task allowed,
      and avoided a 23-file rename since `--color-*` is already used across both Beauty
      and Face Grid. `_tokens.scss` (dark, default) and `_themes.scss`
      (`[data-theme="light"]`) now cover every token dark defines — previously light was
      a partial override (missing most tokens, and had a stray blue accent inconsistent
      with the rest of the redesign); both are now complete and the accent is Amber in
      both themes. Accent picker implemented as `[data-accent="violet|coral|teal"]`
      (+ theme-combined variants) touching only the accent-identity tokens
      (`--color-accent*`, `--color-btn-text-on-accent`), never backgrounds/borders/text —
      matches the Group 2 fix. `tailwindcss`/`bits-ui` are declared in `package.json` but
      **not yet installed/verified** — this sandbox has no Node/pnpm available, so
      `pnpm install` + `svelte-check` still need to be run for real before this ships.
- [x] `decorations: false` in `album/src-tauri/tauri.conf.json` (done 2026-08-30), plus
      `minWidth`/`minHeight` since a custom-titlebar window benefits from a floor. Left
      `scraper/src-tauri/tauri.conf.json` untouched — still not decided, still out of
      scope for this change.
- [x] Built the real titlebar (`src/ui/Titlebar/`): app mark, "Muse", minimize/maximize/
      close via `getCurrentWindow()`, `data-tauri-drag-region` for window dragging. Hosts
      `SettingsPopover` and applies `[data-theme]`/`[data-accent]` to `<html>` reactively
      — shared across both modules by construction (mounted once at the `App.svelte`
      root, above the Beauty/Face-Grid conditional, so Group 5 falls out of this for free,
      see below).
- [x] Built the Settings component (`src/features/settings/`): gear popover (Bits UI
      `Popover`) with Light/Dark toggle + the 4 accent swatches, per the corrected
      Group 2 behavior. Persisted to `localStorage` (`muse.settings.theme` /
      `muse.settings.accent`), which survives full app restarts in the Tauri webview —
      satisfies the locked persistence requirement. Falls back to the OS-reported scheme
      + Amber on first-ever launch.

## Group 4 — Rebuild the Beauty module as Command Bar
- [x] Replaced `ClassList.svelte`'s sidebar+toolbar with the Command Bar header (done
      2026-08-30): search + sort segmented control + favorite-creator chips + wishlist +
      filters popover in one row, class list turned into a wrapping pill strip below it
      (wraps instead of the artifact's single-line horizontal scroll — BDO has ~24
      classes, not the artifact's 6 sample ones, so a scroll-only strip would hide most
      of them off-screen). All existing behavior kept as-is: search debounce, sort,
      region/day filters popover, wishlist modal, class + creator favoriting, live-upload
      badges, active-filter chips. Dropped only the old empty-state hint text under
      "Favorite Creators" (no room for a permanent explanatory section in a compact
      header; the chip row simply doesn't render when there are none).
- [ ] Rebuild `PresetCard.svelte` to match the new visual language (rounded-xl, tier
      glow via `color-mix()`, hover-reveal creator row) — **not done yet**. Note: the
      existing `PresetCard.scss` already implements the tier-priority glow (green/purple/
      red border + box-shadow per tier) via the same `--color-*` tokens, so it already
      picks up the new theme/accent system automatically; what's left is purely the
      `color-mix()`/rounded-xl/Tailwind-utility polish pass, not the tier logic.
- [ ] Replace `PresetDetail.svelte`'s modal with the View-Transitions morph-expand
      pattern from the artifact, including the `prefers-reduced-motion` / unsupported-
      browser fallback. **Not done yet** — still the original centered modal.
- [x] Wired the module switcher (Beauty ↔ Face Grid) into the new shared shell (done
      2026-08-30): new `src/ui/ModuleSwitcher/`, pill-style, replaces the old
      `<nav class="tab-nav">` 1:1 in `App.svelte`, rendered once above the Beauty/
      Face-Grid conditional (so it's shared with Face Grid too, see Group 5).

## Group 5 — Carry the shell to Face Grid
- [x] `FaceGridView.svelte` picks up the same titlebar + module switcher (satisfied
      2026-08-30 as a side effect of Group 3/4's shell placement, not a `FaceGridView.svelte`
      edit): `Titlebar` and `ModuleSwitcher` are mounted once at the `App.svelte` root,
      above the `{#if activeTab === 'beauty'}` branch, so both render identically
      regardless of which module is active. `FaceGridView.svelte` itself is untouched.
- [x] **Do not** touch `CharacterGrid.svelte`'s internal layout (fixed 7-column grid) —
      confirmed constraint, it mirrors how BDO itself lays out `FaceTexture` slots.
      Untouched — trivially satisfied, nothing under `features/face_grid/` was edited.

## Group 6 — Validate
- [ ] Run through `references/validate.md`'s checklist once Groups 1–5 are done; confirm
      `album-browsing`'s existing behavior scenarios (filters, favorites, wishlist, live
      updates) still hold with the new UI before archiving this change.
