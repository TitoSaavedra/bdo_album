<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import ImageCard from '../../../../ui/ImageCard/ImageCard.svelte';
  import { bmpPathFor } from '../../state/face_grid.svelte';

  interface Props {
    characters: CharacterEntry[];
  }
  const { characters }: Props = $props();

  const ordered = $derived([...characters].sort((a, b) => a.order - b.order));

  function getSrc(char: CharacterEntry): string | null {
    const path = bmpPathFor(char.character_no) ?? char.bmp_path;
    return path ? convertFileSrc(path) : null;
  }
</script>

<div class="grid-wrap">
  <div class="char-grid">
    {#each ordered as char (char.character_no)}
      <ImageCard src={getSrc(char)} badge={char.order} />
    {/each}
  </div>
</div>

<style lang="scss">
  @use './CharacterGrid.scss';
</style>
