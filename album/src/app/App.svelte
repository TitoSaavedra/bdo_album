<script lang="ts">
  import './App.scss';
  import { onMount } from 'svelte';
  import { eventBus } from '../lib/events';
  import { getPresets, getWanted } from '../lib/album';
  import {
    beauty,
    selectClass,
    setWantedPresets,
  } from '../features/beauty/state/beauty.svelte';
  import type { ClassEntry, PresetEntry } from '../lib/album';
  import ClassList    from '../features/beauty/components/ClassList/ClassList.svelte';
  import PresetGrid   from '../features/beauty/components/PresetGrid/PresetGrid.svelte';
  import PresetDetail from '../features/beauty/components/PresetDetail/PresetDetail.svelte';

  let presets        = $state<PresetEntry[]>([]);
  let presetsLoading = $state(false);
  let presetsError   = $state('');

  onMount(async () => {
    await eventBus.init();
    return () => eventBus.destroy();
  });

  $effect(() => {
    if (beauty.selectedClass) {
      loadPresets(beauty.selectedClass);
    }
  });

  async function loadPresets(className: string) {
    presetsLoading = true;
    presetsError = '';
    presets = [];
    try {
      const [fetched, wanted] = await Promise.all([
        getPresets(className),
        getWanted(),
      ]);
      presets = fetched;
      setWantedPresets(wanted);
    } catch (e) {
      presetsError = String(e);
    } finally {
      presetsLoading = false;
    }
  }

  function handleSelectClass(cls: ClassEntry) {
    selectClass(cls);
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
    <div class="layout">
      <aside class="sidebar">
        <ClassList
          selectedClass={beauty.selectedClass}
          onselect={handleSelectClass}
        />
      </aside>
      <main class="main custom-scroll">
        <PresetGrid
          {presets}
          selectedClass={beauty.selectedClass}
          loading={presetsLoading}
          error={presetsError}
        />
      </main>
    </div>
  {/if}

  <PresetDetail />
</div>
