<script lang="ts">
  import './FaceGridView.scss';
  import { onMount } from 'svelte';
  import {
    activeAccount,
    faceGrid,
    loadAccounts,
    openSaveDialog,
  } from '../../state/face_grid.svelte';
  import AccountTabs   from '../AccountTabs/AccountTabs.svelte';
  import CharacterGrid from '../CharacterGrid/CharacterGrid.svelte';
  import FaceGridSidebar from '../FaceGridSidebar/FaceGridSidebar.svelte';
  import SaveGridDialog  from '../SaveGridDialog/SaveGridDialog.svelte';

  onMount(() => loadAccounts());

  const account = $derived(activeAccount());
</script>

<div class="face-grid-view">
  <div class="fg-main">
    <FaceGridSidebar />

    <div class="fg-center">
      {#if faceGrid.loading}
        <div class="fg-loading">Leyendo configuración de BDO...</div>
      {:else if faceGrid.error}
        <div class="fg-error">{faceGrid.error}</div>
      {:else if faceGrid.accounts.length === 0}
        <div class="fg-empty">
          <span>No se encontraron cuentas de BDO</span>
          <span class="fg-empty-hint">
            Verifica que BDO esté instalado y hayas iniciado sesión al menos una vez
          </span>
        </div>
      {:else}
        <AccountTabs accounts={faceGrid.accounts} />

        <div class="fg-toolbar">
          <button class="btn-save" onclick={openSaveDialog}>
            Guardar preset
          </button>
        </div>

        {#if account}
          <CharacterGrid characters={account.characters} />
        {/if}
      {/if}
    </div>
  </div>

  {#if faceGrid.saveDialogOpen}
    <SaveGridDialog />
  {/if}
</div>
