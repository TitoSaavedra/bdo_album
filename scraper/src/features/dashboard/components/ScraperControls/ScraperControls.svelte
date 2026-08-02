<script lang="ts">
  import './ScraperControls.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { _ } from 'svelte-i18n';
  import { Input, Button, PillSelector } from '$ui/index';
  import {
    getStatus,
    getPhase,
    getCurrent,
    getTotal,
    getCurrentClass,
    getCurrentMsg,
    getCurrentMsgKey,
    getCurrentMsgValues,
    getProgress,
    getImgProgress,
    getImagesDone,
    getImagesTotal,
    getErrors,
    requestStop,
  } from '../../../scraper/state/scraper.svelte';

  let cfToken     = $state('');
  let parallelism = $state(3);
  let mode        = $state<'images' | 'both' | 'fetch'>('both');

  const MODE_OPTIONS = [
    { value: 'images', label: 'Images only' },
    { value: 'both',   label: 'Fetch + Images' },
    { value: 'fetch',  label: 'Fetch only' },
  ];

  const status      = $derived(getStatus());
  const phase       = $derived(getPhase());
  const current     = $derived(getCurrent());
  const total       = $derived(getTotal());
  const activeClass = $derived(getCurrentClass());
  const msg         = $derived(getCurrentMsg());
  const msgKey      = $derived(getCurrentMsgKey());
  const msgValues   = $derived(getCurrentMsgValues());
  const displayMsg  = $derived(msgKey ? $_(msgKey, { values: msgValues }) : msg);
  const pct         = $derived(getProgress());
  const imgPct      = $derived(getImgProgress());
  const imgDone     = $derived(getImagesDone());
  const imgTotal    = $derived(getImagesTotal());
  const errCount    = $derived(getErrors());

  const PHASES = ['fetch', 'download', 'upload'] as const;

  const isBusy = $derived(status === 'running' || status === 'stopping');

  async function start() {
    await invoke('run_scraper', { cfToken, parallelism, mode });
  }

  async function stop() {
    requestStop();
    await invoke('cancel_scraper');
  }
</script>

<div class="controls">

  <!-- Status -->
  <div class="ctrl-section">
    <span class="ctrl-label">{$_('scraper.title')}</span>
    <span class="status-badge status-{status}">{$_(`scraper.status.${status}`)}</span>
  </div>

  <!-- Phase indicator -->
  {#if status === 'running'}
    <div class="ctrl-section">
      <div class="phase-steps">
        {#each PHASES as p}
          <div class="phase-step" class:active={phase === p} class:done={PHASES.indexOf(p) < PHASES.indexOf(phase)}>
            <span class="phase-dot"></span>
            <span class="phase-name">{p}</span>
          </div>
          {#if p !== 'upload'}<div class="phase-line"></div>{/if}
        {/each}
      </div>
    </div>

    <!-- Progress -->
    <div class="ctrl-section">
      <span class="ctrl-label">{$_('scraper.progress.label')}</span>
      <div class="progress-wrap">
        <div class="progress-track">
          <div class="progress-fill" style="width: {pct}%"></div>
        </div>
        <span class="progress-text">{current} / {total} — {pct}%</span>
        {#if activeClass}
          <span class="progress-detail">{activeClass}</span>
        {/if}
        {#if msg}
          <span class="progress-detail muted">{displayMsg}</span>
        {/if}
      </div>
    </div>

    {#if imgTotal > 0}
      <div class="ctrl-section">
        <span class="ctrl-label">Images</span>
        <div class="progress-wrap">
          <div class="progress-track">
            <div class="progress-fill green" style="width: {imgPct}%"></div>
          </div>
          <span class="progress-text">{imgDone} / {imgTotal} — {imgPct}%</span>
        </div>
      </div>
    {/if}

    {#if errCount > 0}
      <span class="error-count">⚠ {errCount} errors</span>
    {/if}
  {/if}

  <!-- CF Token -->
  <div class="ctrl-section">
    <span class="ctrl-label">{$_('scraper.token.label')}</span>
    <Input
      bind:value={cfToken}
      type="password"
      placeholder={$_('scraper.token.placeholder')}
      disabled={isBusy}
    />
  </div>

  <!-- Parallelism -->
  <div class="ctrl-section">
    <span class="ctrl-label">Parallelism — {parallelism}</span>
    <input
      type="range"
      min="1" max="10"
      bind:value={parallelism}
      disabled={isBusy}
      class="range-slider"
    />
    <span class="range-hint">{parallelism} parallel regions</span>
  </div>

  <!-- Mode -->
  <div class="ctrl-section">
    <span class="ctrl-label">Mode</span>
    <PillSelector
      value={mode}
      options={MODE_OPTIONS}
      onchange={(v) => { mode = v as 'images' | 'both' | 'fetch'; }}
    />
  </div>

  <!-- Action -->
  {#if status === 'running'}
    <Button variant="ghost" onclick={stop}>
      {$_('scraper.actions.stop')}
    </Button>
  {:else if status === 'stopping'}
    <Button variant="ghost" disabled>
      {$_('scraper.actions.stopping')}
    </Button>
  {:else}
    <Button variant="primary" onclick={start} disabled={!cfToken}>
      {$_('scraper.actions.start')}
    </Button>
  {/if}

</div>
