<script lang="ts">
  import { onMount } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { _ } from 'svelte-i18n';
  import type { ClassEntry, PresetEntry } from '../../../../lib/album';
  import { getClassFavorites, setClassFavorite, getWantedPresets } from '../../../../lib/album';
  import Button              from '../../../../ui/Button/Button.svelte';
  import Input               from '../../../../ui/Input/Input.svelte';
  import PillSelector        from '../../../../ui/PillSelector/PillSelector.svelte';
  import WantedDownloadModal from '../WantedDownloadModal/WantedDownloadModal.svelte';
  import {
    beauty,
    setClassFavorites,
    toggleClassFavorite,
    setSelectedRegion,
    setSelectedDays,
    setSelectedSort,
  } from '../../state/beauty.svelte';

  interface Props {
    selectedClass: string | null;
    onselect:      (cls: ClassEntry) => void;
  }

  const { selectedClass, onselect } = $props<Props>();

  let searchInput  = $state('');
  let filterOpen   = $state(false);
  let modalPresets = $state<PresetEntry[]>([]);
  let modalOpen    = $state(false);

  const SORT_PILLS = $derived([
    { value: 'downloads', label: $_('beauty.class_list.sort_downloads') },
    { value: 'views',     label: $_('beauty.class_list.sort_views')     },
    { value: 'likes',     label: $_('beauty.class_list.sort_likes')     },
  ]);

  const DAY_PILLS = $derived([
    { value: 'ever', label: $_('beauty.class_list.day_all') },
    { value: '20',   label: $_('beauty.class_list.day_20')  },
    { value: '30',   label: $_('beauty.class_list.day_30')  },
    { value: '60',   label: $_('beauty.class_list.day_60')  },
    { value: '90',   label: $_('beauty.class_list.day_90')  },
    { value: '180',  label: $_('beauty.class_list.day_180') },
    { value: '365',  label: $_('beauty.class_list.day_365') },
  ]);

  const regionPills = $derived([
    { value: '', label: $_('beauty.class_list.day_all') },
    ...beauty.availableRegions.map(r => ({ value: r, label: r.toUpperCase() })),
  ]);

  $effect(() => {
    const q = searchInput;
    const t = setTimeout(() => { beauty.searchQuery = q; }, 300);
    return () => clearTimeout(t);
  });

  onMount(async () => {
    try {
      const favs = await getClassFavorites();
      setClassFavorites(favs);
    } catch { /* ignore on first launch */ }
  });

  function displayCount(cls: ClassEntry): number {
    const hasFilter = beauty.searchQuery.trim() || beauty.selectedRegion || beauty.selectedDays !== 'ever';
    if (hasFilter) return beauty.searchCounts[cls.class_id] ?? 0;
    return cls.preset_count + (beauty.liveUploaded[cls.class_id] ?? 0);
  }

  const sorted = $derived.by(() => {
    const hasFilter   = !!(beauty.searchQuery.trim() || beauty.selectedRegion || beauty.selectedDays !== 'ever');
    const countsReady = hasFilter && beauty.searchCountsLoaded;
    return [...beauty.classes]
      .filter(cls => !countsReady || displayCount(cls) > 0)
      .sort((a, b) => {
        const aFav = beauty.classFavorites.has(a.name);
        const bFav = beauty.classFavorites.has(b.name);
        if (aFav !== bFav) return aFav ? -1 : 1;
        return displayCount(b) - displayCount(a);
      });
  });

  const filterActive = $derived(
    beauty.sortBy !== 'downloads' || beauty.selectedRegion !== '' || beauty.selectedDays !== 'ever'
  );

  async function openWantedPabs() {
    const presets = await getWantedPresets().catch(() => []);
    if (presets.length === 0) return;
    modalPresets = presets;
    modalOpen    = true;
  }

  async function handleToggleFavorite(name: string, e: MouseEvent) {
    e.stopPropagation();
    toggleClassFavorite(name);
    const isFav = beauty.classFavorites.has(name);
    try {
      await setClassFavorite(name, isFav);
    } catch { /* non-fatal */ }
  }
</script>

<div class="filter-box">
  <div class="search-row">
    <Input bind:value={searchInput} placeholder={$_('beauty.class_list.search_placeholder')} />
    <button
      class="open-wanted-btn"
      class:has-wanted={beauty.wantedPresets.size > 0}
      title={$_('beauty.class_list.open_wanted_title', { values: { count: beauty.wantedPresets.size } })}
      onclick={openWantedPabs}
      disabled={beauty.wantedPresets.size === 0}
    >♥</button>
    <Button
      variant="icon"
      active={filterActive}
      title={$_('beauty.class_list.filter_sort')}
      onclick={() => (filterOpen = !filterOpen)}
    >
      <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
        <line x1="2" y1="4" x2="14" y2="4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="2" y1="8" x2="14" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <line x1="2" y1="12" x2="14" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <circle cx="5.5" cy="4" r="1.75" fill="var(--color-bg-surface)" stroke="currentColor" stroke-width="1.3"/>
        <circle cx="10.5" cy="8" r="1.75" fill="var(--color-bg-surface)" stroke="currentColor" stroke-width="1.3"/>
        <circle cx="6" cy="12" r="1.75" fill="var(--color-bg-surface)" stroke="currentColor" stroke-width="1.3"/>
      </svg>
      {#if filterActive}
        <span class="filter-dot"></span>
      {/if}
    </Button>
  </div>

  {#if filterOpen}
    <div class="filter-panel" transition:slide={{ duration: 180 }}>
      <div class="fp-field">
        <span class="fp-field-label">{$_('beauty.class_list.sort_label')}</span>
        <PillSelector
          value={beauty.sortBy}
          options={SORT_PILLS}
          onchange={v => setSelectedSort(v as 'downloads' | 'views' | 'likes')}
        />
      </div>

      {#if beauty.availableRegions.length > 0}
        <div class="fp-group">
          <span class="fp-group-label">{$_('beauty.class_list.region_label')}</span>
          <PillSelector
            value={beauty.selectedRegion}
            options={regionPills}
            onchange={v => setSelectedRegion(String(v))}
          />
        </div>
      {/if}

      <div class="fp-group">
        <span class="fp-group-label">{$_('beauty.class_list.uploaded_label')}</span>
        <PillSelector
          value={beauty.selectedDays}
          options={DAY_PILLS}
          onchange={v => setSelectedDays(String(v))}
        />
      </div>
    </div>
  {/if}
</div>

{#if modalOpen}
  <WantedDownloadModal
    presets={modalPresets}
    onclose={() => { modalOpen = false; modalPresets = []; }}
  />
{/if}

<div class="list custom-scroll">
  {#if beauty.classes.length === 0}
    <p class="status">{$_('beauty.class_list.waiting_db')}</p>
  {:else if sorted.length === 0}
    <p class="status">
      {beauty.searchQuery.trim() || beauty.selectedRegion || beauty.selectedDays !== 'ever'
        ? $_('beauty.class_list.no_match_filters')
        : $_('beauty.class_list.no_results')}
    </p>
  {:else}
    {#each sorted as cls (cls.name)}
      <div animate:flip={{ duration: 220 }} in:fly={{ y: 8, duration: 180 }} class="class-row">
        <button
          class="class-btn"
          class:active={cls.name === selectedClass}
          onclick={() => onselect(cls)}
        >
          <span class="cls-name">{cls.name}</span>
          <div class="cls-right">
            {#if !beauty.searchQuery.trim() && (beauty.liveUploaded[cls.class_id] ?? 0) > 0}
              <span class="live-badge">+{beauty.liveUploaded[cls.class_id]}</span>
            {/if}
            {#if displayCount(cls) > 0}
              <span class="count">{displayCount(cls)}</span>
            {/if}
            <span
              class="heart"
              class:active={beauty.classFavorites.has(cls.name)}
              onclick={(e) => handleToggleFavorite(cls.name, e)}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && handleToggleFavorite(cls.name, e as unknown as MouseEvent)}
              title={$_('beauty.class_list.pin_to_top')}
            >♥</span>
            {#if cls.icon_svg}
              <span class="cls-icon">{@html cls.icon_svg}</span>
            {/if}
          </div>
        </button>
      </div>
    {/each}
  {/if}
</div>

<style lang="scss">
  @use './ClassList.scss';
</style>
