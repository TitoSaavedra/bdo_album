import type { PresetEntry } from '../../../lib/album';
import type { BdoAccount, FaceGridRow, FaceGridSlotRow, FaceTextureEntry, SlotAssignment } from '../../../lib/face_grid';
import {
  applyFaceGrid as cmdApplyFaceGrid,
  applyFaceToSlot,
  deleteFaceGrid as cmdDeleteFaceGrid,
  getFaceGrids,
  getFaceGridSlots,
  listFaceTextures,
  saveFaceGrid as cmdSaveFaceGrid,
  scanBdoAccounts,
} from '../../../lib/face_grid';

export const faceGrid = $state({
  // BDO accounts detected from UserCache
  accounts:        [] as BdoAccount[],
  activeAccountId: null as string | null,

  // All BMPs in FaceTexture (for orphan detection)
  faceTextures:    [] as FaceTextureEntry[],

  // Pending slot assignments (in-memory before save/apply)
  // character_no → { preset, image_url }
  pendingSlots:    {} as Record<string, { preset: PresetEntry; image_url: string }>,

  // Saved grids
  savedGrids:      [] as FaceGridRow[],
  gridsLoading:    false,

  // Active grid for slot inspection
  activeGridSlots: [] as FaceGridSlotRow[],

  // Drag state
  draggingPreset:  null as PresetEntry | null,

  // UI
  loading:         false,
  error:           null as string | null,
  applyingGrid:    false,
  saveDialogOpen:  false,
});

// ── Loaders ───────────────────────────────────────────────────

export async function loadAccounts() {
  faceGrid.loading = true;
  faceGrid.error   = null;
  try {
    const [accounts, textures, grids] = await Promise.all([
      scanBdoAccounts(),
      listFaceTextures(),
      getFaceGrids(),
    ]);
    faceGrid.accounts     = accounts;
    faceGrid.faceTextures = textures;
    faceGrid.savedGrids   = grids;
    if (accounts.length > 0 && faceGrid.activeAccountId === null) {
      faceGrid.activeAccountId = accounts[0].account_id;
    }
  } catch (e) {
    faceGrid.error = String(e);
  } finally {
    faceGrid.loading = false;
  }
}

// ── Active account ────────────────────────────────────────────

export function selectAccount(accountId: string) {
  faceGrid.activeAccountId = accountId;
  faceGrid.pendingSlots    = {};
}

export function activeAccount(): BdoAccount | undefined {
  return faceGrid.accounts.find(a => a.account_id === faceGrid.activeAccountId);
}

// ── Face texture lookup ───────────────────────────────────────

export function bmpPathFor(characterNo: string): string | null {
  return faceGrid.faceTextures.find(t => t.character_no === characterNo)?.path ?? null;
}

export function isOrphan(characterNo: string): boolean {
  const inAnyAccount = faceGrid.accounts.some(a =>
    a.characters.some(c => c.character_no === characterNo)
  );
  return !inAnyAccount;
}

// ── Slot assignments ──────────────────────────────────────────

export function assignPresetToSlot(characterNo: string, preset: PresetEntry) {
  const image_url = preset.image_1_url ?? '';
  faceGrid.pendingSlots = {
    ...faceGrid.pendingSlots,
    [characterNo]: { preset, image_url },
  };
}

export function clearSlot(characterNo: string) {
  const next = { ...faceGrid.pendingSlots };
  delete next[characterNo];
  faceGrid.pendingSlots = next;
}

export function clearAllSlots() {
  faceGrid.pendingSlots = {};
}

// ── Apply single slot ─────────────────────────────────────────

export async function applySlot(characterNo: string): Promise<void> {
  const slot = faceGrid.pendingSlots[characterNo];
  if (!slot || !slot.image_url) return;
  await applyFaceToSlot(characterNo, slot.image_url);
  // Refresh face textures list
  faceGrid.faceTextures = await listFaceTextures();
}

// ── Save grid ─────────────────────────────────────────────────

export async function saveGrid(name: string): Promise<void> {
  const account = activeAccount();
  if (!account) return;

  const slots: SlotAssignment[] = Object.entries(faceGrid.pendingSlots).map(([charNo, val]) => ({
    character_no: charNo,
    preset_id:    val.preset.preset_id,
    slot_order:   account.characters.find(c => c.character_no === charNo)?.order ?? 0,
    image_url:    val.image_url,
  }));

  const grid = await cmdSaveFaceGrid(name, account.account_id, slots);
  faceGrid.savedGrids = [grid, ...faceGrid.savedGrids];
  faceGrid.saveDialogOpen = false;
}

// ── Apply saved grid ──────────────────────────────────────────

export async function applyGrid(gridId: string): Promise<void> {
  faceGrid.applyingGrid = true;
  try {
    await cmdApplyFaceGrid(gridId);
    faceGrid.faceTextures = await listFaceTextures();
  } finally {
    faceGrid.applyingGrid = false;
  }
}

// ── Delete grid ───────────────────────────────────────────────

export async function deleteGrid(gridId: string): Promise<void> {
  await cmdDeleteFaceGrid(gridId);
  faceGrid.savedGrids = faceGrid.savedGrids.filter(g => g.id !== gridId);
}

// ── Grid slots ────────────────────────────────────────────────

export async function loadGridSlots(gridId: string): Promise<void> {
  faceGrid.activeGridSlots = await getFaceGridSlots(gridId);
}

// ── Drag ──────────────────────────────────────────────────────

export function setDraggingPreset(preset: PresetEntry | null) {
  faceGrid.draggingPreset = preset;
}

// ── UI ────────────────────────────────────────────────────────

export function openSaveDialog() {
  faceGrid.saveDialogOpen = true;
}

export function closeSaveDialog() {
  faceGrid.saveDialogOpen = false;
}
