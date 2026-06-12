<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '../../../../ui/Button/Button.svelte';
  import {
    getLoading,
    getError,
    getAccounts,
    getActiveAccountId,
    getShowAccountPicker,
    getActiveAccount,
    loadAccounts,
    openSaveGridDialog,
    openAccountPicker,
  } from '../../state/face_grid.svelte';
  import CharacterGrid from '../CharacterGrid/CharacterGrid.svelte';
  import Dialog from '../Dialog/Dialog.svelte';
  import FaceGridSidebar from '../FaceGridSidebar/FaceGridSidebar.svelte';
  import GridAccountPicker from '../GridAccountPicker/GridAccountPicker.svelte';

  onMount(async () => {
    await loadAccounts();
  });
</script>

<div class="face-grid-view">
  <div class="fg-main">
    <FaceGridSidebar />

    <div class="fg-center">
      {#if getLoading()}
        <div class="fg-loading">Reading BDO configuration...</div>
      {:else if getError()}
        <div class="fg-error">{getError()}</div>
      {:else if getAccounts().length === 0}
        <div class="fg-empty">
          <span>No BDO accounts found</span>
          <span class="fg-empty-hint">
            Make sure BDO is installed and you've logged in at least once
          </span>
        </div>
      {:else}
        <div class="fg-toolbar">
          <button class="account-chip" onclick={openAccountPicker} title="Change account">
            <span class="chip-label">Account</span>
            <span class="chip-id">{getActiveAccountId()}</span>
            <span class="chip-arrow">&#x25BE;</span>
          </button>
          <div class="toolbar-actions">
            <Button variant="primary" onclick={openSaveGridDialog}>Save Preset</Button>
            <Button variant="ghost" onclick={loadAccounts} title="Reload from disk">Refresh</Button>
          </div>
        </div>

        {#if getActiveAccount()}
          <CharacterGrid characters={getActiveAccount()!.characters} />
        {/if}
      {/if}
    </div>
  </div>

  <Dialog />
  {#if getShowAccountPicker()}
    <GridAccountPicker />
  {/if}
</div>

<style lang="scss">
  @use './FaceGridView.scss';
</style>
