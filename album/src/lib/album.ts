import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────

export interface ClassEntry {
  class_id:     number;
  name:         string;
  icon_svg:     string | null;
  preset_count: number;
  is_favorite:  boolean;
}

export interface PresetEntry {
  preset_id:      string;
  class_id:       number;
  title:          string | null;
  user_nickname:  string | null;
  character_name: string | null;
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
  offset   = 0,
  limit    = 50,
  sortBy   = 'downloads',
  search   = '',
): Promise<PresetEntry[]> =>
  invoke('get_presets', { className, offset, limit, sortBy, search });

export const discardPreset = (presetId: string): Promise<void> =>
  invoke('discard_preset', { presetId });

export const toggleWanted = (presetId: string): Promise<boolean> =>
  invoke('toggle_wanted', { presetId });

export const getWanted = (): Promise<string[]> =>
  invoke('get_wanted');

// ── URL ───────────────────────────────────────────────────────

export const openUrl = (url: string): Promise<void> =>
  invoke('open_url', { url });
