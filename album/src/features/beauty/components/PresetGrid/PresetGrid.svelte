<script lang="ts">
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { _ } from 'svelte-i18n';
  import type { PresetEntry } from '../../../../lib/album';
  import { beauty } from '../../state/beauty.svelte';
  import PresetCard from '../PresetCard/PresetCard.svelte';

  interface Props {
    presets:       PresetEntry[];
    livePresets?:  PresetEntry[];
    selectedClass: string | null;
    loading?:      boolean;
    error?:        string;
    loadingMore?:  boolean;
  }

  const {
    presets,
    livePresets = [],
    selectedClass,
    loading = false,
    error = '',
    loadingMore = false,
  }: Props = $props();

  let discarded = $state(new Set<string>());

  const hasFilters = $derived(
    !!beauty.searchQuery.trim() || !!beauty.selectedRegion || beauty.selectedDays !== 'ever' || !!beauty.creatorFilter
  );

  const liveIds = $derived(new Set(livePresets.map(p => p.preset_id)));

  // Tier 0: has PAB file (green) — top
  // Tier 1: wanted (purple) — after greens
  // Tier 2: favorite creator (red) — only when neither of the above already applies
  // Tier 3: everything else
  function tier(p: PresetEntry): number {
    if (p.has_pab) return 0;
    if (beauty.wantedPresets.has(p.preset_id)) return 1;
    if (p.user_nickname && beauty.creatorFavorites.has(p.user_nickname)) return 2;
    return 3;
  }

  const localPresets = $derived.by(() => {
    const seen = new Set<string>();
    return [...livePresets, ...presets]
      .filter(p => {
        if (discarded.has(p.preset_id) || seen.has(p.preset_id)) return false;
        seen.add(p.preset_id);
        return true;
      })
      .sort((a, b) => tier(a) - tier(b));
  });

  function handleDiscard(id: string) {
    discarded = new Set([...discarded, id]);
  }
</script>

{#if !selectedClass}
  <div class="state-msg">
    <div class="state-hint">{$_('beauty.preset_grid.select_class')}</div>
  </div>
{:else if loading}
  <div class="grid">
    {#each Array.from({ length: 12 }) as _, i}
      <div class="skel-card" style="animation-delay: {i * 50}ms"></div>
    {/each}
  </div>
{:else if error}
  <div class="state-msg">
    <div class="state-hint error">{error}</div>
  </div>
{:else if localPresets.length === 0 && livePresets.length === 0}
  <div class="state-msg">
    <div class="state-hint">
      {hasFilters ? $_('beauty.preset_grid.no_match_filters') : $_('beauty.preset_grid.no_presets')}
    </div>
  </div>
{:else}
  <div class="grid">
    {#each localPresets as preset, i (preset.preset_id ?? i)}
      <div
        in:fly={{ y: 20, duration: 220, easing: cubicOut }}
        class:live-card={liveIds.has(preset.preset_id)}
      >
        <PresetCard {preset} ondiscard={handleDiscard} />
      </div>
    {/each}
    {#if loadingMore}
      {#each Array.from({ length: 6 }) as _, i}
        <div class="skel-card" style="animation-delay: {i * 60}ms"></div>
      {/each}
    {/if}
  </div>
{/if}

<style lang="scss">
  @use './PresetGrid.scss';
</style>
