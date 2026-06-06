<script lang="ts">
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { PresetEntry } from '../../../../lib/album';
  import { beauty } from '../../state/beauty.svelte';
  import PresetCard from '../PresetCard/PresetCard.svelte';

  interface Props {
    presets:       PresetEntry[];
    selectedClass: string | null;
    loading?:      boolean;
    error?:        string;
    loadingMore?:  boolean;
  }

  const {
    presets,
    selectedClass,
    loading = false,
    error = '',
    loadingMore = false,
  } = $props<Props>();

  let discarded = $state(new Set<string>());

  const localPresets = $derived(
    [...presets]
      .filter(p => !discarded.has(p.preset_id))
      .sort((a, b) => {
        // Tier 1: has PAB file (green) — top
        if (a.has_pab !== b.has_pab) return a.has_pab ? -1 : 1;
        // Tier 2: wanted (purple) — after greens
        const aW = beauty.wantedPresets.has(a.preset_id);
        const bW = beauty.wantedPresets.has(b.preset_id);
        if (aW !== bW) return aW ? -1 : 1;
        return 0;
      })
  );

  function handleDiscard(id: string) {
    discarded = new Set([...discarded, id]);
  }
</script>

{#if !selectedClass}
  <div class="state-msg">
    <div class="state-hint">Select a class to browse presets</div>
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
{:else if localPresets.length === 0}
  <div class="state-msg">
    <div class="state-hint">No presets found in this class</div>
  </div>
{:else}
  <div class="grid">
    {#each localPresets as preset, i (preset.preset_id ?? i)}
      <div in:fly={{ y: 20, duration: 220, easing: cubicOut }}>
        <PresetCard {preset} {selectedClass} ondiscard={handleDiscard} />
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
