import { convertFileSrc } from '@tauri-apps/api/core';
import type { FaceGridProgress } from '../../../lib/events/types';
import type {
  BdoAccount,
  CharacterEntry,
  DialogState,
  FaceGridRow,
  FaceGridSlotRow,
  FaceTextureEntry,
  SlotAssignment,
} from '../../../lib/face_grid/types';
import {
  applyFaceGrid    as cmdApplyFaceGrid,
  deleteFaceGrid   as cmdDeleteFaceGrid,
  getCharacterFaces,
  getFaceGrids,
  getFaceGridSlots,
  listFaceTextures,
  saveFaceGrid     as cmdSaveFaceGrid,
  saveFaceToDisk   as cmdSaveFaceToDisk,
  scanBdoAccounts,
} from '../../../lib/face_grid/commands';

// ── State ─────────────────────────────────────────────────────

let accounts         = $state<BdoAccount[]>([]);
let activeAccountId  = $state<string | null>(null);
let showAccountPicker = $state(false);

let faceTextures     = $state<FaceTextureEntry[]>([]);
let customFaces      = $state<Record<string, string>>({});   // character_no → r2 url
let bmpTimestamps    = $state<Record<string, number>>({});   // character_no → cache-bust ts
let accountThumbs    = $state<Record<string, string[]>>({});

let savedGrids       = $state<FaceGridRow[]>([]);
let activeGridId     = $state<number | null>(null);
let activeGridSlots  = $state<FaceGridSlotRow[]>([]);

let loading          = $state(false);
let error            = $state<string | null>(null);
let applyingGrid     = $state(false);
let applyingChars    = $state<Set<string>>(new Set());
let creatingPreset   = $state(false);

let createProgress   = $state<FaceGridProgress>({ current: 0, total: 0, character_no: '' });

let dialog           = $state<DialogState>({
  open:       false,
  title:      '',
  message:    null,
  inputs:     [],
  error:      null,
  submitText: '',
  submitting: false,
  onSubmit:   null,
});

// ── Getters ───────────────────────────────────────────────────

export const getAccounts        = () => accounts;
export const getActiveAccountId = () => activeAccountId;
export const getShowAccountPicker = () => showAccountPicker;

export const getFaceTextures    = () => faceTextures;
export const getCustomFaces     = () => customFaces;
export const getBmpTimestamps   = () => bmpTimestamps;
export const getAccountThumbs   = () => accountThumbs;

export const getSavedGrids      = () => savedGrids;
export const getActiveGridId    = () => activeGridId;
export const getActiveGridSlots = () => activeGridSlots;

export const getLoading         = () => loading;
export const getError           = () => error;
export const getApplyingGrid    = () => applyingGrid;
export const getApplyingChars   = () => applyingChars;
export const getCreatingPreset  = () => creatingPreset;
export const getCreateProgress  = () => createProgress;

export const getDialog          = () => dialog;

export const getActiveAccount = (): BdoAccount | undefined =>
  accounts.find(a => a.account_id === activeAccountId);

export const getBmpPathFor = (characterNo: string): string | null =>
  faceTextures.find(t => t.character_no === characterNo)?.path ?? null;

export const getCharacterSrc = (char: CharacterEntry): string | null => {
  const custom = customFaces[char.character_no];
  if (custom) return custom;
  const path = getBmpPathFor(char.character_no) ?? char.bmp_path;
  if (!path) return null;
  const ts = bmpTimestamps[char.character_no];
  return convertFileSrc(path) + (ts ? `?t=${ts}` : '');
};

export const getGridThumb = (accountId: string): string | null => {
  const account = accounts.find(a => a.account_id === accountId);
  if (!account) return null;
  const first = [...account.characters].sort((a, b) => a.order - b.order)[0];
  if (!first) return null;
  return getCharacterSrc(first);
};

// ── Event handler (called from event bus) ─────────────────────

export function onFaceGridProgress(p: FaceGridProgress): void {
  createProgress = p;
}

export function onFaceApplyDone(characterNo: string): void {
  const next = new Set(applyingChars);
  next.delete(characterNo);
  applyingChars = next;
  bmpTimestamps = { ...bmpTimestamps, [characterNo]: Date.now() };
}

// ── Loaders ───────────────────────────────────────────────────

export async function loadAccounts(): Promise<void> {
  loading = true;
  error   = null;
  try {
    const [accs, textures, grids, faces] = await Promise.all([
      scanBdoAccounts(),
      listFaceTextures(),
      getFaceGrids(),
      getCharacterFaces(),
    ]);

    accounts      = accs;
    faceTextures  = textures;
    savedGrids    = grids;
    customFaces   = Object.fromEntries(faces);
    // Bust cache for all known BMPs so the grid reloads images from disk
    const now = Date.now();
    const busted: Record<string, number> = {};
    for (const t of textures) busted[t.character_no] = now;
    bmpTimestamps = busted;

    // Build account thumbnails (first 5 characters)
    const thumbs: Record<string, string[]> = {};
    for (const acc of accs) {
      thumbs[acc.account_id] = acc.characters
        .slice(0, 5)
        .map(char => {
          const custom = customFaces[char.character_no];
          if (custom) return custom;
          const bmp = textures.find(t => t.character_no === char.character_no);
          return bmp?.path ? convertFileSrc(bmp.path) : '';
        })
        .filter(Boolean);
    }
    accountThumbs = thumbs;

    _restoreAccountSelection(accs);
  } catch (e) {
    error = String(e);
  } finally {
    loading = false;
  }
}

function _restoreAccountSelection(accs: BdoAccount[]): void {
  const saved = localStorage.getItem('fg_selected_account');
  if (saved && accs.some(a => a.account_id === saved)) {
    activeAccountId = saved;
    return;
  }
  // No saved selection → open picker
  showAccountPicker = true;
}

// ── Account picker ────────────────────────────────────────────

export function openAccountPicker(): void {
  showAccountPicker = true;
}

export function closeAccountPicker(): void {
  showAccountPicker = false;
}

export function selectAccount(accountId: string): void {
  activeAccountId = accountId;
  localStorage.setItem('fg_selected_account', accountId);
  showAccountPicker = false;
  _ensureDefaultPreset(accountId);
}

async function _ensureDefaultPreset(accountId: string): Promise<void> {
  if (creatingPreset) return;
  if (savedGrids.some(g => g.account_id === accountId && g.name === '_default')) return;

  try {
    creatingPreset = true;
    const grid = await cmdSaveFaceGrid('_default', accountId, []);
    savedGrids   = [grid, ...savedGrids];
    activeGridId  = grid.id;
    activeGridSlots = await getFaceGridSlots(grid.id);
  } catch (e) {
    console.error('Failed to create default preset:', e);
  } finally {
    creatingPreset = false;
  }
}

// ── Grid actions ──────────────────────────────────────────────

export async function loadGrid(gridId: number): Promise<void> {
  activeGridId    = gridId;
  activeGridSlots = await getFaceGridSlots(gridId);
}

export async function saveGrid(name: string): Promise<void> {
  const account = getActiveAccount();
  if (!account) return;

  const slots: SlotAssignment[] = account.characters.map(c => ({
    character_no: c.character_no,
    slot_order:   c.order,
    image_url:    customFaces[c.character_no] ?? '',
  }));

  const grid  = await cmdSaveFaceGrid(name, account.account_id, slots);
  savedGrids  = [grid, ...savedGrids];
  dialog.open = false;
}

export async function applyGrid(gridId: number): Promise<void> {
  const slots = await getFaceGridSlots(gridId);
  const chars = slots.filter(s => s.image_url).map(s => s.character_no);
  applyingChars = new Set(chars);
  applyingGrid  = true;
  try {
    await cmdApplyFaceGrid(gridId);
    faceTextures = await listFaceTextures();
  } finally {
    applyingGrid  = false;
    applyingChars = new Set();
  }
}

export async function deleteGrid(gridId: number): Promise<void> {
  await cmdDeleteFaceGrid(gridId);
  savedGrids = savedGrids.filter(g => g.id !== gridId);
  if (activeGridId === gridId) activeGridId = null;
}

// ── Character face ────────────────────────────────────────────

export async function saveFaceToDisk(characterNo: string, filePath: string): Promise<void> {
  await cmdSaveFaceToDisk(characterNo, filePath);
  bmpTimestamps = { ...bmpTimestamps, [characterNo]: Date.now() };
}

// ── Dialog ────────────────────────────────────────────────────

export function openDialog(
  title: string,
  options?: {
    message?:    string;
    inputs?:     Array<{ placeholder: string }>;
    submitText?: string;
    onSubmit?:   (values: string[]) => void | Promise<void>;
  }
): void {
  dialog = {
    open:       true,
    title,
    message:    options?.message ?? null,
    inputs:     options?.inputs?.map(i => ({ value: '', placeholder: i.placeholder })) ?? [],
    error:      null,
    submitText: options?.submitText ?? 'Confirm',
    submitting: false,
    onSubmit:   options?.onSubmit ?? null,
  };
}

export function closeDialog(): void {
  dialog.open = false;
}

export function openSaveGridDialog(): void {
  openDialog('Save Grid', {
    inputs:     [{ placeholder: 'Grid name' }],
    submitText: 'Save',
    onSubmit:   async (values) => {
      try {
        dialog.submitting = true;
        await saveGrid(values[0]);
      } catch (e) {
        dialog.error      = String(e);
        dialog.submitting = false;
      }
    },
  });
}

export function openConfirmDialog(
  title:       string,
  message:     string,
  onConfirm:   () => void | Promise<void>,
  confirmText = 'Confirm',
): void {
  openDialog(title, {
    message,
    submitText: confirmText,
    onSubmit:   async () => {
      try {
        dialog.submitting = true;
        await onConfirm();
      } catch (e) {
        dialog.error      = String(e);
        dialog.submitting = false;
      }
    },
  });
}
