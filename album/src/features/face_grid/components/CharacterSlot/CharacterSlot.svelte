<script lang="ts">
  import './CharacterSlot.scss';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import { bmpPathFor, clearSlot, faceGrid } from '../../state/face_grid.svelte';

  interface Props {
    character: CharacterEntry;
  }
  const { character }: Props = $props();

  let isDragOver = $state(false);

  const bmpPath = $derived(bmpPathFor(character.character_no) ?? character.bmp_path);
  const bmpSrc  = $derived(bmpPath ? convertFileSrc(bmpPath) : null);
  const pending = $derived(faceGrid.pendingSlots[character.character_no]);

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
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="bdo-card"
  class:drag-over={isDragOver}
  {ondragover}
  {ondragleave}
  {ondrop}
>
  {#if bmpSrc}
    <img src={bmpSrc} alt="" class="card-bg" />
  {/if}

  <div class="card-content">
    <span class="order-badge">{character.order}</span>
  </div>

  {#if pending}
    <img src={pending.image_url} alt="" class="pending-img" />
    <button class="clear-btn" onclick={() => clearSlot(character.character_no)}>✕</button>
  {/if}
</div>
