<script lang="ts">
  import { scale } from 'svelte/transition';
  import {
    getAccounts,
    getAccountThumbs,
    getActiveAccountId,
    selectAccount,
    closeAccountPicker,
  } from '../../state/face_grid.svelte';
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="picker-backdrop"
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
      {#each getAccounts().filter(a => a.characters.some(c => c.has_bmp)) as account (account.account_id)}
        <button
          class="account-item"
          class:active={getActiveAccountId() === account.account_id}
          onclick={() => selectAccount(account.account_id)}
        >
          <div class="account-info">
            <span class="account-name">Account {account.account_id}</span>
            <span class="account-badge">{account.characters.length}</span>
          </div>
          <div class="account-thumbs">
            {#each (getAccountThumbs()[account.account_id] || []) as thumb}
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
