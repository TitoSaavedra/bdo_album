import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────

export interface CharacterEntry {
  character_no: string;
  order:        number;
  has_bmp:      boolean;
  bmp_path:     string | null;
}

export interface BdoAccount {
  account_id: string;
  characters: CharacterEntry[];
}

export interface FaceTextureEntry {
  character_no: string;
  path:         string;
}

export interface FaceGridRow {
  id:            number;
  name:          string;
  account_id:    string;
  thumbnail_url: string | null;
  created_at:    number;
}

export interface FaceGridSlotRow {
  id:           number;
  grid_id:      number;
  character_no: string;
  image_url:    string;
  slot_order:   number;
}

export interface SlotAssignment {
  character_no: string;
  slot_order:   number;
  image_url:    string;
}

// ── Commands ──────────────────────────────────────────────────

export const scanBdoAccounts = (): Promise<BdoAccount[]> =>
  invoke('scan_bdo_accounts');

export const listFaceTextures = (): Promise<FaceTextureEntry[]> =>
  invoke('list_face_textures');

export const applyFaceToSlot = (characterNo: string, imageUrl: string): Promise<void> =>
  invoke('apply_face_to_slot', { characterNo, imageUrl });

export const saveFaceGrid = (
  name:      string,
  accountId: string,
  slots:     SlotAssignment[],
): Promise<FaceGridRow> =>
  invoke('save_face_grid', { name, accountId, slots });

export const getFaceGrids = (): Promise<FaceGridRow[]> =>
  invoke('get_face_grids');

export const getFaceGridSlots = (gridId: number): Promise<FaceGridSlotRow[]> =>
  invoke('get_face_grid_slots', { gridId });

export const applyFaceGrid = (gridId: number): Promise<void> =>
  invoke('apply_face_grid', { gridId });

export const deleteFaceGrid = (gridId: number): Promise<void> =>
  invoke('delete_face_grid', { gridId });

export const uploadCharacterFace = (characterNo: string, imageB64: string): Promise<string> =>
  invoke('upload_character_face', { characterNo, imageB64 });

export const getCharacterFaces = (): Promise<Array<[string, string]>> =>
  invoke('get_character_faces');
