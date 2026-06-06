import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { RustEventMap, RustEventName } from './types';
import { setDbReady } from '../../features/beauty/state/beauty.svelte';

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
    // Register listeners first, then check if DB is already ready
    // (event may have fired before the listener was registered)
    await Promise.all([
      this.on('db_ready', (p) => setDbReady(p.success, p.error)),
    ]);
    const alreadyReady = await invoke<boolean>('is_db_ready');
    if (alreadyReady) setDbReady(true, null);
  }

  // ── Cleanup ──────────────────────────────────────────────────

  destroy(): void {
    this.unlisten.forEach(fn => fn());
    this.unlisten = [];
  }
}

export const eventBus = new AlbumEventBus();
