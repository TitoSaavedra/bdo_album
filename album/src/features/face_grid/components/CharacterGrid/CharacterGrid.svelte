<script lang="ts">
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import { faceGrid } from '../../state/face_grid.svelte';
  import CharacterSlot from '../CharacterSlot/CharacterSlot.svelte';

  interface Props {
    characters: CharacterEntry[];
  }
  const { characters }: Props = $props();

  // Characters from the CharacterOrderList, sorted by Order
  const ordered = $derived([...characters].sort((a, b) => a.order - b.order));

  // BMPs that exist in FaceTexture but are NOT in this account's list
  const accountNos   = $derived(new Set(characters.map(c => c.character_no)));
  const orphanBmps   = $derived(
    faceGrid.faceTextures
      .filter(t => !accountNos.has(t.character_no))
      .map(t => ({
        character_no: t.character_no,
        order:        999,
        has_bmp:      true,
        bmp_path:     t.path,
      } satisfies CharacterEntry))
  );
</script>

<div class="grid-wrap">
  <div class="char-grid">
    {#each ordered as char (char.character_no)}
      <CharacterSlot character={char} />
    {/each}
  </div>

  {#if orphanBmps.length > 0}
    <div class="orphans-section">
      <div class="section-label">Sin cuenta asignada ({orphanBmps.length})</div>
      <div class="orphan-grid">
        {#each orphanBmps as char (char.character_no)}
          <CharacterSlot character={char} />
        {/each}
      </div>
    </div>
  {/if}
</div>
