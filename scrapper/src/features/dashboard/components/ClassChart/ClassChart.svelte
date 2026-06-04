<script lang="ts">
  import './ClassChart.scss';
  import { getClassMap } from '../../../scrapper/state/scrapper.svelte';
  import { CLASSES } from '$lib/classes';

  const classMap = $derived(getClassMap());

  const rows = $derived(() => {
    const list = CLASSES.map(c => ({
      name:   c.name,
      fetched: classMap[c.id]?.fetched ?? 0,
      images:  classMap[c.id]?.images  ?? 0,
      errors:  0,
      active:  classMap[c.id]?.active  ?? false,
    }));

    const max = Math.max(...list.map(r => r.fetched), 1);
    return list
      .map(r => ({ ...r, pct: Math.round((r.fetched / max) * 100) }))
      .sort((a, b) => b.fetched - a.fetched);
  });
</script>

<div class="chart">
  {#each rows() as row (row.name)}
    <div class="chart-row" class:active={row.active}>
      <span class="chart-label">{row.name}</span>
      <div class="chart-bar-wrap">
        <div class="chart-bar" style="width: {row.pct}%">
          {#if row.images > 0}
            <div class="chart-bar-images" style="width: {row.fetched > 0 ? Math.round((row.images / row.fetched) * 100) : 0}%"></div>
          {/if}
        </div>
      </div>
      <span class="chart-val">{row.fetched > 0 ? row.fetched : '—'}</span>
    </div>
  {/each}
</div>
