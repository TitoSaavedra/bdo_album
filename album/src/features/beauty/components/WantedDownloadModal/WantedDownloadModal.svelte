<script lang="ts">
  import { untrack } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import type { PresetEntry } from '../../../../lib/album';
  import { openUrl, queueAutoDownload, queuePabUploads, pabUploadStatus } from '../../../../lib/album';
  import { beauty } from '../../state/beauty.svelte';

  interface Props {
    presets: PresetEntry[];
    onclose: () => void;
  }

  const { presets, onclose }: Props = $props();

  const PAGE_SIZE = 16; // 4×4

  let items    = $state(untrack(() => [...presets]));
  let page     = $state(0);
  let selected = $state(new Set<string>());
  let sending  = $state(false);

  // .pab drag-and-drop staging — the scraper validates and uploads them.
  // `uploading` covers reading + staging the files; after that the staged row
  // ids stay in `tracked` until the scraper purges them (done) or sets `error`.
  let isDragging  = $state(false);
  let uploading   = $state(false);
  let pabFailed   = $state(false);
  let tracked     = $state<number[]>([]);
  let pabTotal    = $state(0);
  let pabDone     = $state(0);
  let pabRejected = $state<string[]>([]);

  const processing = $derived(tracked.length > 0);

  const POLL_MS = 1500;

  // Follow staged rows while any are pending; restarts only on the
  // processing on/off transition, not on every poll.
  $effect(() => {
    if (!processing) return;
    let inFlight = false;
    const timer = setInterval(async () => {
      if (inFlight) return;
      inFlight = true;
      const snapshot = [...tracked];
      try {
        const rows = await pabUploadStatus(snapshot);
        const still  = new Set(rows.filter(r => r.error == null).map(r => r.id));
        const failed = rows.filter(r => r.error != null);
        const gone   = snapshot.filter(id => !still.has(id) && !failed.some(r => r.id === id));
        if (gone.length === 0 && failed.length === 0) return;
        const resolved = new Set([...gone, ...failed.map(r => r.id)]);
        pabDone    += gone.length;
        pabRejected = [...pabRejected, ...failed.map(r => r.error as string)];
        tracked     = tracked.filter(id => !resolved.has(id));
      } catch (_) {
        // transient DB hiccup — next tick retries
      } finally {
        inFlight = false;
      }
    }, POLL_MS);
    return () => clearInterval(timer);
  });

  // Window-level Tauri drag events; only registered while this modal is mounted.
  $effect(() => {
    let unlisteners: UnlistenFn[] = [];
    let disposed = false;
    Promise.all([
      listen<{ paths: string[] }>('tauri://drag-enter', () => { isDragging = true; }),
      listen<{ paths: string[] }>('tauri://drag-drop',  (e) => { isDragging = false; uploadPabPaths(e.payload.paths); }),
      listen<void>('tauri://drag-leave',                 () => { isDragging = false; }),
    ]).then(fns => {
      if (disposed) fns.forEach(fn => fn());
      else unlisteners = fns;
    });
    return () => {
      disposed = true;
      unlisteners.forEach(fn => fn());
      unlisteners = [];
      isDragging = false;
    };
  });

  // A preset leaves the wanted list once its PAB lands: preset_uploaded
  // (bus.ts) patches the fresh copy into beauty.livePresets with has_pab set.
  $effect(() => {
    const uploaded = new Set<string>();
    for (const list of Object.values(beauty.livePresets)) {
      for (const p of list) if (p.has_pab) uploaded.add(p.preset_id);
    }
    if (uploaded.size === 0) return;
    untrack(() => {
      if (!items.some(p => uploaded.has(p.preset_id))) return;
      items = items.filter(p => !uploaded.has(p.preset_id));
      selected = new Set([...selected].filter(id => !uploaded.has(id)));
      clampPage();
    });
  });

  async function uploadPabPaths(paths: string[]) {
    const pabs = paths.filter(p => p.toLowerCase().endsWith('.pab'));
    if (pabs.length === 0 || uploading) return;
    uploading = true;
    pabFailed = false;
    if (!processing) {
      pabTotal    = 0;
      pabDone     = 0;
      pabRejected = [];
    }
    try {
      const ids = await queuePabUploads(pabs);
      if (ids.length > 0) {
        pabTotal += ids.length;
        tracked   = [...tracked, ...ids];
      } else {
        pabFailed = true;
      }
    } catch (_) {
      pabFailed = true;
    } finally {
      uploading = false;
    }
  }

  const totalPages = $derived(Math.max(1, Math.ceil(items.length / PAGE_SIZE)));
  const pageItems  = $derived(items.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE));

  const activeClasses = $derived.by(() => {
    const map = new Map<number, { name: string; icon_svg: string | null; count: number }>();
    for (const p of items) {
      const entry = beauty.classes.find(c => c.class_id === p.class_id);
      if (!entry) continue;
      const existing = map.get(p.class_id);
      if (existing) existing.count++;
      else map.set(p.class_id, { name: entry.name, icon_svg: entry.icon_svg, count: 1 });
    }
    return [...map.entries()].map(([class_id, info]) => ({ class_id, ...info }));
  });

  function clampPage() {
    const maxPage = Math.max(0, Math.ceil(items.length / PAGE_SIZE) - 1);
    if (page > maxPage) page = maxPage;
  }

  function remove(id: string) {
    items = items.filter(p => p.preset_id !== id);
    selected.delete(id);
    clampPage();
  }

  function removeByClass(classId: number) {
    const removedIds = new Set(items.filter(p => p.class_id === classId).map(p => p.preset_id));
    items = items.filter(p => p.class_id !== classId);
    selected = new Set([...selected].filter(id => !removedIds.has(id)));
    clampPage();
  }

  function toggleSelect(id: string) {
    const next = new Set(selected);
    if (next.has(id)) next.delete(id); else next.add(id);
    selected = next;
  }

  async function sendToAutoDownload() {
    if (selected.size === 0 || sending) return;
    sending = true;
    try {
      const ids = [...selected];
      await queueAutoDownload(ids);
      const now = Date.now() / 1000;
      items = items.map(p =>
        ids.includes(p.preset_id)
          ? { ...p, auto_download_requested_at: now, auto_download_error: null }
          : p
      );
      selected = new Set();
    } catch (_) {
      // surfaced via the queued badge staying unset; user can retry
    } finally {
      sending = false;
    }
  }

  function prevPage() { if (page > 0) page--; }
  function nextPage() { if (page < totalPages - 1) page++; }

  async function openAll() {
    for (const p of items) {
      await openUrl(`https://garmoth.com/beauty-album/preset/${p.preset_id}`);
    }
    onclose();
  }
</script>

<div
  class="backdrop"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === 'Escape' && onclose()}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >

    <div class="modal-header">
      <span class="modal-title">{$_('beauty.wanted_modal.title')}</span>
      <span class="modal-count">{items.length}</span>
      <button class="close-btn" onclick={onclose}>✕</button>
    </div>

    {#if activeClasses.length > 0}
      <div class="class-pills">
        {#each activeClasses as cls (cls.class_id)}
          <div class="class-pill">
            {#if cls.icon_svg}
              <span class="pill-icon">{@html cls.icon_svg}</span>
            {/if}
            <span class="pill-name">{cls.name}</span>
            <span class="pill-count">{cls.count}</span>
            <button
              class="pill-remove"
              onclick={() => removeByClass(cls.class_id)}
              title={$_('beauty.wanted_modal.remove_class', { values: { class: cls.name } })}
            >✕</button>
          </div>
        {/each}
      </div>
    {/if}

    <div class="pab-drop" class:dragging={isDragging} class:busy={uploading || processing}>
      {#if uploading || processing}
        <span class="pab-drop-spinner" aria-hidden="true"></span>
      {:else}
        <span class="pab-drop-icon">⬇</span>
      {/if}
      <span class="pab-drop-text">
        <span class="pab-drop-title">{$_('beauty.wanted_modal.pab_drop_title')}</span>
        <span class="pab-drop-hint">{$_('beauty.wanted_modal.pab_drop_hint')}</span>
      </span>
      {#if uploading}
        <span class="pab-drop-result busy">{$_('beauty.wanted_modal.pab_sending')}</span>
      {:else if processing}
        <span class="pab-drop-result busy">
          {$_('beauty.wanted_modal.pab_processing', { values: { done: pabDone + pabRejected.length, total: pabTotal } })}
        </span>
      {:else if pabFailed}
        <span class="pab-drop-result fail">{$_('beauty.wanted_modal.pab_failed')}</span>
      {:else if pabTotal > 0}
        {#if pabDone > 0}
          <span class="pab-drop-result ok">{$_('beauty.wanted_modal.pab_done', { values: { count: pabDone } })}</span>
        {/if}
        {#if pabRejected.length > 0}
          <span class="pab-drop-result fail" title={pabRejected.join('
')}>
            {$_('beauty.wanted_modal.pab_rejected', { values: { count: pabRejected.length } })}
          </span>
        {/if}
      {/if}
    </div>

    {#if items.length === 0}
      <div class="empty">{$_('beauty.wanted_modal.empty')}</div>
    {:else}
      <div class="select-hint">{$_('beauty.wanted_modal.select_hint')}</div>
      <div class="grid">
        {#each pageItems as p (p.preset_id)}
          {@const img      = p.image_1_url ?? p.image_2_url}
          {@const label    = p.title || p.character_name || `#${p.preset_id}`}
          {@const isSel    = selected.has(p.preset_id)}
          {@const isQueued = !!p.auto_download_requested_at && !p.auto_download_error}
          {@const isFailed = !!p.auto_download_error}
          <div class="card">
            <div
              class="thumb-wrap"
              class:selected={isSel}
              role="button"
              tabindex="0"
              onclick={() => toggleSelect(p.preset_id)}
              onkeydown={(e) => e.key === 'Enter' && toggleSelect(p.preset_id)}
            >
              {#if img}
                <img src={img} alt={label} class="thumb" loading="lazy" />
              {:else}
                <div class="thumb-placeholder"></div>
              {/if}
              {#if isSel}
                <span class="select-check">✓</span>
              {/if}
              {#if isQueued}
                <span class="status-badge queued">{$_('beauty.wanted_modal.queued_badge')}</span>
              {:else if isFailed}
                <span class="status-badge failed" title={p.auto_download_error ?? ''}>{$_('beauty.wanted_modal.error_badge')}</span>
              {/if}
              <button
                class="remove-btn"
                onclick={(e) => { e.stopPropagation(); remove(p.preset_id); }}
                title={$_('beauty.wanted_modal.skip_this')}
              >✕</button>
            </div>
            <div class="card-label" title={label}>{label}</div>
          </div>
        {/each}
      </div>
    {/if}

    {#if totalPages > 1}
      <div class="pagination">
        <button class="page-btn" onclick={prevPage} disabled={page === 0}>‹</button>
        <span class="page-info">{page + 1} / {totalPages}</span>
        <button class="page-btn" onclick={nextPage} disabled={page === totalPages - 1}>›</button>
      </div>
    {/if}

    <div class="modal-footer">
      <button class="cancel-btn" onclick={onclose}>{$_('beauty.wanted_modal.cancel')}</button>
      <button
        class="queue-btn"
        onclick={sendToAutoDownload}
        disabled={selected.size === 0 || sending}
      >
        {$_('beauty.wanted_modal.send_to_auto_download', { values: { count: selected.size } })}
      </button>
      <button class="open-btn" onclick={openAll} disabled={items.length === 0}>
        {$_('beauty.wanted_modal.open_in_browser', { values: { count: items.length } })}
      </button>
    </div>

  </div>
</div>

<style lang="scss">
  @use './WantedDownloadModal.scss';
</style>
