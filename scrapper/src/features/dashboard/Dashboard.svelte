<script lang="ts">
  import './Dashboard.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Sidebar, MultiPill, PillSelector } from '$ui/index';
  import Overview       from './components/Overview/Overview.svelte';
  import PresetStats    from './components/PresetStats/PresetStats.svelte';
  import SessionHistory from './components/SessionHistory/SessionHistory.svelte';
  import ImportPabModal from './components/ImportPabModal/ImportPabModal.svelte';

  type MainTab = 'overview' | 'presets' | 'sessions';
  let mainTab = $state<MainTab>('overview');
  import {
    getStatus, getPhase,
    getCurrent, getTotal,
    getProgress, getImgProgress, getUpProgress,
    getTotalFetched, getImagesDone, getImagesTotal,
    getUploadsDone, getErrors,
    requestStop, getDbReady,
  } from '../scrapper/state/scrapper.svelte';

  const ALL_DAYS    = ['20', '30', '60', '90', '180', '365', 'ever'];
  const ALL_REGIONS = ['all', 'eu', 'na', 'ru', 'jp', 'kr', 'tw', 'sa', 'sea', 'asia', 'mena'];

  let dbClasses       = $state<{id: number, name: string}[]>([]);
  const ALL_CLASSES   = $derived(['all', ...dbClasses.map(c => c.name)]);

  const MODE_OPTIONS = [
    { value: 'images', label: 'Images only'    },
    { value: 'fetch',  label: 'Fetch only'  },
    { value: 'both',   label: 'Fetch + Images' },
  ];

  let parallelism     = $state(3);
  let mode            = $state<'images' | 'both' | 'fetch'>('both');
  let sidebarOpen     = $state(false);
  let pabModalOpen    = $state(false);
  let sidebarTab      = $state<'config' | 'status'>('config');
  let selectedDays    = $state<string[]>([ALL_DAYS[0]]);
  let selectedRegions = $state<string[]>(['all']);
  let selectedClasses = $state<string[]>(['all']);


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
  const current  = $derived(getCurrent());
  const total    = $derived(getTotal());

  const dbReady = $derived(getDbReady());
  const isBusy  = $derived(status === 'running' || status === 'stopping');

  $effect(() => {
    if (dbReady === true) {
      invoke<{id: number, name: string}[]>('get_classes').then(cls => {
        dbClasses = cls;
      });
    }
  });


  const PHASES = [
    { id: 'fetch',    label: 'Fetch JSON', color: 'var(--color-accent)'          },
    { id: 'download', label: 'Download',   color: 'var(--color-accent-secondary)' },
    { id: 'upload',   label: 'Upload R2',  color: 'var(--color-accent-tertiary)'  },
  ] as const;

  async function start()  {
    const classIds = selectedClasses.map(name =>
      name === 'all' ? 'all' : dbClasses.find(c => c.name === name)!.id
    );
    await invoke('run_scraper', {
      parallelism,
      mode,
      days:    selectedDays,
      regions: selectedRegions,
      classes: classIds,
    });
  }
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
        <div class="pipe-step" class:active={status !== 'idle' && phase === p.id} class:done={status !== 'idle' && ['fetch','download','upload'].indexOf(phase) > i} style="--c: {p.color}">
          <span class="pipe-dot"></span>
          <span class="pipe-label">{p.label}</span>
        </div>
        {#if i < 2}<div class="pipe-arrow">→</div>{/if}
      {/each}
    </div>

    <div class="dash-header-right">
      <Button variant="success" onclick={() => pabModalOpen = true} disabled={dbReady !== true}>Import</Button>
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

    <Sidebar bind:open={sidebarOpen} width={220}>
      <!-- Sidebar tabs -->
      <div class="sidebar-tabs">
        <button class="sidebar-tab" class:active={sidebarTab === 'config'} onclick={() => sidebarTab = 'config'}>Config</button>
        <button class="sidebar-tab" class:active={sidebarTab === 'status'} onclick={() => sidebarTab = 'status'}>Status</button>
      </div>

      {#if sidebarTab === 'config'}
        <div class="sidebar-section">
          <span class="sidebar-label">Parallelism — <strong>{parallelism}</strong></span>
          <input class="range" type="range" min="1" max="10" bind:value={parallelism} disabled={isBusy} />
          <span class="range-hint">{parallelism * 2} concurrent downloads</span>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Mode</span>
          <PillSelector
            value={mode}
            options={MODE_OPTIONS}
            onchange={(v) => { if (!isBusy && v !== 'fetch') mode = v as 'images' | 'both'; }}
          />
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Days <span class="sidebar-count">{selectedDays.length}/{ALL_DAYS.length}</span></span>
          <MultiPill
            options={ALL_DAYS}
            selected={selectedDays}
            onchange={(v) => selectedDays = v}
            disabled={isBusy}
            selectAll
            reset
          />
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Regions <span class="sidebar-count">{selectedRegions.length}/{ALL_REGIONS.length}</span></span>
          <MultiPill
            options={ALL_REGIONS}
            selected={selectedRegions}
            onchange={(v) => selectedRegions = v}
            disabled={isBusy}
            selectAll
            reset
          />
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Classes <span class="sidebar-count">{selectedClasses.length}/{ALL_CLASSES.length}</span></span>
          <MultiPill
            options={ALL_CLASSES}
            selected={selectedClasses}
            onchange={(v) => selectedClasses = v}
            disabled={isBusy}
            selectAll
            reset
          />
          <span class="range-hint">
            ~{selectedDays.length * selectedRegions.length * selectedClasses.length} requests
          </span>
        </div>

      {:else}
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
      {/if}
    </Sidebar>

    <!-- Main content -->
    <div class="dash-main">

      <!-- Main tabs -->
      <div class="main-tabs">
        {#each ([['overview','Overview'],['presets','Presets'],['sessions','Sessions']] as const) as [id, label]}
          <button class="main-tab" class:active={mainTab === id} onclick={() => mainTab = id}>
            {label}
          </button>
        {/each}
      </div>

      <!-- Tab content -->
      {#if mainTab === 'overview'}
        <Overview />

      {:else if mainTab === 'presets'}
        <div class="chart-section">
          <span class="section-label">Preset Database</span>
          <PresetStats />
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

<ImportPabModal bind:open={pabModalOpen} />
