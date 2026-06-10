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
  faceGrid.dialog.open = false;
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
