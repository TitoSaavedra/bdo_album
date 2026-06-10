<script lang="ts">
  import './CharacterGrid.scss';
  import type { CharacterEntry } from '../../../../lib/face_grid';
  import CharacterSlot from '../CharacterSlot/CharacterSlot.svelte';

  interface Props {
    characters: CharacterEntry[];
  }
  const { characters }: Props = $props();

  const ordered = $derived([...characters].sort((a, b) => a.order - b.order));
</script>

<div class="grid-wrap">
  <div class="char-grid">
    {#each ordered as char (char.character_no)}
      <CharacterSlot character={char} />
    {/each}
  </div>
</div>
