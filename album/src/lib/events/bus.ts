import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { RustEventMap, RustEventName } from './types';
import { setDbReady, onPresetUploaded, setListenerConnected } from '../../features/beauty/state/beauty.svelte';
import { onFaceGridProgress, onFaceApplyDone } from '../../features/face_grid/state/face_grid.svelte';
import { getPreset } from '../album';
import type { PresetEntry } from '../album';

class AlbumEventBus {
  private unlisten: UnlistenFn[] = [];

  private async on<K extends RustEventName>(
    event: K,
    handler: (payload: RustEventMap[K]) => void,
  ): Promise<void> {
    const fn = await listen<RustEventMap[K]>(event, ({ payload }) => handler(payload));
    this.unlisten.push(fn);
  }

  // ── Init ─────────────────────────────────────────────────────

  async init(): Promise<void> {
    // Register listeners first, then poll current state
    // (events may have fired before the frontend was ready)
    await Promise.all([
      this.on('db_ready',        (p) => setDbReady(p.success, p.error)),
      this.on('preset_uploaded', async (p) => {
        try {
          const full = await getPreset(String(p.preset_id));
          if (full) { onPresetUploaded(full); return; }
        } catch { /* fall through to sparse fallback */ }
        const sparse: PresetEntry = {
          preset_id:      String(p.preset_id),
          class_id:       p.class_id,
          title:          null,
          user_nickname:  null,
          character_name: null,
          image_1_url:    p.image_1_url,
          image_2_url:    p.image_2_url,
          pab_url:        null,
          has_pab:        false,
          downloads:      null,
          views:          null,
          likes:          null,
          is_discarded:   false,
          is_wanted:      false,
          creation_at:    null,
          updated_at:     null,
        };
        onPresetUploaded(sparse);
      }),
      this.on('listener_status',    (p) => setListenerConnected(p.connected)),
      this.on('face_grid_progress', (p) => onFaceGridProgress(p)),
      this.on('face_apply_done',    (p) => onFaceApplyDone(p)),
    ]);
    const alreadyReady = await invoke<boolean>('is_db_ready');
    if (alreadyReady) {
      setDbReady(true, null);
      const connected = await invoke<boolean>('is_listener_connected');
      setListenerConnected(connected);
    }
  }

  // ── Cleanup ──────────────────────────────────────────────────

  destroy(): void {
    this.unlisten.forEach(fn => fn());
    this.unlisten = [];
  }
}

export const eventBus = new AlbumEventBus();
