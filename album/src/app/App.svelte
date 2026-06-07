<script lang="ts">
  import './App.scss';
  import { onMount, tick } from 'svelte';
  import { eventBus } from '../lib/events';
  import { getPresets, getWanted, getRegions } from '../lib/album';
  import {
    beauty,
    selectClass,
    setWantedPresets,
    clearLiveForClass,
    setSelectedRegion,
    setAvailableRegions,
  } from '../features/beauty/state/beauty.svelte';
  import type { ClassEntry, PresetEntry } from '../lib/album';
  import ClassList    from '../features/beauty/components/ClassList/ClassList.svelte';
  import PresetGrid   from '../features/beauty/components/PresetGrid/PresetGrid.svelte';
  import PresetDetail from '../features/beauty/components/PresetDetail/PresetDetail.svelte';
  import LiveDot       from '../ui/LiveDot/LiveDot.svelte';
  import PillSelector  from '../ui/PillSelector/PillSelector.svelte';
  import FaceGridView  from '../features/face_grid/components/FaceGridView/FaceGridView.svelte';

  type Tab = 'beauty' | 'face_grid';
  let activeTab = $state<Tab>('beauty');

  const LIMIT = 40;

  let presets        = $state<PresetEntry[]>([]);
  let presetsLoading = $state(false);
  let loadingMore    = $state(false);
  let presetsError   = $state('');
  let hasMore        = $state(false);
  let offset         = $state(0);

  let mainEl:              HTMLElement | null = $state(null);
  let sentinelEl:          HTMLElement | null = $state(null);
  let sentinelIntersecting                    = $state(false);

  const selectedClassId = $derived(
    beauty.classes.find(c => c.name === beauty.selectedClass)?.class_id ?? null
  );

  const livePresets = $derived(
    selectedClassId !== null ? (beauty.livePresets[selectedClassId] ?? []) : []
  );

  const regionPills = $derived([
    { value: '', label: 'All' },
    ...beauty.availableRegions.map(r => ({ value: r, label: r.toUpperCase() })),
  ]);

  onMount(async () => {
    await eventBus.init();
    const regions = await getRegions().catch(() => []);
    setAvailableRegions(regions);
    return () => eventBus.destroy();
  });

  // Reload when selected class OR region changes
  $effect(() => {
    const cls    = beauty.selectedClass;
    const region = beauty.selectedRegion;
    if (cls) resetAndLoad(cls, region);
  });

  // Infinite scroll — fires when sentinel enters the scroll container
  $effect(() => {
    if (!sentinelEl || !mainEl) return;
    const obs = new IntersectionObserver(
      ([entry]) => {
        sentinelIntersecting = entry.isIntersecting;
        if (entry.isIntersecting) loadMore();
      },
      { root: mainEl, threshold: 0.1 },
    );
    obs.observe(sentinelEl);
    return () => obs.disconnect();
  });

  async function resetAndLoad(cls: string, region: string) {
    offset = 0;
    hasMore = false;
    presets = [];
    presetsError = '';
    const entry = beauty.classes.find(c => c.name === cls);
    if (entry) clearLiveForClass(entry.class_id);
    await doLoad(cls, 0, region, true);
  }

  async function loadMore() {
    if (!beauty.selectedClass || !hasMore || loadingMore || presetsLoading) return;
    loadingMore = true;  // lock immediately — prevents concurrent calls racing past the guard
    const nextOffset = offset + LIMIT;
    offset = nextOffset;
    await doLoad(beauty.selectedClass, nextOffset, beauty.selectedRegion, false);
  }

  async function doLoad(cls: string, off: number, region: string, initial: boolean) {
    if (initial) presetsLoading = true;
    else         loadingMore    = true;
    presetsError = '';
    try {
      if (initial) {
        const [fetched, wanted] = await Promise.all([
          getPresets(cls, off, LIMIT, 'downloads', '', region),
          getWanted(),
        ]);
        presets = fetched;
        setWantedPresets(wanted);
        hasMore = fetched.length >= LIMIT;
      } else {
        const fetched = await getPresets(cls, off, LIMIT, 'downloads', '', region);
        const seen = new Set(presets.map(p => p.preset_id));
        presets = [...presets, ...fetched.filter(p => !seen.has(p.preset_id))];
        hasMore = fetched.length >= LIMIT;
      }
    } catch (e) {
      presetsError = String(e);
    } finally {
      presetsLoading = false;
      loadingMore    = false;
      // On large screens the sentinel may stay visible after a batch loads —
      // the observer won't re-fire, so we kick another load manually.
      await tick();
      if (hasMore && sentinelIntersecting) loadMore();
    }
  }

  function handleSelectClass(cls: ClassEntry) {
    setSelectedRegion('');
    selectClass(cls);
  }

  function handleRegionChange(region: string | number) {
    setSelectedRegion(String(region));
  }
</script>

<div class="app">
  {#if !beauty.dbReady && !beauty.dbError}
    <div class="splash">
      <div class="splash-text">Connecting to database...</div>
    </div>
  {:else if beauty.dbError}
    <div class="splash">
      <div class="splash-text error">{beauty.dbError}</div>
    </div>
  {:else}
    <nav class="tab-nav">
      <button class:active={activeTab === 'beauty'}    onclick={() => activeTab = 'beauty'}>
        Beauty Album
      </button>
      <button class:active={activeTab === 'face_grid'} onclick={() => activeTab = 'face_grid'}>
        Character Grid
      </button>
    </nav>

    {#if activeTab === 'beauty'}
      <div class="layout">
        <aside class="sidebar">
          <ClassList
            selectedClass={beauty.selectedClass}
            onselect={handleSelectClass}
          />
        </aside>
        <main class="main custom-scroll" bind:this={mainEl}>
          {#if beauty.selectedClass && beauty.availableRegions.length > 0}
            <div class="region-bar">
              <PillSelector
                value={beauty.selectedRegion}
                options={regionPills}
                onchange={handleRegionChange}
              />
            </div>
          {/if}
          <PresetGrid
            {presets}
            {livePresets}
            selectedClass={beauty.selectedClass}
            loading={presetsLoading}
            error={presetsError}
            {loadingMore}
          />
          <div bind:this={sentinelEl} class="scroll-sentinel"></div>
        </main>
      </div>
    {:else}
      <FaceGridView />
    {/if}
  {/if}

  <PresetDetail />
  <LiveDot />
</div>
