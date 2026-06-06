import { CLASSES } from '$lib/classes';
import type {
  ScrapperStatus,
  ScrapperPhase,
  ScrapperProgress,
  FetchProgress,
  ImageProgress,
  UploadProgress,
  ClassStatsUpdated,
  PresetSynced,
  LogEntry,
} from '$lib/events/types';

// Re-export types consumed by components
export type { ScrapperStatus, ScrapperPhase, LogEntry };

type LogEntryTagged = LogEntry & { _uid: number };

interface ClassState {
  id:      number;
  name:    string;
  fetched: number;
  images:  number;
  active:  boolean;
}

// ── State ────────────────────────────────────────────────────

let classIcons        = $state<Record<number, string>>({});
let lastPresetSynced  = $state<PresetSynced | null>(null);

let dbReady  = $state<boolean | null>(null); // null = pending, true = ok, false = error
let dbError  = $state<string | null>(null);

let status       = $state<ScrapperStatus>('idle');
let phase        = $state<ScrapperPhase>('fetch');
let current      = $state(0);
let total        = $state(0);
let currentClass = $state('');
let currentMsg   = $state('');
let imagesDone   = $state(0);
let imagesTotal  = $state(0);
let uploadsDone  = $state(0);
let uploadsTotal = $state(0);
let errors       = $state(0);
let discarded    = $state(0);
let startedAt    = $state<number | null>(null);
let _logUid      = 0;
let logs         = $state<LogEntryTagged[]>([]);
let sessionLogs  = $state<LogEntryTagged[]>([]);
let _ticker: ReturnType<typeof setInterval> | null = null;
let _now         = $state(Date.now());

const classMap = $state<Record<number, ClassState>>(
  Object.fromEntries(
    CLASSES.map((c): [number, ClassState] => [c.id, { id: c.id, name: c.name, fetched: 0, images: 0, active: false }])
  )
);

// ── Derived (wrapped in functions — Svelte 5 module rule) ────

const _progress     = $derived(total > 0        ? Math.round((current     / total)        * 100) : 0);
const _imgProgress  = $derived(imagesTotal > 0  ? Math.round((imagesDone  / imagesTotal)  * 100) : 0);
const _upProgress   = $derived(uploadsTotal > 0 ? Math.round((uploadsDone / uploadsTotal) * 100) : 0);
const _elapsed      = $derived(startedAt ? Math.floor((_now - startedAt) / 1000) : 0);
const _totalFetched = $derived(Object.values(classMap).reduce((s, c: ClassState) => s + c.fetched, 0));
const _totalImages  = $derived(Object.values(classMap).reduce((s, c: ClassState) => s + c.images,  0));

// ── Getters ──────────────────────────────────────────────────

// DB state
export const getDbReady          = () => dbReady;
export const getDbError          = () => dbError;
export const getClassIcons       = () => classIcons;
export const getLastPresetSynced = () => lastPresetSynced;

// Raw state
export const getStatus       = () => status;
export const getPhase        = () => phase;
export const getCurrent      = () => current;
export const getTotal        = () => total;
export const getCurrentClass = () => currentClass;
export const getCurrentMsg   = () => currentMsg;
export const getImagesDone   = () => imagesDone;
export const getImagesTotal  = () => imagesTotal;
export const getUploadsDone  = () => uploadsDone;
export const getUploadsTotal = () => uploadsTotal;
export const getErrors       = () => errors;
export const getDiscarded    = () => discarded;
export const getClassMap     = () => classMap;
export const getLogs         = () => logs;
export const getSessionLogs  = () => sessionLogs;

// Computed
export const getProgress     = () => _progress;
export const getImgProgress  = () => _imgProgress;
export const getUpProgress   = () => _upProgress;
export const getElapsed      = () => _elapsed;
export const getTotalFetched = () => _totalFetched;
export const getTotalImages  = () => _totalImages;

// ── Mutators — called exclusively from EventBus ───────────────

export function setClassIcons(entries: { id: number; icon_svg: string | null }[]): void {
  entries.forEach(e => { if (e.icon_svg) classIcons[e.id] = e.icon_svg; });
}

export function setDbReady(success: boolean, error?: string | null): void {
  dbReady = success;
  dbError = error ?? null;
}

export function setStatus(s: ScrapperStatus): void {
  status = s;
  if (s === 'running') {
    errors       = 0;
    discarded    = 0;
    imagesDone   = 0;
    imagesTotal  = 0;
    uploadsDone  = 0;
    uploadsTotal = 0;
    current      = 0;
    total        = 0;
    CLASSES.forEach(c => {
      classMap[c.id].fetched = 0;
      classMap[c.id].images  = 0;
      classMap[c.id].active  = false;
    });
    startedAt   = Date.now();
    _now        = startedAt;
    sessionLogs = [];
    _ticker = setInterval(() => { _now = Date.now(); }, 1000);
  }
  if (s === 'done' || s === 'idle' || s === 'cancelled' || s === 'error') {
    if (_ticker) { clearInterval(_ticker); _ticker = null; }
    Object.values(classMap).forEach(c => (c.active = false));
  }
}

export function requestStop(): void {
  if (status === 'running') status = 'stopping';
}

export function setPhase(p: ScrapperPhase): void {
  phase = p;
}

export function onProgress(p: ScrapperProgress): void {
  current      = p.current;
  total        = p.total;
  currentClass = p.class_name;
  currentMsg   = p.message;

  const cls = classMap[p.class_id];
  if (!cls) return;

  cls.active = p.status !== 'done';
  if (p.status === 'done') {
    if (p.progress_type === 'preset') cls.fetched += 1;
  }
}

export function onFetchProgress(p: FetchProgress): void {
  currentClass = p.class_name;
  current      = p.fetched;
  total        = p.total;
}

export function onImageProgress(p: ImageProgress): void {
  imagesDone  = p.done;
  imagesTotal = p.total;
  currentMsg  = `${p.class_name} — img ${p.image_num}`;
  const entry = Object.values(classMap).find((c: ClassState) => c.name === p.class_name);
  if (entry && p.image_num === 1) entry.images += 1;
}

export function onUploadProgress(p: UploadProgress): void {
  uploadsDone  = p.done;
  uploadsTotal = p.total;
}

export function onClassStatsUpdated(p: ClassStatsUpdated): void {
  const cls = classMap[p.class_id];
  if (cls) {
    cls.fetched = p.fetched;
    cls.images  = p.images;
  }
  if (p.errors  > 0) errors    += p.errors;
  if (p.skipped > 0) discarded += p.skipped;
}

export function onPresetSynced(p: PresetSynced): void {
  if (p.image_1_url || p.image_2_url) {
    const count = (p.image_1_url ? 1 : 0) + (p.image_2_url ? 1 : 0);
    currentMsg = `Preset ${p.preset_id} — ${count} image(s) synced to R2`;
    const cls = classMap[p.class_id];
    if (cls) cls.images += count;
  }
  lastPresetSynced = p;
}

export function onSyncLoading(msg: string): void {
  currentMsg = msg;
}

export function pushLog(entry: LogEntry): void {
  const tagged = { ...entry, _uid: _logUid++ };
  logs.unshift(tagged);
  if (logs.length > 2000) logs.pop();
  sessionLogs.unshift(tagged);
  if (sessionLogs.length > 500) sessionLogs.pop();
}

export function prependLogs(entries: LogEntry[]): void {
  const existing = new Set(logs.map(l => `${l.ts}|${l.tag}|${l.source}|${l.msg}`));
  const fresh = entries.filter(e => !existing.has(`${e.ts}|${e.tag}|${e.source}|${e.msg}`));
  if (fresh.length === 0) return;
  logs = [...logs, ...fresh.map(e => ({ ...e, _uid: _logUid++ }))];
}

export function clearLogs(): void {
  logs = [];
}

export function reset(): void {
  status       = 'idle';
  phase        = 'fetch';
  current      = 0;
  total        = 0;
  currentClass = '';
  currentMsg   = '';
  imagesDone   = 0;
  imagesTotal  = 0;
  uploadsDone  = 0;
  uploadsTotal = 0;
  errors       = 0;
  discarded    = 0;
  startedAt    = null;
  CLASSES.forEach(c => {
    classMap[c.id].fetched = 0;
    classMap[c.id].images  = 0;
    classMap[c.id].active  = false;
  });
}
