import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { RustEventMap, RustEventName } from './types';
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
  pushLog,
} from '../../features/scrapper/state/scrapper.svelte';

class ScrapperEventBus {
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
    await Promise.all([

      // Lifecycle
      this.on('scrapper_started',   ()  => { setStatus('running'); setPhase('fetch'); }),
      this.on('scrapper_done',      (p) => { setUpdated(p.total_updated); setStatus('done'); }),
      this.on('scrapper_cancelled', ()  => setStatus('cancelled')),
      this.on('scrapper_error',     ()  => { setStatus('error'); }),

      // Progress
      this.on('scrapper_progress',  (p) => onProgress(p)),
      this.on('fetch_progress',     (p) => { setPhase('fetch');    onFetchProgress(p);   }),
      this.on('scrapper_fetch_done',()  => { setPhase('download'); }),
      this.on('image_progress',     (p) => { onImageProgress(p);   }),
      this.on('upload_progress',    (p) => { setPhase('upload');   onUploadProgress(p);  }),

      // Data
      this.on('class_stats_updated', (p) => onClassStatsUpdated(p)),
      this.on('preset_synced',       (p) => onPresetSynced(p)),

      // Infrastructure
      this.on('db_ready',     (p) => { setDbReady(p.success, p.error); if (!p.success && p.error) pushLog({ ts: now(), tag: 'ERR', source: 'db', msg: p.error }); }),
      this.on('log_entry',    (p) => pushLog(p)),
      this.on('sync_loading', (p) => onSyncLoading(p)),

    ]);
  }

  // ── Cleanup ──────────────────────────────────────────────────

  destroy(): void {
    this.unlisten.forEach(fn => fn());
    this.unlisten = [];
  }
}

const now = () => Math.floor(Date.now() / 1000);

export const eventBus = new ScrapperEventBus();
