<script lang="ts">
  import './ClassChart.scss';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { CLASSES } from '$lib/classes';
  import { getClassIcons, getClassMap, getStatus } from '../../../scrapper/state/scrapper.svelte';

  interface DbClass { class_id: number; total: number; with_images: number; }

  const classIcons = $derived(getClassIcons());
  const classMap   = $derived(getClassMap());
  const status     = $derived(getStatus());

  let dbByClass = $state<Record<number, DbClass>>({});

  async function load() {
    try {
      const stats: { by_class: DbClass[] } = await invoke('get_preset_stats');
      dbByClass = Object.fromEntries(stats.by_class.map(r => [r.class_id, r]));
    } catch {}
  }

  onMount(load);
  $effect(() => { if (status === 'done' || status === 'cancelled') load(); });

  const rows = $derived(() => {
    const list = CLASSES.map(c => {
      const db      = dbByClass[c.id];
      const session = classMap[c.id];
      // DB total + anything added this session not yet in DB
      const total      = db?.total       ?? 0;
      const withImages = db?.with_images ?? 0;
      const pending    = total - withImages;
      return {
        id:       c.id,
        name:     c.name,
        total,
        withImages,
        pending,
        active:   session?.active ?? false,
      };
    });

    const max = Math.max(...list.map(r => r.total), 1);
    return list
      .map(r => ({ ...r, pct: Math.round((r.total / max) * 100) }))
      .sort((a, b) => {
        if (a.active !== b.active) return a.active ? -1 : 1;
        return b.total - a.total;
      });
  });
</script>

<div class="chart">
  {#each rows() as row (row.id)}
    <div class="chart-row" class:active={row.active}>

      <!-- Name + icon -->
      <div class="chart-name">
        <span class="chart-label">{row.name}</span>
        {#if classIcons[row.id]}
          <span class="chart-icon">{@html classIcons[row.id]}</span>
        {/if}
      </div>

      <!-- Bar -->
      <div class="chart-bar-wrap">
        <div class="chart-bar" style="width: {row.pct}%">
          {#if row.withImages > 0 && row.total > 0}
            <div class="chart-bar-images"
              style="width: {Math.round((row.withImages / row.total) * 100)}%">
            </div>
          {/if}
        </div>
      </div>

      <!-- Values -->
      <div class="chart-vals">
        <span class="chart-total">{row.total > 0 ? row.total.toLocaleString('es') : '—'}</span>
        {#if row.pending > 0}
          <span class="chart-pending">⏳{row.pending.toLocaleString('es')}</span>
        {:else if row.withImages > 0}
          <span class="chart-done">✓</span>
        {/if}
      </div>

    </div>
  {/each}
</div>
