<script lang="ts">
  import './PresetStats.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { CLASSES } from '$lib/classes';
  import { getClassIcons, getStatus, getDbReady } from '../../../scraper/state/scraper.svelte';

  interface Stats {
    total:       number;
    with_images: number;
    not_found:   number;
    pending:     number;
    by_class:    { class_id: number; total: number; with_images: number; not_found: number }[];
  }

  let stats = $state<Stats | null>(null);

  const classIcons = $derived(getClassIcons());
  const status     = $derived(getStatus());
  const dbReady    = $derived(getDbReady());

  const classMap = Object.fromEntries(CLASSES.map(c => [c.id, c.name]));

  let _poll: ReturnType<typeof setInterval> | null = null;

  async function load() {
    try {
      stats = await invoke<Stats>('get_preset_stats');
    } catch {}
  }

  function stopPoll() {
    if (_poll) { clearInterval(_poll); _poll = null; }
  }

  // Initial load once DB is ready
  $effect(() => {
    if (dbReady === true) load();
  });

  // Poll while running; 5s extra on done/cancelled; cleanup on unmount
  $effect(() => {
    const s = status;
    if (s === 'running') {
      stopPoll();
      _poll = setInterval(load, 1000);
    } else {
      stopPoll();
      if (s === 'done' || s === 'cancelled') {
        let ticks = 0;
        _poll = setInterval(() => { load(); if (++ticks >= 5) stopPoll(); }, 1000);
      }
    }
    return stopPoll;
  });

  const pct = (a: number, b: number) => b > 0 ? Math.round((a / b) * 100) : 0;
  const fmt = (n: number) => n.toLocaleString('es');
</script>

<div class="preset-stats">
  {#if !stats}
    <div class="ps-loading">Loading...</div>
  {:else}
    <!-- ── Global KPIs ── -->
    <div class="ps-kpis">
      <div class="ps-kpi" style="--kc: var(--color-accent)">
        <span class="ps-kpi-val">{fmt(stats.total)}</span>
        <span class="ps-kpi-label">Total Presets</span>
      </div>
      <div class="ps-kpi" style="--kc: var(--color-status-success)">
        <span class="ps-kpi-val">{fmt(stats.with_images)}</span>
        <span class="ps-kpi-label">With Images</span>
        <span class="ps-kpi-sub">{pct(stats.with_images, stats.total)}%</span>
      </div>
      <div class="ps-kpi" style="--kc: var(--color-accent-tertiary)">
        <span class="ps-kpi-val">{fmt(stats.pending)}</span>
        <span class="ps-kpi-label">Pending Images</span>
      </div>
      <div class="ps-kpi" style="--kc: var(--color-text-muted)">
        <span class="ps-kpi-val">{fmt(stats.not_found)}</span>
        <span class="ps-kpi-label">Not Found</span>
      </div>
    </div>

    <!-- ── Progress bar global ── -->
    <div class="ps-global-bar">
      <div class="ps-bar-track">
        <div class="ps-bar-fill" style="width: {pct(stats.with_images, stats.total - stats.not_found)}%"></div>
      </div>
      <span class="ps-bar-label">{pct(stats.with_images, stats.total - stats.not_found)}% images ready</span>
    </div>

    <!-- ── Per class ── -->
    <div class="ps-class-list">
      {#each stats.by_class as row (row.class_id)}
        {@const name   = classMap[row.class_id] ?? `Class ${row.class_id}`}
        {@const imgPct = pct(row.with_images, row.total - row.not_found)}
        <div class="ps-class-row">
          <div class="ps-class-icon">
            {#if classIcons[row.class_id]}
              {@html classIcons[row.class_id]}
            {:else}
              {name[0]}
            {/if}
          </div>
          <span class="ps-class-name">{name}</span>
          <div class="ps-class-bar-wrap">
            <div class="ps-class-bar">
              <div class="ps-class-bar-img" class:complete={imgPct === 100} style="width: {imgPct}%"></div>
            </div>
          </div>
          <span class="ps-class-total">{fmt(row.total)}</span>
          <span class="ps-class-img" class:zero={row.with_images === 0}>
            ↑{fmt(row.with_images)}
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>
