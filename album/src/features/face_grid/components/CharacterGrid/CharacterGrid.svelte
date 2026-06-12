<script lang="ts">
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import ImageCard from '../../../../ui/ImageCard/ImageCard.svelte';
  import { getCharacterSrc, getApplyingChars, saveFaceToDisk } from '../../state/face_grid.svelte';

  interface Props {
    characters: CharacterEntry[];
  }
  const { characters }: Props = $props();

  const ordered = $derived([...characters].sort((a, b) => a.order - b.order));

  let hoveredChar = $state<string | null>(null);
  let isDragging  = $state(false);

  let _unlisteners: UnlistenFn[] = [];

  const IMAGE_EXTS = new Set(['png', 'jpg', 'jpeg', 'bmp', 'webp', 'gif', 'tiff']);

  function charAtPoint(x: number, y: number): string | null {
    const dpr = window.devicePixelRatio || 1;
    const el  = document.elementFromPoint(x / dpr, y / dpr);
    return (el?.closest('[data-char]') as HTMLElement | null)?.dataset.char ?? null;
  }

  onMount(() => {
    Promise.all([
      listen('tauri://drag-enter', () => { isDragging = true; }),
      listen('tauri://drag-leave', () => { isDragging = false; hoveredChar = null; }),

      listen<{ paths: string[]; position: { x: number; y: number } }>(
        'tauri://drag-over',
        (e) => {
          isDragging  = true;
          hoveredChar = charAtPoint(e.payload.position.x, e.payload.position.y);
        },
      ),

      listen<{ paths: string[]; position: { x: number; y: number } }>(
        'tauri://drag-drop',
        async (e) => {
          isDragging  = false;
          const char  = hoveredChar;
          hoveredChar = null;
          if (!char) return;

          const path = e.payload.paths[0];
          if (!path) return;

          const ext = path.split('.').pop()?.toLowerCase() ?? '';
          if (!IMAGE_EXTS.has(ext)) return;

          try {
            await saveFaceToDisk(char, path);
          } catch (err) {
            console.error('Failed to save face to disk:', err);
          }
        },
      ),
    ]).then(fns => { _unlisteners = fns; });
  });

  onDestroy(() => _unlisteners.forEach(fn => fn()));
</script>

<div class="grid-wrap">
  <div class="char-grid">
    {#each ordered as char (char.character_no)}
      <div
        data-char={char.character_no}
        class="card-wrap"
        class:drag-over={isDragging && hoveredChar === char.character_no}
      >
        {#if getApplyingChars().has(char.character_no)}
          <div class="skeleton-card"></div>
        {:else}
          <ImageCard
            src={getCharacterSrc(char)}
            badge={char.order}
          />
        {/if}
      </div>
    {/each}
  </div>
</div>

<style lang="scss">
  @use './CharacterGrid.scss';
</style>
