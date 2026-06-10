<script lang="ts">
  import './Logs.scss';
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$ui/index';
  import { getLogs, clearLogs, prependLogs } from '../scrapper/state/scrapper.svelte';

  const TAG_COLORS: Record<string, string> = {
    ERR:'#f87171', WARN:'#fbbf24', FETCH:'#06b6d4', SYNC:'#2dd4bf',
    ORCH:'#a78bfa', USER:'#60a5fa', INFO:'#94a3b8', UPDT:'#34d399',
  };

  const TAG_OPTIONS = ['ALL', 'ERR', 'WARN', 'ORCH', 'FETCH', 'SYNC', 'UPDT', 'USER', 'INFO'];

  const PAGE_SIZE = 100;

  let filterTag    = $state('ALL');
  let filterSearch = $state('');
  let page         = $state(1);

  const logs = $derived(getLogs());

  const filtered = $derived(() => {
    const q = filterSearch.toLowerCase().trim();
    return logs.filter(l => {
      if (filterTag !== 'ALL' && l.tag !== filterTag) return false;
      if (q && !l.msg.toLowerCase().includes(q) && !l.tag.toLowerCase().includes(q)) return false;
      return true;
    });
  });

  const totalPages = $derived(Math.max(1, Math.ceil(filtered().length / PAGE_SIZE)));
  const paginated  = $derived(filtered().slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE));

  const hasFilters = $derived(filterTag !== 'ALL' || filterSearch !== '');

  $effect(() => {
    filterTag; filterSearch;
    page = 1;
  });

  function reset() { filterTag = 'ALL'; filterSearch = ''; }

  const fmt = (ts: number) => new Date(ts * 1000).toLocaleTimeString('es', { hour12: false });

  onMount(async () => {
    try {
      const rows = await invoke<{ ts: number; tag: string; source: string; msg: string }[]>('get_logs', { limit: 500 });
      if (rows.length > 0) {
        prependLogs(rows.map(r => ({ ts: Math.floor(r.ts / 1000), tag: r.tag, source: r.source, msg: r.msg })));
      }
    } catch (_) {}
  });
</script>

<div class="logs-view">
  <div class="logs-main">

    <!-- ── Header ── -->
    <header class="logs-header">
      <span class="logs-title">Logs</span>
      <span class="logs-count-badge">
        {filtered().length}<span class="count-sep">/</span>{logs.length}
      </span>
      {#if hasFilters}
        <span class="logs-filtered-hint">filtered</span>
        <button class="logs-reset-btn" onclick={reset}>✕ reset</button>
      {/if}
      <div class="logs-header-spacer"></div>
      {#if logs.length > 0}
        <Button variant="ghost" onclick={clearLogs}>Clear</Button>
      {/if}
    </header>

    <!-- ── Tag chips + search toolbar ── -->
    <div class="logs-toolbar">
      {#each TAG_OPTIONS as tag}
        <button
          class="tag-chip"
          class:active={filterTag === tag}
          style="--tc:{TAG_COLORS[tag] ?? 'var(--color-text-muted)'}"
          onclick={() => filterTag = tag}
        >
          {tag}
        </button>
      {/each}
      <div class="toolbar-spacer"></div>
      <input
        class="toolbar-search"
        type="text"
        placeholder="Filter messages…"
        bind:value={filterSearch}
      />
    </div>

    <!-- ── Log list ── -->
    <div class="logs-list-col">
      <div class="logs-list">
        {#if filtered().length === 0}
          <div class="logs-empty">
            <span class="empty-label">{logs.length === 0 ? 'no logs yet' : 'no logs match filters'}</span>
          </div>
        {:else}
          {#each paginated as log (log._uid)}
            <div class="log-row" data-tag={log.tag} title={new Date(log.ts * 1000).toLocaleString('es')}
              in:fly={{ y: -6, duration: 180, opacity: 0 }}>
              <span class="log-ts">{fmt(log.ts)}</span>
              <span class="log-tag-badge" style="--tc:{TAG_COLORS[log.tag] ?? '#475569'}">{log.tag}</span>
              <span class="log-msg">{log.msg}<span class="log-source"> · {log.source}</span></span>
              <button class="log-copy" onclick={() => navigator.clipboard.writeText(log.msg)} title="Copy">⎘</button>
            </div>
          {/each}
        {/if}
      </div>

      {#if totalPages > 1}
        <div class="logs-pagination">
          <span class="page-range">
            {(page - 1) * PAGE_SIZE + 1}–{Math.min(page * PAGE_SIZE, filtered().length)}
            <span class="page-total">of {filtered().length}</span>
          </span>
          <div class="page-controls">
            <button class="page-btn" disabled={page === 1} onclick={() => page = 1} title="First">«</button>
            <button class="page-btn" disabled={page === 1} onclick={() => page--} title="Previous">‹</button>
            <span class="page-current">{page} / {totalPages}</span>
            <button class="page-btn" disabled={page === totalPages} onclick={() => page++} title="Next">›</button>
            <button class="page-btn" disabled={page === totalPages} onclick={() => page = totalPages} title="Last">»</button>
          </div>
        </div>
      {/if}
    </div>

  </div>
</div>
