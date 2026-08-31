<script lang="ts">
  import { fade } from 'svelte/transition';
  import { _ } from 'svelte-i18n';
  import { openUrl, toggleWanted, discardPreset, exportToBdo, setCreatorFavorite } from '../../../../lib/album';
  import {
    beauty,
    closePreset,
    toggleWantedPreset,
    toggleCreatorFavorite,
  } from '../../state/beauty.svelte';
  import { withViewTransition } from '../../../../lib/viewTransition';
  import Button from '../../../../ui/Button/Button.svelte';

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
  const images = $derived(
    p ? [p.image_1_url, p.image_2_url].filter((u): u is string => !!u) : []
  );
  const title     = $derived(p ? (p.title || p.character_name || `#${p.preset_id}`) : '');
  const id        = $derived(p?.preset_id ?? '');
  const nickname  = $derived(p?.user_nickname || null);
  const downloads = $derived(p?.downloads ?? 0);
  const views     = $derived(p?.views ?? 0);
  const likes     = $derived(p?.likes ?? 0);
  const className     = $derived(p?.class_display ?? '');
  const hasPab        = $derived(p?.has_pab ?? false);
  const isWanted      = $derived(p ? beauty.wantedPresets.has(p.preset_id) : false);
  const isFavoriteCreator = $derived(nickname ? beauty.creatorFavorites.has(nickname) : false);
  // Same tier priority as PresetCard's badge — the hero carries the same
  // "why this preset looks the way it does" signal the grid card had.
  const showFavCreator = $derived(isFavoriteCreator && !hasPab && !isWanted);
  const tierBadge      = $derived(hasPab ? 'pab' : isWanted ? 'wanted' : showFavCreator ? 'creator' : null);
  const tierBadgeLabel = $derived(
    tierBadge === 'pab' ? $_('beauty.preset_card.tier_downloaded')
    : tierBadge === 'wanted' ? $_('beauty.preset_card.tier_wishlist')
    : tierBadge === 'creator' ? $_('beauty.preset_card.tier_favorite_creator')
    : ''
  );
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') goBack();
  }

  // Mouse "back" side button (common on gaming/productivity mice) closes the
  // detail view same as the back button / Escape. The "forward" side button
  // is handled in App.svelte — it only makes sense while the grid (not this
  // component) is showing, to reopen whatever was last closed.
  function onMouseDown(e: MouseEvent) {
    if (e.button === 3) { e.preventDefault(); goBack(); }
  }

  // Reverses the grow-in-place morph on the way back to the grid. Unlike the
  // expand direction, no specific grid card is re-targeted with the shared
  // view-transition-name (the grid hasn't re-rendered yet when this fires),
  // so this side just cross-fades via the generic root transition.
  function goBack() {
    withViewTransition(() => closePreset());
  }

  async function handleToggleWant() {
    if (!p) return;
    toggleWantedPreset(p.preset_id);
    try { await toggleWanted(p.preset_id); } catch { /* non-fatal */ }
  }

  async function handleToggleCreatorFavorite() {
    if (!nickname) return;
    toggleCreatorFavorite(nickname);
    const isFav = beauty.creatorFavorites.has(nickname);
    try { await setCreatorFavorite(nickname, isFav); } catch { /* non-fatal */ }
  }

  async function handleDiscard() {
    if (!p) return;
    const pid = p.preset_id;
    goBack();
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

<svelte:window onkeydown={onKeydown} onmousedown={onMouseDown} />

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
  <div class="detail-panel custom-scroll">
    <button class="back-btn" onclick={goBack}>
      <span class="back-arrow">‹</span> {$_('beauty.preset_detail.back_to_grid')}
    </button>

    <div class="detail-hero">
      {#if activeImage}
        {#key activeImage}
          <img
            src={activeImage}
            alt={title}
            class="hero-img"
            class:sharpen={sharpenOn}
            onerror={onImgError}
            in:fade={{ duration: 100 }}
            out:fade={{ duration: 100 }}
          />
        {/key}
      {:else}
        <span class="no-media">{$_('beauty.preset_detail.no_media')}</span>
      {/if}

      {#if tierBadge}
        <span class="tier-badge tier-badge-{tierBadge}">{tierBadgeLabel}</span>
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

    <div class="detail-body">
      <div class="header-meta">
        {#if className}
          <span class="class-tag">{className}</span>
        {/if}
        <span class="preset-id">#{id}</span>
      </div>

      <h1 class="detail-title">{title}</h1>

      {#if nickname}
        <div class="creator-row">
          <span class="creator-at">@</span><span class="creator-name">{nickname}</span>
          <button
            class="creator-fav-btn"
            class:on={isFavoriteCreator}
            title={isFavoriteCreator ? $_('beauty.preset_detail.unfavorite_creator') : $_('beauty.preset_detail.favorite_creator')}
            onclick={handleToggleCreatorFavorite}
          >♥</button>
        </div>
      {/if}

      <div class="stats-row">
        <span><b>{Number(downloads).toLocaleString()}</b> {$_('beauty.preset_detail.downloads')}</span>
        <span><b>{Number(views).toLocaleString()}</b> {$_('beauty.preset_detail.views')}</span>
        <span><b>{Number(likes).toLocaleString()}</b> {$_('beauty.preset_detail.likes')}</span>
      </div>

      <div class="actions">
        {#if hasPab}
          <Button variant="primary" class="btn-export" onclick={handleExport} disabled={exporting}>
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
        <div class="want-row">
          <Button variant="ghost" class="btn-want" active={isWanted} onclick={handleToggleWant}>
            ♥ {isWanted ? $_('beauty.preset_detail.remove_wishlist') : $_('beauty.preset_detail.add_wishlist')}
          </Button>
          <Button variant="icon" class="btn-discard-sm" title={$_('beauty.preset_detail.discard')} onclick={handleDiscard}>✕</Button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style lang="scss">
  @use './PresetDetail.scss';
</style>
