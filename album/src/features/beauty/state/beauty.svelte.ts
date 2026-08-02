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

export function onPresetUploaded(preset: PresetEntry) {
  beauty.liveUploaded[preset.class_id] = (beauty.liveUploaded[preset.class_id] ?? 0) + 1;
  beauty.livePresets[preset.class_id]  = [preset, ...(beauty.livePresets[preset.class_id] ?? [])];
  const cls = beauty.classes.find(c => c.class_id === preset.class_id);
  if (cls) cls.preset_count += 1;
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
