<script lang="ts">
  import './Dashboard.scss';
  import { _ } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$ui/index';
  import StatCard      from './components/StatCard/StatCard.svelte';
  import ClassGrid     from './components/ClassGrid/ClassGrid.svelte';
  import ClassChart    from './components/ClassChart/ClassChart.svelte';
  import ActivityFeed  from './components/ActivityFeed/ActivityFeed.svelte';
  import SessionHistory from './components/SessionHistory/SessionHistory.svelte';

  type MainTab = 'overview' | 'classes' | 'sessions';
  let mainTab = $state<MainTab>('overview');
  import {
    getStatus, getPhase,
    getCurrent, getTotal,
    getProgress, getImgProgress, getUpProgress,
    getTotalFetched, getImagesDone, getImagesTotal,
    getUploadsDone, getErrors, getElapsed,
    requestStop, getDbReady,
  } from '../scrapper/state/scrapper.svelte';

  let parallelism = $state(3);
  let sidebarOpen = $state(true);

  const status   = $derived(getStatus());
  const phase    = $derived(getPhase());
  const pct      = $derived(getProgress());
  const imgPct   = $derived(getImgProgress());
  const upPct    = $derived(getUpProgress());
  const fetched  = $derived(getTotalFetched());
  const imgDone  = $derived(getImagesDone());
  const imgTotal = $derived(getImagesTotal());
  const upDone   = $derived(getUploadsDone());
  const errors   = $derived(getErrors());
  const secs     = $derived(getElapsed());
  const current  = $derived(getCurrent());
  const total    = $derived(getTotal());

  const dbReady = $derived(getDbReady());
  const isBusy  = $derived(status === 'running' || status === 'stopping');

  const PHASES = [
    { id: 'fetch',    label: 'Fetch JSON', color: 'var(--color-accent)'          },
    { id: 'download', label: 'Download',   color: 'var(--color-accent-secondary)' },
    { id: 'upload',   label: 'Upload R2',  color: 'var(--color-accent-tertiary)'  },
  ] as const;

  const fmtTime = (s: number) => {
    const m = Math.floor(s / 60);
    return m > 0 ? `${m}m ${s % 60}s` : `${s}s`;
  };

  async function start()  { await invoke('run_scraper', { parallelism }); }
  async function stop()   { requestStop(); await invoke('cancel_scraper'); }
</script>

<div class="dashboard">

  <!-- ── Top header ── -->
  <header class="dash-header">
    <div class="dash-header-left">
      <span class="dash-title">BDO Scrapper</span>
      <span class="status-pill status-{status}">{status}</span>
    </div>

    <!-- Phase pipeline -->
    <div class="phase-pipeline">
      {#each PHASES as p, i}
        <div class="pipe-step" class:active={phase === p.id} class:done={['fetch','download','upload'].indexOf(phase) > i} style="--c: {p.color}">
          <span class="pipe-dot"></span>
          <span class="pipe-label">{p.label}</span>
        </div>
        {#if i < 2}<div class="pipe-arrow">→</div>{/if}
      {/each}
    </div>

    <div class="dash-header-right">
      {#if status === 'running'}
        <Button variant="ghost" onclick={stop}>■ Stop</Button>
      {:else if status === 'stopping'}
        <Button variant="ghost" disabled>⏳ Stopping...</Button>
      {:else}
        <Button variant="primary" onclick={start} disabled={dbReady !== true || isBusy}>▶ Start</Button>
      {/if}
    </div>
  </header>

  <!-- ── Body ── -->
  <div class="dash-body">

    <!-- Left: sidebar -->
    <aside class="dash-sidebar" class:collapsed={!sidebarOpen}>
      <!-- Config -->
      <div class="sidebar-section">
        <span class="sidebar-label">Parallelism — <strong>{parallelism}</strong></span>
        <input class="range" type="range" min="1" max="10" bind:value={parallelism} disabled={isBusy} />
        <span class="range-hint">{parallelism * 2} concurrent downloads</span>
      </div>

      <div class="sidebar-divider"></div>

      <!-- Progress bars -->
      {#if status === 'running' || fetched > 0}
        <div class="sidebar-section">
          <div class="prog-row">
            <span class="prog-label">Fetch</span>
            <span class="prog-val">{current}/{total}</span>
          </div>
          <div class="prog-track"><div class="prog-fill fetch" style="width:{pct}%"></div></div>
        </div>

        {#if imgTotal > 0}
          <div class="sidebar-section">
            <div class="prog-row">
              <span class="prog-label">Download</span>
              <span class="prog-val">{imgDone}/{imgTotal}</span>
            </div>
            <div class="prog-track"><div class="prog-fill download" style="width:{imgPct}%"></div></div>
          </div>
        {/if}

        {#if upDone > 0}
          <div class="sidebar-section">
            <div class="prog-row">
              <span class="prog-label">Upload R2</span>
              <span class="prog-val">{upDone}</span>
            </div>
            <div class="prog-track"><div class="prog-fill upload" style="width:{upPct}%"></div></div>
          </div>
        {/if}
      {:else}
        <span class="sidebar-idle">No active session</span>
      {/if}

      {#if errors > 0}
        <div class="sidebar-errors">⚠ {errors} error{errors > 1 ? 's' : ''}</div>
      {/if}
    </aside>

    <!-- Collapser — outside sidebar so it stays visible when collapsed -->
    <button class="sidebar-collapser" onclick={() => sidebarOpen = !sidebarOpen}>
      {sidebarOpen ? '‹' : '›'}
    </button>

    <!-- Main content -->
    <div class="dash-main">

      <!-- Stat cards -->
      <div class="stats-row">
        <StatCard label="Presets"  value={fetched}  color="var(--color-accent)"           icon="⬇" />
        <StatCard label="Images"   value={imgDone}  color="var(--color-accent-secondary)" icon="🖼" sub={imgTotal > 0 ? `of ${imgTotal}` : undefined} />
        <StatCard label="Uploaded" value={upDone}   color="var(--color-accent-tertiary)"  icon="☁" />
        <StatCard label="Errors"   value={errors}   color={errors > 0 ? 'var(--color-status-error)' : undefined} icon="⚠" />
        {#if secs > 0}
          <StatCard label="Elapsed"  value={fmtTime(secs)} color="var(--color-cyan)" icon="⏱" />
        {/if}
      </div>

      <!-- Main tabs -->
      <div class="main-tabs">
        {#each ([['overview','Overview'],['classes','Classes'],['sessions','Sessions']] as const) as [id, label]}
          <button class="main-tab" class:active={mainTab === id} onclick={() => mainTab = id}>
            {label}
          </button>
        {/each}
      </div>

      <!-- Tab content -->
      {#if mainTab === 'overview'}
        <div class="dash-grid-row">
          <div class="class-section">
            <span class="section-label">Classes <span class="section-count">31</span></span>
            <ClassGrid />
          </div>
          <div class="feed-section">
            <span class="section-label">Live Activity</span>
            <ActivityFeed />
          </div>
        </div>

      {:else if mainTab === 'classes'}
        <div class="chart-section">
          <span class="section-label">
            Presets per class
            <span class="chart-legend">
              <span class="legend-dot fetched"></span> Fetched
              <span class="legend-dot images"></span> Images
            </span>
          </span>
          <ClassChart />
        </div>

      {:else}
        <div class="sessions-section">
          <span class="section-label">Session history</span>
          <SessionHistory />
        </div>
      {/if}

    </div>
  </div>


</div>
