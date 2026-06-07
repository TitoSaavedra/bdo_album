<script lang="ts">
  import type { BdoAccount } from '../../../../lib/face_grid';
  import { faceGrid, selectAccount } from '../../state/face_grid.svelte';

  interface Props {
    accounts: BdoAccount[];
  }
  const { accounts }: Props = $props();
</script>

{#if accounts.length > 1}
  <div class="account-tabs">
    {#each accounts as acc (acc.account_id)}
      <button
        class:active={faceGrid.activeAccountId === acc.account_id}
        onclick={() => selectAccount(acc.account_id)}
      >
        Cuenta {acc.account_id}
        <span style="font-size:0.7em;opacity:0.6"> ({acc.characters.length})</span>
      </button>
    {/each}
  </div>
{/if}
