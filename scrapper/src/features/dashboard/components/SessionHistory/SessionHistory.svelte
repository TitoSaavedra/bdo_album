<script lang="ts">
  import './SessionHistory.scss';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { CLASSES } from '$lib/classes';
  import { getStatus } from '../../../scrapper/state/scrapper.svelte';

  interface Session {
    id:             number;
    started_at:     number;
    finished_at:    number | null;
    status:         'running' | 'done' | 'error' | 'cancelled';
    total_fetched:  number;
    total_images:   number;
    total_uploaded: number;
    errors:         number;
    elapsed_secs:   number | null;
    cf_used:        boolean;
  }

  interface ClassStat {
    class_id:  number;
    fetched:   number;
    images_ok: number;
    errors:    number;
  }

  const CLASS_NAME: Record<number, string> = Object.fromEntries(CLASSES.map(c => [c.id, c.name]));

  let sessions      = $state<Session[]>([]);
  let expandedId    = $state<number | null>(null);
  let expandedStats = $state<ClassStat[]>([]);
  let loadingStats  = $state(false);

  const scrappingStatus = $derived(getStatus());

  async function loadSessions() {
    sessions = await invoke<Session[]>('get_sessions');
  }

  async function toggleRow(id: number) {
    if (expandedId === id) {
      expandedId = null;
      return;
    }
    expandedId    = id;
    loadingStats  = true;
    expandedStats = await invoke<ClassStat[]>('get_class_stats_cmd', { sessionId: id });
    loadingStats  = false;
  }

  $effect(() => {
    if (scrappingStatus === 'done' || scrappingStatus === 'cancelled') {
      loadSessions();
    }
  });

  onMount(loadSessions);

  function fmtDate(ms: number) {
    return new Date(ms).toLocaleString('es', {
      month: 'short', day: '2-digit',
      hour: '2-digit', minute: '2-digit', hour12: false,
    });
  }

  function fmtDuration(secs: number | null) {
    if (!secs) return '—';
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return m > 0 ? `${m}m ${s}s` : `${s}s`;
  }

  const maxFetched = $derived(
    expandedStats.length > 0 ? Math.max(...expandedStats.map(c => c.fetched)) : 1
  );
</script>

<div class="sessions-wrap">
  {#if sessions.length === 0}
    <div class="empty-state">
      <div class="empty-icon">◎</div>
      <span>No sessions yet</span>
      <span class="empty-hint">Run the scraper to see history here</span>
    </div>
  {:else}
    <table class="sh-table">
      <thead>
        <tr>
          <th class="col-expand"></th>
          <th class="col-id">#</th>
          <th class="col-date">Started</th>
          <th class="col-dur">Duration</th>
          <th class="col-status">Status</th>
          <th class="col-num">Fetched</th>
          <th class="col-num">Images</th>
          <th class="col-num">Uploaded</th>
          <th class="col-num">Errors</th>
        </tr>
      </thead>
      <tbody>
        {#each sessions as s (s.id)}
          <tr
            class="sh-row"
            class:is-expanded={expandedId === s.id}
            class:is-running={s.status === 'running'}
            class:is-error={s.status === 'error'}
            onclick={() => toggleRow(s.id)}
          >
            <td class="col-expand">
              <span class="chevron" class:open={expandedId === s.id}>›</span>
            </td>
            <td class="col-id">#{s.id}</td>
            <td class="col-date">{fmtDate(s.started_at)}</td>
            <td class="col-dur">{fmtDuration(s.elapsed_secs)}</td>
            <td class="col-status">
              <span class="status-dot dot-{s.status}"></span>
              <span class="status-label">{s.status}</span>
            </td>
            <td class="col-num accent">{s.total_fetched.toLocaleString()}</td>
            <td class="col-num">{s.total_images.toLocaleString()}</td>
            <td class="col-num">{s.total_uploaded.toLocaleString()}</td>
            <td class="col-num" class:col-err={s.errors > 0}>{s.errors}</td>
          </tr>

          {#if expandedId === s.id}
            <tr class="detail-row">
              <td colspan="9">
                <div class="detail-panel">
                  {#if loadingStats}
                    <div class="detail-loading">
                      <span class="spinner"></span>
                      Loading class breakdown...
                    </div>
                  {:else if expandedStats.length === 0}
                    <div class="detail-empty">No class data recorded for this session</div>
                  {:else}
                    <div class="detail-header">
                      <span class="detail-title">Class breakdown</span>
                      <span class="detail-count">{expandedStats.length} classes</span>
                    </div>
                    <div class="class-bars">
                      {#each expandedStats as cs (cs.class_id)}
                        {@const pct = maxFetched > 0 ? Math.round((cs.fetched / maxFetched) * 100) : 0}
                        <div class="class-bar-row">
                          <span class="cb-name">{CLASS_NAME[cs.class_id] ?? `#${cs.class_id}`}</span>
                          <div class="cb-track">
                            <div class="cb-fill" style="width: {pct}%"></div>
                          </div>
                          <span class="cb-val">{cs.fetched.toLocaleString()}</span>
                          {#if cs.errors > 0}
                            <span class="cb-err">⚠ {cs.errors}</span>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  {/if}
</div>
