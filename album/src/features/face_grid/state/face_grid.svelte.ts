import { convertFileSrc } from '@tauri-apps/api/core';
import type { BdoAccount, FaceGridRow, FaceGridSlotRow, FaceTextureEntry, SlotAssignment } from '../../../lib/face_grid';
import {
  applyFaceGrid as cmdApplyFaceGrid,
  deleteFaceGrid as cmdDeleteFaceGrid,
  getFaceGrids,
  getFaceGridSlots,
  listFaceTextures,
  saveFaceGrid as cmdSaveFaceGrid,
  scanBdoAccounts,
  uploadCharacterFace as cmdUploadCharacterFace,
  getCharacterFaces,
} from '../../../lib/face_grid';

export const faceGrid = $state({
  accounts:         [] as BdoAccount[],
  activeAccountId:  null as string | null,
  visibleAccountIds: null as string[] | null,
  showAccountPicker: false,
  faceTextures:     [] as FaceTextureEntry[],
  customFaces:      {} as Record<string, string>,  // character_no → image_url
  accountThumbs:    {} as Record<string, string[]>, // account_id → [image_urls]
  pendingSlots:     {} as Record<string, string>,  // character_no → image_url (unused for now)
  savedGrids:       [] as FaceGridRow[],
  activeGridId:     null as number | null,
  activeGridSlots:  [] as FaceGridSlotRow[],
  loading:          false,
  error:            null as string | null,
  applyingGrid:     false,
  dialog: {
    open:       false,
    title:      '',
    message:    '' as string | null,
    inputs:     [] as Array<{ value: string; placeholder: string }>,
    error:      null as string | null,
    submitText: '',
    submitting: false,
    onSubmit:   null as ((values: string[]) => void | Promise<void>) | null,
  },
});

// ── Loaders ───────────────────────────────────────────────────

export async function loadAccounts() {
  faceGrid.loading = true;
  faceGrid.error   = null;
  try {
    const [accounts, textures, grids, customFaces] = await Promise.all([
      scanBdoAccounts(),
      listFaceTextures(),
      getFaceGrids(),
      getCharacterFaces(),
    ]);
    faceGrid.accounts     = accounts;
    faceGrid.faceTextures = textures;
    faceGrid.savedGrids   = grids;
    faceGrid.customFaces  = Object.fromEntries(customFaces);

    // Generate thumbnails for each account (first 5 characters)
    faceGrid.accountThumbs = {};
    for (const acc of accounts) {
      const thumbs = acc.characters
        .slice(0, 5)
        .map(char => {
          const custom = faceGrid.customFaces[char.character_no];
          if (custom) return custom;
          const bmp = textures.find(t => t.character_no === char.character_no);
          return bmp?.path ? convertFileSrc(bmp.path) : '';
        })
        .filter(url => url);
      faceGrid.accountThumbs[acc.account_id] = thumbs;
    }

    loadVisibleAccounts(accounts);
  } catch (e) {
    faceGrid.error = String(e);
  } finally {
    faceGrid.loading = false;
  }
}

function loadVisibleAccounts(accounts: BdoAccount[]) {
  const saved = localStorage.getItem('fg_selected_accounts');
  if (saved) {
    try {
      faceGrid.visibleAccountIds = JSON.parse(saved);
      // Ensure active account is in visible list
      if (faceGrid.visibleAccountIds && faceGrid.visibleAccountIds.length > 0) {
        faceGrid.activeAccountId = faceGrid.visibleAccountIds[0];
      }
      return;
    } catch (_) {}
  }
  // No saved selection → open picker
  faceGrid.showAccountPicker = true;
}

// ── Account picker ────────────────────────────────────────────

export function openAccountPicker() {
  faceGrid.showAccountPicker = true;
}

export function closeAccountPicker() {
  faceGrid.showAccountPicker = false;
}

export function setVisibleAccounts(accountIds: string[]) {
  faceGrid.visibleAccountIds = accountIds;
  localStorage.setItem('fg_selected_accounts', JSON.stringify(accountIds));
  if (accountIds.length > 0) {
    faceGrid.activeAccountId = accountIds[0];
    // Create default preset for this account if it doesn't have one
    createDefaultPreset(accountIds[0]).catch(e => console.error('Failed to create default preset:', e));
  }
  faceGrid.showAccountPicker = false;
}

async function createDefaultPreset(accountId: string) {
  const account = faceGrid.accounts.find(a => a.account_id === accountId);
  if (!account) return;

  // Check if account already has a preset
  const existingPresets = faceGrid.savedGrids.filter(g => g.account_id === accountId);
  if (existingPresets.length > 0) return; // Already has presets

  // Create default preset with current character images
  const slots: SlotAssignment[] = account.characters.map(c => {
    const custom = faceGrid.customFaces[c.character_no];
    if (custom) return { character_no: c.character_no, slot_order: c.order, image_url: custom };

    const bmp = faceGrid.faceTextures.find(t => t.character_no === c.character_no);
    const bmpPath = bmp?.path ? convertFileSrc(bmp.path) : '';
    return { character_no: c.character_no, slot_order: c.order, image_url: bmpPath };
  });

  try {
    const grid = await cmdSaveFaceGrid(`Default`, accountId, slots);
    faceGrid.savedGrids = [grid, ...faceGrid.savedGrids];
    faceGrid.activeGridId = grid.id;
    faceGrid.activeGridSlots = await getFaceGridSlots(grid.id);
  } catch (e) {
    console.error('Failed to create default preset:', e);
  }
}

export async function loadGrid(gridId: number) {
  try {
    faceGrid.activeGridId = gridId;
    faceGrid.activeGridSlots = await getFaceGridSlots(gridId);
  } catch (e) {
    console.error('Failed to load grid:', e);
  }
}

export function visibleAccounts(): BdoAccount[] {
  if (faceGrid.visibleAccountIds === null) {
    return faceGrid.accounts;
  }
  return faceGrid.accounts.filter(a => faceGrid.visibleAccountIds!.includes(a.account_id));
}

// ── Active account ────────────────────────────────────────────

export function selectAccount(accountId: string) {
  faceGrid.activeAccountId = accountId;
}

export function removeAccount(accountId: string) {
  faceGrid.accounts = faceGrid.accounts.filter(a => a.account_id !== accountId);
  if (faceGrid.activeAccountId === accountId) {
    faceGrid.activeAccountId = faceGrid.accounts[0]?.account_id ?? null;
  }
}

export function activeAccount(): BdoAccount | undefined {
  return faceGrid.accounts.find(a => a.account_id === faceGrid.activeAccountId);
}

// ── Face texture lookup ───────────────────────────────────────

export function bmpPathFor(characterNo: string): string | null {
  return faceGrid.faceTextures.find(t => t.character_no === characterNo)?.path ?? null;
}

// ── Custom character faces ───────────────────────────────────

export function setCustomFace(characterNo: string, imageUrl: string) {
  faceGrid.customFaces[characterNo] = imageUrl;
}

export async function uploadCharacterFace(characterNo: string, imageB64: string): Promise<string> {
  const url = await cmdUploadCharacterFace(characterNo, imageB64);
  setCustomFace(characterNo, url);
  return url;
}

// ── Save grid (snapshot of current account's characters) ──────

export async function saveGrid(name: string): Promise<void> {
  const account = activeAccount();
  if (!account) return;

  const slots: SlotAssignment[] = account.characters.map(c => ({
    character_no: c.character_no,
    slot_order:   c.order,
    image_url:    faceGrid.customFaces[c.character_no] ?? '',
  }));

  const grid = await cmdSaveFaceGrid(name, account.account_id, slots);
  faceGrid.savedGrids = [grid, ...faceGrid.savedGrids];
  faceGrid.dialog.open = false;
}

// ── Apply saved grid ──────────────────────────────────────────

export async function applyGrid(gridId: number): Promise<void> {
  faceGrid.applyingGrid = true;
  try {
    await cmdApplyFaceGrid(gridId);
    faceGrid.faceTextures = await listFaceTextures();
  } finally {
    faceGrid.applyingGrid = false;
  }
}

// ── Delete grid ───────────────────────────────────────────────

export async function deleteGrid(gridId: number): Promise<void> {
  await cmdDeleteFaceGrid(gridId);
  faceGrid.savedGrids = faceGrid.savedGrids.filter(g => g.id !== gridId);
}

// ── Grid slots ────────────────────────────────────────────────

export async function loadGridSlots(gridId: number): Promise<void> {
  faceGrid.activeGridSlots = await getFaceGridSlots(gridId);
}

// ── UI ────────────────────────────────────────────────────────

export function openDialog(
  title: string,
  options?: {
    message?: string;
    inputs?: Array<{ placeholder: string }>;
    submitText?: string;
    onSubmit?: (values: string[]) => void | Promise<void>;
  }
) {
  faceGrid.dialog.title = title;
  faceGrid.dialog.message = options?.message ?? null;
  faceGrid.dialog.inputs = options?.inputs?.map(i => ({ value: '', placeholder: i.placeholder })) ?? [];
  faceGrid.dialog.submitText = options?.submitText || 'Confirm';
  faceGrid.dialog.submitting = false;
  faceGrid.dialog.error = null;
  faceGrid.dialog.onSubmit = options?.onSubmit || null;
  faceGrid.dialog.open = true;
}

export function closeDialog() {
  faceGrid.dialog.open = false;
}

export function openSaveGridDialog() {
  openDialog('Save Grid', {
    inputs: [{ placeholder: 'Grid name' }],
    submitText: 'Save',
    onSubmit: async (values) => {
      try {
        faceGrid.dialog.submitting = true;
        await saveGrid(values[0]);
      } catch (e) {
        faceGrid.dialog.error = String(e);
        faceGrid.dialog.submitting = false;
      }
    },
  });
}

export function openConfirmDialog(
  title: string,
  message: string,
  onConfirm: () => void | Promise<void>,
  confirmText = 'Confirm'
) {
  openDialog(title, {
    message,
    submitText: confirmText,
    onSubmit: async () => {
      try {
        faceGrid.dialog.submitting = true;
        await onConfirm();
      } catch (e) {
        faceGrid.dialog.error = String(e);
        faceGrid.dialog.submitting = false;
      }
    },
  });
}
