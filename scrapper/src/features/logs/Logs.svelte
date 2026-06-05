<script lang="ts">
  import './Logs.scss';
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Sidebar } from '$ui/index';
  import { getLogs, clearLogs, prependLogs } from '../scrapper/state/scrapper.svelte';
  import type { LogEntry } from '$lib/events/types';

  const TAG_COLORS: Record<string, string> = {
    ERR:'#f87171', WARN:'#fbbf24', POP:'#2dd4bf', SYNC:'#2dd4bf',
    ORCH:'#a78bfa', PERSONAL:'#22d3ee', USER:'#60a5fa', INFO:'#94a3b8',
  };

  const TAG_OPTIONS = ['ALL', 'ERR', 'WARN', 'ORCH', 'SYNC', 'POP', 'PERSONAL', 'USER', 'INFO'];
  const TIME_OPTIONS = [
    { value: 'all', label: 'All time' },
    { value: '60',  label: 'Last 1h'  },
    { value: '30',  label: 'Last 30m' },
    { value: '5',   label: 'Last 5m'  },
    { value: '1',   label: 'Last 1m'  },
  ];

  let sidebarOpen  = $state(false);
  let filterTag    = $state('ALL');
  let filterTime   = $state('all');
  let filterSource = $state('all');
  let filterSearch = $state('');

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

  const hasFilters = $derived(
    filterTag !== 'ALL' || filterTime !== 'all' || filterSource !== 'all' || filterSearch !== ''
  );

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

  <!-- ── Left nav sidebar ── -->
  <Sidebar bind:open={sidebarOpen} width={200}>
  <div class="logs-nav">

    <div class="nav-section">
      <span class="nav-label">Search</span>
      <input
        class="nav-search"
        type="text"
        placeholder="Filter messages..."
        bind:value={filterSearch}
      />
    </div>

    <div class="nav-section">
      <span class="nav-label">Tag</span>
      <select class="nav-select" bind:value={filterTag}>
        {#each TAG_OPTIONS as t}
          <option value={t}>{t === 'ALL' ? 'All tags' : t}</option>
        {/each}
      </select>
    </div>

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

    <div class="nav-footer">
      {#if hasFilters}
        <button class="btn-reset" onclick={reset}>✕ Reset filters</button>
      {/if}
      {#if logs.length > 0}
        <Button variant="ghost" onclick={clearLogs}>Clear logs</Button>
      {/if}
    </div>

  </div>
  </Sidebar>

  <!-- ── Main: header + entries ── -->
  <div class="logs-main">

    <header class="logs-header">
      <span class="logs-title">Logs</span>
      <span class="logs-count-badge">
        {filtered().length}<span class="count-sep">/</span>{logs.length}
      </span>
      <span class="logs-hint">
        {#if hasFilters}filtered{:else}all entries{/if}
      </span>
    </header>

    <div class="logs-list">
      {#if filtered().length === 0}
        <div class="logs-empty">
          <span class="empty-icon">📋</span>
          <span>{logs.length === 0 ? 'No logs yet' : 'No logs match the filters'}</span>
        </div>
      {:else}
        {#each filtered() as log (log._uid)}
          <div class="log-row" data-tag={log.tag} title={new Date(log.ts * 1000).toLocaleString('es')}
            in:fly={{ y: -6, duration: 180, opacity: 0 }}>
            <span class="log-ts">{fmt(log.ts)}</span>
            <span class="log-tag-badge" style="--tc:{TAG_COLORS[log.tag] ?? '#475569'}">{log.tag}</span>
            <span class="log-source">{log.source}</span>
            <span class="log-msg">{log.msg}</span>
            <button class="log-copy" onclick={() => navigator.clipboard.writeText(log.msg)} title="Copy">⎘</button>
          </div>
        {/each}
      {/if}
    </div>

    <footer class="logs-statusbar">
      <span class="statusbar-dot" class:active={logs.length > 0}></span>
      <span class="statusbar-text">
        {filtered().length} entries shown{#if hasFilters} · filtered{/if}{#if logs.length > 0} · {logs.length} total{/if}
      </span>
    </footer>

  </div>
</div>
