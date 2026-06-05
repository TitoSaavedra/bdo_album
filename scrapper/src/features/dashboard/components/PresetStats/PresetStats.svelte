<script lang="ts">
  import './PresetStats.scss';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { CLASSES } from '$lib/classes';
  import { getClassIcons, getStatus } from '../../../scrapper/state/scrapper.svelte';

  interface Stats {
    total:       number;
    with_images: number;
    not_found:   number;
    pending:     number;
    by_class:    { class_id: number; total: number; with_images: number }[];
  }

  let stats = $state<Stats | null>(null);

  const classIcons = $derived(getClassIcons());
  const status     = $derived(getStatus());

  const classMap = Object.fromEntries(CLASSES.map(c => [c.id, c.name]));

  async function load() {
    try {
      stats = await invoke<Stats>('get_preset_stats');
    } catch {}
  }

  onMount(load);

  // Refresh when scraper finishes
  $effect(() => {
    if (status === 'done' || status === 'cancelled') load();
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
        <div class="ps-bar-fill" style="width: {pct(stats.with_images, stats.total)}%"></div>
      </div>
      <span class="ps-bar-label">{pct(stats.with_images, stats.total)}% images ready</span>
    </div>

    <!-- ── Per class ── -->
    <div class="ps-class-list">
      {#each stats.by_class as row (row.class_id)}
        {@const name = classMap[row.class_id] ?? `Class ${row.class_id}`}
        {@const imgPct = pct(row.with_images, row.total)}
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
            <div class="ps-class-bar" style="width: {pct(row.total, stats.total) * 2}%">
              <div class="ps-class-bar-img" style="width: {imgPct}%"></div>
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
