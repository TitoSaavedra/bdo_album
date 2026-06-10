<script lang="ts">
  import './PresetPicker.scss';
  import { onMount } from 'svelte';
  import type { ClassEntry, PresetEntry } from '../../../../lib/album';
  import { getClasses, getPresets } from '../../../../lib/album';
  import { setDraggingPreset } from '../../state/face_grid.svelte';

  let classes   = $state<ClassEntry[]>([]);
  let presets   = $state<PresetEntry[]>([]);
  let classId   = $state<number | null>(null);
  let offset    = $state(0);
  let hasMore   = $state(true);
  const limit   = 40;

  onMount(async () => {
    classes = await getClasses();
    if (classes.length > 0) classId = classes[0].class_id;
  });

  $effect(() => {
    if (classId === null) return;
    const cls = classes.find(c => c.class_id === classId);
    if (!cls) return;
    offset  = 0;
    presets = [];
    loadMore(cls.name, 0);
  });

  async function loadMore(className?: string, fromOffset?: number) {
    const cls  = classes.find(c => c.class_id === classId);
    if (!cls) return;
    const name = className ?? cls.name;
    const off  = fromOffset ?? offset;
    const batch = await getPresets(name, off, limit);
    presets = off === 0 ? batch : [...presets, ...batch];
    offset  = off + batch.length;
    hasMore = batch.length === limit;
  }
</script>

<div class="preset-picker">
  <div class="picker-header">
    <div class="picker-title">Arrastrar preset</div>
    <select bind:value={classId}>
      {#each classes as cls (cls.class_id)}
        <option value={cls.class_id}>{cls.name}</option>
      {/each}
    </select>
  </div>

  <div class="preset-list">
    {#if presets.length === 0}
      <div class="empty-msg" style="grid-column:1/-1">Sin presets</div>
    {:else}
      {#each presets as preset (preset.preset_id)}
        {#if preset.image_1_url}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="pick-item"
            draggable="true"
            ondragstart={(e) => {
              e.dataTransfer?.setData('preset_json', JSON.stringify(preset));
              setDraggingPreset(preset);
            }}
            ondragend={() => setDraggingPreset(null)}
          >
            <img src={preset.image_1_url} alt={preset.title ?? ''} loading="lazy" />
          </div>
        {/if}
      {/each}
    {/if}
  </div>

  {#if hasMore}
    <button class="load-more" onclick={() => loadMore()}>Cargar más</button>
  {/if}
</div>
