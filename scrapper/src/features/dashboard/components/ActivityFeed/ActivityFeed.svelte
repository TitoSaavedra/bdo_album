<script lang="ts">
  import './ActivityFeed.scss';
  import { getSessionLogs } from '../../../scrapper/state/scrapper.svelte';

  // Each row: 4px+4px padding + 11px*1.5 line-height + 1px border ≈ 24px rendered
  const ITEM_H = 24;

  let containerH = $state(0);
  const maxVisible = $derived(containerH > 0 ? Math.max(1, Math.floor(containerH / ITEM_H)) : 20);
  const recent = $derived(getSessionLogs().slice(0, maxVisible));

  const TAG_COLOR: Record<string, string> = {
    ERR:      '#f87171',
    WARN:     '#fbbf24',
    POP:      '#2dd4bf',
    SYNC:     '#2dd4bf',
    UPDT:     '#34d399',
    ORCH:     '#a78bfa',
    PERSONAL: '#22d3ee',
    USER:     '#60a5fa',
    INFO:     '#94a3b8',
  };

  const fmt = (ts: number) =>
    new Date(ts * 1000).toLocaleTimeString('es', { hour12: false });
</script>

<div class="feed" bind:clientHeight={containerH}>
  {#if recent.length === 0}
    <div class="feed-empty">
      <div class="feed-empty-icon">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
        </svg>
      </div>
      <span class="feed-empty-text">Waiting for activity<span class="feed-dots"><span>.</span><span>.</span><span>.</span></span></span>
    </div>
  {:else}
    {#each recent as entry (entry._uid)}
      <div class="feed-item">
        <span class="feed-time">{fmt(entry.ts)}</span>
        <span class="feed-tag" style="color: {TAG_COLOR[entry.tag] ?? '#475569'}">{entry.tag}</span>
        <span class="feed-msg">{entry.msg}</span>
      </div>
    {/each}
  {/if}
</div>
