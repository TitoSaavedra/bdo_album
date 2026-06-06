<script lang="ts">
  import { onMount } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import type { ClassEntry } from '../../../../lib/album';
  import { getClassFavorites, setClassFavorite } from '../../../../lib/album';
  import Button from '../../../../ui/Button/Button.svelte';
  import Input from '../../../../ui/Input/Input.svelte';
  import Select from '../../../../ui/Select/Select.svelte';
  import Toggle from '../../../../ui/Toggle/Toggle.svelte';
  import {
    beauty,
    setClassFavorites,
    toggleClassFavorite,
  } from '../../state/beauty.svelte';

  interface Props {
    selectedClass: string | null;
    onselect:      (cls: ClassEntry) => void;
  }

  const { selectedClass, onselect } = $props<Props>();

  let search     = $state('');
  let filterOpen = $state(false);
  let sortBy     = $state<'downloads' | 'views' | 'likes'>('downloads');

  onMount(async () => {
    try {
      const favs = await getClassFavorites();
      setClassFavorites(favs);
    } catch { /* ignore on first launch */ }
  });

  const filtered = $derived(
    search.trim()
      ? beauty.classes.filter(c => c.name.toLowerCase().includes(search.toLowerCase()))
      : beauty.classes
  );

  const sorted = $derived(
    [...filtered].sort((a, b) => {
      const aFav = beauty.classFavorites.has(a.name);
      const bFav = beauty.classFavorites.has(b.name);
      if (aFav !== bFav) return aFav ? -1 : 1;
      return b.preset_count - a.preset_count;
    })
  );

  const filterActive = $derived(sortBy !== 'downloads');

  async function handleToggleFavorite(name: string, e: MouseEvent) {
    e.stopPropagation();
    toggleClassFavorite(name);
    const isFav = beauty.classFavorites.has(name);
    try {
      await setClassFavorite(name, isFav);
    } catch { /* non-fatal */ }
  }
</script>

<div class="search-box">
  <div class="search-row">
    <Input bind:value={search} placeholder="Search classes..." />
    <Button
      variant="icon"
      active={filterActive}
      title="Filter & Sort"
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
        <span class="fp-field-label">Sort by</span>
        <Select
          bind:value={sortBy}
          options={[
            { value: 'downloads', label: 'Downloads' },
            { value: 'views',     label: 'Views' },
            { value: 'likes',     label: 'Likes' },
          ]}
        />
      </div>
    </div>
  {/if}
</div>

<div class="list custom-scroll">
  {#if beauty.classes.length === 0}
    <p class="status">Waiting for database...</p>
  {:else if sorted.length === 0}
    <p class="status">No results</p>
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
            {#if cls.preset_count > 0}
              <span class="count">{cls.preset_count}</span>
            {/if}
            <span
              class="heart"
              class:active={beauty.classFavorites.has(cls.name)}
              onclick={(e) => handleToggleFavorite(cls.name, e)}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && handleToggleFavorite(cls.name, e as unknown as MouseEvent)}
              title="Pin to top"
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
