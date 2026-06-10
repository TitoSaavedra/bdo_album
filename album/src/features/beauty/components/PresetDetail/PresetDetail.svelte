<script lang="ts">
  import { fade, scale } from 'svelte/transition';
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

  const p         = $derived(beauty.presetDetail);
  const title     = $derived(p ? (p.title || p.character_name || `#${p.preset_id}`) : '');
  const id        = $derived(p?.preset_id ?? '');
  const nickname  = $derived(p?.user_nickname || null);
  const downloads = $derived(p?.downloads ?? 0);
  const views     = $derived(p?.views ?? 0);
  const likes     = $derived(p?.likes ?? 0);
  const className = $derived(p?.class_display ?? '');
  const isWanted  = $derived(p ? beauty.wantedPresets.has(p.preset_id) : false);
  const uploadedAt = $derived(
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
            <img src={activeImage} alt={title} class="main-img" onerror={onImgError} />
          {:else}
            <span class="no-media">NO MEDIA</span>
          {/if}
        </div>
        {#if images.length > 1}
          <div class="thumbs custom-scroll">
            {#each images as img}
              <div
                class="thumb"
                class:thumb-active={img === activeImage}
                onclick={() => (activeImage = img)}
                role="button"
                tabindex="0"
                onkeydown={(e) => e.key === 'Enter' && (activeImage = img)}
              >
                <img src={img} alt="" class="thumb-img" onerror={onImgError} />
              </div>
            {/each}
          </div>
        {/if}
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
            <span class="stat-lbl">Downloads</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-icon">◉</span>
            <span class="stat-val">{Number(views).toLocaleString()}</span>
            <span class="stat-lbl">Views</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item stat-fav">
            <span class="stat-icon">♥</span>
            <span class="stat-val">{Number(likes).toLocaleString()}</span>
            <span class="stat-lbl">Likes</span>
          </div>
        </div>

        {#if uploadedAt || syncedAt}
          <div class="meta-grid">
            {#if uploadedAt}
              <span class="meta-key">Uploaded</span>
              <span class="meta-val">{uploadedAt}</span>
            {/if}
            {#if syncedAt}
              <span class="meta-key">Synced</span>
              <span class="meta-val">{syncedAt}</span>
            {/if}
          </div>
        {/if}

        <div class="actions">
          <div class="want-row">
            <Button variant="icon" class="btn-want-sm" active={isWanted} onclick={handleToggleWant} title={isWanted ? 'Remove from wishlist' : 'Add to wishlist'}>♥</Button>
            <Button variant="icon" class="btn-discard-sm" title="Discard" onclick={handleDiscard}>✕</Button>
          </div>
          {#if p?.has_pab}
            <Button variant="ghost" class="btn-export" onclick={handleExport} disabled={exporting}>
              <span class="icon">⬆</span> {exporting ? 'Exporting...' : 'Export to Black Desert'}
            </Button>
            {#if exportError}
              <p class="export-error">{exportError}</p>
            {/if}
          {:else}
            <Button variant="ghost" class="btn-garmoth" onclick={openOnGarmoth}>
              <span class="icon">◈</span> View on Garmoth
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
