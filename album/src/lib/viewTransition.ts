// Shared by any grid<->detail navigation (card click, back button, Escape,
// mouse back/forward) so they all animate consistently. Falls back to an
// instant state change when the API is unsupported or reduced-motion is on.
export function withViewTransition(run: () => void) {
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (!document.startViewTransition || reduceMotion) {
    run();
    return;
  }
  document.startViewTransition(run);
}
