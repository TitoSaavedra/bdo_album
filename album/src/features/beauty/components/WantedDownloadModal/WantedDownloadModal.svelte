<script lang="ts">
  import { untrack } from 'svelte';
  import type { PresetEntry } from '../../../../lib/album';
  import { openUrl } from '../../../../lib/album';
  import { beauty } from '../../state/beauty.svelte';

  interface Props {
    presets: PresetEntry[];
    onclose: () => void;
  }

  const { presets, onclose } = $props<Props>();

  const PAGE_SIZE = 16; // 4×4

  let items = $state(untrack(() => [...presets]));
  let page  = $state(0);

  const totalPages = $derived(Math.max(1, Math.ceil(items.length / PAGE_SIZE)));
  const pageItems  = $derived(items.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE));

  const activeClasses = $derived.by(() => {
    const map = new Map<number, { name: string; icon_svg: string | null; count: number }>();
    for (const p of items) {
      const entry = beauty.classes.find(c => c.class_id === p.class_id);
      if (!entry) continue;
      const existing = map.get(p.class_id);
      if (existing) existing.count++;
      else map.set(p.class_id, { name: entry.name, icon_svg: entry.icon_svg, count: 1 });
    }
    return [...map.entries()].map(([class_id, info]) => ({ class_id, ...info }));
  });

  function clampPage() {
    const maxPage = Math.max(0, Math.ceil(items.length / PAGE_SIZE) - 1);
    if (page > maxPage) page = maxPage;
  }

  function remove(id: string) {
    items = items.filter(p => p.preset_id !== id);
    clampPage();
  }

  function removeByClass(classId: number) {
    items = items.filter(p => p.class_id !== classId);
    clampPage();
  }

  function prevPage() { if (page > 0) page--; }
  function nextPage() { if (page < totalPages - 1) page++; }

  async function openAll() {
    for (const p of items) {
      await openUrl(`https://garmoth.com/beauty-album/preset/${p.preset_id}`);
    }
    onclose();
  }
</script>

<div
  class="backdrop"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === 'Escape' && onclose()}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >

    <div class="modal-header">
      <span class="modal-title">Presets to download</span>
      <span class="modal-count">{items.length}</span>
      <button class="close-btn" onclick={onclose}>✕</button>
    </div>

    {#if activeClasses.length > 0}
      <div class="class-pills">
        {#each activeClasses as cls (cls.class_id)}
          <div class="class-pill">
            {#if cls.icon_svg}
              <span class="pill-icon">{@html cls.icon_svg}</span>
            {/if}
            <span class="pill-name">{cls.name}</span>
            <span class="pill-count">{cls.count}</span>
            <button
              class="pill-remove"
              onclick={() => removeByClass(cls.class_id)}
              title="Remove all {cls.name} presets"
            >✕</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if items.length === 0}
      <div class="empty">No presets queued.</div>
    {:else}
      <div class="grid">
        {#each pageItems as p (p.preset_id)}
          {@const img   = p.image_1_url ?? p.image_2_url}
          {@const label = p.title || p.character_name || `#${p.preset_id}`}
          <div class="card">
            <div class="thumb-wrap">
              {#if img}
                <img src={img} alt={label} class="thumb" loading="lazy" />
              {:else}
                <div class="thumb-placeholder"></div>
              {/if}
              <button
                class="remove-btn"
                onclick={() => remove(p.preset_id)}
                title="Skip this preset"
              >✕</button>
            </div>
            <div class="card-label" title={label}>{label}</div>
          </div>
        {/each}
      </div>
    {/if}

    {#if totalPages > 1}
      <div class="pagination">
        <button class="page-btn" onclick={prevPage} disabled={page === 0}>‹</button>
        <span class="page-info">{page + 1} / {totalPages}</span>
        <button class="page-btn" onclick={nextPage} disabled={page === totalPages - 1}>›</button>
      </div>
    {/if}

    <div class="modal-footer">
      <button class="cancel-btn" onclick={onclose}>Cancel</button>
      <button class="open-btn" onclick={openAll} disabled={items.length === 0}>
        Open {items.length} in browser
      </button>
    </div>

  </div>
</div>

<style lang="scss">
  @use './WantedDownloadModal.scss';
</style>
