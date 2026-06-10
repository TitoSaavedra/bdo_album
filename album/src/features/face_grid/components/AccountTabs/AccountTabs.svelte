<script lang="ts">
  import './AccountTabs.scss';
  import type { BdoAccount } from '../../../../lib/face_grid';
  import { faceGrid, removeAccount, selectAccount } from '../../state/face_grid.svelte';

  interface Props {
    accounts: BdoAccount[];
  }
  const { accounts }: Props = $props();
</script>

<div class="account-tabs">
  {#each accounts as acc (acc.account_id)}
    <div class="tab" class:active={faceGrid.activeAccountId === acc.account_id}>
      <button class="tab-btn" onclick={() => selectAccount(acc.account_id)}>
        Cuenta {acc.account_id}
        <span class="count">({acc.characters.length})</span>
      </button>
      <button
        class="delete-btn"
        title="Eliminar cuenta"
        onclick={() => removeAccount(acc.account_id)}
      >✕</button>
    </div>
  {/each}
</div>
