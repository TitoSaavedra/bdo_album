<script lang="ts">
  import './Logs.scss';
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Sidebar } from '$ui/index';
  import { getLogs, clearLogs, prependLogs } from '../scrapper/state/scrapper.svelte';

  const TAG_COLORS: Record<string, string> = {
    ERR:'#f87171', WARN:'#fbbf24', FETCH:'#06b6d4', SYNC:'#2dd4bf',
    ORCH:'#a78bfa', USER:'#60a5fa', INFO:'#94a3b8',
  };

  const TAG_OPTIONS = ['ALL', 'ERR', 'WARN', 'ORCH', 'FETCH', 'SYNC', 'USER', 'INFO'];
  const TIME_OPTIONS = [
    { value: 'all', label: 'All time' },
    { value: '60',  label: 'Last 1h'  },
    { value: '30',  label: 'Last 30m' },
    { value: '5',   label: 'Last 5m'  },
    { value: '1',   label: 'Last 1m'  },
  ];

  const PAGE_SIZE = 100;

  let sidebarOpen  = $state(false);
  let filterTag    = $state('ALL');
  let filterTime   = $state('all');
  let filterSource = $state('all');
  let filterSearch = $state('');
  let page         = $state(1);

  const logs    = $derived(getLogs());
  const sources = $derived(['all', ...new Set(logs.map(l => l.source).filter(Boolean))]);

  const filtered = $derived(() => {
    const now  = Math.floor(Date.now() / 1000);
    const mins = filterTime !== 'all' ? parseInt(filterTime) : null;
    const q    = filterSearch.toLowerCase().trim();
    return logs.filter(l => {
      if (filterTag !== 'ALL' && l.tag !== filterTag)          return false;
      if (filterSource !== 'all' && l.source !== filterSource) return false;
      if (mins && now - l.ts > mins * 60)                      return false;
      if (q && !l.msg.toLowerCase().includes(q) && !l.tag.toLowerCase().includes(q)) return false;
      return true;
    });
  });

  const totalPages = $derived(Math.max(1, Math.ceil(filtered().length / PAGE_SIZE)));
  const paginated  = $derived(filtered().slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE));

  const hasFilters = $derived(
    filterTag !== 'ALL' || filterTime !== 'all' || filterSource !== 'all' || filterSearch !== ''
  );

  $effect(() => {
    filterTag; filterTime; filterSource; filterSearch;
    page = 1;
  });

  function reset() { filterTag = 'ALL'; filterTime = 'all'; filterSource = 'all'; filterSearch = ''; }

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

    <!-- ── Body: sidebar (time/source) + log list ── -->
    <div class="logs-body">

      <Sidebar bind:open={sidebarOpen} width={180}>
        <div class="logs-nav">

          <div class="nav-section">
            <span class="nav-label">Time</span>
            <select class="nav-select" bind:value={filterTime}>
              {#each TIME_OPTIONS as t}
                <option value={t.value}>{t.label}</option>
              {/each}
            </select>
          </div>

          <div class="nav-section">
            <span class="nav-label">Source</span>
            <select class="nav-select" bind:value={filterSource}>
              {#each sources as src}
                <option value={src}>{src === 'all' ? 'All sources' : src}</option>
              {/each}
            </select>
          </div>

          <div class="nav-divider"></div>

          <div class="nav-section">
            <span class="nav-label">Sources</span>
            {#each sources.filter(s => s !== 'all') as src}
              <button
                class="source-item"
                class:active={filterSource === src}
                onclick={() => filterSource = filterSource === src ? 'all' : src}
              >
                <span class="source-dot" style="background:{TAG_COLORS[src.toUpperCase()] ?? 'var(--color-text-muted)'}"></span>
                <span class="source-name">{src}</span>
                <span class="source-count">{logs.filter(l => l.source === src).length}</span>
              </button>
            {:else}
              <span class="source-empty">No sources yet</span>
            {/each}
          </div>

        </div>
      </Sidebar>

      <!-- ── Log entries + pagination ── -->
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
</div>
