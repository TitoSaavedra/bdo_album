# Spec delta — album-ui-redesign

The edit this change makes to the living capability spec(s). On archive, these merge into
`specs/capabilities/<capability>/spec.md`.

## Capability: album-browsing   (modified)

### Added / changed requirements
- The browsing UI is a single-window **Command Bar** shell: no persistent sidebar. A
  sticky header holds search, the class-pills strip, favorite-creator chips, and a
  Filters affordance; the preset grid fills the rest of the window. This replaces the
  old sidebar + toolbar arrangement — the filter/sort/favorite/wishlist/tier *behavior*
  itself (see existing requirements above) does not change.
- Opening a preset no longer opens a centered modal. Clicking a card grows its thumbnail
  in place into a full detail view, in the same content area, via the View Transitions
  API (`document.startViewTransition()`), closed via a "‹ Back to grid" affordance. On
  browsers/webviews without View Transitions support, or with `prefers-reduced-motion`
  set, the detail view swaps in instantly with no animation — same end state, no morph.
- The window draws its own titlebar (`decorations: false`) instead of the OS one: app
  mark + name ("Muse"), minimize/maximize/close, wired to
  `getCurrentWindow().minimize()/toggleMaximize()/close()`. Shared with the Face Grid
  module (see below).
- A Beauty ↔ Face Grid module switcher lives inside this shared shell (replacing/
  absorbing the old `<nav class="tab-nav">` in `App.svelte`).
- Adds a Settings surface (gear icon in the Command Bar header): a Light/Dark theme
  toggle, and an accent-color picker. The accent picker changes **only** the accent
  token(s) (buttons, focus/active states, modal/dialog accent touches) — backgrounds,
  surfaces, borders, and text always come from the Light/Dark toggle alone, never from
  the accent choice. Default on first launch: **Amber** accent + the OS-detected theme.
  Both the theme and accent choice persist across full app restarts (not just the
  running session).
- Adds a light theme alongside the existing dark-only theme, via a `[data-theme]`
  pattern; both themes must hold up to a "modern commercial product" visual bar (see
  `specs/constitution/mission.md`), not just a personal-tool polish pass.
- The app is renamed/rebranded to **"Muse"**, applied to the titlebar and Settings
  surface; the faceted-gem mark is the app's logo.

### Added / changed scenarios
- **Given** the user clicks a preset card in the grid **when** the browser/webview
  supports View Transitions and reduced-motion is not requested **then** the card's
  thumbnail grows in place into the detail view's hero image while the rest of the
  detail content cross-fades in, and clicking "‹ Back to grid" reverses the transition
  back to the grid at the same scroll position.
- **Given** the browser/webview lacks View Transitions support, or the user has
  `prefers-reduced-motion` set **when** a preset card is clicked **then** the detail view
  replaces the grid instantly with no morph/cross-fade animation.
- **Given** the user opens Settings and picks a non-default accent color **when** they
  look at the grid/detail view **then** only accent-colored elements (buttons, active
  states, focus rings, tier-badge accents that use the accent token) change color —
  backgrounds, card surfaces, borders, and body text are unchanged.
- **Given** the user sets a theme and/or accent color in Settings **when** they close and
  reopen the app **then** the same theme and accent are still active (no reset to
  defaults).
- **Given** the user is in the Face Grid module **when** they use the module switcher
  **then** they land back in the Beauty module's last-viewed state (selected class or
  favorite-creator filter, scroll position not required to be preserved).

### Removed
- `PresetDetail.svelte`'s centered-modal-with-backdrop pattern (replaced by the
  grow-in-place morph view described above).
- The sidebar + toolbar arrangement in `ClassList.svelte` (replaced by the Command Bar
  header).

## Capability: face-grid   (modified)

### Added / changed requirements
- `FaceGridView.svelte` shares the same titlebar and Beauty ↔ Face Grid module switcher
  as the Beauty module (see `album-browsing` above) — this is purely shell/chrome.
- `CharacterGrid.svelte`'s internal layout (the fixed 7-column grid) is explicitly **not**
  touched by this change — it mirrors how Black Desert itself lays out `FaceTexture`
  slots on disk and is not a UI choice this redesign is free to make.

### Added / changed scenarios
- **Given** the user is in the Face Grid module **when** they use the shared titlebar's
  minimize/maximize/close controls **then** the window responds the same way it would
  from the Beauty module (shared titlebar component, not a per-module reimplementation).

### Removed
- (none — Face Grid's own behavior and internal layout are unchanged)
