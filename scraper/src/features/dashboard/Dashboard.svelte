<script lang="ts">
  import './Dashboard.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Sidebar, MultiPill, PillSelector } from '$ui/index';
  import Overview       from './components/Overview/Overview.svelte';
  import PresetStats    from './components/PresetStats/PresetStats.svelte';
  import SessionHistory from './components/SessionHistory/SessionHistory.svelte';
  import ImportPabModal from './components/ImportPabModal/ImportPabModal.svelte';
  import RepairPabModal from './components/RepairPabModal/RepairPabModal.svelte';
  import ImportSessionModal from './components/ImportSessionModal/ImportSessionModal.svelte';

  type MainTab = 'overview' | 'presets' | 'sessions';
  let mainTab = $state<MainTab>('overview');
  import {
    getStatus, getPhase,
    getCurrent, getTotal,
    getProgress, getImgProgress, getUpProgress,
    getTotalFetched, getImagesDone, getImagesTotal,
    getUploadsDone, getErrors, getUpdated,
    requestStop, getDbReady, getClassIcons,
    getAutoDownloadStatus,
  } from '../scraper/state/scraper.svelte';

  const ALL_DAYS    = ['20', '30', '60', '90', '180', '365', 'ever'];
  const ALL_REGIONS = ['all', 'eu', 'na', 'ru', 'jp', 'kr', 'tw', 'sa', 'sea', 'asia', 'mena'];

  let dbClasses       = $state<{id: number, name: string}[]>([]);
  const ALL_CLASSES   = $derived(['all', ...dbClasses.map(c => c.name)]);

  const MODE_OPTIONS = [
    { value: 'images', label: 'Images' },
    { value: 'fetch',  label: 'Fetch'  },
    { value: 'both',   label: 'Both'   },
  ];

  let parallelism     = $state(3);
  let mode            = $state<'images' | 'both' | 'fetch'>('both');
  let sidebarOpen     = $state(false);
  let pabModalOpen     = $state(false);
  let repairModalOpen  = $state(false);
  let sessionModalOpen = $state(false);
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
  const updated  = $derived(getUpdated());
  const errors   = $derived(getErrors());
  const autoDl   = $derived(getAutoDownloadStatus());
  const current  = $derived(getCurrent());
  const total    = $derived(getTotal());

  const dbReady      = $derived(getDbReady());
  const isBusy       = $derived(status === 'running' || status === 'stopping');
  const classIconsSvg = $derived(getClassIcons());
  const classIconMap  = $derived(
    Object.fromEntries(
      dbClasses
        .filter(c => classIconsSvg[c.id])
        .map(c => [c.name, classIconsSvg[c.id]])
    )
  );

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
    const effectiveClasses = selectedClasses.includes('all') ? ['all'] : selectedClasses;
    const classIds = effectiveClasses.map(name =>
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
      <span class="header-gem"></span>
      <span class="dash-title">Scraper</span>
      <span class="status-pill status-{status}">{status}</span>
      {#if autoDl.state === 'downloading'}
        <span class="auto-dl-pill downloading" title="Auto-download (independiente de la sesión de scraping)">
          ⬇ Auto-descarga: preset #{autoDl.preset_id}
        </span>
      {:else if autoDl.state === 'quota_exceeded'}
        <span class="auto-dl-pill quota" title="Auto-download (independiente de la sesión de scraping)">
          ⏸ Auto-descarga: cuota agotada ({autoDl.used}/{autoDl.limit})
        </span>
      {/if}
      {#if status !== 'idle'}
        <div class="phase-mini">
          {#each PHASES as p, i}
            <div
              class="phase-mini-step"
              class:active={phase === p.id}
              class:done={['fetch','download','upload'].indexOf(phase) > i}
              style="--c: {p.color}"
              title={p.label}
            >
              <span class="phase-mini-dot"></span>
            </div>
            {#if i < 2}<span class="phase-mini-arrow">›</span>{/if}
          {/each}
        </div>
      {/if}
    </div>

    <!-- Segmented tabs -->
    <div class="header-tabs">
      {#each ([['overview','Overview'],['presets','Presets'],['sessions','Sessions']] as const) as [id, label]}
        <button class="header-tab" class:active={mainTab === id} onclick={() => mainTab = id}>
          {label}
        </button>
      {/each}
    </div>

    <div class="dash-header-right">
      <Button variant="success" onclick={() => pabModalOpen = true} disabled={dbReady !== true}>Import</Button>
      <Button variant="ghost" onclick={() => repairModalOpen = true} disabled={dbReady !== true}>Reparar PAB</Button>
      <Button variant="ghost" onclick={() => sessionModalOpen = true} disabled={dbReady !== true}>Sesión</Button>
      {#if status === 'running'}
        <Button variant="ghost" onclick={stop}>■ Stop</Button>
      {:else if status === 'stopping'}
        <Button variant="ghost" disabled>Stopping...</Button>
      {:else}
        <Button variant="primary" onclick={start} disabled={dbReady !== true || isBusy}>▶ Start</Button>
      {/if}
    </div>

  </header>

  <!-- ── Body ── -->
  <div class="dash-body">

    <Sidebar bind:open={sidebarOpen} width={220}>
      <!-- Sidebar tabs -->
      <div class="sidebar-seg">
        <button class="seg-btn" class:active={sidebarTab === 'config'} onclick={() => sidebarTab = 'config'}>Config</button>
        <button class="seg-btn" class:active={sidebarTab === 'status'} onclick={() => sidebarTab = 'status'}>Status</button>
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
            disabled={isBusy}
            onchange={(v) => { mode = v as 'images' | 'both' | 'fetch'; }}
          />
        </div>

        {#if mode !== 'images'}
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
        {/if}

        <div class="sidebar-section">
          <span class="sidebar-label">Classes <span class="sidebar-count">{selectedClasses.length}/{ALL_CLASSES.length}</span></span>
          <MultiPill
            options={ALL_CLASSES}
            selected={selectedClasses}
            onchange={(v) => selectedClasses = v}
            disabled={isBusy}
            icons={classIconMap}
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

          {#if updated > 0}
            <div class="sidebar-section">
              <div class="prog-row">
                <span class="prog-label">Stats updated</span>
                <span class="prog-val">{updated}</span>
              </div>
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
<RepairPabModal bind:open={repairModalOpen} />
<ImportSessionModal bind:open={sessionModalOpen} />
