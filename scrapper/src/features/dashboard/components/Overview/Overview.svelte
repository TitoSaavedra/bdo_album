<script lang="ts">
  import './Overview.scss';
  import { onMount } from 'svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import ActivityFeed from '../ActivityFeed/ActivityFeed.svelte';
  import {
    getStatus, getTotalFetched, getImagesDone,
    getUploadsDone, getElapsed, getErrors, getDiscarded,
    getClassIcons, getLastPresetSynced,
  } from '../../../scrapper/state/scrapper.svelte';
  import { CLASSES } from '$lib/classes';
  import type { PresetSynced } from '$lib/events/types';

  let dbPresets   = $state(0);
  let dbImages    = $state(0);
  let dbUploaded  = $state(0);
  let dbSessions  = $state(0);
  let dbErrors    = $state(0);
  let dbSkipped   = $state(0);

  const tPresets  = tweened(0, { duration: 700, easing: cubicOut });
  const tImages   = tweened(0, { duration: 700, easing: cubicOut });
  const tUploaded = tweened(0, { duration: 700, easing: cubicOut });
  const tErrors   = tweened(0, { duration: 700, easing: cubicOut });
  const tSkipped  = tweened(0, { duration: 700, easing: cubicOut });

  const status    = $derived(getStatus());
  const fetched   = $derived(getTotalFetched());
  const imgDone   = $derived(getImagesDone());
  const upDone    = $derived(getUploadsDone());
  const elapsed   = $derived(getElapsed());
  const errors    = $derived(getErrors());
  const discarded = $derived(getDiscarded());
  const isActive  = $derived(status === 'running' || status === 'stopping' || status === 'done' || status === 'cancelled');

  // ── Preset card ──────────────────────────────────────────────
  let cardData    = $state<PresetSynced | null>(null);
  let cardVisible = $state(false);
  let cardLeaving = $state(false);
  let imgIndex    = $state(0);
  let imgErrors   = $state([false, false]);
  let _cardTimer: ReturnType<typeof setTimeout> | null = null;
  let _imgTimer:  ReturnType<typeof setInterval> | null = null;

  function classNameById(id: number) {
    return CLASSES.find(c => c.id === id)?.name ?? '';
  }

  function showCard(p: NonNullable<ReturnType<typeof getLastPresetSynced>>) {
    cardData    = p;
    cardLeaving = false;
    cardVisible = true;
    imgIndex    = 0;
    imgErrors   = [false, false];

    if (_cardTimer) clearTimeout(_cardTimer);
    if (_imgTimer)  clearInterval(_imgTimer);

    if (p.image_1_url && p.image_2_url) {
      _imgTimer = setInterval(() => {
        imgIndex = imgIndex === 0 ? 1 : 0;
      }, 1800);
    }

    _cardTimer = setTimeout(() => {
      cardLeaving = true;
      setTimeout(() => {
        cardVisible = false;
        cardLeaving = false;
        if (_imgTimer) { clearInterval(_imgTimer); _imgTimer = null; }
      }, 250);
    }, 2000);
  }

  $effect(() => {
    const p = getLastPresetSynced();
    if (p) showCard(p);
  });

  $effect(() => { tPresets.set(dbPresets   + fetched);    });
  $effect(() => { tImages.set(dbImages    + imgDone);     });
  $effect(() => { tUploaded.set(dbUploaded + upDone);     });
  $effect(() => { tErrors.set(dbErrors    + errors);      });
  $effect(() => { tSkipped.set(dbSkipped  + discarded);   });

  const fmt = (n: number) => Math.round(n).toLocaleString('es');
  const fmtTime = (s: number) => {
    const m = Math.floor(s / 60);
    return m > 0 ? `${m}m ${s % 60}s` : `${s}s`;
  };

  onMount(async () => {
    try {
      const [stats, sessions] = await Promise.all([
        invoke<any>('get_preset_stats'),
        invoke<any[]>('get_sessions'),
      ]);
      dbPresets   = stats.total ?? 0;
      dbSessions  = sessions.length;
      dbImages    = sessions.reduce((s: number, r: any) => s + (r.total_images   ?? 0), 0);
      dbUploaded  = sessions.reduce((s: number, r: any) => s + (r.total_uploaded ?? 0), 0);
      dbErrors    = sessions.reduce((s: number, r: any) => s + (r.errors         ?? 0), 0);
      dbSkipped   = sessions.reduce((s: number, r: any) => s + (r.skipped        ?? 0), 0);
      tPresets.set(dbPresets,   { duration: 0 });
      tImages.set(dbImages,     { duration: 0 });
      tUploaded.set(dbUploaded, { duration: 0 });
      tErrors.set(dbErrors,     { duration: 0 });
      tSkipped.set(dbSkipped,   { duration: 0 });
    } catch {}
  });
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
      <div class="ov-kpi" style="--kc: var(--color-cyan)">
        <span class="ov-kpi-val">{dbSessions + (isActive ? 1 : 0)}</span>
        <span class="ov-kpi-meta">📋 Sessions</span>
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
          {@const icons      = getClassIcons()}
          {@const className  = classNameById(cardData.class_id)}
          {@const iconSvg    = icons[cardData.class_id]}
          {@const images     = [cardData.image_1_url, cardData.image_2_url].filter(Boolean) as string[]}
          {@const allErrored = images.length > 0 && images.every((_, i) => imgErrors[i])}
          <div class="preset-card" class:card-out={cardLeaving}>

            <div class="preset-card-header">
              {#if iconSvg}
                <div class="preset-card-icon">{@html iconSvg}</div>
              {/if}
              <div class="preset-card-names">
                <span class="preset-card-class">{className}</span>
                {#if cardData.character_name}
                  <span class="preset-card-char">{cardData.character_name}</span>
                {/if}
              </div>
            </div>

            <div class="preset-card-img-wrap">
              {#each images as url, i}
                <img
                  class="preset-card-img"
                  class:img-visible={imgIndex === i && !imgErrors[i]}
                  class:img-hidden={imgIndex !== i || imgErrors[i]}
                  src={url}
                  alt=""
                  onerror={() => { imgErrors[i] = true; }}
                />
              {/each}
              {#if allErrored}
                <div class="preset-card-img-placeholder">
                  <span style="font-size: 26px">🖼</span>
                  <span>No preview</span>
                </div>
              {/if}
              {#if images.length > 1 && !allErrored}
                <div class="preset-card-dot-row">
                  {#each images as _, i}
                    <div class="preset-card-dot" class:dot-active={imgIndex === i}></div>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="preset-card-stats">
              <div class="preset-card-stat">
                <span class="stat-val">{(cardData.downloads ?? 0).toLocaleString('es')}</span>
                <span class="stat-label">DL</span>
              </div>
              <div class="preset-card-stat">
                <span class="stat-val">{(cardData.views ?? 0).toLocaleString('es')}</span>
                <span class="stat-label">Views</span>
              </div>
              <div class="preset-card-stat">
                <span class="stat-val">{(cardData.likes ?? 0).toLocaleString('es')}</span>
                <span class="stat-label">Likes</span>
              </div>
            </div>

          </div>
        {:else}
          <div class="preset-card-empty">
            <span style="font-size: 24px">🖼</span>
            <span>Waiting for images...</span>
          </div>
        {/if}
      </div>

    </div>
  </section>

</div>
