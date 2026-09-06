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
  import { getPresets, getWanted, getClassSearchCounts } from '../lib/album';
  import {
    beauty,
    setWantedPresets,
    clearLiveForClass,
    setSearchCounts,
    reopenLastPreset,
  } from '../features/beauty/state/beauty.svelte';
  import { withViewTransition } from '../lib/viewTransition';
  import type { PresetEntry } from '../lib/album';
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

  // Only the single-class-and-no-creator case has one class's feed to patch —
  // 0/2+ classes or a creator filter falls back to "no live patch, shows up
  // on next natural reload" (same as creator-browsing already did before
  // filters composed; just a generalized condition now).
  const singleSelectedClassId = $derived.by(() => {
    if (beauty.selectedClasses.size !== 1) return null;
    const name = [...beauty.selectedClasses][0];
    return beauty.classes.find(c => c.name === name)?.class_id ?? null;
  });

  const livePresets = $derived.by(() => {
    if (beauty.creatorFilter) return [];
    if (singleSelectedClassId === null) return [];
    let all = beauty.livePresets[singleSelectedClassId] ?? [];
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

  onMount(() => {
    eventBus.init();
    check().then((u) => { if (u) pendingUpdate = u; }).catch(() => {});
    return () => eventBus.destroy();
  });

  // Reload whenever any filter changes — class(es), creator, region, search,
  // days, sort, or any status toggle. One unified path now (see
  // preset_repo.rs::get_filtered): every filter is just an optional narrowing
  // condition on the same query, none of them exclusive branches anymore.
  // Guarded on classes having loaded so this doesn't fire a "browse every
  // class" query during the brief startup window before loadClasses()'s
  // auto-select-first-class runs (selectedClasses is empty then too, but for
  // "nothing's loaded yet", not "the user chose to browse everything").
  // Debounced (unlike the search box's own 300ms upstream in ClassList, this
  // one covers every OTHER filter — pills/toggles fire with no debounce of
  // their own, so a quick run through several of them used to queue up one
  // full round-trip per click) — same clearTimeout/setTimeout shape as
  // liveCountsDebounce below.
  let reloadDebounce: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    if (beauty.classes.length === 0) return;
    const classIds = [...beauty.selectedClasses]
      .map(name => beauty.classes.find(c => c.name === name)?.class_id)
      .filter((id): id is number => id !== undefined);
    const creator       = beauty.creatorFilter;
    const region         = beauty.selectedRegion;
    const search         = beauty.searchQuery;
    const days           = beauty.selectedDays;
    const sort           = beauty.sortBy;
    const hasMods        = beauty.hasModificationsFilter;
    const wanted         = beauty.wantedFilter;
    const hasPab         = beauty.hasPabFilter;
    const showDiscarded  = beauty.showDiscardedFilter;
    clearTimeout(reloadDebounce);
    reloadDebounce = setTimeout(() => {
      resetAndLoad(classIds, creator, region, search, days, sort, hasMods, wanted, hasPab, showDiscarded);
    }, 150);
  });

  // Update per-class counts whenever any non-class filter changes (counts are
  // already class-agnostic — one row per class regardless of what's selected).
  // Same debounce reasoning as the reload effect above.
  let countsDebounce: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const search        = beauty.searchQuery;
    const region         = beauty.selectedRegion;
    const days           = beauty.selectedDays;
    const hasMods        = beauty.hasModificationsFilter;
    const wanted         = beauty.wantedFilter;
    const hasPab         = beauty.hasPabFilter;
    const showDiscarded  = beauty.showDiscardedFilter;
    const hasFilter = !!search.trim() || !!region || days !== 'ever' || hasMods || wanted || hasPab || showDiscarded;
    clearTimeout(countsDebounce);
    if (!hasFilter) { setSearchCounts([], false); return; }
    countsDebounce = setTimeout(() => {
      getClassSearchCounts(search, region, days, hasMods, wanted, hasPab, showDiscarded).then(r => setSearchCounts(r, true)).catch(() => {});
    }, 150);
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
      const search        = beauty.searchQuery;
      const region         = beauty.selectedRegion;
      const days           = beauty.selectedDays;
      const hasMods        = beauty.hasModificationsFilter;
      const wanted         = beauty.wantedFilter;
      const hasPab         = beauty.hasPabFilter;
      const showDiscarded  = beauty.showDiscardedFilter;
      const hasFilter = !!search.trim() || !!region || days !== 'ever' || hasMods || wanted || hasPab || showDiscarded;
      if (!hasFilter) return;
      clearTimeout(liveCountsDebounce);
      liveCountsDebounce = setTimeout(() => {
        getClassSearchCounts(search, region, days, hasMods, wanted, hasPab, showDiscarded).then(r => setSearchCounts(r, true)).catch(() => {});
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

  // Set by resetAndLoad, called by doLoad/loadMore — keeps pagination logic
  // in one place regardless of how many classes/which creator is active.
  let fetchPage: ((off: number) => Promise<PresetEntry[]>) | null = null;

  async function resetAndLoad(
    classIds: number[], creator: string | null, region: string, search: string,
    days: string, sort: string, hasModifications: boolean, isWanted: boolean,
    hasPab: boolean, showDiscarded: boolean,
  ) {
    offset = 0;
    hasMore = false;
    presets = [];
    presetsError = '';
    if (classIds.length === 1) clearLiveForClass(classIds[0]);
    fetchPage = (off) => getPresets(classIds, creator, off, LIMIT, sort, search, region, days, hasModifications, isWanted, hasPab, showDiscarded);
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
      <ClassList moduleSwitcher={moduleSwitcherSnippet} />
      <main class="main custom-scroll" bind:this={mainEl}>
        {#if beauty.presetDetail}
          <PresetDetail />
        {:else}
          <PresetGrid
            {presets}
            {livePresets}
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
