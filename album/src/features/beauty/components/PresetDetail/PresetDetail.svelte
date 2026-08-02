<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { _ } from 'svelte-i18n';
  import { openUrl, toggleWanted, discardPreset, exportToBdo } from '../../../../lib/album';
  import {
    beauty,
    closePreset,
    toggleWantedPreset,
  } from '../../state/beauty.svelte';
  import Button from '../../../../ui/Button/Button.svelte';

  const images = $derived(
    p ? [p.image_1_url, p.image_2_url].filter((u): u is string => !!u) : []
  );

  let activeImage = $state('');

  $effect(() => {
    activeImage = images[0] ?? '';
  });

  $effect(() => {
    if (images.length <= 1) return;
    const cur = activeImage;
    const timer = setTimeout(() => {
      const idx = images.indexOf(cur);
      activeImage = images[(idx + 1) % images.length];
    }, 900);
    return () => clearTimeout(timer);
  });

  const p         = $derived(beauty.presetDetail);
  const title     = $derived(p ? (p.title || p.character_name || `#${p.preset_id}`) : '');
  const id        = $derived(p?.preset_id ?? '');
  const nickname  = $derived(p?.user_nickname || null);
  const downloads = $derived(p?.downloads ?? 0);
  const views     = $derived(p?.views ?? 0);
  const likes     = $derived(p?.likes ?? 0);
  const className     = $derived(p?.class_display ?? '');
  const characterName = $derived(p?.character_name || null);
  const region        = $derived(p?.region || null);
  const isWanted      = $derived(p ? beauty.wantedPresets.has(p.preset_id) : false);
  const uploadedAt    = $derived(
    p?.creation_at ? new Date(p.creation_at * 1000).toLocaleDateString('en-CA') : null
  );
  const syncedAt  = $derived(
    p?.updated_at ? new Date(p.updated_at * 1000).toLocaleDateString('en-CA') : null
  );

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') closePreset();
  }

  async function handleToggleWant() {
    if (!p) return;
    toggleWantedPreset(p.preset_id);
    try { await toggleWanted(p.preset_id); } catch { /* non-fatal */ }
  }

  async function handleDiscard() {
    if (!p) return;
    const pid = p.preset_id;
    closePreset();
    try { await discardPreset(pid); } catch { /* non-fatal */ }
  }

  function openOnGarmoth() {
    openUrl(`https://garmoth.com/beauty-album/preset/${id}`);
  }

  let exporting     = $state(false);
  let exportError   = $state('');
  let sharpenOn     = $state(true);

  async function handleExport() {
    if (!p?.pab_url) return;
    exporting   = true;
    exportError = '';
    try {
      await exportToBdo(p.pab_url);
    } catch (e) {
      exportError = String(e);
    } finally {
      exporting = false;
    }
  }

  function onImgError(e: Event) {
    (e.currentTarget as HTMLImageElement).style.display = 'none';
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- Sharpen kernel: mild unsharp mask applied in-browser, no image reprocessing -->
<svg style="display:none" aria-hidden="true">
  <defs>
    <filter id="detail-sharpen" color-interpolation-filters="sRGB">
      <feConvolveMatrix
        order="3"
        kernelMatrix="0 -0.4 0 -0.4 2.6 -0.4 0 -0.4 0"
        preserveAlpha="true"
      />
    </filter>
  </defs>
</svg>

{#if p}
  <div
    class="backdrop"
    role="presentation"
    onclick={closePreset}
    onkeydown={(e) => e.key === 'Escape' && closePreset()}
    transition:fade={{ duration: 200 }}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      transition:scale={{ duration: 250, start: 0.95 }}
    >

      <!-- LEFT: image preview -->
      <div class="panel-left">
        <div class="main-preview">
          {#if activeImage}
            {#key activeImage}
              <img
                src={activeImage}
                alt={title}
                class="main-img"
                class:sharpen={sharpenOn}
                onerror={onImgError}
                in:fade={{ duration: 100 }}
                out:fade={{ duration: 100 }}
              />
            {/key}
          {:else}
            <span class="no-media">{$_('beauty.preset_detail.no_media')}</span>
          {/if}
          <button
            class="sharpen-toggle"
            class:active={sharpenOn}
            onclick={() => (sharpenOn = !sharpenOn)}
            title={sharpenOn ? $_('beauty.preset_detail.sharpen_on') : $_('beauty.preset_detail.sharpen_off')}
          >⬡</button>

          {#if images.length > 1}
            <div class="carousel-dots">
              {#each images as img, i}
                <button
                  class="dot"
                  class:dot-active={img === activeImage}
                  onclick={() => (activeImage = img)}
                  aria-label={$_('beauty.preset_detail.image_n', { values: { n: i + 1 } })}
                ></button>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <!-- RIGHT: info -->
      <div class="panel-right custom-scroll">

        <div class="panel-header">
          <div class="header-top">
            <div class="header-meta">
              {#if className}
                <span class="class-tag">{className}</span>
              {/if}
              <span class="preset-id">#{id}</span>
            </div>
            <Button variant="ghost" class="close-btn" onclick={closePreset}>✕</Button>
          </div>
          <h2 class="preset-title">{title}</h2>
          {#if nickname}
            <div class="creator-row">
              <span class="creator-at">@</span><span class="creator-name">{nickname}</span>
            </div>
          {/if}
        </div>

        <div class="stats-row">
          <div class="stat-item">
            <span class="stat-icon">↓</span>
            <span class="stat-val">{Number(downloads).toLocaleString()}</span>
            <span class="stat-lbl">{$_('beauty.preset_detail.downloads')}</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-icon">◉</span>
            <span class="stat-val">{Number(views).toLocaleString()}</span>
            <span class="stat-lbl">{$_('beauty.preset_detail.views')}</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item stat-fav">
            <span class="stat-icon">♥</span>
            <span class="stat-val">{Number(likes).toLocaleString()}</span>
            <span class="stat-lbl">{$_('beauty.preset_detail.likes')}</span>
          </div>
        </div>

        <div class="meta-grid">
          {#if characterName && characterName !== title}
            <span class="meta-key">{$_('beauty.preset_detail.character')}</span>
            <span class="meta-val">{characterName}</span>
          {/if}
          {#if region}
            <span class="meta-key">{$_('beauty.preset_detail.region')}</span>
            <span class="meta-val region-val">{region}</span>
          {/if}
          {#if uploadedAt}
            <span class="meta-key">{$_('beauty.preset_detail.uploaded')}</span>
            <span class="meta-val">{uploadedAt}</span>
          {/if}
          {#if syncedAt}
            <span class="meta-key">{$_('beauty.preset_detail.synced')}</span>
            <span class="meta-val">{syncedAt}</span>
          {/if}
          <span class="meta-key">{$_('beauty.preset_detail.images')}</span>
          <span class="meta-val">{images.length}</span>
          <span class="meta-key">{$_('beauty.preset_detail.pab')}</span>
          <span class="meta-val" class:pab-yes={p?.has_pab} class:pab-no={!p?.has_pab}>
            {p?.has_pab ? $_('beauty.preset_detail.available') : $_('beauty.preset_detail.not_available')}
          </span>
        </div>

        <div class="actions">
          <div class="want-row">
            <Button variant="icon" class="btn-want-sm" active={isWanted} onclick={handleToggleWant} title={isWanted ? $_('beauty.preset_detail.remove_wishlist') : $_('beauty.preset_detail.add_wishlist')}>♥</Button>
            <Button variant="icon" class="btn-discard-sm" title={$_('beauty.preset_detail.discard')} onclick={handleDiscard}>✕</Button>
          </div>
          {#if p?.has_pab}
            <Button variant="ghost" class="btn-export" onclick={handleExport} disabled={exporting}>
              <span class="icon">⬆</span> {exporting ? $_('beauty.preset_detail.exporting') : $_('beauty.preset_detail.export_to_bdo')}
            </Button>
            {#if exportError}
              <p class="export-error">{exportError}</p>
            {/if}
          {:else}
            <Button variant="ghost" class="btn-garmoth" onclick={openOnGarmoth}>
              <span class="icon">◈</span> {$_('beauty.preset_detail.view_on_garmoth')}
            </Button>
          {/if}
        </div>

      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @use './PresetDetail.scss';
</style>
