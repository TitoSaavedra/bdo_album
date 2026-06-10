<script lang="ts">
  import type { PresetEntry } from '../../../../lib/album';
  import { discardPreset, toggleWanted } from '../../../../lib/album';
  import {
    beauty,
    toggleWantedPreset,
    openPreset,
  } from '../../state/beauty.svelte';

  interface Props {
    preset:         PresetEntry;
    selectedClass?: string | null;
    ondiscard?:     (id: string) => void;
  }

  const { preset, selectedClass = null, ondiscard } = $props<Props>();

  const imageUrl  = $derived(preset.image_1_url ?? preset.image_2_url ?? null);
  const title     = $derived(preset.title || preset.character_name || `#${preset.preset_id}`);
  const downloads = $derived(preset.downloads ?? 0);
  const views     = $derived(preset.views ?? 0);
  const likes     = $derived(preset.likes ?? 0);
  const id        = $derived(preset.preset_id);
  const hasPab    = $derived(preset.has_pab);
  const isWanted  = $derived(beauty.wantedPresets.has(id));

  function handleClick() {
    openPreset(preset, selectedClass ?? '');
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

  let imageLoaded = $state(false);

  function onThumbLoad() {
    imageLoaded = true;
  }

  function onThumbError(e: Event) {
    (e.currentTarget as HTMLImageElement).style.display = 'none';
    imageLoaded = true;
  }
</script>

<div
  class="preset-card"
  class:wished={isWanted}
  class:has-pab={hasPab}
  onclick={handleClick}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === 'Enter' && handleClick()}
>
  <div class="thumb-wrap">
    {#if imageUrl}
      {#if !imageLoaded}
        <div class="skeleton-thumb"></div>
      {/if}
      <img
        src={imageUrl}
        alt={title}
        class="thumb"
        class:loaded={imageLoaded}
        onerror={onThumbError}
        onload={onThumbLoad}
        loading="lazy"
      />
    {:else}
      <div class="skeleton-thumb"></div>
    {/if}

    <div class="gradient-overlay">
      <h3 class="card-title" title={title}>{title}</h3>
    </div>

    <div class="card-actions">
      <button
        class="action-want"
        class:want-active={isWanted}
        title={isWanted ? 'Remove from wishlist' : 'Add to wishlist'}
        onclick={handleToggleWant}
      >
        <span class="action-icon">♥</span>
      </button>
      <button class="action-discard" title="Discard" onclick={handleDiscard}>✕</button>
    </div>
  </div>

  <div class="card-footer">
    <span class="stat" title="Downloads">↓ {Number(downloads).toLocaleString()}</span>
    <span class="stat" title="Views">◉ {Number(views).toLocaleString()}</span>
    <span class="stat stat-fav" title="Likes">♥ {Number(likes).toLocaleString()}</span>
  </div>
</div>

<style lang="scss">
  @use './PresetCard.scss';
</style>
