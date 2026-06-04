<script lang="ts">
  import './ActivityFeed.scss';
  import { getLogs } from '../../../scrapper/state/scrapper.svelte';

  const recent = $derived(getLogs().slice(0, 30));

  const TAG_COLOR: Record<string, string> = {
    ERR:      '#f87171',
    WARN:     '#fbbf24',
    POP:      '#2dd4bf',
    SYNC:     '#2dd4bf',
    ORCH:     '#a78bfa',
    PERSONAL: '#22d3ee',
    USER:     '#60a5fa',
    INFO:     '#94a3b8',
  };

  const fmt = (ts: number) =>
    new Date(ts * 1000).toLocaleTimeString('es', { hour12: false });
</script>

<div class="feed">
  {#if recent.length === 0}
    <div class="feed-empty">
      <span class="feed-empty-icon">⚡</span>
      <span>Waiting for activity...</span>
    </div>
  {:else}
    {#each recent as entry (entry.ts + entry.tag + entry.msg)}
      <div class="feed-item">
        <span class="feed-time">{fmt(entry.ts)}</span>
        <span class="feed-tag" style="color: {TAG_COLOR[entry.tag] ?? '#475569'}">{entry.tag}</span>
        <span class="feed-msg">{entry.msg}</span>
      </div>
    {/each}
  {/if}
</div>
