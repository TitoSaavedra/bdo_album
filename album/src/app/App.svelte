<script lang="ts">
  import './App.scss';
  import { onMount, tick, untrack } from 'svelte';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { _ } from 'svelte-i18n';
  import { eventBus } from '../lib/events';
  import Toast from '../ui/Toast/Toast.svelte';
  import type { ToastItem } from '../ui/Toast/Toast.svelte';
  import { getPresets, getWanted, getRegions, getClassSearchCounts } from '../lib/album';
  import {
    beauty,
    selectClass,
    setWantedPresets,
    clearLiveForClass,
    setSelectedRegion,
    setAvailableRegions,
    setSearchCounts,
  } from '../features/beauty/state/beauty.svelte';
  import type { ClassEntry, PresetEntry } from '../lib/album';
  import ClassList    from '../features/beauty/components/ClassList/ClassList.svelte';
  import PresetGrid   from '../features/beauty/components/PresetGrid/PresetGrid.svelte';
  import PresetDetail from '../features/beauty/components/PresetDetail/PresetDetail.svelte';
  import LiveDot      from '../ui/LiveDot/LiveDot.svelte';
  import FaceGridView from '../features/face_grid/components/FaceGridView/FaceGridView.svelte';

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

  const livePresets = $derived.by(() => {
    if (selectedClassId === null) return [];
    let all = beauty.livePresets[selectedClassId] ?? [];
    const region = beauty.selectedRegion;
    if (region) all = all.filter(p => p.region === region);
    const days = beauty.selectedDays;
    if (days !== 'ever') {
      const n = parseInt(days, 10);
      if (!isNaN(n)) {
        const cutoff = Math.floor(Date.now() / 1000) - n * 86400;
        all = all.filter(p => p.creation_at != null && p.creation_at >= cutoff);
      }
    }
    return all;
  });

  let pendingUpdate    = $state<Update | null>(null);
  let updateInstalling = $state(false);

  async function installUpdate() {
    if (!pendingUpdate || updateInstalling) return;
    updateInstalling = true;
    await pendingUpdate.downloadAndInstall();
    await relaunch();
  }

  const toasts = $derived<ToastItem[]>(
    pendingUpdate
      ? [{
          id:      2,
          type:    'success' as const,
          text:    updateInstalling ? $_('app.update.installing') : $_('app.update.available', { values: { version: pendingUpdate.version } }),
          onClick: installUpdate,
        }]
      : []
  );

  onMount(async () => {
    await eventBus.init();
    const regions = await getRegions().catch(() => []);
    setAvailableRegions(regions);
    check().then((u) => { if (u) pendingUpdate = u; }).catch(() => {});
    return () => eventBus.destroy();
  });

  // Reload when selected class, region, search, days, OR sort changes
  $effect(() => {
    const cls    = beauty.selectedClass;
    const region = beauty.selectedRegion;
    const search = beauty.searchQuery;
    const days   = beauty.selectedDays;
    const sort   = beauty.sortBy;
    if (cls) resetAndLoad(cls, region, search, days, sort);
  });

  // Update per-class counts whenever search, region, OR days changes
  $effect(() => {
    const search = beauty.searchQuery;
    const region = beauty.selectedRegion;
    const days   = beauty.selectedDays;
    const hasFilter = !!search.trim() || !!region || days !== 'ever';
    if (!hasFilter) { setSearchCounts([], false); return; }
    getClassSearchCounts(search, region, days).then(r => setSearchCounts(r, true)).catch(() => {});
  });

  // Live uploads (scraper/auto-download finishing while the album is open) only
  // patch the currently-selected class's card list in place — they don't touch
  // the server-computed filtered counts. Re-run that same count query, debounced,
  // whenever new live activity arrives and a filter is active, so counts for
  // other classes climb too instead of staying frozen until the filter changes.
  const totalLiveUploaded = $derived(
    Object.values(beauty.liveUploaded).reduce((a, b) => a + b, 0)
  );
  let liveCountsDebounce: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    totalLiveUploaded;
    untrack(() => {
      const search = beauty.searchQuery;
      const region = beauty.selectedRegion;
      const days   = beauty.selectedDays;
      const hasFilter = !!search.trim() || !!region || days !== 'ever';
      if (!hasFilter) return;
      clearTimeout(liveCountsDebounce);
      liveCountsDebounce = setTimeout(() => {
        getClassSearchCounts(search, region, days).then(r => setSearchCounts(r, true)).catch(() => {});
      }, 800);
    });
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

  async function resetAndLoad(cls: string, region: string, search: string, days: string, sort: string) {
    offset = 0;
    hasMore = false;
    presets = [];
    presetsError = '';
    const entry = beauty.classes.find(c => c.name === cls);
    if (entry) clearLiveForClass(entry.class_id);
    await doLoad(cls, 0, region, search, days, sort, true);
  }

  async function loadMore() {
    if (!beauty.selectedClass || !hasMore || loadingMore || presetsLoading) return;
    loadingMore = true;
    const nextOffset = offset + LIMIT;
    offset = nextOffset;
    await doLoad(beauty.selectedClass, nextOffset, beauty.selectedRegion, beauty.searchQuery, beauty.selectedDays, beauty.sortBy, false);
  }

  async function doLoad(cls: string, off: number, region: string, search: string, days: string, sort: string, initial: boolean) {
    if (initial) presetsLoading = true;
    else         loadingMore    = true;
    presetsError = '';
    try {
      if (initial) {
        const [fetched, wanted] = await Promise.all([
          getPresets(cls, off, LIMIT, sort, search, region, days),
          getWanted(),
        ]);
        presets = fetched;
        setWantedPresets(wanted);
        hasMore = fetched.length >= LIMIT;
      } else {
        const fetched = await getPresets(cls, off, LIMIT, sort, search, region, days);
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
</script>

<div class="app">
  {#if !beauty.dbReady && !beauty.dbError}
    <div class="splash">
      <div class="splash-text">{$_('app.connecting')}</div>
    </div>
  {:else if beauty.dbError}
    <div class="splash">
      <div class="splash-text error">{$_(`errors.db.${beauty.dbError}`)}</div>
    </div>
  {:else}
    <nav class="tab-nav">
      <button class:active={activeTab === 'beauty'}    onclick={() => activeTab = 'beauty'}>{$_('app.tabs.beauty')}</button>
      <button class:active={activeTab === 'face_grid'} onclick={() => activeTab = 'face_grid'}>{$_('app.tabs.face_grid')}</button>
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

<Toast {toasts} />
