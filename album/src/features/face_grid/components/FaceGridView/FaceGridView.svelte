<script lang="ts">
  import './FaceGridView.scss';
  import { onMount } from 'svelte';
  import {
    activeAccount,
    faceGrid,
    loadAccounts,
    openSaveGridDialog,
  } from '../../state/face_grid.svelte';
  import AccountTabs   from '../AccountTabs/AccountTabs.svelte';
  import CharacterGrid from '../CharacterGrid/CharacterGrid.svelte';
  import Dialog from '../Dialog/Dialog.svelte';
  import FaceGridSidebar from '../FaceGridSidebar/FaceGridSidebar.svelte';

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
        <AccountTabs accounts={faceGrid.accounts} onSave={openSaveGridDialog} onReset={loadAccounts} />

        {#if account}
          <CharacterGrid characters={account.characters} />
        {/if}
      {/if}
    </div>
  </div>

  <Dialog />
</div>
