<script lang="ts">
  import './App.scss';
  import { onMount, tick, untrack } from 'svelte';
  import { check, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { _ } from 'svelte-i18n';
  import { eventBus } from '../lib/events';
  import Toast from '../ui/Toast/Toast.svelte';
  import type { ToastItem } from '../ui/Toast/Toast.svelte';
  import Titlebar from '../ui/Titlebar/Titlebar.svelte';
  import ModuleSwitcher from '../ui/ModuleSwitcher/ModuleSwitcher.svelte';
  import { getPresets, getPresetsByCreator, getWanted, getRegions, getClassSearchCounts } from '../lib/album';
  import {
    beauty,
    selectClass,
    setWantedPresets,
    clearLiveForClass,
    setSelectedRegion,
    setAvailableRegions,
    setSearchCounts,
    setCreatorFilter,
    reopenLastPreset,
  } from '../features/beauty/state/beauty.svelte';
  import { withViewTransition } from '../lib/viewTransition';
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
    // Cross-class creator browsing doesn't map onto any single class's live feed.
    if (beauty.creatorFilter) return [];
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

  // Reload when selected class, region, search, days, sort, OR the favorite-creator
  // filter changes. The creator filter takes over the grid entirely — it deliberately
  // ignores region/days so a favorited creator's full catalog always shows up.
  $effect(() => {
    const cls     = beauty.selectedClass;
    const region  = beauty.selectedRegion;
    const search  = beauty.searchQuery;
    const days    = beauty.selectedDays;
    const sort    = beauty.sortBy;
    const creator = beauty.creatorFilter;
    if (creator) resetAndLoadCreator(creator, search, sort);
    else if (cls) resetAndLoad(cls, region, search, days, sort);
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

  // Preserve the grid's scroll position across opening/closing a preset —
  // PresetDetail replaces PresetGrid inside the same scrollable `mainEl`, so
  // without this the browser clamps scrollTop to 0 while the (shorter)
  // detail content is showing and that's lost for good once we're back.
  // `$effect.pre` runs before the DOM patches for a given change, so on the
  // opening transition `mainEl` still holds the outgoing grid when we read
  // its scrollTop; the plain `$effect` below runs after the DOM patches, so
  // on the closing transition `mainEl` already holds the restored grid.
  let savedGridScrollTop = 0;
  let presetDetailWasOpen = false;
  $effect.pre(() => {
    const isOpen = !!beauty.presetDetail;
    if (isOpen && !presetDetailWasOpen && mainEl) savedGridScrollTop = mainEl.scrollTop;
    presetDetailWasOpen = isOpen;
  });
  $effect(() => {
    if (!beauty.presetDetail && mainEl) mainEl.scrollTop = savedGridScrollTop;
  });

  // Mouse side buttons: back (3) closes the open preset, forward (4) reopens
  // whatever was last closed — same one-slot back/forward feel as a browser,
  // scoped to the Beauty tab since Face Grid has no equivalent navigation.
  function handleMouseNav(e: MouseEvent) {
    if (activeTab !== 'beauty' || e.button !== 4) return;
    e.preventDefault();
    withViewTransition(() => reopenLastPreset());
  }

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

  // Set by resetAndLoad/resetAndLoadCreator, called by doLoad/loadMore — keeps
  // pagination logic in one place regardless of which mode is active.
  let fetchPage: ((off: number) => Promise<PresetEntry[]>) | null = null;

  async function resetAndLoad(cls: string, region: string, search: string, days: string, sort: string) {
    offset = 0;
    hasMore = false;
    presets = [];
    presetsError = '';
    const entry = beauty.classes.find(c => c.name === cls);
    if (entry) clearLiveForClass(entry.class_id);
    fetchPage = (off) => getPresets(cls, off, LIMIT, sort, search, region, days);
    await doLoad(true);
  }

  async function resetAndLoadCreator(creator: string, search: string, sort: string) {
    offset = 0;
    hasMore = false;
    presets = [];
    presetsError = '';
    fetchPage = (off) => getPresetsByCreator(creator, off, LIMIT, sort, search);
    await doLoad(true);
  }

  async function loadMore() {
    if (!fetchPage || !hasMore || loadingMore || presetsLoading) return;
    loadingMore = true;
    offset = offset + LIMIT;
    await doLoad(false);
  }

  async function doLoad(initial: boolean) {
    if (!fetchPage) return;
    if (initial) presetsLoading = true;
    else         loadingMore    = true;
    presetsError = '';
    try {
      if (initial) {
        const [fetched, wanted] = await Promise.all([
          fetchPage(0),
          getWanted(),
        ]);
        presets = fetched;
        setWantedPresets(wanted);
        hasMore = fetched.length >= LIMIT;
      } else {
        const fetched = await fetchPage(offset);
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
    setCreatorFilter(null);
    selectClass(cls);
  }
</script>

<svelte:window onmousedown={handleMouseNav} />

<div class="app">
  <Titlebar />

  {#if !beauty.dbReady && !beauty.dbError}
    <div class="splash">
      <div class="splash-text">{$_('app.connecting')}</div>
    </div>
  {:else if beauty.dbError}
    <div class="splash">
      <div class="splash-text error">{$_(`errors.db.${beauty.dbError}`)}</div>
    </div>
  {:else}
    {#snippet moduleSwitcherSnippet()}
      <ModuleSwitcher active={activeTab} onchange={(m) => (activeTab = m)} />
    {/snippet}

    {#if activeTab === 'beauty'}
      <ClassList
        selectedClass={beauty.selectedClass}
        onselect={handleSelectClass}
        moduleSwitcher={moduleSwitcherSnippet}
      />
      <main class="main custom-scroll" bind:this={mainEl}>
        {#if beauty.presetDetail}
          <PresetDetail />
        {:else}
          {#if beauty.creatorFilter}
            <div class="creator-banner">
              {$_('beauty.preset_grid.creator_banner', { values: { creator: beauty.creatorFilter } })}
              <button class="creator-banner-back" onclick={() => setCreatorFilter(null)}>
                {$_('beauty.preset_grid.back_to_class', { values: { class: beauty.selectedClass ?? '' } })}
              </button>
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
        {/if}
      </main>
    {:else}
      <div class="module-row-solo">
        {@render moduleSwitcherSnippet()}
      </div>
      <FaceGridView />
    {/if}
  {/if}

  <LiveDot />
</div>

<Toast {toasts} />
