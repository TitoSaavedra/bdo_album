<script lang="ts">
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import { _ } from 'svelte-i18n';
  import type { ClassEntry, PresetEntry } from '../../../../lib/album';
  import {
    getClassFavorites, setClassFavorite,
    getCreatorFavorites, setCreatorFavorite,
    getWantedPresets,
  } from '../../../../lib/album';
  import Input       from '../../../../ui/Input/Input.svelte';
  import PillSelector        from '../../../../ui/PillSelector/PillSelector.svelte';
  import Dialog              from '../../../../ui/Dialog/Dialog.svelte';
  import WantedDownloadModal from '../WantedDownloadModal/WantedDownloadModal.svelte';
  import {
    beauty,
    setClassFavorites,
    toggleClassFavorite,
    setCreatorFavorites,
    toggleCreatorFavorite,
    setCreatorFilter,
    setSelectedRegion,
    setSelectedDays,
    setSelectedSort,
  } from '../../state/beauty.svelte';

  interface Props {
    selectedClass: string | null;
    onselect:      (cls: ClassEntry) => void;
    moduleSwitcher?: import('svelte').Snippet;
  }

  const { selectedClass, onselect, moduleSwitcher }: Props = $props();

  let searchInput   = $state('');
  let popoverOpen   = $state(false);
  let modalPresets  = $state<PresetEntry[]>([]);
  let modalOpen     = $state(false);
  let classPillsEl: HTMLElement | undefined = $state();

  // The class strip only scrolls horizontally — redirect vertical wheel
  // input into it (standard treatment for a horizontal-scroll row) instead
  // of requiring a shift-scroll or a drag on the thin scrollbar. Only when
  // there's actually overflow to scroll, so the page's own scroll isn't
  // hijacked when every class pill already fits on screen.
  function onClassPillsWheel(e: WheelEvent) {
    if (!classPillsEl || e.deltaY === 0) return;
    if (classPillsEl.scrollWidth <= classPillsEl.clientWidth) return;
    e.preventDefault();
    classPillsEl.scrollLeft += e.deltaY;
  }

  // Click-and-drag ("grab to scroll") on the same strip. `dragMoved` gates
  // the click-capture handler below so a drag that ends on top of a pill
  // doesn't also select that class — only a genuine stationary click does.
  let isDragging   = $state(false);
  let dragMoved    = $state(false);
  let dragStartX   = 0;
  let dragStartScroll = 0;

  function onClassPillsMouseDown(e: MouseEvent) {
    if (!classPillsEl || e.button !== 0) return;
    isDragging   = true;
    dragMoved    = false;
    dragStartX   = e.clientX;
    dragStartScroll = classPillsEl.scrollLeft;
  }

  function onWindowMouseMove(e: MouseEvent) {
    if (!isDragging || !classPillsEl) return;
    const dx = e.clientX - dragStartX;
    if (Math.abs(dx) > 4) dragMoved = true;
    if (dragMoved) classPillsEl.scrollLeft = dragStartScroll - dx;
  }

  function onWindowMouseUp() {
    isDragging = false;
  }

  function onClassPillsClickCapture(e: MouseEvent) {
    if (dragMoved) { e.preventDefault(); e.stopPropagation(); }
  }

  let confirmUnfavoriteCreator: string | null = $state(null);

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
    const [classFavs, creatorFavs] = await Promise.allSettled([
      getClassFavorites(),
      getCreatorFavorites(),
    ]);
    if (classFavs.status === 'fulfilled')   setClassFavorites(classFavs.value);
    if (creatorFavs.status === 'fulfilled') setCreatorFavorites(creatorFavs.value);
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

  const activeFilterCount = $derived(
    (beauty.selectedRegion ? 1 : 0) + (beauty.selectedDays !== 'ever' ? 1 : 0)
  );

  const favoriteCreatorNames = $derived(Array.from(beauty.creatorFavorites).sort());

  // Deterministic hue per creator so their avatar dot in the chip cloud stays
  // stable across renders without needing to store a color anywhere.
  function creatorHue(name: string): number {
    let h = 0;
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) % 360;
    return h;
  }

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

  async function handleToggleCreatorFavorite(name: string) {
    toggleCreatorFavorite(name);
    const isFav = beauty.creatorFavorites.has(name);
    try {
      await setCreatorFavorite(name, isFav);
    } catch { /* non-fatal */ }
  }

  function handleCreatorChipClick(name: string) {
    setCreatorFilter(beauty.creatorFilter === name ? null : name);
  }
</script>

<svelte:window onmousemove={onWindowMouseMove} onmouseup={onWindowMouseUp} />

<header class="command-bar">
  <div class="bar-row">
    {#if moduleSwitcher}
      {@render moduleSwitcher()}
      <div class="bar-divider"></div>
    {/if}

    <div class="search-wrap">
      <Input bind:value={searchInput} placeholder={$_('beauty.class_list.search_placeholder')}>
        {#snippet icon()}
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.4"/>
            <line x1="10.8" y1="10.8" x2="14" y2="14" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
          </svg>
        {/snippet}
        {#snippet trailing()}
          {#if searchInput}
            <button class="search-clear" onclick={() => (searchInput = '')} title={$_('ui.remove')}>✕</button>
          {/if}
        {/snippet}
      </Input>
    </div>

    <div class="seg">
      {#each SORT_PILLS as opt (opt.value)}
        <button
          class="seg-btn"
          class:active={beauty.sortBy === opt.value}
          onclick={() => setSelectedSort(opt.value as 'downloads' | 'views' | 'likes')}
        >{opt.label}</button>
      {/each}
    </div>

    <div class="bar-actions">
      <button
        class="tool-btn wishlist"
        class:has={beauty.wantedPresets.size > 0}
        title={$_('beauty.class_list.open_wanted_title', { values: { count: beauty.wantedPresets.size } })}
        onclick={openWantedPabs}
        disabled={beauty.wantedPresets.size === 0}
      >
        <span>♥ {$_('beauty.class_list.wishlist')}</span>
        <span class="badge">{beauty.wantedPresets.size}</span>
      </button>

      <div class="pop-anchor">
        <button
          class="tool-btn filters"
          class:active={activeFilterCount > 0}
          title={$_('beauty.class_list.filters')}
          onclick={() => (popoverOpen = !popoverOpen)}
        >
          <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
            <line x1="2" y1="4" x2="14" y2="4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="2" y1="8" x2="14" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="2" y1="12" x2="14" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <circle cx="5.5" cy="4" r="1.6" fill="var(--color-bg-input)" stroke="currentColor" stroke-width="1.3"/>
            <circle cx="10.5" cy="8" r="1.6" fill="var(--color-bg-input)" stroke="currentColor" stroke-width="1.3"/>
            <circle cx="6" cy="12" r="1.6" fill="var(--color-bg-input)" stroke="currentColor" stroke-width="1.3"/>
          </svg>
          <span>{$_('beauty.class_list.filters')}</span>
          {#if activeFilterCount > 0}
            <span class="badge">{activeFilterCount}</span>
          {/if}
        </button>

        {#if popoverOpen}
          <div class="backdrop" onclick={() => (popoverOpen = false)}></div>
          <div class="popover" transition:fly={{ y: -6, duration: 140 }}>
            <div class="pop-head">
              <span class="pop-title">{$_('beauty.class_list.filters')}</span>
              <button class="pop-close" onclick={() => (popoverOpen = false)}>✕</button>
            </div>

            {#if beauty.availableRegions.length > 0}
              <div class="pop-group" class:disabled={!!beauty.creatorFilter}>
                <span class="pop-label">{$_('beauty.class_list.region_label')}</span>
                <PillSelector
                  value={beauty.selectedRegion}
                  options={regionPills}
                  onchange={v => setSelectedRegion(String(v))}
                />
              </div>
            {/if}

            <div class="pop-group" class:disabled={!!beauty.creatorFilter}>
              <span class="pop-label">{$_('beauty.class_list.uploaded_label')}</span>
              <PillSelector
                value={beauty.selectedDays}
                options={DAY_PILLS}
                onchange={v => setSelectedDays(String(v))}
              />
            </div>

            {#if beauty.creatorFilter}
              <p class="pop-note">{$_('beauty.class_list.filters_ignored_creator')}</p>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  {#if beauty.creatorFilter || beauty.selectedRegion || beauty.selectedDays !== 'ever'}
    <div class="active-chips">
      {#if beauty.creatorFilter}
        <span class="chip creator">
          ♥ <b>{$_('beauty.class_list.chip_creator')}:</b> {beauty.creatorFilter}
          <button class="x" onclick={() => setCreatorFilter(null)}>✕</button>
        </span>
      {:else}
        {#if beauty.selectedRegion}
          <span class="chip">
            <b>{$_('beauty.class_list.chip_region')}:</b> {beauty.selectedRegion.toUpperCase()}
            <button class="x" onclick={() => setSelectedRegion('')}>✕</button>
          </span>
        {/if}
        {#if beauty.selectedDays !== 'ever'}
          <span class="chip">
            <b>{$_('beauty.class_list.chip_uploaded')}:</b> {DAY_PILLS.find(d => d.value === beauty.selectedDays)?.label}
            <button class="x" onclick={() => setSelectedDays('ever')}>✕</button>
          </span>
        {/if}
        {#if activeFilterCount > 1}
          <button class="clear-all" onclick={() => { setSelectedRegion(''); setSelectedDays('ever'); }}>
            {$_('beauty.class_list.clear_all')}
          </button>
        {/if}
      {/if}
    </div>
  {/if}

  <div
    class="class-pills"
    class:dimmed={!!beauty.creatorFilter}
    class:dragging={isDragging && dragMoved}
    aria-label={$_('beauty.class_list.classes')}
    bind:this={classPillsEl}
    onwheel={onClassPillsWheel}
    onmousedown={onClassPillsMouseDown}
    onclickcapture={onClassPillsClickCapture}
  >
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
        <div animate:flip={{ duration: 220 }} in:fly={{ y: 8, duration: 180 }} class="pill-wrap">
          <button
            class="class-pill"
            class:active={cls.name === selectedClass && !beauty.creatorFilter}
            onclick={() => onselect(cls)}
          >
            {#if cls.icon_svg}
              <span class="pill-icon">{@html cls.icon_svg}</span>
            {/if}
            <span class="pill-name">{cls.name}</span>
            {#if !beauty.searchQuery.trim() && (beauty.liveUploaded[cls.class_id] ?? 0) > 0}
              <span class="live-badge">+{beauty.liveUploaded[cls.class_id]}</span>
            {/if}
            {#if displayCount(cls) > 0}
              <span class="pill-count">{displayCount(cls)}</span>
            {/if}
            <span
              class="pill-heart"
              class:active={beauty.classFavorites.has(cls.name)}
              onclick={(e) => handleToggleFavorite(cls.name, e)}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && handleToggleFavorite(cls.name, e as unknown as MouseEvent)}
              title={$_('beauty.class_list.pin_to_top')}
            >♥</span>
          </button>
        </div>
      {/each}
    {/if}
  </div>

  {#if favoriteCreatorNames.length > 0}
    <div class="creator-chips" aria-label={$_('beauty.class_list.favorite_creators')}>
      {#each favoriteCreatorNames as name (name)}
        <span class="creator-chip" class:active={beauty.creatorFilter === name}>
          <button class="creator-chip-main" onclick={() => handleCreatorChipClick(name)}>
            <span class="avatar" style="background: hsl({creatorHue(name)} 55% 40%)">{name.charAt(0).toUpperCase()}</span>
            <span class="name">{name}</span>
          </button>
          <button
            class="creator-chip-remove"
            onclick={() => (confirmUnfavoriteCreator = name)}
            title={$_('beauty.class_list.unfavorite_creator')}
          >✕</button>
        </span>
      {/each}
    </div>
  {/if}
</header>

{#if confirmUnfavoriteCreator}
  <Dialog
    title={$_('beauty.class_list.unfavorite_creator_title')}
    message={$_('beauty.class_list.unfavorite_creator_msg', { values: { creator: confirmUnfavoriteCreator } })}
    submitText={$_('beauty.class_list.unfavorite_creator_confirm')}
    onsubmit={() => {
      if (confirmUnfavoriteCreator) handleToggleCreatorFavorite(confirmUnfavoriteCreator);
      confirmUnfavoriteCreator = null;
    }}
    oncancel={() => (confirmUnfavoriteCreator = null)}
  />
{/if}

{#if modalOpen}
  <WantedDownloadModal
    presets={modalPresets}
    onclose={() => { modalOpen = false; modalPresets = []; }}
  />
{/if}

<style lang="scss">
  @use './ClassList.scss';
</style>
