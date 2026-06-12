// ── BDO Account ───────────────────────────────────────────────

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

// ── Face Grid ─────────────────────────────────────────────────

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

// ── Dialog ────────────────────────────────────────────────────

export interface DialogInput {
  value:       string;
  placeholder: string;
}

export interface DialogState {
  open:       boolean;
  title:      string;
  message:    string | null;
  inputs:     DialogInput[];
  error:      string | null;
  submitText: string;
  submitting: boolean;
  onSubmit:   ((values: string[]) => void | Promise<void>) | null;
}
