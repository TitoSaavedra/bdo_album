import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────

export interface ClassEntry {
  class_id:     number;
  name:         string;
  icon_svg:     string | null;
  preset_count: number;
  is_favorite:  boolean;
}

export interface ClassCount {
  class_id: number;
  count:    number;
}

export interface PresetEntry {
  preset_id:      string;
  class_id:       number;
  title:          string | null;
  user_nickname:  string | null;
  character_name: string | null;
  region:         string | null;
  image_1_url:    string | null;
  image_2_url:    string | null;
  pab_url:        string | null;
  has_pab:        boolean;
  downloads:      number | null;
  views:          number | null;
  likes:          number | null;
  is_discarded:   boolean;
  is_wanted:      boolean;
  creation_at:    number | null;
  updated_at:     number | null;
  auto_download_requested_at: number | null;
  auto_download_error:        string | null;
}

// ── Database state ────────────────────────────────────────────

export const isDbReady = (): Promise<boolean> =>
  invoke('is_db_ready');

// ── Classes ───────────────────────────────────────────────────

export const getClasses = (): Promise<ClassEntry[]> =>
  invoke('get_classes');

export const getClassFavorites = (): Promise<string[]> =>
  invoke('get_class_favorites');

export const setClassFavorite = (className: string, isFavorite: boolean): Promise<void> =>
  invoke('set_class_favorite', { className, isFavorite });

// ── Presets ───────────────────────────────────────────────────

export const getPresets = (
  className: string,
  offset    = 0,
  limit     = 40,
  sortBy    = 'downloads',
  search    = '',
  region    = '',
  days      = 'ever',
): Promise<PresetEntry[]> =>
  invoke('get_presets', { className, offset, limit, sortBy, search, region, days });

export const getPreset = (presetId: string): Promise<PresetEntry | null> =>
  invoke('get_preset', { presetId });

export const getRegions = (): Promise<string[]> =>
  invoke('get_regions');

export const getClassSearchCounts = (
  search = '',
  region = '',
  days   = 'ever',
): Promise<ClassCount[]> =>
  invoke('get_class_search_counts', { search, region, days });

export const discardPreset = (presetId: string): Promise<void> =>
  invoke('discard_preset', { presetId });

export const toggleWanted = (presetId: string): Promise<boolean> =>
  invoke('toggle_wanted', { presetId });

export const getWanted = (): Promise<string[]> =>
  invoke('get_wanted');

export const getWantedPabUrls = (): Promise<string[]> =>
  invoke('get_wanted_pab_urls');

export const getWantedPresets = (): Promise<PresetEntry[]> =>
  invoke('get_wanted_presets');

export const queueAutoDownload = (presetIds: string[]): Promise<void> =>
  invoke('queue_auto_download', { presetIds });

export const exportToBdo = (pabUrl: string): Promise<void> =>
  invoke('export_to_bdo', { pabUrl });

// ── URL ───────────────────────────────────────────────────────

export const openUrl = (url: string): Promise<void> =>
  invoke('open_url', { url });
