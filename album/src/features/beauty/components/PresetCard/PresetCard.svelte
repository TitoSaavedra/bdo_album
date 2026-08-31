<script lang="ts">
  import { _ } from 'svelte-i18n';
  import type { PresetEntry } from '../../../../lib/album';
  import { discardPreset, toggleWanted } from '../../../../lib/album';
  import {
    beauty,
    toggleWantedPreset,
    openPreset,
  } from '../../state/beauty.svelte';
  import { withViewTransition } from '../../../../lib/viewTransition';

  interface Props {
    preset:     PresetEntry;
    ondiscard?: (id: string) => void;
  }

  const { preset, ondiscard } = $props<Props>();

  const imageUrl  = $derived(preset.image_1_url ?? preset.image_2_url ?? null);
  const title     = $derived(preset.title || preset.character_name || `#${preset.preset_id}`);
  const downloads = $derived(preset.downloads ?? 0);
  const views     = $derived(preset.views ?? 0);
  const likes     = $derived(preset.likes ?? 0);
  const id        = $derived(preset.preset_id);
  const hasPab    = $derived(preset.has_pab);
  const isWanted  = $derived(beauty.wantedPresets.has(id));
  const nickname  = $derived(preset.user_nickname);
  const isFavoriteCreator = $derived(nickname ? beauty.creatorFavorites.has(nickname) : false);
  // Only tint the card for a favorite creator when it's not already flagged
  // green (downloaded) or purple (wishlisted) — those two already earn a look.
  const showFavCreator = $derived(isFavoriteCreator && !hasPab && !isWanted);

  const tierBadge = $derived(
    hasPab ? 'pab' : isWanted ? 'wanted' : showFavCreator ? 'creator' : null
  );
  const tierBadgeLabel = $derived(
    tierBadge === 'pab' ? $_('beauty.preset_card.tier_downloaded')
    : tierBadge === 'wanted' ? $_('beauty.preset_card.tier_wishlist')
    : tierBadge === 'creator' ? $_('beauty.preset_card.tier_favorite_creator')
    : ''
  );

  let cardEl: HTMLElement | undefined = $state();

  // Grow-in-place morph into PresetDetail via the View Transitions API — the
  // thumb (not the whole card) carries the shared view-transition-name so
  // only the image morphs into the detail hero; falls back to an instant
  // swap when the API is unsupported or reduced-motion is requested.
  function handleClick() {
    const thumb = cardEl?.querySelector<HTMLElement>('.thumb, .skeleton-thumb');
    if (thumb) thumb.style.viewTransitionName = 'morph-thumb';
    withViewTransition(() => openPreset(preset, preset.class_name));
  }

  async function handleToggleWant(e: MouseEvent) {
    e.stopPropagation();
    toggleWantedPreset(id);
    try { await toggleWanted(id); } catch { /* non-fatal */ }
  }

  async function handleDiscard(e: MouseEvent) {
    e.stopPropagation();
    ondiscard?.(id);
    try { await discardPreset(id); } catch { /* non-fatal */ }
  }

  function onThumbError(e: Event) {
    (e.currentTarget as HTMLImageElement).style.display = 'none';
  }
</script>

<div
  class="preset-card"
  class:wished={isWanted}
  class:has-pab={hasPab}
  class:fav-creator={showFavCreator}
  bind:this={cardEl}
  onclick={handleClick}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === 'Enter' && handleClick()}
>
  <div class="thumb-wrap">
    {#if imageUrl}
      <img src={imageUrl} alt={title} class="thumb" onerror={onThumbError} loading="lazy" />
    {:else}
      <div class="skeleton-thumb"></div>
    {/if}

    {#if beauty.creatorFilter}
      <span class="class-tag">{preset.class_name}</span>
    {/if}

    {#if tierBadge}
      <span class="tier-badge tier-badge-{tierBadge}">{tierBadgeLabel}</span>
    {/if}

    <div class="gradient-overlay">
      <h3 class="card-title" title={title}>{title}</h3>
      {#if nickname}
        <div class="creator-row">
          <span class="creator-name" title={nickname}>@{nickname}</span>
        </div>
      {/if}
    </div>

    <div class="card-actions">
      <button
        class="action-want"
        class:want-active={isWanted}
        title={isWanted ? $_('beauty.preset_card.remove_wishlist') : $_('beauty.preset_card.add_wishlist')}
        onclick={handleToggleWant}
      >
        <span class="action-icon">♥</span>
      </button>
      <button class="action-discard" title={$_('beauty.preset_card.discard')} onclick={handleDiscard}>✕</button>
    </div>
  </div>

  <div class="card-footer">
    <span class="stat" title={$_('beauty.preset_card.downloads')}>↓ {Number(downloads).toLocaleString()}</span>
    <span class="stat" title={$_('beauty.preset_card.views')}>◉ {Number(views).toLocaleString()}</span>
    <span class="stat stat-fav" title={$_('beauty.preset_card.likes')}>♥ {Number(likes).toLocaleString()}</span>
  </div>
</div>

<style lang="scss">
  @use './PresetCard.scss';
</style>
