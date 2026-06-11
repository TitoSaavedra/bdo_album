<script lang="ts">
  import './GridAccountPicker.scss';
  import { fade, scale } from 'svelte/transition';
  import type { BdoAccount } from '../../../../lib/face_grid';
  import { faceGrid, setVisibleAccounts, closeAccountPicker } from '../../state/face_grid.svelte';

  interface Props {
    accounts: BdoAccount[];
  }
  const { accounts }: Props = $props();

  let selected = $state<string | null>(
    faceGrid.visibleAccountIds?.[0] ?? null
  );

  function selectAccount(accountId: string) {
    selected = accountId;
    setVisibleAccounts([accountId]);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="picker-backdrop"
  transition:fade={{ duration: 150 }}
  onclick={closeAccountPicker}
  onkeydown={(e) => e.key === 'Escape' && closeAccountPicker()}
>
  <div
    class="picker-dialog"
    role="dialog"
    tabindex="-1"
    transition:scale={{ duration: 150, start: 0.95 }}
    onclick={(e) => e.stopPropagation()}
  >
    <h3>Select Account</h3>
    <p class="picker-subtitle">Choose your BDO account</p>

    <div class="accounts-list">
      {#each accounts as account (account.account_id)}
        <button
          class="account-item"
          class:active={selected === account.account_id}
          onclick={() => selectAccount(account.account_id)}
        >
          <div class="account-info">
            <span class="account-name">Account {account.account_id}</span>
            <span class="account-badge">{account.characters.length}</span>
          </div>
          <div class="account-thumbs">
            {#each (faceGrid.accountThumbs[account.account_id] || []) as thumb}
              <img src={thumb} alt="" />
            {/each}
          </div>
        </button>
      {/each}
    </div>
  </div>
</div>

<style lang="scss">
  @use './GridAccountPicker.scss';
</style>
