# Roadmap

Small, sequenced phases. Each phase is a chunk a human can review in one sitting.
Check items off as their change is archived. Keep phase 1 intentionally tiny.

## Phase 1 — Album visual redesign (modern UI, light/dark)
- [ ] Explore layout directions for the Album shell (sidebar + grid) — visual-only
      comparison, no wiring, reviewed as an artifact before any code changes.
- [ ] Pick a design direction and a frontend framework/approach for it (Tailwind-based
      component kit vs. hand-rolled tokens+components — see backlog research note).
- [ ] Land a `specs/capabilities/album-ui/spec.md` describing the new design system
      (tokens, light/dark strategy, component inventory).
- [ ] Implement across the Album's components (ClassList/toolbar, PresetCard, PresetGrid,
      PresetDetail) behind the existing feature set — no behavior change, visuals only.
- [ ] Ship as a MINOR version bump.

## Phase 2 — Playwright first-run download UX
- [ ] Surface real progress in the Dashboard UI for `browser.rs::bootstrap_driver`
      instead of a silent blocking step (see README TODO).

## Backlog / research (not yet scheduled)
- **Frontend framework choice for Phase 1** — candidates to evaluate: Tailwind v4 +
  Skeleton/shadcn-svelte/Bits UI/Melt UI style headless components, vs. DaisyUI, vs.
  staying hand-rolled SCSS with a proper design-token system. Needs a quick spike before
  Phase 1's design-direction step locks it in.
- **`.env`/credential distribution to end users** — deferred per the mission's current
  personal-use-only scope; revisit only if the project is ever shared beyond Tito.
