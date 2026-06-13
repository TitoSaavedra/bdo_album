import { invoke } from '@tauri-apps/api/core';
import type {
  BdoAccount,
  FaceTextureEntry,
  FaceGridRow,
  FaceGridSlotRow,
  SlotAssignment,
} from './types';

export const scanBdoAccounts = (): Promise<BdoAccount[]> =>
  invoke('scan_bdo_accounts');

export const listFaceTextures = (): Promise<FaceTextureEntry[]> =>
  invoke('list_face_textures');

export const getFaceGrids = (): Promise<FaceGridRow[]> =>
  invoke('get_face_grids');

export const getFaceGridSlots = (gridId: number): Promise<FaceGridSlotRow[]> =>
  invoke('get_face_grid_slots', { gridId });

export const saveFaceGrid = (
  name:      string,
  accountId: string,
  slots:     SlotAssignment[],
): Promise<FaceGridRow> =>
  invoke('save_face_grid', { name, accountId, slots });

export const applyFaceGrid = (gridId: number): Promise<void> =>
  invoke('apply_face_grid', { gridId });

export const deleteFaceGrid = (gridId: number): Promise<void> =>
  invoke('delete_face_grid', { gridId });

export const overwriteFaceGrid = (gridId: number, accountId: string): Promise<FaceGridRow> =>
  invoke('overwrite_face_grid', { gridId, accountId });

export const getCharacterFaces = (): Promise<Array<[string, string]>> =>
  invoke('get_character_faces');

export const saveFaceToDisk = (characterNo: string, filePath: string): Promise<void> =>
  invoke('save_face_to_disk', { characterNo, filePath });
