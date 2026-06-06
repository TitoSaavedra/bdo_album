import type { ClassEntry, PresetEntry } from '../../../lib/album';
import { getClasses } from '../../../lib/album';

export const beauty = $state({
  // DB
  dbReady:  false,
  dbError:  null as string | null,

  // Classes
  classes:        [] as ClassEntry[],
  classesLoading: false,

  // Selected class
  selectedClass: null as string | null,

  // Preset detail modal
  presetDetail: null as (PresetEntry & { class_display: string }) | null,

  // Class favorites
  classFavorites: new Set<string>(),

  // Wanted presets
  wantedPresets: new Set<string>(),
});

// ── DB ────────────────────────────────────────────────────────

export function setDbReady(ok: boolean, error: string | null) {
  beauty.dbReady = ok;
  beauty.dbError = error;
  if (ok) loadClasses();
}

// ── Classes ───────────────────────────────────────────────────

export async function loadClasses() {
  beauty.classesLoading = true;
  try {
    beauty.classes = await getClasses();
  } catch { /* non-fatal */ }
  finally { beauty.classesLoading = false; }
}

// ── Selected class ────────────────────────────────────────────

export function selectClass(cls: ClassEntry) {
  beauty.selectedClass = cls.name;
}

// ── Preset detail modal ───────────────────────────────────────

export function openPreset(preset: PresetEntry, classDisplay: string) {
  beauty.presetDetail = { ...preset, class_display: classDisplay };
}

export function closePreset() {
  beauty.presetDetail = null;
}

// ── Class favorites ───────────────────────────────────────────

export function setClassFavorites(favs: string[]) {
  beauty.classFavorites = new Set(favs);
}

export function toggleClassFavorite(name: string) {
  const next = new Set(beauty.classFavorites);
  if (next.has(name)) next.delete(name);
  else next.add(name);
  beauty.classFavorites = next;
}

// ── Wanted presets ────────────────────────────────────────────

export function setWantedPresets(ids: string[]) {
  beauty.wantedPresets = new Set(ids);
}

export function toggleWantedPreset(id: string) {
  const next = new Set(beauty.wantedPresets);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  beauty.wantedPresets = next;
}
