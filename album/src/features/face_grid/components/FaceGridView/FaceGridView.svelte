<script lang="ts">
  import './FaceGridView.scss';
  import { onMount } from 'svelte';
  import Button from '../../../../ui/Button/Button.svelte';
  import {
    activeAccount,
    faceGrid,
    loadAccounts,
    openSaveGridDialog,
    openAccountPicker,
  } from '../../state/face_grid.svelte';
  import CharacterGrid from '../CharacterGrid/CharacterGrid.svelte';
  import Dialog from '../Dialog/Dialog.svelte';
  import FaceGridSidebar from '../FaceGridSidebar/FaceGridSidebar.svelte';
  import GridAccountPicker from '../GridAccountPicker/GridAccountPicker.svelte';

  onMount(() => loadAccounts());

  const account = $derived(activeAccount());
</script>

<div class="face-grid-view">
  <div class="fg-main">
    <FaceGridSidebar />

    <div class="fg-center">
      {#if faceGrid.loading}
        <div class="fg-loading">Reading BDO configuration...</div>
      {:else if faceGrid.error}
        <div class="fg-error">{faceGrid.error}</div>
      {:else if faceGrid.accounts.length === 0}
        <div class="fg-empty">
          <span>No BDO accounts found</span>
          <span class="fg-empty-hint">
            Make sure BDO is installed and you've logged in at least once
          </span>
        </div>
      {:else}
        <div class="fg-toolbar">
          <div class="toolbar-left">
            <span class="account-name">Account {faceGrid.activeAccountId}</span>
            <div class="toolbar-actions">
              <Button variant="primary" onclick={openSaveGridDialog} title="Save current grid as preset">Save</Button>
              <Button variant="ghost" onclick={loadAccounts} title="Refresh accounts">Refresh</Button>
            </div>
          </div>
          <Button variant="icon" title="Select accounts" onclick={openAccountPicker}>⚙</Button>
        </div>

        {#if account}
          <CharacterGrid characters={account.characters} />
        {/if}
      {/if}
    </div>
  </div>

  <Dialog />
  {#if faceGrid.showAccountPicker}
    <GridAccountPicker accounts={faceGrid.accounts} />
  {/if}
</div>
