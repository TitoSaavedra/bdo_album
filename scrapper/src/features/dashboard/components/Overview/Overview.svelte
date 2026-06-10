<script lang="ts">
  import './Overview.scss';
  import { onMount, untrack } from 'svelte';
  import { fade } from 'svelte/transition';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import ActivityFeed from '../ActivityFeed/ActivityFeed.svelte';
  import {
    getStatus, getTotalFetched, getImagesDone,
    getUploadsDone, getElapsed, getErrors, getDiscarded, getUpdated,
    getClassIcons, getLastPresetSynced,
  } from '../../../scrapper/state/scrapper.svelte';
  import { CLASSES } from '$lib/classes';
  import type { PresetSynced } from '$lib/events/types';

  let dbPresets   = $state(0);
  let dbImages    = $state(0);
  let dbUploaded  = $state(0);
  let dbUpdated   = $state(0);
  let dbSessions  = $state(0);
  let dbErrors    = $state(0);
  let dbSkipped   = $state(0);

  const tPresets  = tweened(0, { duration: 700, easing: cubicOut });
  const tImages   = tweened(0, { duration: 700, easing: cubicOut });
  const tUploaded = tweened(0, { duration: 700, easing: cubicOut });
  const tUpdated  = tweened(0, { duration: 700, easing: cubicOut });
  const tErrors   = tweened(0, { duration: 700, easing: cubicOut });
  const tSkipped  = tweened(0, { duration: 700, easing: cubicOut });

  const status    = $derived(getStatus());
  const fetched   = $derived(getTotalFetched());
  const imgDone   = $derived(getImagesDone());
  const upDone    = $derived(getUploadsDone());
  const elapsed   = $derived(getElapsed());
  const errors    = $derived(getErrors());
  const discarded = $derived(getDiscarded());
  const sessionUpdated = $derived(getUpdated());
  const isActive  = $derived(status === 'running' || status === 'stopping' || status === 'done' || status === 'cancelled');

  // ── Preset card queue ────────────────────────────────────────
  const DISPLAY_MS    = 2000;
  const TRANSITION_MS = 250;

  let cardQueue    = $state<PresetSynced[]>([]);
  let cardData     = $state<PresetSynced | null>(null);
  let cardVisible  = $state(false);
  let hasHadPreset = $state(false);
  let transitioning = false;
  let waiting       = false;
  let imgIndex     = $state(0);
  let imgErrors    = $state<boolean[]>([false, false]);

  const cardClassName  = $derived(cardData ? classNameById(cardData.class_id) : '');
  const cardIconSvg    = $derived(cardData ? (getClassIcons()[cardData.class_id] ?? null) : null);
  const cardImages     = $derived(cardData ? [cardData.image_1_url, cardData.image_2_url].filter(Boolean) as string[] : []);
  const cardAllErrored = $derived(cardImages.length > 0 && cardImages.every((_, i) => imgErrors[i]));

  function openPresetPage() {
    if (cardData) {
      invoke('open_url', { url: `https://garmoth.com/beauty-album/preset/${cardData.preset_id}` });
    }
  }
  let _displayTimer: ReturnType<typeof setTimeout> | null = null;
  let _imgTimer:    ReturnType<typeof setInterval> | null = null;

  function classNameById(id: number) {
    return CLASSES.find(c => c.id === id)?.name ?? '';
  }

  function stopImgTimer() {
    if (_imgTimer) { clearInterval(_imgTimer); _imgTimer = null; }
  }

  function stopDisplayTimer() {
    if (_displayTimer) { clearTimeout(_displayTimer); _displayTimer = null; }
  }

  function startImgCycle(p: PresetSynced) {
    stopImgTimer();
    imgIndex  = 0;
    imgErrors = [false, false];
    if (p.image_1_url && p.image_2_url) {
      _imgTimer = setInterval(() => {
        imgIndex = imgIndex === 0 ? 1 : 0;
      }, 1000);
    }
  }

  function startDisplayTimer() {
    stopDisplayTimer();
    waiting       = false;
    _displayTimer = setTimeout(() => {
      _displayTimer = null;
      if (cardQueue.length > 0) advance();
      else waiting = true;
    }, DISPLAY_MS);
  }

  function preloadPreset(p: PresetSynced): Promise<void> {
    const urls = [p.image_1_url, p.image_2_url].filter(Boolean) as string[];
    if (urls.length === 0) return Promise.resolve();
    return Promise.race([
      Promise.all(urls.map(url => new Promise<void>(resolve => {
        const img = new Image();
        img.onload  = () => resolve();
        img.onerror = () => resolve();
        img.src = url;
      }))),
      new Promise<void>(resolve => setTimeout(resolve, 3000)),
    ]) as Promise<void>;
  }

  async function advance() {
    if (transitioning || cardQueue.length === 0) return;
    waiting       = false;
    const next    = cardQueue[0];
    cardQueue     = cardQueue.slice(1);
    transitioning = true;

    await preloadPreset(next);

    cardData = next;
    startImgCycle(next);
    startDisplayTimer();
    setTimeout(() => { transitioning = false; }, TRANSITION_MS);
  }

  function enqueuePreset(p: PresetSynced) {
    if (!p.image_1_url && !p.image_2_url) return;
    hasHadPreset = true;

    if (!cardVisible) {
      cardData    = p;
      cardVisible = true;
      startImgCycle(p);
      startDisplayTimer();
    } else if (waiting) {
      // Timer ya venció y no había cola — avanzar de inmediato
      cardQueue = [p];
      advance();
    } else {
      // Timer corriendo — encolar para cuando venza
      cardQueue = [...cardQueue, p];
    }
  }

  $effect(() => {
    const p = getLastPresetSynced();
    if (p) untrack(() => enqueuePreset(p));
  });

  $effect(() => {
    cardQueue.slice(0, 2).forEach(p => {
      if (p.image_1_url) { const img = new Image(); img.src = p.image_1_url; }
      if (p.image_2_url) { const img = new Image(); img.src = p.image_2_url; }
    });
  });

  // Full reset only when status TRANSITIONS to 'running' (new session), not on initial mount
  let _prevStatus = getStatus();
  $effect(() => {
    const s = getStatus();
    if (s === 'running' && _prevStatus !== 'running') {
      cardQueue     = [];
      cardData      = null;
      cardVisible   = false;
      hasHadPreset  = false;
      transitioning = false;
      waiting       = false;
      imgIndex      = 0;
      imgErrors     = [false, false];
      stopDisplayTimer();
      stopImgTimer();
      // Refresh base stats so All Time KPIs are accurate for this session.
      // fetched/discarded have already been reset to 0 at this point.
      loadStats(true);
    }
    _prevStatus = s;
  });

  $effect(() => { tPresets.set(dbPresets   + fetched);         });
  $effect(() => { tImages.set(dbImages    + imgDone);          });
  $effect(() => { tUploaded.set(dbUploaded + upDone);          });
  $effect(() => { tUpdated.set(dbUpdated  + sessionUpdated);   });
  $effect(() => { tErrors.set(dbErrors    + errors);           });
  $effect(() => { tSkipped.set(dbSkipped  + discarded);        });

  const fmt = (n: number) => Math.round(n).toLocaleString('es');
  const fmtTime = (s: number) => {
    const m = Math.floor(s / 60);
    return m > 0 ? `${m}m ${s % 60}s` : `${s}s`;
  };

  async function loadStats(animate = true) {
    try {
      const [stats, totals] = await Promise.all([
        invoke<any>('get_preset_stats'),
        invoke<any>('get_session_totals'),
      ]);
      dbPresets  = stats.total      ?? 0;
      dbSessions = totals.count     ?? 0;
      dbImages   = totals.total_images   ?? 0;
      dbUploaded = totals.total_uploaded ?? 0;
      dbUpdated  = totals.total_updated  ?? 0;
      dbErrors   = totals.errors         ?? 0;
      dbSkipped  = totals.skipped        ?? 0;
      if (!animate) {
        tPresets.set(dbPresets,   { duration: 0 });
        tImages.set(dbImages,     { duration: 0 });
        tUploaded.set(dbUploaded, { duration: 0 });
        tUpdated.set(dbUpdated,   { duration: 0 });
        tErrors.set(dbErrors,     { duration: 0 });
        tSkipped.set(dbSkipped,   { duration: 0 });
      }
    } catch {}
  }

  onMount(() => loadStats(false));
</script>

<div class="overview">

  <!-- ── Global all-time ── -->
  <section class="ov-block">
    <div class="ov-label">All Time</div>
    <div class="ov-kpi-row">
      <div class="ov-kpi" style="--kc: var(--color-accent)">
        <span class="ov-kpi-val">{fmt($tPresets)}</span>
        <span class="ov-kpi-meta">⬇ Presets</span>
      </div>
      <div class="ov-kpi" style="--kc: var(--color-accent-secondary)">
        <span class="ov-kpi-val">{fmt($tImages)}</span>
        <span class="ov-kpi-meta">🖼 Images</span>
      </div>
      <div class="ov-kpi" style="--kc: var(--color-accent-tertiary)">
        <span class="ov-kpi-val">{fmt($tUploaded)}</span>
        <span class="ov-kpi-meta">☁ Uploaded</span>
      </div>
      <div class="ov-kpi" style="--kc: #34d399">
        <span class="ov-kpi-val">{fmt($tUpdated)}</span>
        <span class="ov-kpi-meta">↺ Updated</span>
      </div>
      <div class="ov-kpi" style="--kc: var(--color-cyan)">
        <span class="ov-kpi-val">{dbSessions + (isActive ? 1 : 0)}</span>
        <span class="ov-kpi-meta">≡ Sessions</span>
      </div>
      <div class="ov-kpi" style="--kc: var(--color-text-muted)">
        <span class="ov-kpi-val">{fmt($tSkipped)}</span>
        <span class="ov-kpi-meta">⏭ Skipped</span>
      </div>
      <div class="ov-kpi" style="--kc: {$tErrors > 0 ? 'var(--color-status-error)' : 'var(--color-text-muted)'}">
        <span class="ov-kpi-val">{fmt($tErrors)}</span>
        <span class="ov-kpi-meta">⚠ Errors</span>
      </div>
    </div>
  </section>

  <!-- ── Session delta ── -->
  {#if isActive}
    <section class="ov-block ov-session-block">
      <div class="ov-label">
        This Session
        {#if elapsed > 0}<span class="ov-badge ov-time">⏱ {fmtTime(elapsed)}</span>{/if}
        {#if errors > 0}<span class="ov-badge ov-err">⚠ {errors} err</span>{/if}
      </div>
      <div class="ov-delta-row">
        <div class="ov-delta" class:zero={fetched === 0}>
          <span class="ov-delta-val">+{fetched.toLocaleString('es')}</span>
          <span class="ov-delta-label">Presets fetched</span>
        </div>
        <div class="ov-delta" class:zero={imgDone === 0}>
          <span class="ov-delta-val">+{imgDone.toLocaleString('es')}</span>
          <span class="ov-delta-label">Images downloaded</span>
        </div>
        <div class="ov-delta" class:zero={upDone === 0}>
          <span class="ov-delta-val">+{upDone.toLocaleString('es')}</span>
          <span class="ov-delta-label">Uploaded to R2</span>
        </div>
        <div class="ov-delta" class:zero={sessionUpdated === 0}>
          <span class="ov-delta-val">↺{sessionUpdated.toLocaleString('es')}</span>
          <span class="ov-delta-label">Stats updated</span>
        </div>
        <div class="ov-delta" class:zero={discarded === 0}>
          <span class="ov-delta-val">{discarded.toLocaleString('es')}</span>
          <span class="ov-delta-label">Skipped (DB)</span>
        </div>
        {#if errors > 0}
          <div class="ov-delta ov-delta-err">
            <span class="ov-delta-val">{errors.toLocaleString('es')}</span>
            <span class="ov-delta-label">Errors</span>
          </div>
        {/if}
      </div>
    </section>
  {/if}

  <!-- ── Activity feed ── -->
  <section class="ov-block ov-feed-block">
    <div class="ov-label">Live Activity</div>
    <div class="ov-feed-row">

      <div class="ov-feed-col">
        <ActivityFeed />
      </div>

      <div class="ov-class-panel">
        {#if cardVisible && cardData}
          <div class="preset-card">
            {#key cardData.preset_id}
              <div class="preset-card-content" in:fade={{ duration: 250 }} out:fade={{ duration: 250 }}>

                <div class="preset-card-header">
                  {#if cardIconSvg}
                    <div class="preset-card-icon">{@html cardIconSvg}</div>
                  {/if}
                  <div class="preset-card-names">
                    <span class="preset-card-class">{cardClassName}</span>
                    {#if cardData.character_name}
                      <span class="preset-card-char">{cardData.character_name}</span>
                    {/if}
                    <span class="preset-card-id">#{cardData.preset_id}</span>
                  </div>
                </div>

                <div class="preset-card-img-wrap" onclick={openPresetPage} onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && openPresetPage()} role="button" tabindex="0" title="View on Garmoth">
                  {#each cardImages as url, i}
                    <img
                      class="preset-card-img"
                      class:img-visible={imgIndex === i && !imgErrors[i]}
                      class:img-hidden={imgIndex !== i || imgErrors[i]}
                      src={url}
                      alt=""
                      onerror={() => { imgErrors[i] = true; }}
                    />
                  {/each}
                  {#if cardAllErrored}
                    <div class="preset-card-img-placeholder">
                      <span style="font-size: 26px">🖼</span>
                      <span>No preview</span>
                    </div>
                  {/if}
                  {#if cardImages.length > 1 && !cardAllErrored}
                    <div class="preset-card-dot-row">
                      {#each cardImages as _, i}
                        <div class="preset-card-dot" class:dot-active={imgIndex === i}></div>
                      {/each}
                    </div>
                  {/if}
                </div>

                <div class="preset-card-stats">
                  <div class="stat-item">
                    <svg class="stat-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M8 2v7M5 6.5l3 3 3-3"/>
                      <path d="M3 13h10"/>
                    </svg>
                    <span class="stat-num">{(cardData.downloads ?? 0).toLocaleString('es')}</span>
                  </div>
                  <div class="stat-sep"></div>
                  <div class="stat-item">
                    <svg class="stat-icon" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M1 8s3-5 7-5 7 5 7 5-3 5-7 5-7-5-7-5z"/>
                      <circle cx="8" cy="8" r="2"/>
                    </svg>
                    <span class="stat-num">{(cardData.views ?? 0).toLocaleString('es')}</span>
                  </div>
                  <div class="stat-sep"></div>
                  <div class="stat-item stat-item--likes">
                    <svg class="stat-icon" viewBox="0 0 16 16" fill="currentColor">
                      <path d="M8 13.5c-.2 0-.4-.1-.6-.2C4.5 11 1 8.2 1 5c0-2 1.6-3.5 3.5-3.5 1 0 2 .5 2.7 1.3L8 3.7l.8-.9C9.5 2 10.5 1.5 11.5 1.5 13.4 1.5 15 3 15 5c0 3.2-3.5 6-6.4 8.3-.2.1-.4.2-.6.2z"/>
                    </svg>
                    <span class="stat-num">{(cardData.likes ?? 0).toLocaleString('es')}</span>
                  </div>
                </div>

              </div>
            {/key}
          </div>
        {:else if !hasHadPreset}
          <div class="preset-card-empty">
            <div class="pce-icon">
              <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <circle cx="8.5" cy="8.5" r="1.5"/>
                <polyline points="21 15 16 10 5 21"/>
              </svg>
            </div>
            <span class="pce-text">Waiting for images<span class="feed-dots"><span>.</span><span>.</span><span>.</span></span></span>
          </div>
        {/if}
      </div>

    </div>
  </section>

</div>
