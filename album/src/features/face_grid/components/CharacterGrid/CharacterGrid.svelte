<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import ImageCard from '../../../../ui/ImageCard/ImageCard.svelte';
  import { bmpPathFor, faceGrid, uploadCharacterFace } from '../../state/face_grid.svelte';

  interface Props {
    characters: CharacterEntry[];
  }
  const { characters }: Props = $props();

  const ordered = $derived([...characters].sort((a, b) => a.order - b.order));

  function getSrc(char: CharacterEntry): string | null {
    const custom = faceGrid.customFaces[char.character_no];
    if (custom) return custom;
    const path = bmpPathFor(char.character_no) ?? char.bmp_path;
    return path ? convertFileSrc(path) : null;
  }

  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        const base64 = result.split(',')[1];
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  async function handleDrop(char: CharacterEntry, e: DragEvent) {
    const file = e.dataTransfer?.files[0];
    if (!file || !file.type.startsWith('image/')) return;
    try {
      const b64 = await fileToBase64(file);
      await uploadCharacterFace(char.character_no, b64);
    } catch (err) {
      console.error('Failed to upload character face:', err);
    }
  }
</script>

<div class="grid-wrap">
  <div class="char-grid">
    {#each ordered as char (char.character_no)}
      <ImageCard
        src={getSrc(char)}
        badge={char.order}
        ondrop={(e) => handleDrop(char, e)}
      />
    {/each}
  </div>
</div>

<style lang="scss">
  @use './CharacterGrid.scss';
</style>
