export type ThemeChoice = 'light' | 'dark';
export type AccentChoice = 'amber' | 'violet' | 'coral' | 'teal';

const THEME_KEY = 'muse.settings.theme';
const ACCENT_KEY = 'muse.settings.accent';

const THEMES: readonly ThemeChoice[] = ['light', 'dark'];
const ACCENTS: readonly AccentChoice[] = ['amber', 'violet', 'coral', 'teal'];

// Falls back to the OS-reported scheme on first-ever launch (no stored
// pref yet); once the user picks in Settings, that choice persists across
// restarts via localStorage regardless of what the OS does afterwards.
function detectSystemTheme(): ThemeChoice {
  try {
    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  } catch {
    return 'dark';
  }
}

function readStored<T extends string>(key: string, valid: readonly T[], fallback: T): T {
  try {
    const v = localStorage.getItem(key);
    return v && (valid as readonly string[]).includes(v) ? (v as T) : fallback;
  } catch {
    return fallback;
  }
}

export const settings = $state({
  theme: readStored<ThemeChoice>(THEME_KEY, THEMES, detectSystemTheme()),
  accent: readStored<AccentChoice>(ACCENT_KEY, ACCENTS, 'amber'),
});

export function setTheme(theme: ThemeChoice) {
  settings.theme = theme;
  try { localStorage.setItem(THEME_KEY, theme); } catch { /* non-fatal */ }
}

export function setAccent(accent: AccentChoice) {
  settings.accent = accent;
  try { localStorage.setItem(ACCENT_KEY, accent); } catch { /* non-fatal */ }
}
