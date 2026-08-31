import type { ClassCount, ClassEntry, PresetEntry } from '../../../lib/album';
import { getClasses } from '../../../lib/album';
import type { DbErrorCode } from '../../../lib/events/types';

export const beauty = $state({
  // DB
  dbReady:  false,
  dbError:  null as DbErrorCode | null,

  // Classes
  classes:        [] as ClassEntry[],
  classesLoading: false,

  // Selected class
  selectedClass: null as string | null,

  // Preset detail modal
  presetDetail: null as (PresetEntry & { class_display: string }) | null,

  // Class favorites
  classFavorites: new Set<string>(),

  // Creator favorites
  creatorFavorites: new Set<string>(),

  // Active "browse by favorite creator" filter — a creator nickname, or null
  creatorFilter: null as string | null,

  // Wanted presets
  wantedPresets: new Set<string>(),

  // Region filter
  selectedRegion:   '',
  availableRegions: [] as string[],

  // Sort
  sortBy: 'downloads' as 'downloads' | 'views' | 'likes',

  // Days filter
  selectedDays: 'ever',

  // Search
  searchQuery:        '',
  searchCounts:       {} as Record<number, number>,
  searchCountsLoaded: false,

  // Live upload tracking (from sscraper via PG LISTEN/NOTIFY)
  liveUploaded:      {} as Record<number, number>,
  livePresets:       {} as Record<number, PresetEntry[]>,
  listenerConnected: false,
});

// ── DB ────────────────────────────────────────────────────────

export function setDbReady(ok: boolean, error: DbErrorCode | null) {
  beauty.dbReady = ok;
  beauty.dbError = error;
  if (ok) loadClasses();
}

// ── Classes ───────────────────────────────────────────────────

export async function loadClasses() {
  beauty.classesLoading = true;
  try {
    beauty.classes = await getClasses();
    if (beauty.selectedClass === null && beauty.classes.length > 0) {
      beauty.selectedClass = beauty.classes[0].name;
    }
  } catch { /* non-fatal */ }
  finally { beauty.classesLoading = false; }
}

// ── Selected class ────────────────────────────────────────────

export function selectClass(cls: ClassEntry) {
  beauty.selectedClass = cls.name;
}

// ── Preset detail modal ───────────────────────────────────────

// Not reactive state on purpose — nothing renders off this directly, it's
// only read when the mouse "forward" button asks to reopen whatever was
// last closed (one-slot back/forward history, same idea as browser nav).
let lastClosedPreset: (PresetEntry & { class_display: string }) | null = null;

export function openPreset(preset: PresetEntry, classDisplay: string) {
  beauty.presetDetail = { ...preset, class_display: classDisplay };
}

export function closePreset() {
  if (beauty.presetDetail) lastClosedPreset = beauty.presetDetail;
  beauty.presetDetail = null;
}

export function reopenLastPreset() {
  if (lastClosedPreset && !beauty.presetDetail) {
    beauty.presetDetail = lastClosedPreset;
  }
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

// ── Creator favorites ─────────────────────────────────────────

export function setCreatorFavorites(names: string[]) {
  beauty.creatorFavorites = new Set(names);
}

export function toggleCreatorFavorite(name: string) {
  const next = new Set(beauty.creatorFavorites);
  if (next.has(name)) next.delete(name);
  else next.add(name);
  beauty.creatorFavorites = next;
}

export function setCreatorFilter(name: string | null) {
  beauty.creatorFilter = name;
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

// ── Region filter ─────────────────────────────────────────────

export function setSelectedRegion(region: string) {
  beauty.selectedRegion = region;
}

export function setAvailableRegions(regions: string[]) {
  beauty.availableRegions = regions;
}

export function setSelectedDays(days: string) {
  beauty.selectedDays = days;
}

export function setSelectedSort(sort: 'downloads' | 'views' | 'likes') {
  beauty.sortBy = sort;
}

// ── Live upload tracking ──────────────────────────────────────

// `isNewPreset` distinguishes a preset's first-ever image upload (it just
// entered the class's counted set) from a PAB-only completion on a preset
// that already had images and was already counted — bumping the live/total
// counters for the latter would double-count it. The card itself still needs
// to refresh either way (has_pab flips, etc.), so `livePresets` always gets
// the fresh copy; PresetGrid dedupes by preset_id and prefers this live entry
// over the stale one already in its loaded list.
export function onPresetUploaded(preset: PresetEntry, isNewPreset: boolean) {
  if (isNewPreset) {
    beauty.liveUploaded[preset.class_id] = (beauty.liveUploaded[preset.class_id] ?? 0) + 1;
    const cls = beauty.classes.find(c => c.class_id === preset.class_id);
    if (cls) cls.preset_count += 1;
  }
  beauty.livePresets[preset.class_id] = [
    preset,
    ...(beauty.livePresets[preset.class_id] ?? []).filter(p => p.preset_id !== preset.preset_id),
  ];
}

export function clearLiveForClass(classId: number) {
  beauty.liveUploaded[classId] = 0;
  beauty.livePresets[classId]  = [];
}

export function setListenerConnected(connected: boolean) {
  beauty.listenerConnected = connected;
}

// ── Search ────────────────────────────────────────────────────

export function setSearchCounts(counts: ClassCount[], loaded = true) {
  const map: Record<number, number> = {};
  for (const { class_id, count } of counts) map[class_id] = count;
  beauty.searchCounts       = map;
  beauty.searchCountsLoaded = loaded;
}
