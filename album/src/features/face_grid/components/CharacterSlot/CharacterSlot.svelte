<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { PresetEntry } from '../../../../lib/album';
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import { assignPresetToSlot, bmpPathFor, clearSlot, faceGrid, isOrphan } from '../../state/face_grid.svelte';

  interface Props {
    character: CharacterEntry;
    onassign?: (characterNo: string) => void;
  }
  const { character, onassign }: Props = $props();

  let isDragOver = $state(false);

  const bmpPath    = $derived(bmpPathFor(character.character_no) ?? character.bmp_path);
  const bmpSrc     = $derived(bmpPath ? convertFileSrc(bmpPath) : null);
  const pending    = $derived(faceGrid.pendingSlots[character.character_no]);
  const orphan     = $derived(isOrphan(character.character_no));

  function ondragover(e: DragEvent) {
    e.preventDefault();
    isDragOver = true;
  }

  function ondragleave() {
    isDragOver = false;
  }

  function ondrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    const raw = e.dataTransfer?.getData('preset_json');
    if (!raw) return;
    try {
      const preset: PresetEntry = JSON.parse(raw);
      assignPresetToSlot(character.character_no, preset);
      onassign?.(character.character_no);
    } catch { /* ignore */ }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="slot"
  class:drag-over={isDragOver}
  class:has-pending={!!pending}
  class:orphan={orphan}
  {ondragover}
  {ondragleave}
  {ondrop}
>
  {#if bmpSrc}
    <img src={bmpSrc} alt="" class="bmp-img" />
  {:else}
    <div class="empty">{character.order}</div>
  {/if}

  {#if pending?.image_url}
    <img src={pending.image_url} alt="" class="pending-img" />
  {/if}

  <span class="order-badge">{character.order}</span>

  {#if orphan}
    <span class="orphan-badge">sin cuenta</span>
  {/if}

  {#if pending}
    <button class="clear-btn" onclick={() => clearSlot(character.character_no)}>✕</button>
  {/if}
</div>
