import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { RustEventMap, RustEventName, DbErrorCode } from './types';
import {
  onProgress,
  onFetchProgress,
  onImageProgress,
  onUploadProgress,
  onClassStatsUpdated,
  onPresetSynced,
  onSyncLoading,
  setStatus,
  setPhase,
  setDbReady,
  setUpdated,
  setAutoDownloadStatus,
  pushLog,
} from '../../features/scraper/state/scraper.svelte';

class ScraperEventBus {
  private unlisten: UnlistenFn[] = [];

  // ── Typed listener ───────────────────────────────────────────
  // TypeScript infers payload type from the event name automatically.
  // Adding a new event = add it to RustEventMap + add on() call in init().

  private async on<K extends RustEventName>(
    event: K,
    handler: (payload: RustEventMap[K]) => void,
  ): Promise<void> {
    const fn = await listen<RustEventMap[K]>(event, ({ payload }) => handler(payload));
    this.unlisten.push(fn);
  }

  // ── Init ─────────────────────────────────────────────────────

  async init(): Promise<void> {
    // Defensive: if init() is ever called again without a matching destroy()
    // (e.g. a dev-mode HMR remount), drop any listeners already registered
    // instead of stacking duplicates that each re-fire the same event.
    this.destroy();

    await Promise.all([

      // Lifecycle
      this.on('scraper_started',   ()  => { setStatus('running'); setPhase('fetch'); }),
      this.on('scraper_done',      (p) => { setUpdated(p.total_updated); setStatus('done'); }),
      this.on('scraper_cancelled', ()  => setStatus('cancelled')),
      this.on('scraper_error',     ()  => { setStatus('error'); }),

      // Progress
      this.on('scraper_progress',  (p) => onProgress(p)),
      this.on('fetch_progress',     (p) => { setPhase('fetch');    onFetchProgress(p);   }),
      this.on('scraper_fetch_done',()  => { setPhase('download'); }),
      this.on('image_progress',     (p) => { onImageProgress(p);   }),
      this.on('upload_progress',    (p) => { setPhase('upload');   onUploadProgress(p);  }),

      // Data
      this.on('class_stats_updated', (p) => onClassStatsUpdated(p)),
      this.on('preset_synced',       (p) => onPresetSynced(p)),

      // Infrastructure
      this.on('db_ready',     (p) => {
        setDbReady(p.success, p.error);
        if (!p.success && p.error) {
          pushLog({ ts: now(), tag: 'ERR', source: 'db', msg: DB_ERROR_TEXT[p.error], code: null });
        }
      }),
      this.on('log_entry',    (p) => pushLog(p)),
      this.on('sync_loading', (p) => onSyncLoading(p)),
      this.on('auto_download_status', (p) => setAutoDownloadStatus(p)),

    ]);
  }

  // ── Cleanup ──────────────────────────────────────────────────

  destroy(): void {
    this.unlisten.forEach(fn => fn());
    this.unlisten = [];
  }
}

const now = () => Math.floor(Date.now() / 1000);

// Logs feed stays English-only (no i18n) — see i18n/locales for the translated
// popup text; this is just the fallback line pushed into the same feed.
const DB_ERROR_TEXT: Record<DbErrorCode, string> = {
  docker_not_running: 'Could not connect to the database. Is Docker running? (docker compose up -d)',
  env_var_missing:    'DATABASE_URL is not set',
};

export const eventBus = new ScraperEventBus();
