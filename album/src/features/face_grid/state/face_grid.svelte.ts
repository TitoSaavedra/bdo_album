import type { BdoAccount, FaceGridRow, FaceGridSlotRow, FaceTextureEntry, SlotAssignment } from '../../../lib/face_grid';
import {
  applyFaceGrid as cmdApplyFaceGrid,
  deleteFaceGrid as cmdDeleteFaceGrid,
  getFaceGrids,
  getFaceGridSlots,
  listFaceTextures,
  saveFaceGrid as cmdSaveFaceGrid,
  scanBdoAccounts,
} from '../../../lib/face_grid';

export const faceGrid = $state({
  accounts:        [] as BdoAccount[],
  activeAccountId: null as string | null,
  faceTextures:    [] as FaceTextureEntry[],
  pendingSlots:    {} as Record<string, string>,  // character_no → image_url (unused for now)
  savedGrids:      [] as FaceGridRow[],
  activeGridSlots: [] as FaceGridSlotRow[],
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

// ── Save grid (snapshot of current account's characters) ──────

export async function saveGrid(name: string): Promise<void> {
  const account = activeAccount();
  if (!account) return;

  const slots: SlotAssignment[] = account.characters.map(c => ({
    character_no: c.character_no,
    preset_id:    '0',
    slot_order:   c.order,
    image_url:    '',
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

// ── UI ────────────────────────────────────────────────────────

export function openSaveDialog()  { faceGrid.saveDialogOpen = true;  }
export function closeSaveDialog() { faceGrid.saveDialogOpen = false; }
